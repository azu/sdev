# sdev

[English](README.md)

sdevは、人とAIが仕様をもとに開発を進めるためのフレームワークです。
目的や要件、設計を文書に残し、実装やレビューで判断の理由をたどれるようにします。
文書を書くためのガイドとテンプレートは、CLIから参照できます。

仕様は次の3種類に分けます。
利用先に既存の文書や規約がある場合は、それらを使って進めます。

| 文書 | 整理すること |
| --- | --- |
| Product | 誰のどんな課題を解決するか、何を提供するか |
| Requirements | 機能の振る舞いと、受け入れ条件 |
| Design | 要件を実現する方法と、その方法を選んだ理由 |

## 導入する

### macOS・Linux

使いたいプロジェクトのルートで、次のコマンドを実行します。
OS・CPUに合う最新の安定版を取得し、初期設定します。
RustやNode.jsの準備、PATHの変更は必要ありません。

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/azu/sdev/main/install.sh | sh -s -- --init
./bin/sdev help
```

初期設定では、次のファイルを用意します。
既存のAGENTS.mdにある指示は保持します。

| ファイル | 役割 |
| --- | --- |
| `bin/sdev` | 使用するバージョンを固定した起動スクリプト |
| `AGENTS.md` | AIがsdevのガイドを読むための指示 |
| `sdev.toml` | プロジェクトの既定言語と翻訳言語 |
| `.gitignore` | 取得した本体の保存先`.sdev/`をGit管理から除外する設定 |

この4ファイルをコミットすると、cloneした環境でも`./bin/sdev help`から使えます。
本体がなければ起動時に取得し、取得済みなら通信せずに実行します。
仕様ファイルや空の仕様ディレクトリは作りません。

取得時にはチェックサムとバージョンを照合します。
GitHubによる配布元の証明も確認したい場合は、インストーラーへ`--verify`を付けてください。
詳しい手順は[bootstrapと証明の検証](docs/engineering/guidelines/releasing.ja.md#bootstrapと証明の検証)を参照してください。

### 起動スクリプトの配置先を変える

初期設定の`--path`で、`bin/sdev`以外の場所を選べます。
本体を別の場所へ取得してから初期設定すると、AGENTS.mdも指定先を参照します。

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/azu/sdev/main/install.sh | sh -s -- --install-dir ./.sdev/setup
./.sdev/setup/sdev init --path ./scripts/sdev
./scripts/sdev help
```

既存の別ファイルや編集済みの起動スクリプトは上書きしません。
初期設定済みの配置先変更やバージョン更新は、[配布と更新](docs/engineering/guidelines/releasing.ja.md)を参照してください。

### Windows

[GitHub Releases](https://github.com/azu/sdev/releases)からWindows x64向けの実行ファイルを取得し、`bin/sdev.exe`として配置します。
PowerShellで初期設定し、ガイドを表示します。

```powershell
.\bin\sdev.exe init
.\bin\sdev.exe help
```

Windowsでは起動スクリプトを生成せず、配置した実行ファイルを使います。
対応するOSの条件と手動の導入手順は、[配布対象と選択](docs/engineering/guidelines/releasing.ja.md#配布対象と選択)を参照してください。

## プロジェクトの言語を選ぶ

言語設定がない場合は英語を使います。
新規プロジェクトでは`sdev init --lang en --translations ja`で、英語を既定、日本語を翻訳言語としてsdev.tomlへ保存できます。
初期設定済みのプロジェクトでは、設定ファイルを編集して変更します。

```toml
language = "en"
translations = ["ja"]
```

既定言語は`.md`、別言語版は`requirements.ja.md`などの名前で管理します。
両方を同じ仕様として、同じ変更で更新します。
一度だけ日本語で読む場合は`./bin/sdev help design --lang ja`と指定できます。
`--lang`を省略すると、ガイドとテンプレートもプロジェクトの既定言語で表示します。

## ガイドを読む

`help`に作業名を渡すと、その作業に必要なガイドを読めます。
まずは一覧から選んでください。

```sh
./bin/sdev help
```

| コマンド | 読む内容 |
| --- | --- |
| `./bin/sdev help product` | プロダクトの目的・利用者・価値 |
| `./bin/sdev help requirements` | 機能の振る舞い・受け入れ条件 |
| `./bin/sdev help design` | 要件を実現する方法 |
| `./bin/sdev help documents` | 文書の責務・配置・更新 |
| `./bin/sdev help workflow` | 対話・調査・実装・検証の進め方 |
| `./bin/sdev help scenarios` | 受け入れ条件のシナリオ |

AIはこれらのガイドに従って、利用者と目的や作業の範囲を整理します。
「作りたい」という相談だけで実装を始めず、文書の作成だけを頼まれた場合は、その提示で区切ります。
詳しい進め方は[エージェントの作業原則](docs/guidelines/agent.ja.md)にまとめています。

### テンプレートを使う

利用先で指定されたテンプレートがあれば、それを使います。
指定がない場合は、内蔵のテンプレートを一覧から選べます。

```sh
./bin/sdev template
./bin/sdev template requirements
```

コマンドはテンプレートの本文を表示します。
既存の仕様を変更するときは、その文書の構造を尊重してください。
ガイドとテンプレートは本体に同梱しているため、個別に取得する必要はありません。

## 文書のリンクを検査する

プロジェクトのルートで`lint`を実行すると、文書のリンク切れと参照方向を検査できます。
対象はREADME.md、AGENTS.md、各言語版、docs以下のMarkdownです。

```sh
./bin/sdev lint
./bin/sdev lint --format json
```

通常はテキストで結果を表示し、他のツールから使う場合はJSONで取得できます。
検査の範囲や終了コードは[文書リンク検査のRequirements](docs/products/sdev/features/document-lint/requirements.ja.md)を参照してください。

## sdevを開発する

Rustの環境を用意し、開発用のツールと検査コマンドは[mise](https://mise.jdx.dev/getting-started.html)でそろえます。
初回はリポジトリのルートで環境と依存を導入します。

```sh
mise trust
mise install --locked
```

英語のREADMEは[slopless](https://github.com/berelevant-ai/slopless)、日本語のREADMEはtextlintの技術文書向けプリセットで検査します。
postinstallで[hk](https://hk.jdx.dev/)のhookも登録し、コミット前にREADMEとRustの変更を検査します。

```sh
mise run md:lint
mise run md:fix     # 自動修正した後は差分を確認
mise run rs:check   # Rustの整形・静的検査・テスト
```

環境の導入と検証の手順は[Rust開発ガイド](docs/engineering/guidelines/rust-development.ja.md)にまとめています。
配布物の作成やリリースの手順は、[配布と更新](docs/engineering/guidelines/releasing.ja.md)を参照してください。

目的や仕様を確認する場合は、[sdevのProduct](docs/products/sdev/product.ja.md)と[文書と作業の入口](docs/README.ja.md)からたどれます。
sdevは[MIT License](LICENSE)で公開しています。
