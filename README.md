# sdev

sdevは、人とAIが仕様を起点にオープンソースを開発するためのフレームワークです。
仕様からコードへ具体化し、コードから要件や設計、残された前提と判断理由をたどってレビューできる状態を目指します。
Product・Requirements・Designのガイドラインと既定テンプレートを提供し、対話・調査・執筆・実装の進行は利用するエージェントが担当します。

目的は[Product](docs/products/sdev/product.md)、GitHubで読む資料の入口は[文書と作業の入口](docs/README.md)に記述しています。

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

## プロジェクトから参照する

`sdev`が起動できる環境では、たとえば次のように指示できます。

```text
仕様の作成・変更・実装では、利用先の指示・規約と既存文書を先に確認してください。sdev helpで作業に合うガイドを選んで読み、対象文書用のテンプレートが指定されていなければsdev templateで内蔵の既定を参照してください。Product・Requirements・Designは作業するプロジェクトのものを使います。
```

必要ならコマンドを実際の配置場所に置き換えてください。
このリポジトリ自身を開発するときは、[AGENTS.md](AGENTS.md)から配置済みの資料を読む方法も使えます。

配置済みMarkdownをコマンドなしで参照する場合は、実際に読める配置先を使って次のように案内する方法もあります。

```text
仕様の作成・変更・実装を始める前に、<sdevの配置先>/docs/README.mdを読み、利用先の指示・規約と作業に対応するガイドラインに従ってください。
```

## 配布と更新

GitHub Releasesからバージョンと対象環境を選んで取得し、新しい実行ファイルへの差し替えでコマンドと資料を一緒に更新する方針です。
初期対象はmacOS arm64/x64、Linux arm64/x64、Windows x64です。
OS条件、入手・起動・更新、公開前の確認は[配布と更新](docs/engineering/guidelines/releasing.md)を参照してください。
公開リリースはこの変更では実行しておらず、未確認のダウンロードURLは案内していません。

CLIと配布準備の仕様は[Requirements](docs/products/sdev/features/command-distribution/requirements.md)と[Design](docs/products/sdev/features/command-distribution/design.md)に記述しています。
必要なライセンス通知は配布物に別添します。sdev自身のライセンスは公開前の未決事項です。

## 開発する

環境準備と通常の検証は[Rust開発ガイド](docs/engineering/guidelines/rust-development.md)、バージョンと共通CIの方針は[共通設計](docs/engineering/design/rust-development.md)を参照します。
開発時にはRustupとPython 3を使います。
Cargo.tomlのバージョンと、rust-toolchain.tomlに固定したRustで実行ファイルと資料を一緒にビルドします。

```sh
python3 scripts/check.py
python3 scripts/build-release.py --target aarch64-apple-darwin
```

ターゲットは自分の環境に合わせて指定します。
`dist/`の成果物はレビュー用で、ビルドによって自動的に公開されることはありません。
文書は目的・要件・設計の変更に合わせて更新し、過去の内容はGitの履歴で参照します。

lintや可視化、既存文書の移行、自動更新は製品CLIに含めていません。
配布方式の比較用コードは`experiments/command-distribution/`に残し、製品の機能と区別します。
