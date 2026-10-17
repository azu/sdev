use std::{
    fs::{self, OpenOptions, Permissions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const BEGIN: &str = "<!-- sdev:begin -->";
const END: &str = "<!-- sdev:end -->";
#[cfg(not(windows))]
const COMMAND: &str = "./bin/sdev";
#[cfg(windows)]
const COMMAND: &str = "./bin/sdev.exe";

struct Document {
    path: PathBuf,
    body: String,
    permissions: Option<Permissions>,
}
impl Document {
    fn read(root: &Path, name: &str) -> Result<Self, String> {
        let path = root.join(name);
        let permissions = match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_file() => Some(metadata.permissions()),
            Ok(_) => return Err(format!("{name}: 通常のファイル以外は更新できません。")),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(format!("{name}: {error}")),
        };
        let body = if permissions.is_some() {
            fs::read_to_string(&path).map_err(|error| format!("{name}: {error}"))?
        } else {
            String::new()
        };
        Ok(Self {
            path,
            body,
            permissions,
        })
    }
    fn newline(&self) -> &'static str {
        if self.body.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        }
    }
    fn append(&self, addition: &str) -> String {
        let newline = self.newline();
        let mut body = self.body.clone();
        if !body.is_empty() {
            if !body.ends_with('\n') {
                body.push_str(newline);
            }
            body.push_str(newline);
        }
        body.push_str(&addition.replace('\n', newline));
        body
    }
    fn stage(&self, body: &str) -> io::Result<Option<Staged>> {
        if body == self.body {
            return Ok(None);
        }
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let (mut file, temporary) = loop {
            let temporary = self.path.with_file_name(format!(
                ".sdev-init-{}-{}.tmp",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => break (file, temporary),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        };
        let staged = Staged {
            temporary,
            destination: self.path.clone(),
        };
        file.write_all(body.as_bytes())?;
        if let Some(permissions) = &self.permissions {
            file.set_permissions(permissions.clone())?;
        }
        file.sync_all()?;
        Ok(Some(staged))
    }
}
struct Staged {
    temporary: PathBuf,
    destination: PathBuf,
}
impl Drop for Staged {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.temporary);
    }
}

fn instructions(document: &Document) -> Result<String, String> {
    let starts: Vec<_> = document.body.match_indices(BEGIN).collect();
    let ends: Vec<_> = document.body.match_indices(END).collect();
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => {}
        ([(start, _)], [(end, _)]) if start < end => return Ok(document.body.clone()),
        _ => {
            return Err(
                "AGENTS.md: sdevの指示の区切りが壊れています。確認してから再実行してください。"
                    .into(),
            );
        }
    }
    Ok(document.append(&format!(
        "{BEGIN}\n## 仕様を起点にした開発\n\n仕様の作成・変更、設計、実装、検証では、まず`{COMMAND} help`を実行し、表示される一覧から作業に対応するガイドを選んで読んでください。\nプロダクトの目的整理、機能の要件定義、実現方法の設計、文書の更新などが対象です。\n\n「作りたい」だけで実装までの依頼と解釈せず、まず対話で目的と範囲を整理してください。\n文書作成だけの依頼は文書と要点の提示で区切り、実装まで任されている場合は要件・設計の要点を途中で示してください。\n目的・範囲・利用者の振る舞いについて未決の判断が必要なら、利用者へ確認してください。\n目的や条件を仮定で補わず、使える質問ツールを優先して質問し、回答に応じて具体例や条件を掘り下げてください。\n詳しい進め方は`{COMMAND} help workflow`を参照してください。\n\n利用先の指示・規約と既存の仕様を優先してください。\n新規文書に適用するテンプレートの指定がなければ、`{COMMAND} template`の一覧から対応するものを選んでください。\n既存文書の変更では、その構造を尊重してください。\n\n仕様は、このプロジェクトのProduct・Requirements・Designを使います。\n配置の指定がなければ、`docs/products/`以下に作成してください。\n{END}\n"
    )))
}
fn ignores(document: &Document) -> String {
    let mut addition = String::new();
    for pattern in ["/bin/sdev", "/bin/sdev.exe"] {
        if !document.body.lines().any(|line| line == pattern) {
            addition.push_str(pattern);
            addition.push('\n');
        }
    }
    if addition.is_empty() {
        document.body.clone()
    } else {
        document.append(&addition)
    }
}

pub fn initialize(root: &Path) -> Result<String, String> {
    let agents = Document::read(root, "AGENTS.md")?;
    let gitignore = Document::read(root, ".gitignore")?;
    let bodies = [instructions(&agents)?, ignores(&gitignore)];
    let documents = [agents, gitignore];
    // Validate both inputs and stage both writes before changing either file.
    let staged = documents
        .iter()
        .zip(&bodies)
        .map(|(document, body)| {
            document
                .stage(body)
                .map_err(|error| format!("{}: {error}", document.path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut result = String::new();
    for (document, staged) in documents.iter().zip(staged) {
        let name = document.path.file_name().unwrap().to_string_lossy();
        if let Some(staged) = staged {
            fs::rename(&staged.temporary, &staged.destination)
                .map_err(|error| format!("{name}: 保存できません: {error}"))?;
            result.push_str(&format!("{name}: 追加しました。\n"));
        } else {
            result.push_str(&format!("{name}: 既存の設定を保持しました。\n"));
        }
    }
    result.push_str(&format!(
        "次に`{COMMAND} help`からガイドを選んでください。\n"
    ));
    Ok(result)
}
