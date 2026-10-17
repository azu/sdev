# Rust開発ガイド

sdev自身のRust実装は、リポジトリに固定した環境とCargoの標準機能で開発・検証します。
人とAIで同じ手順を使い、確認できた内容をレビューへ渡します。
このガイドは[Rust開発基盤のProduct](../../products/development-platform/product.md)に対応し、配置と責務は[共通開発ガイドのDesign](../../products/development-platform/features/common-development-guide/design.md)を参照します。
sdevを使う他のプロジェクトに、Rustやこの開発環境を必須にするものではありません。

## 作業を始める

[文書と作業の入口](../../README.md)から対象Product、Requirements、Designの順に読み、変更理由と範囲を確認します。
文書作成では[テンプレートの選び方](../../guidelines/documents.md#テンプレートの選び方)に従い、利用先で指定された対象用テンプレートを優先し、ない場合にsdevの既定を使います。
既存仕様の変更では、その文書の構造を尊重します。
文書だけを直す場合と、実装・依存・環境を変える場合を区別し、必要な確認を選びます。
資料参照用のCLIを使う場合も、利用先の仕様をsdev自身の仕様で置き換えません。

以下の実行手順は、ルートに`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`と対象コードが配置された状態に適用します。
文書だけのブランチなどで不足する場合は、その対象を報告します。
別の作業から未確定の実装をコピーしたり、手順を試すためだけの設定を生成したりせず、Rustの動作は未確認として扱います。
設定やコードへの反映状況は実ファイルで確かめ、このガイドが存在することを実装完了の根拠にしません。

## 環境を導入する

共通の入口として[mise](https://mise.jdx.dev/getting-started.html)を用意し、ルートで次を実行します。
Node.js、pnpm、hkはmise.lockの取得先とチェックサムを使って導入します。
Rustの管理には、後述のrustupとrust-toolchain.tomlを使います。

```sh
mise trust
mise install --locked
```

postinstallがpnpm-lock.yamlに固定した依存を導入し、hkでこのリポジトリのGit hookを登録します。
Git 2.54以降では、設定によるhook登録で既存のhookと併用します。
古いGitで既存hookと競合した場合は、強制上書きせず登録方法を確認します。
postinstallを無効にした場合は、`mise run setup`で同じ処理を手動実行できます。
以降は`mise run`から検査を実行でき、シェルの自動切り替え設定は必須ではありません。

ツールの版を変える場合はmise.tomlを編集して`mise lock`を実行し、mise.lockも一緒に更新します。
pnpmを変更した場合はpackage.jsonのpackageManagerも合わせ、pnpm-lock.yamlの互換性を確認します。

Rustの管理には、[公式の導入案内](https://rust-lang.org/tools/install/)が推奨するrustupを使います。
コンパイラーのrustcと、ビルド・依存・テストを扱うCargoを同じ環境でそろえます。
特定のエディターは必須にせず、補完や保存時の整形を使う場合も、後述のコマンドによる確認を共通の基準にします。

OS側のリンク用ツールも必要です。
macOSではXcode Command Line Tools、LinuxではCのコンパイラーとリンカーを用意します。
必要な道具は[Rust公式の導入手順](https://doc.rust-lang.org/book/ch01-01-installation.html)で確認できます。
WindowsのMSVC向け開発は、[rustupの前提条件](https://rust-lang.github.io/rustup/installation/windows-msvc.html)に従い、C++用ツールとWindows SDKを用意します。
OSごとの導入方法は[rustupの公式案内](https://rust-lang.github.io/rustup/installation/index.html)に従い、手元で使わないOSの開発環境を一律に追加しません。

## バージョンを固定して確認する

採用するバージョンと共通CIの構成は[Rust開発環境と共通CIの設計](../design/rust-development.md)で定義します。
開発用のRustバージョンは、ルートの`rust-toolchain.toml`で具体的な安定版のバージョンに固定します。
`profile = "minimal"`に整形用の`rustfmt`と静的検査用の`clippy`を追加する構成を使います。
検証のたびに別のバージョンが選ばれる`stable`や、必要性がない`nightly`へ切り替えません。
設定の意味と優先順位は[rustupのバージョン選択規則](https://rust-lang.github.io/rustup/overrides.html)に従います。

リポジトリのルートで、実際に選ばれる環境を確認します。

```sh
rustup show active-toolchain
rustc --version
cargo --version
```

環境変数やディレクトリ単位の上書き、`RUSTC`などで設定と異なる環境が選ばれていないか調べます。
食い違う場合は原因を明らかにします。
`cargo xtask`の開発用タスクは、設定に固定したCargoとrustcを選んで実行します。
新しい仕組みを追加して回避する前に、既存の設定と実行経路を確認します。

`Cargo.toml`の`rust-version`は、開発用の固定バージョンとは役割が異なります。
これは[ソースからビルドする際に対応するRustの条件](https://doc.rust-lang.org/cargo/reference/rust-version.html)です。
値を小さく書くだけで対応を保証せず、その環境での確認が必要かを変更時に判断します。
`edition`もRustのバージョンとは分けて扱い、更新の目的なしに書き換えません。

## 開発中に実行する

実装中にコンパイルを確認するときは`cargo check --locked`、CLIを起動するときは`cargo run --locked -- --help`を使えます。
引数は対象機能の仕様に従います。
ファイルを変更する機能は一時領域を対象にして確認します。
実行と検査の役割は、Cargo公式の[`cargo run`](https://doc.rust-lang.org/cargo/commands/cargo-run.html)と[`cargo check`](https://doc.rust-lang.org/cargo/commands/cargo-check.html)を参照します。
これらの成功だけで、変更後のテストや配布物の確認を省きません。

## 整形・静的検査・テストを行う

Rustの変更後は、リポジトリのルートで`mise run rs:check`を実行します。
検査本体の`cargo xtask check`を直接実行することもできます。
これは固定環境を選び、製品と開発用タスクの整形・静的検査・テストをまとめて確認します。
個別に調べる場合は次のCargo標準コマンドを使い、別の合否基準にはしません。

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
```

整形が必要なら`cargo fmt --all`で直して差分を確認します。
rustfmtの既定を使い、独自の整形設定は必要な理由がある場合だけ追加します。
整形と差分を変えない検査方法は[rustfmtの公式説明](https://github.com/rust-lang/rustfmt)を参照します。

Clippyはコードの問題を調べ、警告を失敗として扱います。
この扱いは[ClippyのCI向け説明](https://doc.rust-lang.org/stable/clippy/continuous_integration/index.html)に沿ったsdevの方針です。
既定の検査を出発点にし、厳しい追加ルールを一括で有効にしません。
警告を抑える必要がある場合は、該当箇所に範囲を絞って理由を残し、全体の規則を緩めて検査を通しません。

テストはRequirementsの観測できる振る舞いと、変更の影響を確認します。
内部処理だけで確かめるものは単体テスト、CLIの入出力・終了状態・ファイルへの影響は`tests/`配下の結合テストなど、確認する対象で配置を選びます。
実装の書き写しや件数の増加を目的にしません。
`cargo test`が扱う対象は[公式説明](https://doc.rust-lang.org/cargo/commands/cargo-test.html)を参照し、文書中の実行例を検証するテストは対象となるライブラリがある場合に確認します。

`--all-targets`はCargo内のテストや実行例なども検査する指定であり、全OS・CPUでの実行を意味しません。
機能切り替えの設定を追加した場合は、有効な組み合わせを確認して必要な検査を選びます。
相互に併用できない機能まで一律に有効にする指定は使いません。

失敗した場合は原因を修正して影響する確認を再実行します。
環境不足や取得失敗はコードの失敗と区別し、未実行の検査を成功として報告しません。

## READMEの文章を検査する

環境の導入後、READMEを変更するときは次を実行します。
CIもmise.tomlのNode.jsとpnpmを使います。文書検査の依存は、製品を使う環境には導入しません。

```sh
mise run md:lint
```

自動修正できる表記は`mise run md:fix`で直せます。
修正後は差分を読み返し、検査を再実行します。
textlintは文章の表記を検査し、見出しの構成や内容の正しさは人とAIが確認します。
Markdownのリンクと参照方向は`./target/debug/sdev lint`で確認します。
採用したルールとCIの構成は、共通設計で管理します。

コミット前のhookは、READMEの変更時にmd:lint、RustのソースやCargoの設定の変更時にrs:checkを実行します。
未ステージの変更はhkが一時的に退避し、検査後に復元します。自動修正や再ステージは行いません。
手動で確認する場合は`mise run pre-commit`、登録を確認する場合は`git hook list pre-commit`を使います。
登録を外す場合は、次を実行します。

```sh
mise exec -- hk uninstall
```

## 依存とロックファイルを変更する

依存の追加前に、標準ライブラリと既存の依存で目的を満たせるか確認します。
追加する場合は、用途、対応するRustとOS、ライセンス、必要な機能、ビルド時に動く処理を調べ、選定理由をDesignへ残します。
テストだけに必要な依存は開発用、ビルド時だけ必要な依存はビルド用として役割を分けます。

追加には[`cargo add`](https://doc.rust-lang.org/cargo/commands/cargo-add.html)、更新には[`cargo update`](https://doc.rust-lang.org/cargo/commands/cargo-update.html)などCargoの操作を使います。
更新は対象を絞り、関係のない依存更新を混ぜません。
`Cargo.lock`はCargoに生成させてGitで管理し、`Cargo.toml`と一緒に差分をレビューします。
ロックファイルを手で編集しない方針は[Cargo公式の役割説明](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)に従います。

通常の検証では`--locked`で、意図しない依存の解決変更を検出します。
依存を変更する作業では先にCargoでロックファイルを更新し、その後に固定した状態の検証へ戻します。
この指定はネットワークの不要性や、ビルド出力のバイト単位の一致まで保証するものではありません。
ビルド処理や外部コマンドを実行する依存もあるため、取得したコードの実行を単なる文書の読み取りと同じ扱いにしません。

## CIと異なるOSで確かめる

CIでは、ローカルと同じ固定環境・ロックされた依存・基本検査を使います。
実際の対象と呼び出し方法は`.github/workflows/`以下と、そこから呼ばれる`cargo xtask`の実装を確認します。
CIのインストール指定を別に持つ場合は、Rustの更新と一緒に見直し、設定とのずれを残しません。
キャッシュは時間短縮の手段とし、成功結果の代わりにしません。

確認するOS・CPUはsdevの対応環境と変更の影響から選びます。
パス、改行、実行ファイル名、環境変数、外部プロセスなどに関係する変更は、該当するOSでの振る舞いを確認します。
テスト用の一時領域を使い、開発者の作業ディレクトリや偶然の環境設定に依存させません。
別ターゲット向けのビルド成功と、その環境でのテスト実行成功は分けて報告します。

配布用のビルド、単体実行ファイルの確認、成果物の形式、公開手順は、[配布機能のRequirements](../../products/sdev/features/command-distribution/requirements.md)・[Design](../../products/sdev/features/command-distribution/design.md)と[配布と更新](releasing.md)を参照します。
このガイドに配布の対象一覧や公開コマンドを複製せず、CIで検証できたことを公開済みと扱いません。

## 文書・生成物・一時実験を分ける

仕様は`docs/products/`、共通の規約は`docs/engineering/guidelines/`、複数機能に共通する設計は`docs/engineering/design/`へ置きます。
細かな責務と更新の判断は[文書の責務と更新](../../guidelines/documents.md)に従います。
Rustの公開項目の使い方はコード近くのドキュメントコメントに書き、要件や設計の判断理由を全文複製しません。
コードの文書を確認するときは[`cargo doc`](https://doc.rust-lang.org/cargo/commands/cargo-doc.html)による生成を使えます。

`target/`などCargoが生成するものと、配布準備で生成するものはソースと分け、通常はGitへ追加しません。
一時実験は目的と実行条件が分かる専用の場所に分離し、通常のソースや検査対象へ暗黙に組み込みません。
実験で使った道具を本体の必須依存にせず、採用する場合は必要な部分を仕様・通常のコードへ移します。
保持する実験資料と破棄できる生成物は区別します。

変更の目的や提供範囲が変わる場合はProduct、振る舞いが変わる場合はRequirementsとDesign、実現方法や判断理由が変わる場合はDesignを更新します。
基本手順が変わる場合はこのガイドも更新します。
Rustの更新ではCI、依存との互換性、整形差分や新しい警告、配布に必要な通知資料への影響も確認します。

## 結果を報告する

レビューには、変更理由、実行した確認、実行環境、結果を残します。
実行したコマンドや開発用タスク、対象のRustバージョンとOS・ターゲットを示し、失敗と未確認には理由を添えます。
CIの結果を使う場合は対象コミットを確認します。
検査の成功と、Requirementsの受け入れ条件の確認はそれぞれ報告します。
