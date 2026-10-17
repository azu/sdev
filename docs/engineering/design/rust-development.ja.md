# Rust開発環境と共通CIの設計

[English](rust-development.md)

[Rust開発基盤のProduct](../../products/development-platform/product.ja.md)に基づき、開発環境と複数機能に共通するCIの構成を定めます。
日常の操作と更新手順は[Rust開発ガイド](../guidelines/rust-development.ja.md)へ分けます。
この文書は採用する構成の定義であり、設定の反映やCI実行の完了記録ではありません。

## Rustと設定の責務

開発とCIに使うRustは1.96.0とし、`rust-toolchain.toml`の`channel`で固定します。
導入する構成は`profile = "minimal"`と`rustfmt`・`clippy`です。
Cargoとrustcはこの固定環境のものを使い、環境変数やローカルの上書きで異なるコンパイラーを選ばないようにします。
固定方法は[rustupの公式説明](https://rust-lang.github.io/rustup/overrides.html)に従います。

`Cargo.toml`では`edition = "2024"`、`rust-version = "1.96"`を採用します。
`rust-version`はソースビルドで対応するRustの条件であり、開発用のバージョン固定とは分けます。
役割は[Cargoの公式説明](https://doc.rust-lang.org/cargo/reference/rust-version.html)に従います。
製品バージョン、最適化、OS固有の配布設定は製品側で扱い、この設計で重ねて定義しません。

依存は`Cargo.toml`、解決された具体的な依存は`Cargo.lock`で管理します。
固定環境とロックファイルを使う理由は、同じ変更の検証で依存の解決が意図せず変わることを防ぐためです。
ビルド出力のバイト単位の一致を保証する設計ではありません。

## 共通CIの構成

共通の検査はGitHub Actionsの`.github/workflows/`以下から実行し、ローカルと同じ固定環境とロックファイルを使います。
Rustの設定を読み取って導入する方法を優先し、導入指定を別に記述する場合は同じ変更で更新する対象とします。
共通チェックと配布準備は、ワークスペース内のRust製開発用パッケージ`xtask`へまとめます。
Cargoの別名設定から起動し、同じ固定環境でCargoとrustcを実行する責務を持たせます。
子プロセスのPATHでは固定したRustのbinを先頭に置き、Clippyやrustfmtが別の導入先から選ばれないようにします。
製品パッケージからは依存せず、通常の製品ビルドの対象も製品に限定します。
Rust製の開発用タスクには別の言語の実行環境や汎用ランナーを要求しません。

開発用タスクのJSONの読み書きには`serde_json`、SHA-256には`sha2`を使います。
Cargoのメタデータと配布の確認用JSONを扱い、既存のチェックサム形式を保つためです。
一時領域やプロセスの実行は標準ライブラリで扱います。
開発用タスクの依存を製品の依存へ暗黙に加えず、製品で必要なライブラリは各機能の設計で判断します。

整形、静的検査、テストを共通の検査対象とし、具体的なコマンドは[Rust開発ガイド](../guidelines/rust-development.ja.md#整形静的検査テストを行う)に集約します。
静的検査では警告を失敗として扱います。
検査が失敗した場合は、その結果をジョブの失敗として返し、後続処理で成功へ置き換えません。

実行契機はプルリクエスト、mainへの変更、手動実行とします。
対象のOS・CPUは製品の対応環境に従い、変更したコミットに対する実行結果を確認します。
OSごとに実行する検査と配布用のビルド・成果物確認を組み合わせる場合も、後者の具体的な対象や形式は配布機能で定義します。
共通CIでの検査と、公開の可否・公開操作は分けます。

ワークフローの権限は必要な範囲に絞り、共通の読み取り検査では`contents: read`を使います。
使用するActionsはコミットに固定し、認証情報を検査対象のコードや成果物へ残しません。
同じ変更への実行を識別して古い実行を取り消し、対象環境ごとの失敗を確認できる構成にします。
キャッシュや成果物保存は補助手段であり、検査を省く条件にしません。

## 開発コマンドと環境の入口

開発環境と日常のコマンドの入口には[mise](https://mise.jdx.dev/tasks/)を使います。
Node.js、pnpm、hkはmise.tomlで具体的な版に固定し、取得先とチェックサムをmise.lockへ記録します。
導入では`mise install --locked`を使い、pnpmはpackage.jsonのpackageManagerと一致させます。
Rustはrustupと既存のrust-toolchain.tomlを使い、mise.tomlには版を重複して指定しません。
miseによるRustのバージョンファイルの検出と管理を無効にし、lockedでの導入対象をmise.lockに記録した道具へ限定します。
Rustの検査本体は引き続きcargo xtask checkに置き、miseを使わない実行経路も保ちます。

文章検査は`mise run md:lint`、自動修正は`mise run md:fix`、Rustの検査は`mise run rs:check`にそろえます。
[postinstall](https://mise.jdx.dev/hooks.html)からsetupタスクを呼び、固定した文書検査の依存とこのリポジトリのGit hookを導入します。
CIではMISE_NO_HOOKSを有効にしてpostinstallを無効にし、文書検査の依存だけを明示的に導入します。

## READMEの文章検査

英語のREADME.mdは[slopless](https://github.com/berelevant-ai/slopless)、日本語のREADME.ja.mdは[textlint](https://textlint.org/docs/getting-started/)の[技術文書向けプリセット](https://github.com/textlint-ja/textlint-rule-preset-ja-technical-writing)で検査します。
一文の長さ、助詞の重複、表記の揺れなどを検出し、構成や説明の順序は人とAIが読み返して判断します。
文章検査の対象は両言語のREADMEとし、ガイド・テンプレート・仕様の構成と内容は読み返しと文書検査で確認します。

文書検査の依存は開発用のpackage.jsonとpnpm-lock.yamlで管理し、pnpmの版はpackageManagerで固定します。
textlintとプリセットは具体的な版を指定し、ルールの変更はREADMEへの影響と一緒に確認します。
英語の設定は.textlintrc.json、日本語は.textlintrc.ja.jsonに置き、プリセットの既定値から始めます。
sloplessはtextlintが読み込めるパッケージ名へのnpm aliasで固定します。
ルールの例外を加える場合は、その文書で必要な理由を残します。

文章検査はNode.jsとpnpmで実行し、Rustの共通検査とは別のCIジョブにします。
CIもmise.tomlから環境を導入し、固定した依存とローカルと同じ検査コマンドを使います。
md:lintには言語版の組と相互リンクの確認、sdev自身によるリンク・参照方向の検査も含めます。後者には固定したRustを使います。
この開発用の依存は製品の実行・インストール・Rustのビルドには要求しません。
具体的な操作は[Rust開発ガイド](../guidelines/rust-development.ja.md#readmeの文章を検査する)に記述します。

## コミット前の検査

Git hookは[hk](https://hk.jdx.dev/)で管理し、hk.pklに対象ファイルと検査コマンドを記述します。
setupから`hk install --mise`を実行してこのリポジトリへ登録し、グローバル設定は変更しません。
Git 2.54以降では設定によるhookを使うため、利用者の既存hookと併用できます。
古いGitではhkのスクリプトによる登録を使い、既存のhookを強制的に上書きしません。

Markdownと文章検査の設定の変更時はmd:lint、RustのソースやCargoの設定の変更時はrs:checkを呼びます。
未ステージの変更の退避と復元はhkへ任せ、ファイルの一部だけをステージした場合もコミットする内容を検査します。
失敗はコミットを止める結果として返し、自動修正や再ステージは行いません。
hookの未登録や回避があっても、CIでは両言語のREADMEと言語版の組、文書のリンクを検査します。

## 更新の方針

Rustの更新は、使う言語機能、依存の要求、修正の取り込みなど、理由を持つ変更として行います。
最新バージョンへ自動追従すること自体を目的にしません。
採用する値はこの文書と`rust-toolchain.toml`で一致させ、CIの導入指定、開発用タスク、必要なら`rust-version`も一緒に見直します。
更新後に必要な検証と、配布用の通知資料などへの影響確認は共通ガイドに従います。

`rust-version`は開発環境を更新しただけで機械的に引き上げず、ソースビルドの対応条件を変える理由と検証範囲を確認します。
整形やClippyの出力が変わる場合は、更新に伴う差分と製品の振る舞いの変更を区別してレビューします。

## 配布機能との境界

各機能のDesignは、[Rust開発ガイド](../guidelines/rust-development.ja.md)の共通規約を参照します。
この文書は構成の採用理由を管理し、各機能が従う規約・手順はガイドで管理します。
ビルド対象、単体実行ファイルとしての条件、通知資料、成果物の命名・公開インターフェースは配布機能で定義します。
開発時の起動や確認コマンドは共通ガイドへリンクし、同じ手順や値を製品機能ごとに書き直しません。

## 代替案と判断理由

可変の`stable`を使えば更新作業を減らせますが、変更していないコードに新しい整形差分や警告が出る可能性があります。
レビュー時の条件をそろえるため、具体的なバージョンに固定し、更新を独立して確認できる変更にします。

Rustの検査基準はCargoの標準機能とRust製開発用タスクでローカルとCIをそろえます。
miseには開発ツールの導入とコマンドの入口をまとめ、既存の検査処理を移し替えません。
標準構成で確認できない具体的な問題が生じた場合に、追加の道具を比較します。
