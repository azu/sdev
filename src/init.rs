use std::{
    fs::{self, OpenOptions, Permissions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const BEGIN: &str = "<!-- sdev:begin -->";
const END: &str = "<!-- sdev:end -->";

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
        self.append_with_newline(addition, self.newline())
    }
    fn append_with_newline(&self, addition: &str, newline: &str) -> String {
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

fn instructions(
    document: &Document,
    command: &str,
    language: crate::language::Language,
) -> Result<String, String> {
    let starts: Vec<_> = document.body.match_indices(BEGIN).collect();
    let ends: Vec<_> = document.body.match_indices(END).collect();
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => {}
        ([(start, _)], [(end, _)]) if start < end => {
            if !document.body[*start..*end].contains(&format!("`{command} help`")) {
                return Err(format!(
                    "AGENTS.md: 既存のsdev指示を`{command} help`の配置先に合わせてから再実行してください。"
                ));
            }
            return Ok(document.body.clone());
        }
        _ => {
            return Err(
                "AGENTS.md: sdevの指示の区切りが壊れています。確認してから再実行してください。"
                    .into(),
            );
        }
    }
    if language == crate::language::Language::En {
        return Ok(document.append(&format!(
            "{BEGIN}\n## Specifications-first development\n\nBefore creating or changing specifications, design, implementation, or verification, run `{command} help` and choose the guide for the task.\nTasks include defining product purpose, specifying feature requirements, designing an implementation, and updating documents.\n\nTreat an idea such as 'I want to build this' as a starting point for discussing purpose and scope, rather than permission to implement.\nWhen only a document is requested, present the document and its key points, then stop. When implementation is requested, present requirement and design decisions along the way.\nAsk the user about unresolved decisions affecting purpose, scope, or observable behavior.\nAsk instead of inventing purposes or constraints; prefer available question tools and follow up according to the answers.\nRead `{command} help workflow` for the full workflow.\n\nPrioritize the consuming project's instructions, conventions, and existing specifications.\nIf no template is designated for a new document, choose one from `{command} template`. Respect an existing document's structure when editing it.\n\nUse this project's Product, Requirements, and Design. Without another placement rule, put specifications under `docs/products/`.\nUse the default language and translations configured in sdev.toml. Default-language documents use `.md`; other language versions use `.<language>.md`.\nKeep language versions equivalent and update them in the same change.\n{END}\n"
        )));
    }
    Ok(document.append(&format!(
        "{BEGIN}\n## 仕様を起点にした開発\n\n仕様の作成・変更、設計、実装、検証では、まず`{command} help`を実行し、表示される一覧から作業に対応するガイドを選んで読んでください。\nプロダクトの目的整理、機能の要件定義、実現方法の設計、文書の更新などが対象です。\n\n「作りたい」だけで実装までの依頼と解釈せず、まず対話で目的と範囲を整理してください。\n文書作成だけの依頼は文書と要点の提示で区切り、実装まで任されている場合は要件・設計の要点を途中で示してください。\n目的・範囲・利用者の振る舞いについて未決の判断が必要なら、利用者へ確認してください。\n目的や条件を仮定で補わず、使える質問ツールを優先して質問し、回答に応じて具体例や条件を掘り下げてください。\n詳しい進め方は`{command} help workflow`を参照してください。\n\n利用先の指示・規約と既存の仕様を優先してください。\n新規文書に適用するテンプレートの指定がなければ、`{command} template`の一覧から対応するものを選んでください。\n既存文書の変更では、その構造を尊重してください。\n\n仕様は、このプロジェクトのProduct・Requirements・Designを使います。\n配置の指定がなければ、`docs/products/`以下に作成してください。\nsdev.tomlの既定言語と翻訳言語に従い、既定言語は`.md`、別言語版は`.<言語>.md`を使ってください。\n各言語版は同じ内容として、同じ変更で更新してください。\n{END}\n"
    )))
}
fn ignores(document: &Document, command: &str) -> String {
    let newline = document.newline();
    #[cfg(unix)]
    let document = Document {
        path: document.path.clone(),
        body: document
            .body
            .split_inclusive('\n')
            .filter(|line| {
                let pattern = line.trim_end_matches(['\r', '\n']);
                ![
                    command,
                    command.strip_prefix('.').unwrap(),
                    command.strip_prefix("./").unwrap(),
                ]
                .contains(&pattern)
            })
            .collect(),
        permissions: document.permissions.clone(),
    };
    #[cfg(windows)]
    let _ = command;
    let mut addition = String::new();
    #[cfg(unix)]
    let patterns = ["/.sdev/"].as_slice();
    #[cfg(windows)]
    let patterns = ["/bin/sdev", "/bin/sdev.exe"].as_slice();
    for &pattern in patterns {
        if !document.body.lines().any(|line| line == pattern) {
            addition.push_str(pattern);
            addition.push('\n');
        }
    }
    if addition.is_empty() {
        document.body.clone()
    } else {
        document.append_with_newline(&addition, newline)
    }
}

fn prepare_launcher(
    root: &Path,
    path: &str,
    documents: &mut Vec<Document>,
    bodies: &mut Vec<String>,
) -> Result<(), String> {
    #[cfg(windows)]
    let _ = (root, path, documents, bodies);
    #[cfg(unix)]
    {
        let script = crate::launcher::render(path);
        crate::launcher::check_parents(root, path)?;
        let target = root.join(path);
        let permissions = match fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.file_type().is_file() => Some(metadata.permissions()),
            Ok(_) => {
                return Err(format!(
                    "{path}: 起動スクリプトには通常のファイルが必要です。"
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(format!("{path}: {error}")),
        };
        let bytes = if permissions.is_some() {
            fs::read(&target).map_err(|error| format!("{path}: {error}"))?
        } else {
            Vec::new()
        };
        let body = if bytes == script.as_bytes() {
            script.clone()
        } else if permissions.is_some() {
            let executable = std::env::current_exe()
                .and_then(fs::read)
                .map_err(|error| format!("sdev: {error}"))?;
            if bytes != executable {
                return Err(format!(
                    "{path}: 既存の別ファイルは上書きできません。内容を確認し、移動・削除するか--pathで別の配置先を指定してください。"
                ));
            }
            String::new()
        } else {
            String::new()
        };
        use std::os::unix::fs::PermissionsExt;
        let permissions = Some(Permissions::from_mode(
            permissions.map_or(0o755, |p| p.mode() | 0o111),
        ));
        documents.push(Document {
            path: target.clone(),
            body,
            permissions,
        });
        bodies.push(script);
        fs::create_dir_all(target.parent().unwrap()).map_err(|error| format!("{path}: {error}"))?;
    }
    Ok(())
}

pub fn initialize(
    root: &Path,
    path: &str,
    config: &crate::language::ProjectConfig,
) -> Result<String, String> {
    #[cfg(unix)]
    let command = format!("./{path}");
    #[cfg(windows)]
    let command = {
        let _ = path;
        "./bin/sdev.exe".to_owned()
    };
    let agents = Document::read(root, "AGENTS.md")?;
    let gitignore = Document::read(root, ".gitignore")?;
    let settings = Document::read(root, "sdev.toml")?;
    let settings_body = if settings.permissions.is_some() {
        settings.body.clone()
    } else {
        config.render()
    };
    let mut bodies = vec![
        instructions(&agents, &command, config.language)?,
        ignores(&gitignore, &command),
        settings_body,
    ];
    let mut documents = vec![agents, gitignore, settings];
    prepare_launcher(root, path, &mut documents, &mut bodies)?;
    // Validate all inputs and stage all writes before changing any file.
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
        let name = document.path.strip_prefix(root).unwrap().display();
        if let Some(staged) = staged {
            fs::rename(&staged.temporary, &staged.destination)
                .map_err(|error| format!("{name}: 保存できません: {error}"))?;
            result.push_str(&format!("{name}: 追加しました。\n"));
        } else {
            result.push_str(&format!("{name}: 既存の設定を保持しました。\n"));
        }
    }
    result.push_str(&format!(
        "次に`{command} help`からガイドを選んでください。\n"
    ));
    #[cfg(unix)]
    result.push_str("起動スクリプトとAGENTS.md・.gitignore・sdev.tomlをコミットしてください。取得した本体は.sdev/に保存します。親ディレクトリ全体を除外している場合は、起動スクリプトをGit管理できるよう除外設定を確認してください。\n");
    Ok(result)
}
