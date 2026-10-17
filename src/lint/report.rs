use super::{markdown::Reference, paths};
use crate::language::Language;
use serde_json::{Value, json};
use std::{collections::BTreeSet, fmt::Write, path::Path};

struct Diagnostic {
    path: String,
    line: Option<usize>,
    column: Option<usize>,
    target: Option<String>,
    code: &'static str,
    reason: String,
}
impl Diagnostic {
    fn violation(&self) -> bool {
        matches!(
            self.code,
            "missing-target" | "missing-heading" | "forbidden-reference" | "invalid-target"
        )
    }
    fn section(&self) -> &'static str {
        if self.code == "forbidden-reference" {
            "文書の参照方向"
        } else {
            "参照の確認"
        }
    }
}
#[derive(Default)]
pub struct Report {
    language: Language,
    diagnostics: Vec<Diagnostic>,
    pub documents: usize,
    pub direction_documents: usize,
    pub internal_references: usize,
    pub external_references: usize,
    pub skipped_heading_checks: usize,
}
impl Report {
    fn section(&self, diagnostic: &Diagnostic) -> &'static str {
        if self.language == Language::Ja {
            diagnostic.section()
        } else if diagnostic.code == "forbidden-reference" {
            "Reference directions"
        } else {
            "Checking references"
        }
    }
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }
    pub fn add(
        &mut self,
        path: &Path,
        reference: Option<&Reference>,
        code: &'static str,
        reason: String,
    ) {
        self.diagnostics.push(Diagnostic {
            path: paths::display(path),
            line: reference.map(|r| r.line),
            column: reference.map(|r| r.column),
            target: reference.map(|r| r.target.clone()),
            code,
            reason,
        });
    }
    pub fn finish(mut self) -> Self {
        self.diagnostics.sort_by(|a, b| {
            (&a.path, a.line, a.column, a.code, &a.target)
                .cmp(&(&b.path, b.line, b.column, b.code, &b.target))
        });
        self
    }
    fn violations(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.violation()).count()
    }
    fn failures(&self) -> usize {
        self.diagnostics.len() - self.violations()
    }
    pub fn code(&self) -> u8 {
        if self.failures() > 0 {
            2
        } else if self.violations() > 0 {
            1
        } else {
            0
        }
    }
    fn status(&self) -> &'static str {
        match self.code() {
            0 => "pass",
            1 => "violations",
            _ => "incomplete",
        }
    }
    pub fn json(&self) -> String {
        let diagnostics: Vec<Value> = self.diagnostics.iter().map(|d| json!({
            "kind": if d.violation() { "violation" } else { "failure" }, "code": d.code,
            "path": d.path, "line": d.line, "column": d.column, "target": d.target, "reason": self.language.message(&d.reason),
            "guidance": { "command": self.language.select("sdev help documents --lang en", "sdev help documents --lang ja"), "section": self.section(d) }
        })).collect();
        let report = json!({"status": self.status(), "diagnostics": diagnostics,
            "summary": { "documents": self.documents, "direction_documents": self.direction_documents,
            "link_only_documents": self.documents - self.direction_documents,
            "internal_references": self.internal_references, "external_references": self.external_references,
            "skipped_heading_checks": self.skipped_heading_checks, "violations": self.violations(), "failures": self.failures() }});
        format!("{report}\n")
    }
    pub fn text(&self) -> String {
        let mut output = String::new();
        for d in &self.diagnostics {
            output.push_str(&escaped(&d.path));
            if let (Some(line), Some(column)) = (d.line, d.column) {
                write!(output, ":{line}:{column}").unwrap();
            }
            write!(
                output,
                " [{}] {}",
                d.code,
                escaped(&self.language.message(&d.reason))
            )
            .unwrap();
            if let Some(target) = &d.target {
                write!(output, " (target: {})", escaped(target)).unwrap();
            }
            output.push('\n');
        }
        let sections: BTreeSet<_> = self.diagnostics.iter().map(|d| self.section(d)).collect();
        if !sections.is_empty() {
            let sections = sections.into_iter().collect::<Vec<_>>().join(", ");
            let guide = self
                .language
                .select(
                    &format!("Guide: sdev help documents --lang en ({sections})"),
                    &format!("ガイド: sdev help documents --lang ja（{sections}）"),
                )
                .to_owned();
            writeln!(output, "{guide}").unwrap();
        }
        let label = match self.code() {
            0 => self
                .language
                .select("Check complete, no violations", "検査完了、違反なし"),
            1 => self
                .language
                .select("Check complete, violations found", "検査完了、違反あり"),
            _ => self.language.select("Check incomplete", "検査未完了"),
        };
        writeln!(output, "{}: {label}", self.status()).unwrap();
        let (documents, directional, links, internal, external, skipped) = (
            self.documents,
            self.direction_documents,
            self.documents - self.direction_documents,
            self.internal_references,
            self.external_references,
            self.skipped_heading_checks,
        );
        let summary = self.language.select(
            &format!("Documents {documents} (direction-checked {directional} / links-only {links}), internal references {internal}, external references {external}, heading checks skipped {skipped}"),
            &format!("文書 {documents}（方向も検査 {directional} / リンク切れのみ検査 {links}）、内部参照 {internal}、外部参照 {external}、見出し照合対象外 {skipped}"),
        ).to_owned();
        writeln!(output, "{summary}").unwrap();
        let (violations, failures) = (self.violations(), self.failures());
        let summary = self
            .language
            .select(
                &format!("Violations {violations}, failures {failures}"),
                &format!("違反 {violations}、処理失敗 {failures}"),
            )
            .to_owned();
        writeln!(output, "{summary}").unwrap();
        output
    }
}
fn escaped(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}
