# 配布と更新

sdevはRust製の単体実行ファイルとして配布します。
ガイドライン・既定テンプレートはコマンドと同じバージョンで同梱し、実行時にRust・Node.js・Bunやネットワークを必要としません。
この文書は利用手順と手動リリースの操作を示します。ワークフローの配置やビルド成功は、Releaseの公開完了を意味しません。

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
Rustの通知は別のTHIRD-PARTY-NOTICES.htmlへコピーし、組み込む通常の依存の通知はTHIRD-PARTY-CRATES.htmlへまとめます。
通知の元資料がない場合は配布準備を失敗にします。
sdev自身のLICENSEがあればそれも添付します。
実行に必要なものは実行ファイルだけで、通知ファイルは実行時に読みません。
生成は公開操作ではありません。

GitHub ActionsのBuild and check sdevは、5組をそれぞれのOSでビルドし、振る舞い・文書の自己lint・テンプレートの元の本文との一致とガイドの参照先・バージョン・実行環境を空にした起動・依存するシステムライブラリを確認して、レビュー用成果物を保存します。
Linuxではglibcの要求、Windowsでは追加のCランタイムDLLへの依存も確認します。
通常のBuild and check sdevは成果物の保存までです。公開は次の手動ワークフローから開始します。

## 公開前に確認する

sdev自身のライセンスを決め、ルートのLICENSEへ配置します。
Rust標準ライブラリの通知はRust 1.96.0のものを配布物へ別添しており、Rustを更新するときは通知も更新します。

バージョンと同梱資料を確定し、CIが5組で成功したコミットから配布物を用意します。
macOS・Windowsの署名・公証や取得後の起動制限、対応環境での動作を確認します。
このリポジトリの設定では署名・公証はまだ自動化していません。

## 手動でリリースする

先にCargo.tomlの製品バージョンを更新し、CargoでCargo.lockを同期して通常のレビューでmainへ反映します。
プロジェクト自身のLICENSEも選択してmainへ配置します。LICENSEがない、空である、指定バージョンとCargoのバージョンが異なる場合はリリースを開始できません。
初回の手動リリースは`0.1.0`のような`X.Y.Z`形式の安定バージョンだけを受け付けます。先頭のv、事前公開やビルド識別子は付けません。

GitHubのActionsでRelease sdevを開き、Run workflowからmainを選び、versionにCargo.tomlと同じ値を指定します。
GitHub CLIから行う場合も、同じ手動実行を使います。次のバージョンは公開する値へ置き換えてください。

```sh
gh workflow run release.yml --ref main -f version=0.1.0
```

起動時のコミットを固定し、全対象のビルド・共通チェック・配布物検証を同じ実行で行います。
全件が成功すると、公開ジョブが対象・コミット・バージョン・チェックサム・通知・LICENSEを照合し、タグと下書きReleaseを作成します。
実行ファイルとSHA-256、THIRD-PARTY-NOTICES.html、THIRD-PARTY-CRATES.html、sdev自身のLICENSEを添付し、GitHub上の全添付の名前・状態・サイズ・ハッシュも確認してから公開します。
mainが開始時から進んでいれば公開しません。進行中の公開は新しい手動実行で取り消しません。
タグのpushだけで公開するワークフローや、実行時にバージョン更新のコミットを作る処理はありません。

### 失敗した場合と再実行

ビルド・検証の失敗では公開ジョブへ進まず、タグとReleaseを作成しません。
同じコミットがmainの先頭である間は、失敗した元の実行からRe-run failed jobsまたはRe-run all jobsを選べます。
成果物は実行IDと対象で区別し、成功済みの対象と再検証した対象を、同じソースか確認してそろえます。成果物の保存期限を過ぎた場合は全ジョブを再実行します。

添付の途中で失敗した下書きは残します。同じ元の実行の再実行だけが同名添付を置き換え、全件を再照合して公開します。
新しい実行から同じバージョンを指定すると、残った下書きが別実行のものであるため停止します。
別コミットのタグ、注釈付きタグ、別実行の下書き、予期しない添付は自動で移動・削除しません。
mainが進んだ場合や他の実行の下書きがある場合は、保守者がタグと下書きの状態を確認し、必要な整理を行ってから現在のmainで新しく開始します。

公開済みReleaseは再実行で上書きしません。公開操作の応答を確認できず失敗した場合は、Releaseの公開状態と添付を先に確認してください。
公開済みならその状態を正として扱い、追加の修正は新しいバージョンで配布します。
Release公開後に実際の取得先と添付を確認してから、README等へダウンロード先を追加します。

## ソースからローカルへ導入する

ソースを手元に置いている場合は、ルートでCargoからインストールできます。
固定されたRustとOSのビルド用ツールが必要です。これはソースから導入する開発者向けの条件であり、配布済みの単体実行ファイルを使う場合には不要です。

```sh
cargo install --path . --locked
sdev --version
sdev --help
```

既定の導入先のbinディレクトリをPATHへ含めて利用します。
既存の同名インストールがある場合はCargoの案内に従い、上書きの対象を確認してください。
隔離した場所へ試す場合は、次のように専用の導入先を指定できます。Windowsでは実行ファイル名に.exeを付けます。

```sh
cargo install --path . --locked --root ./target/local-install
./target/local-install/bin/sdev --version
```

[Cargo公式のinstallの説明](https://doc.rust-lang.org/cargo/commands/cargo-install.html)で導入先と更新時の指定を確認できます。
この操作はGitHub Release・タグ・パッケージレジストリーを作成しません。
