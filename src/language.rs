use std::{fs, io, path::Path};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    En,
    Ja,
}
impl Language {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "en" => Ok(Self::En),
            "ja" => Ok(Self::Ja),
            _ => Err(format!("Unsupported language: {value}. Choose en or ja.")),
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ja => "ja",
        }
    }
    pub fn select<'a>(self, en: &'a str, ja: &'a str) -> &'a str {
        match self {
            Self::En => en,
            Self::Ja => ja,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectConfig {
    pub language: Language,
    pub translations: Vec<Language>,
}
impl ProjectConfig {
    pub fn validate(&self) -> Result<(), String> {
        let mut seen = vec![self.language];
        for language in &self.translations {
            if seen.contains(language) {
                return Err(format!("Duplicate project language: {}", language.code()));
            }
            seen.push(*language);
        }
        Ok(())
    }
    pub fn read(root: &Path) -> Result<Option<Self>, String> {
        let path = root.join("sdev.toml");
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("sdev.toml: {error}")),
            Ok(metadata) if !metadata.file_type().is_file() => {
                return Err("sdev.toml must be a regular file.".into());
            }
            _ => {}
        }
        let body = fs::read_to_string(path).map_err(|error| format!("sdev.toml: {error}"))?;
        let table = body
            .parse::<toml::Table>()
            .map_err(|error| format!("sdev.toml: {error}"))?;
        for key in table.keys() {
            if !["language", "translations"].contains(&key.as_str()) {
                return Err(format!("sdev.toml: unknown setting: {key}"));
            }
        }
        let language = table
            .get("language")
            .map(|value| {
                value
                    .as_str()
                    .ok_or_else(|| "sdev.toml: language must be a string.".to_owned())
                    .and_then(Language::parse)
            })
            .transpose()?
            .unwrap_or_default();
        let translations = table
            .get("translations")
            .map(|value| {
                value
                    .as_array()
                    .ok_or_else(|| {
                        "sdev.toml: translations must be an array of languages.".to_owned()
                    })?
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .ok_or_else(|| {
                                "sdev.toml: translations must contain strings.".to_owned()
                            })
                            .and_then(Language::parse)
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();
        let config = Self {
            language,
            translations,
        };
        config.validate()?;
        Ok(Some(config))
    }
    pub fn render(&self) -> String {
        format!(
            "language = {:?}\ntranslations = [{}]\n",
            self.language.code(),
            self.translations
                .iter()
                .map(|language| format!("{:?}", language.code()))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Language {
    pub fn message(self, text: &str) -> String {
        if self == Self::Ja {
            return text.to_owned();
        }
        let mut text = text.to_owned();
        text = text.replace(
            "ファイルを確認・読み取れません",
            "Cannot inspect or read file",
        );
        text = text.replace(
            "入口文書が通常のファイルではありません",
            "Entry document is not a regular file",
        );
        text = text.replace(
            "docsが通常のディレクトリではありません",
            "docs is not a regular directory",
        );
        text = text.replace(
            "検査対象のMarkdownがありません",
            "No Markdown documents to check",
        );
        text = text.replace(
            "文書のパスがUTF-8ではありません",
            "Document path is not UTF-8",
        );
        text = text.replace(
            "探索中のシンボリックリンク・再解析ポイントはたどりません",
            "Discovery does not follow symlinks or reparse points",
        );
        text = text.replace(
            "参照先の実体がプロジェクトのルート外です",
            "Resolved target is outside the project root",
        );
        text = text.replace("参照先が存在しません", "Target does not exist");
        text = text.replace(
            "参照先が通常のファイルではありません",
            "Target is not a regular file",
        );
        text = text.replace("文書がUTF-8ではありません", "Document is not UTF-8");
        text = text.replace(
            "参照方向の判定に必要な文書種別を特定できません",
            "Cannot classify the target for reference-direction checking",
        );
        text = text.replace(
            "参照先の見出しが存在しません",
            "Target heading does not exist",
        );
        text = text.replace("通常ファイルではありません", "Not a regular file");
        text = text.replace("パーセント表現が不正です", "Invalid percent encoding");
        text = text.replace(
            "復号したパスに区切りが含まれます",
            "Decoded path contains a separator",
        );
        text = text.replace(
            "参照先の符号化がUTF-8ではありません",
            "Encoded target is not UTF-8",
        );
        text = text.replace("参照先にNULが含まれます", "Target contains NUL");
        text = text.replace(
            "Windows固有のローカルパスは扱えません",
            "Windows-specific local paths are unsupported",
        );
        text = text.replace(
            "バックスラッシュを使うローカルパスは扱えません",
            "Backslashes in local paths are unsupported",
        );
        text = text.replace(
            "参照先がプロジェクトのルート外です",
            "Target is outside the project root",
        );
        text = text.replace("扱えないローカルパスです", "Unsupported local path");
        text = text.replace(
            "通常のファイル以外は更新できません。",
            "Only regular files can be updated.",
        );
        text = text.replace(
            "既存のsdev指示を",
            "Update the existing sdev instructions to match ",
        );
        text = text.replace(
            "の配置先に合わせてから再実行してください。",
            " before running init again.",
        );
        text = text.replace(
            "sdevの指示の区切りが壊れています。確認してから再実行してください。",
            "The sdev instruction markers are malformed. Check them before running init again.",
        );
        text = text.replace(
            "起動スクリプトには通常のファイルが必要です。",
            "The launcher must be a regular file.",
        );
        text = text.replace("既存の別ファイルは上書きできません。内容を確認し、移動・削除するか--pathで別の配置先を指定してください。", "Cannot overwrite another existing file. Review and move it, or choose another location with --path.");
        text = text.replace("保存できません", "Cannot save");
        text = text.replace(": 追加しました。", ": added.");
        text = text.replace(
            ": 既存の設定を保持しました。",
            ": preserved existing settings.",
        );
        text = text.replace("次に`", "Next, use `");
        text = text.replace(
            " help`からガイドを選んでください。",
            " help` to choose a guide.",
        );
        text = text.replace("起動スクリプトとAGENTS.md・.gitignore・sdev.tomlをコミットしてください。取得した本体は.sdev/に保存します。親ディレクトリ全体を除外している場合は、起動スクリプトをGit管理できるよう除外設定を確認してください。", "Commit the launcher, AGENTS.md, .gitignore, and sdev.toml. Downloaded binaries are cached in .sdev/. If a parent directory is ignored, adjust the ignore rules so the launcher can be tracked.");
        text = text.replace("--pathにはプロジェクト内の相対パスを指定してください。各名前には英数字・-・_・.を使えます。", "--path must be a relative project path using letters, digits, -, _, and . in each component.");
        text = text.replace(
            "親には通常のディレクトリが必要です。",
            "Parent components must be regular directories.",
        );
        text = text.replace(
            "Productから別文書のDesignへの参照は禁止です",
            "References from Product to another Design are forbidden",
        );
        text = text.replace(
            "Productから実装への参照は禁止です",
            "References from Product to implementation are forbidden",
        );
        text = text.replace(
            "Requirementsから別文書のDesignへの参照は禁止です",
            "References from Requirements to another Design are forbidden",
        );
        text = text.replace(
            "Requirementsから実装への参照は禁止です",
            "References from Requirements to implementation are forbidden",
        );
        text = text.replace(
            "Designから別文書のDesignへの参照は禁止です",
            "References from Design to another Design are forbidden",
        );
        text = text.replace(
            "Designから実装への参照は禁止です",
            "References from Design to implementation are forbidden",
        );
        text
    }
}
