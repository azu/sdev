# 配布と更新

sdevはRust製の単体実行ファイルとして配布します。
ガイドライン・既定テンプレートはコマンドと同じバージョンで同梱し、実行時にRust・Node.js・Bunやネットワークを必要としません。
この文書は利用手順と公開準備を示し、GitHub Releasesで公開済みであることを意味しません。

## 配布対象と選択

公開後はGitHubのsdevリポジトリでReleasesを開き、利用したいバージョンと対応する環境を選びます。
実際に公開して確認するまでは、ダウンロードURLを案内しません。

| 環境 | 実行ファイル名 | 利用条件 |
| --- | --- | --- |
| macOS arm64 | `sdev-v<バージョン>-macos-arm64` | macOS 11以降 |
| macOS x64 | `sdev-v<バージョン>-macos-x64` | macOS 10.12以降 |
| Linux arm64 | `sdev-v<バージョン>-linux-arm64` | glibc 2.35以降、kernel 4.1以降 |
| Linux x64 | `sdev-v<バージョン>-linux-x64` | glibc 2.35以降、kernel 3.2以降 |
| Windows x64 | `sdev-v<バージョン>-windows-x64.exe` | Windows 10以降 |

これ以外のCPU、32ビットx86、Windows arm64、musl環境は初回対象に含めません。
OS条件は[配布のDesign](../../products/sdev/features/command-distribution/design.md)で定めたビルド上の条件です。
最低バージョンの全実機で確認したという意味ではなく、確認した環境は公開時の説明に記載します。

## 入手して使う

自分の環境と選んだバージョンに対応する実行ファイルを取得し、任意の配置場所へ保存します。
別に配布されるSHA-256の値も確認できる場合は、取得した実行ファイルの値と照合します。
対応するファイルがない場合は、別の環境向けのものを使えると推測せず、公開されている対象一覧を確認します。
取得に失敗した場合は、Releasesで選んだバージョンとOS・CPUを確認し、入手できたものとして次の作業へ進めません。

macOS・Linuxでは取得したファイルを`sdev`へ名前を変え、実行できるようにして起動します。

```sh
chmod +x ./sdev
./sdev --version
./sdev help
```

Windowsではファイル名を`sdev.exe`へ変え、PowerShellから起動します。

```powershell
.\sdev.exe --version
.\sdev.exe help
```

OSによる起動制限と署名の状態は、公開者が配布前に確認します。
開発時のローカルビルドで起動できたことだけでは、ブラウザーで取得したファイルの起動を確認したことにはなりません。

## エージェントからガイドとテンプレートを読む

プロジェクトの指示には、実際に起動できるコマンド名・配置場所を指定します。
エージェントは利用先の指示・規約・既存文書を先に確認し、作業に合うガイドを選びます。

```sh
sdev help
sdev help product
sdev help requirements
sdev help design
sdev help documents
sdev help workflow
sdev help scenarios
```

内部のファイル名を指定する必要はなく、ガイド内の関連資料も次のコマンドで案内します。
利用先のProduct・Requirements・Designを先に読む意味を保ち、sdev自身の仕様を代わりにはしません。

対象文書用として指定されたテンプレートがあればそれを優先し、ない場合に内蔵の既定を読みます。

```sh
sdev template
sdev template product
sdev template requirements
sdev template design
```

コマンドは本文を表示するだけで、利用先を自動探索せず、生成・上書きもしません。
任意のMarkdownや既存仕様をテンプレートと混同せず、既存仕様の変更ではその構造を尊重します。
人がGitHub上で資料を読む場合は、使っているバージョンのタグに対応する文書を選びます。

## 更新する

現在の`sdev --version`を確認してから、新しいバージョンと自分の環境に合う実行ファイルを取得します。
新しいファイルを別の名前・場所へ保存し、そのファイルのバージョンが選んだものと一致することを確認してから、使っている実行ファイルと差し替えます。
差し替え後も`sdev --version`を確認します。
更新のために資料を別途ダウンロードする操作は不要です。

取得できなかった場合は旧ファイルを維持し、配布先とOS・CPU・バージョンを確認します。
新しいファイルが起動できない場合も、旧ファイルで使い続けられるよう差し替え前に確認します。
自動更新や複数バージョンを切り替える専用機能は提供しません。

## 開発者が配布物を準備する

開発環境と通常の検証は[Rust開発ガイド](rust-development.md)、Rustの選択と共通CIの構成は[共通設計](../design/rust-development.md)に従います。
配布準備はCargoからRust製の開発用タスクを実行し、sdevのバージョンは`Cargo.toml`から読み取ります。
次はmacOS arm64向けに準備する例です。別の環境ではDesignのターゲット名を指定します。

```sh
cargo xtask build-release --target aarch64-apple-darwin
cargo xtask check --target aarch64-apple-darwin
cargo xtask verify-release dist/sdev-v0.1.0-macos-arm64
```

ファイル名のバージョンはCargo.tomlの値に合わせます。
`dist/`には実行ファイル、SHA-256、ローカルの確認用メタデータを作ります。
必要な通知は別のTHIRD-PARTY-NOTICES.htmlへコピーし、sdev自身のLICENSEがあればそれも添付します。
実行に必要なものは実行ファイルだけで、通知ファイルは実行時に読みません。
生成は公開操作ではありません。

GitHub ActionsのBuild and check sdevは、5組をそれぞれのOSでビルドし、振る舞い・テンプレートの元の本文との一致とガイドの参照先・バージョン・実行環境を空にした起動・依存するシステムライブラリを確認して、レビュー用成果物を保存します。
Linuxではglibcの要求、Windowsでは追加のCランタイムDLLへの依存も確認します。
成果物の保存までとし、タグ・Releaseの作成や公開は自動では行いません。

## 公開前に確認する

sdev自身のライセンスを決め、ルートのLICENSEへ配置します。
Rust標準ライブラリの通知はRust 1.96.0のものを配布物へ別添しており、Rustを更新するときは通知も更新します。

バージョンと同梱資料を確定し、CIが5組で成功したコミットから配布物を用意します。
macOS・Windowsの署名・公証や取得後の起動制限、対応環境での動作を確認します。
このリポジトリの設定では署名・公証はまだ自動化していません。

公開するバージョンのタグとGitHub Releaseを用意し、5つの実行ファイル、必要なチェックサム、THIRD-PARTY-NOTICES.htmlとsdev自身のLICENSEを添付して内容を確認してから公開します。
実行に必要なものは実行ファイル単体で、通知は別添します。
CI成果物を取り出す際のアーカイブをそのまま利用者向けインストール形式にはしません。
公開して実際の取得先と内容を確認した後に、README等へ実在するダウンロード先を追加します。
