use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, VecDeque},
    io::Write,
    sync::mpsc,
    time::Duration,
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Comparison {
    id: String,
    base: String,
    target: String,
    captured_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    revision: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    path: String,
    line: usize,
    side: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Note {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    text: String,
    reference: Option<Reference>,
    item: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot: Option<Comparison>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution: Option<Resolution>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Resolution {
    snapshot: Comparison,
    summary: String,
}
#[derive(Default)]
pub(super) struct Registry {
    notes: BTreeMap<String, (String, Option<Resolution>)>,
}
impl Registry {
    pub(super) fn resolutions(&self) -> Value {
        Value::Array(
            self.notes
                .iter()
                .filter_map(|(id, (_, resolution))| {
                    resolution
                        .as_ref()
                        .map(|r| json!({"id":id,"snapshot":r.snapshot,"summary":r.summary}))
                })
                .collect(),
        )
    }
    pub(super) fn resolve(
        &mut self,
        body: &[u8],
        snapshot: &super::Snapshot,
        revision: &str,
    ) -> Result<Value, String> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Request {
            id: String,
            revision: String,
            summary: String,
        }
        let request: Request = serde_json::from_slice(body).map_err(|_| "Invalid resolution")?;
        if request.revision != revision
            || request.summary.trim().is_empty()
            || request.summary.len() > 4096
        {
            return Err(
                "Resolution needs the current revision and a summary of the verified fix".into(),
            );
        }
        let entry = self
            .notes
            .get_mut(&request.id)
            .ok_or("Comment has not been delivered to this server")?;
        entry.1 = Some(Resolution {
            snapshot: Comparison {
                id: snapshot.id.clone(),
                base: snapshot.base.clone(),
                target: snapshot.target.clone(),
                captured_at: snapshot.captured_at,
                revision: Some(revision.into()),
            },
            summary: request.summary,
        });
        Ok(json!({"resolved":true,"id":request.id}))
    }
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Report {
    version: u8,
    snapshot: Comparison,
    notes: Vec<Note>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Submission {
    id: String,
    report: Report,
}
pub(super) struct Delivery {
    id: String,
    payload: String,
    digest: String,
    notes: Vec<(String, String)>,
    reply: mpsc::SyncSender<Result<Value, String>>,
}
pub(super) type Sender = mpsc::SyncSender<Delivery>;
pub(super) type Seen = VecDeque<(String, String)>;

fn comparison_valid(value: &Comparison, base: &str, target: &str) -> bool {
    !value.id.is_empty()
        && value.id.len() <= 128
        && value.base == base
        && value.target == target
        && value
            .revision
            .as_ref()
            .is_none_or(|value| value.len() <= 128)
}
pub(super) fn submit(
    body: &[u8],
    base: &str,
    target: &str,
    sender: &Sender,
) -> Result<Value, String> {
    let submission: Submission =
        serde_json::from_slice(body).map_err(|_| "Invalid feedback report")?;
    let report = &submission.report;
    if submission.id.is_empty()
        || submission.id.len() > 128
        || !submission
            .id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        || report.version != 1
        || report.notes.is_empty()
        || report.notes.len() > 2000
        || !comparison_valid(&report.snapshot, base, target)
        || report.notes.iter().any(|note| {
            note.id.as_ref().is_some_and(|id| {
                id.is_empty()
                    || id.len() > 128
                    || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            }) || note.resolution.as_ref().is_some_and(|r| {
                !comparison_valid(&r.snapshot, base, target)
                    || r.summary.trim().is_empty()
                    || r.summary.len() > 4096
            }) || note.text.trim().is_empty()
                || note.text.len() > 64 * 1024
                || note.item.as_ref().is_some_and(|item| item.len() > 128)
                || note
                    .snapshot
                    .as_ref()
                    .is_some_and(|snapshot| !comparison_valid(snapshot, base, target))
                || (note.reference.is_none()
                    && note.item.as_ref().is_none_or(|item| item.is_empty()))
                || note.reference.as_ref().is_some_and(|reference| {
                    !super::path_ok(&reference.path)
                        || reference.path.len() > 4096
                        || reference.line == 0
                        || !["old", "new"].contains(&reference.side.as_str())
                        || reference
                            .quote
                            .as_ref()
                            .is_some_and(|quote| quote.len() > 64 * 1024)
                        || reference.id.as_ref().is_some_and(|id| id.len() > 128)
                        || reference
                            .label
                            .as_ref()
                            .is_some_and(|label| label.len() > 4096)
                })
        })
    {
        return Err("Invalid feedback identity, target, or comment".into());
    }
    let notes: Vec<_> = report.notes.iter().filter_map(|note| note.id.as_ref().map(|id| (id.clone(), format!("{:x}",Sha256::digest(serde_json::to_vec(&json!({"text":note.text,"reference":note.reference,"item":note.item,"snapshot":note.snapshot})).unwrap()))))).collect();
    let mut ids = std::collections::BTreeSet::new();
    if notes.iter().any(|(id, _)| !ids.insert(id)) {
        return Err("Duplicate comment ID".into());
    }
    let payload =
        json!({"type":"review.feedback","id":submission.id,"report":submission.report}).to_string();
    let digest = format!("{:x}", Sha256::digest(payload.as_bytes()));
    let (reply, receipt) = mpsc::sync_channel(1);
    sender
        .try_send(Delivery {
            id: submission.id,
            payload,
            digest,
            notes,
            reply,
        })
        .map_err(|_| "Feedback delivery is busy. Retry shortly.")?;
    receipt
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| "No delivery receipt. Retry the same submission.")?
}
pub(super) fn deliver(
    delivery: Delivery,
    seen: &mut Seen,
    registry: &mut Registry,
    writer: &mut impl Write,
) -> Result<(), String> {
    let result = if let Some((_, digest)) = seen.iter().find(|(id, _)| *id == delivery.id) {
        if digest == &delivery.digest {
            Ok(json!({"delivered":true,"duplicate":true,"id":delivery.id}))
        } else {
            Err("This submission ID was already used for different feedback".into())
        }
    } else {
        if delivery.notes.iter().any(|(id, digest)| {
            registry
                .notes
                .get(id)
                .is_some_and(|entry| &entry.0 != digest)
        }) || registry.notes.len()
            + delivery
                .notes
                .iter()
                .filter(|(id, _)| !registry.notes.contains_key(id))
                .count()
            > 2000
        {
            let _ = delivery.reply.send(Err(
                "Comment ID conflict or session comment limit exceeded".into(),
            ));
            return Ok(());
        }
        let result = writeln!(writer, "{}", delivery.payload)
            .and_then(|_| writer.flush())
            .map_err(|error| error.to_string());
        match result {
            Ok(()) => {
                for (id, digest) in delivery.notes {
                    registry.notes.entry(id).or_insert((digest, None));
                }
                seen.push_back((delivery.id.clone(), delivery.digest));
                if seen.len() > 128 {
                    seen.pop_front();
                }
                Ok(json!({"delivered":true,"duplicate":false,"id":delivery.id}))
            }
            Err(error) => {
                let _ = delivery.reply.send(Err(error.clone()));
                return Err(error);
            }
        }
    };
    let _ = delivery.reply.send(result);
    Ok(())
}
