# 配布と更新

[English](releasing.md)

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
OS条件は[配布のDesign](../../products/sdev/features/command-distribution/design.ja.md)で定めたビルド上の条件です。
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

## miseで導入する

[miseのGitHub backend](https://mise.jdx.dev/dev-tools/backends/github.html)から、同じReleaseの本体を導入できます。利用先のmise.tomlへ次の設定を追加します。この例は公開済みのv0.4.0を選びます。

```toml
[tools."github:azu/sdev"]
version = "0.4.0"
format = "raw"
bin = "sdev"

[tools."github:azu/sdev".platforms]
macos-arm64 = { asset_pattern = "sdev-v*-macos-arm64" }
macos-x64 = { asset_pattern = "sdev-v*-macos-x64" }
linux-arm64 = { asset_pattern = "sdev-v*-linux-arm64" }
linux-x64 = { asset_pattern = "sdev-v*-linux-x64" }
windows-x64 = { asset_pattern = "sdev-v*-windows-x64.exe" }
```

raw形式と`bin`の指定により、単体の配布ファイルを`sdev`として実行できます。対象別の名前で本体だけを選び、チェックサムやメタデータを除外します。
設定を確認して信頼した後、全5対象をロックし、固定したバージョンを導入します。

```sh
mise trust
mise lock --platform macos-arm64,macos-x64,linux-arm64,linux-x64,windows-x64
mise install --locked
mise exec -- sdev help
```

mise.tomlとmise.lockをコミットします。AGENTS.mdには、仕様や実装の作業前に`mise exec -- sdev help`を実行し、作業に対応するガイドを選ぶよう記載します。
言語を共有する場合は既存のsdev.tomlの設定を使います。`sdev init`はプロジェクト用の起動スクリプトを作る操作であり、このmise経由の導入では不要です。
更新はmise.tomlのバージョン変更後にmise.lockを生成し直し、URL・チェックサムの差分を確認して`mise install --locked`を実行します。この導入先には`sdev update`を使わず、miseで更新します。

## bootstrapと証明の検証

macOS・Linuxでは、内容と取得元を確認した`install.sh`からプロジェクト内の`./bin/sdev`へ導入できます。
公開後に確認した配布URLから取得するか、このリポジトリの確認済みソースを使います。
リポジトリのルートへ導入する場合は`sh install.sh --version 0.1.0 --init`、別のプロジェクトへ導入する場合はそのルートからスクリプトの実際の配置場所を指定します。
バージョンを省略すると最新の安定Releaseを取得し、`--init`を省略すると実行ファイルだけを用意します。
`--install-dir DIR`で本体の配置先ディレクトリを変更できます。
起動スクリプトに対応する版の`init`は、`bin/sdev`をコミット用の起動スクリプトへ置き換えます。
初期設定では公開済みの全対象のSHA-256を取得して起動スクリプトへ保存するため、curlと通信が必要です。対象のチェックサムに不足や不正があれば生成しません。
初回起動時には期待値を取得し直さず、コミット済みのSHAと本体を照合します。
次回の起動時に固定した本体を`.sdev/bin/<version>/<os>-<arch>/sdev`へ取得し、以後はそれを使います。
配置先を変える場合は、導入済みの本体から`sdev init --path ./scripts/sdev`のように実行します。
AGENTS.md内の参照指示も指定先に合わせて生成されます。すでにある別の配置先の指示は手動で合わせてから再実行します。
`sh install.sh --help`でオプションを確認できます。
言語は`sdev init --lang en --translations ja`のように初期設定でき、既存の設定は保持します。

チェックサムは毎回照合します。
これは破損や取得対象の間違いを見つけるための確認で、配布元を証明するものではありません。
通常の導入では証明の検証を省略したことを表示します。

`--verify`を付けると、GitHub Artifact Attestationsで配布元も検証します。
[検証に関する修正](https://github.com/cli/cli/security/advisories/GHSA-8xvp-7hj6-mcj9)を含む`gh` 2.93.0以降を必要とし、証明の取得・検証の失敗時はチェックサムだけの導入へ切り替えません。
同じReleaseの`attestation.sigstore.json`と`sdev-release.txt`を取得してローカルbundleから検証し、証明済みの記録にあるコミットと、実行ファイルのソース・署名ワークフローのコミットを照合します。
この記録はシェルとして読み込まず、確認済みのバージョン・コミットとしてだけ使います。

```sh
sh /path/to/sdev/install.sh --version 0.1.0 --verify --init
```

利用するソースも固定する場合は、Releaseのタグから独立して確認したコミットを`--source-commit "$SDEV_SOURCE_COMMIT"`で指定します。
指定がない場合は証明済みの配布記録のコミットを使い、Gitのタグを別経路で照合したことにはなりません。

このオプションが検証するのは、これから起動する実行ファイルです。
すでに実行したインストーラー自身を後から検証するものではありません。
スクリプトも実行前に検証する場合は、公開済みの`install.sh`と`attestation.sigstore.json`を保存し、独立して導入したGitHub CLIで先に検証します。
次の例は公開後に同じReleaseの両ファイルを取得した場所で実行します。

```sh
gh attestation verify ./install.sh --hostname github.com --repo azu/sdev \
  --bundle ./attestation.sigstore.json \
  --signer-workflow azu/sdev/.github/workflows/build.yml \
  --source-ref refs/heads/main --deny-self-hosted-runners \
  --source-digest "$SDEV_SOURCE_COMMIT" --signer-digest "$SDEV_SOURCE_COMMIT"
```

`SDEV_SOURCE_COMMIT`には、利用するReleaseのタグから独立して確認したコミットSHAを指定します。
証明は配布元と作成経路の確認であり、コードの無害さやバグがないことの保証ではありません。
証明の意味と出力の扱いは[GitHub CLIの仕様](https://cli.github.com/manual/gh_attestation_verify)を参照してください。

### Claude Code on the webで準備する

プロジェクトの起動スクリプト・AGENTS.md・sdev.toml・.gitignoreをGitへ含め、`.sdev/`以下の本体は含めません。
Web環境でも`./bin/sdev help`を実行すると、スクリプトが固定したバージョンを必要時に取得します。
Setup scriptで先に取得する場合も、同じ起動スクリプトを実行できます。
非対話で実行でき、PATHやシェルの設定ファイルを変更しません。
初回取得に証明検証を求める場合は`SDEV_VERIFY=1 ./bin/sdev help`を使います。
取得済みの本体は再検証しないため、検証方法を変える場合は該当するキャッシュを削除してから取得し直します。

[公式の環境仕様](https://code.claude.com/docs/en/cloud-environments)ではUbuntu 24.04のx86_64環境と、起動前のSetup scriptを提供しています。
既定のTrustedネットワークには公開ファイルとReleaseの配布に使うドメインが含まれますが、別リポジトリのGitHub APIやソースアーカイブには別の制約があります。
bootstrapは最新Releaseの検索と証明の取得にGitHub APIを使いません。

`--verify`で信頼情報の取得先へ通信できない場合は、独立した信頼できる環境で`gh attestation trusted-root`から準備したファイルを、`--trusted-root /path/to/trusted-root.jsonl`で指定します。
信頼情報を未検証の配布物と一緒に取得して、そのまま信用しないでください。
準備と更新の考え方は[GitHubのオフライン検証手順](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/verify-attestations-offline)を参照します。

実際のRelease取得、Web環境のプロキシを通した取得、起動・証明検証は、公開後に確認します。
ローカルの取得テストやLinux向けCIの成功だけで、Web環境で検証済みとは扱いません。

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

言語はsdev.tomlに従い、未設定では英語です。`--lang ja`または`--lang en`で今回の言語を選べます。
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

macOS・Linuxの生成済み起動スクリプトは、プロジェクトのルートで更新します。

```sh
./bin/sdev update
```

バージョンを省略すると最新安定版を選びます。
公開済みの版には`--version X.Y.Z`、本体を直接呼んで別の配置先を更新する場合には`--path ./scripts/sdev`を指定します。
生成済みの起動スクリプトから呼ぶ場合は、その配置先を引き継ぎます。
チェックサムと起動は毎回確認します。`--verify`または`SDEV_VERIFY=1`では証明も検証し、`--source-commit`と`--trusted-root`は導入時と同じ意味で使えます。

元の生成内容を照合し、新しい本体とスクリプトの準備後に切り替えます。
編集済みのスクリプトと、AGENTS.md・.gitignore・sdev.toml・仕様は保持します。
取得・検証・準備に失敗した場合は、旧スクリプトを使い続けられます。
同じ版なら、その旨を表示してスクリプトを書き直しません。
強制終了後に`.sdev/update.lock`が残った場合は、更新処理が動いていないことを確認してから、そのロック用ディレクトリを削除します。

`update`を持たない旧版では、確認済みの導入コマンドを再実行できます。元の`--init`付きの実行でも同じ更新に進みます。
別の配置先には`sh install.sh --update --path ./scripts/sdev`を使います。
更新したスクリプトの差分を確認してコミットすると、別のcloneでもその版を使います。取得した本体はGitの除外対象に残します。
通常の起動では、バックグラウンドで更新しません。

Windowsの単体実行ファイルや別の導入方法では、対応する新版を別の場所へ取得し、バージョンとチェックサムを確認してから、その方法で旧版と差し替えます。
新しい本体の取得と確認が終わるまで、旧版を維持します。パッケージ管理の配置をプロジェクトの更新機能では置き換えません。
起動スクリプトの配置先変更は、引き続き`init --path`を使う別の操作です。元のスクリプトを確認・退避し、既存の参照指示を新しい配置先へ合わせてから初期設定します。

## 開発者が配布物を準備する

開発環境と通常の検証は[Rust開発ガイド](rust-development.ja.md)、Rustの選択と共通CIの構成は[共通設計](../design/rust-development.ja.md)に従います。
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
全件が成功すると、再利用ビルドワークフローの証明生成ジョブが対象・コミット・バージョン・チェックサム・通知・LICENSEを照合し、公開する実行ファイル・インストーラー・記録へ証明を付けます。
このジョブにだけOIDCと証明生成の権限を付け、通常のビルドジョブと、Releaseを書き込む公開ジョブから分離します。
公開ジョブは全ファイルの証明をローカルbundleで検証した後に、タグと下書きReleaseを作成します。
実行ファイルとSHA-256、THIRD-PARTY-NOTICES.html、THIRD-PARTY-CRATES.html、sdev自身のLICENSE、install.shとSHA-256、sdev-release.txt、attestation.sigstore.jsonを添付し、GitHub上の全添付の名前・状態・サイズ・ハッシュも確認してから公開します。
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
