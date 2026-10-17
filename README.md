# sdev

sdevは、人とAIが仕様を起点にプロダクトを開発するためのフレームワークです。
仕様からコードへ具体化し、コードから要件や設計、残された前提と判断理由をたどってレビューできる状態を目指します。
Product・Requirements・Designのガイドラインと既定テンプレートを提供し、対話・調査・執筆・実装の進行は利用するエージェントが担当します。

目的は[Product](docs/products/sdev/product.md)、GitHubで読む資料の入口は[文書と作業の入口](docs/README.md)に記述しています。

## プロジェクトへ導入する

プロジェクト内の`./bin/sdev`を入口にして、その場所からガイドを読めます。
macOS・Linux向けの`install.sh`はOS・CPUに合う公開済みReleaseを取得し、チェックサムとバージョンを確認してから配置します。
PATHやRust・Node.js・Pythonの設定は必要ありません。

プロジェクトのルートで、GitHubからインストーラーを取得して実行します。
バージョンを省略すると最新の安定Releaseを選びます。

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/azu/sdev/main/install.sh | sh -s -- --init
./bin/sdev help
```

`--init`は`./bin/sdev`へ起動スクリプトを生成し、AGENTS.mdへ参照指示を追加します。
起動スクリプト・AGENTS.md・.gitignoreをコミットすると、cloneした環境でも`./bin/sdev help`から使えます。
スクリプトは生成元のバージョンに固定し、本体がなければ対応するOS・CPUのReleaseを取得・検証して`.sdev/`へ保存します。
取得済みなら通信せずに起動します。`.sdev/`は.gitignoreへ追加し、起動スクリプトはGit管理します。
既存の本文とsdevの指示を保持し、再実行しても重複させません。
作業の種類を例示し、ガイドが増えても`./bin/sdev help`の一覧から選べる指示を置きます。
「作りたい」という目的の表明から、実装までの依頼とは解釈しません。
文書作成だけの依頼は提示で区切り、実装まで任された場合は要件・設計の要点を途中で示し、利用者の判断が必要になった場合に確認します。
対話では、使える質問ツールを優先し、目的や条件を仮定で補わずに、回答に応じて掘り下げます。
仕様ファイルや空の仕様ディレクトリは作りません。
配置済みの実行ファイルで初期設定だけ行う場合は`./bin/sdev init`を使います。
起動スクリプトの配置先は、次のように指定できます。AGENTS.mdも指定したパスを参照します。

```sh
sdev init --path ./scripts/sdev
./scripts/sdev help
```

既存の別ファイルや編集済みの起動スクリプトは上書きしません。
Windowsは手動配置した`./bin/sdev.exe`から`init`を実行する従来の手順を使います。
以下の例の`sdev`は、プロジェクト内へ導入した場合は`./bin/sdev`に読み替えてください。

チェックサムはファイルの一致を確認し、配布元の証明とは区別します。
`--verify`を付けると、GitHub CLIで配布元・ワークフロー・ソースと署名のコミットを検証し、検証できなければ実行・配置へ進みません。
インストーラー自身の確認方法と、通信が制限された環境での手順は[bootstrapと証明の検証](docs/engineering/guidelines/releasing.md#bootstrapと証明の検証)を参照してください。
起動スクリプトで本体を初めて取得するときも、`SDEV_VERIFY=1 ./bin/sdev help`で証明検証を指定できます。

## 作業名からガイドを読む

Rust製CLIにガイドとテンプレートを同梱し、実行時にRust・Node.js・Bunやネットワークを必要としません。
ファイル名を資料IDとして指定せず、作業や共通トピックから読みます。

```sh
sdev --help
sdev help product
sdev help requirements
sdev help design
sdev help documents
sdev help workflow
sdev help scenarios
sdev --version
```

ガイド内の関連資料も、対応するコマンドで次に読むものを案内します。
利用先のProduct・Requirements・Designは、そのプロジェクトの文書を参照します。

## 必要な場合に内蔵テンプレートを使う

エージェントは利用先の指示・規約と、対象文書用として指定されたテンプレートを先に確認します。
適用するテンプレートがなければ、sdevの内蔵の既定を読みます。
既存仕様の変更では、その文書の構造を尊重します。

```sh
sdev template
sdev template product
sdev template requirements
sdev template design
```

これらは内蔵テンプレートのMarkdown本文を表示する操作です。
利用先の自動探索や推測、テンプレートの生成・上書きは行いません。
任意のMarkdownや既存の仕様書を、そのままテンプレートとして扱うこともしません。

## 文書のリンクを検査する

プロジェクトのルートで実行します。ルートのREADME.md・AGENTS.mdと、docs以下のMarkdownについて、内部リンクのファイルと見出しの存在、標準配置に基づく文書の参照方向を検査します。

```sh
sdev lint
sdev lint --format json
sdev lint --help
```

既定は人向けのテキスト、JSONはAIや他の処理から読む形式です。
終了コードは検査成功が0、規則違反が1、読み取りなどの失敗による検査未完了が2です。
設定ファイルは不要で、外部URLへの通信や文書の書き換えは行いません。
検査の範囲は[Requirements](docs/products/sdev/features/document-lint/requirements.md)、処理と出力形式は[Design](docs/products/sdev/features/document-lint/design.md)に記述しています。

## プロジェクトから参照する

`sdev`が起動できる環境では、たとえば次のように指示できます。

```text
仕様の作成・変更・実装では、利用先の指示・規約と既存文書を先に確認してください。sdev helpで作業に合うガイドを選んで読み、対象文書用のテンプレートが指定されていなければsdev templateで内蔵の既定を参照してください。Product・Requirements・Designは作業するプロジェクトのものを使います。

「作りたい」だけで実装までの依頼と解釈せず、まず対話で目的と範囲を整理してください。文書作成だけの依頼は文書と要点の提示で区切り、実装まで任されている場合は要件・設計の要点を途中で示してください。目的・範囲・利用者の振る舞いについて未決の判断が必要なら、利用者へ確認してください。

目的や条件を仮定で補わず、使える質問ツールを優先して質問し、回答に応じて具体例や条件を掘り下げてください。
```

必要ならコマンドを実際の配置場所に置き換えてください。
旧版の`init`で追加したsdevの指示も再実行時には保持します。
依頼の範囲・レビュー・質問の案内がない場合は、上の例の該当する段落を既存の指示へ追記してください。
このリポジトリ自身の[AGENTS.md](AGENTS.md)では、作業開始時に`cargo build --locked --package sdev --bin sdev`でビルドし、`./target/debug/sdev`からガイドを読みます。
ガイド自体を変更する場合は元資料も確認し、変更後に再ビルドしてコマンド出力を確認します。

配置済みMarkdownをコマンドなしで参照する場合は、実際に読める配置先を使って次のように案内する方法もあります。

```text
仕様の作成・変更・実装を始める前に、<sdevの配置先>/docs/README.mdを読み、利用先の指示・規約と作業に対応するガイドラインに従ってください。
```

## 配布と更新

GitHub Releasesからバージョンと対象環境を選んで取得し、新しい実行ファイルへの差し替えでコマンドと資料を一緒に更新する方針です。
初期対象はmacOS arm64/x64、Linux arm64/x64、Windows x64です。
OS条件、入手・起動・更新、公開前の確認は[配布と更新](docs/engineering/guidelines/releasing.md)を参照してください。
公開はActionsのRelease sdevをmainから手動実行し、Cargo.tomlと同じバージョンを指定します。
全対象の検証が成功してからタグとReleaseを作成・公開します。ライセンス選択などの前提、再実行の扱いは配布ガイドで確認してください。

CLIと配布準備の仕様は[Requirements](docs/products/sdev/features/command-distribution/requirements.md)と[Design](docs/products/sdev/features/command-distribution/design.md)に記述しています。
必要なライセンス通知は配布物に別添します。sdev自身は[MIT License](LICENSE)で公開します。

## ソースからローカルへインストールする

リポジトリのルートで実行します。ビルドには固定されたRustとOSの開発用ツールが必要です。

```sh
cargo install --path . --locked
sdev --version
```

導入先を分ける場合は`--root ./target/local-install`を指定できます。
詳しい手順は[ローカル導入](docs/engineering/guidelines/releasing.md#ソースからローカルへ導入する)を参照してください。

## 開発する

環境準備と通常の検証は[Rust開発ガイド](docs/engineering/guidelines/rust-development.md)、バージョンと共通CIの方針は[共通設計](docs/engineering/design/rust-development.md)を参照します。
開発・配布準備はRustupとCargoを使い、開発用タスクを`cargo xtask`から実行します。
Cargo.tomlのバージョンと、rust-toolchain.tomlに固定したRustで実行ファイルと資料を一緒にビルドします。

```sh
cargo xtask check
cargo xtask build-release --target aarch64-apple-darwin
```

ターゲットは自分の環境に合わせて指定します。
`dist/`の成果物はレビュー用で、ビルドによって自動的に公開されることはありません。
文書は目的・要件・設計の変更に合わせて更新し、過去の内容はGitの履歴で参照します。

可視化、既存文書の移行、自動更新は製品CLIに含めていません。
