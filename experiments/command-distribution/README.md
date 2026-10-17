# 配布方式の比較用CLI

Bun・scriptc・Rustを、同じ意味のCLIとMarkdown処理で比較する試作品です。
[配布・参照のRequirements](../../docs/products/sdev/features/command-distribution/requirements.md)と[比較用Design](../../docs/products/sdev/features/command-distribution/design.md)を前提とし、製品lintや公開コマンドの完成とは扱いません。

## 比較条件

既存の入口・ガイドライン・テンプレート10文書を同じ一覧からビルド時に取り込みます。
ヘルプ、バージョン、資料一覧・本文、Markdownの内部リンク先の存在確認を提供します。
TypeScript側はMarked 18.1.0、Rust側はpulldown-cmark 0.13.4を使います。
正規表現でMarkdownを解析せず、コード内の見せかけのリンク、参照形式、画像、入れ子を区別します。
見出しの存在、HTMLのリンク、すべてのMarkdown方言への対応は検査しません。
異なるパーサーと実装の比較であり、言語そのものの性能を示すものではありません。

今回の主な比較はBun 1.4.0とRust 1.96.0のrelease最適化済み実行ファイルです。
Rustはopt-level 3、LTO、codegen-units 1、シンボル除去を指定しています。
Rustupで選んだコンパイラーをRUSTCにも明示し、Homebrew等の別コンパイラーが混ざらないようにします。
Cargo.lockとpackage-lock.jsonで依存を固定します。
Rustのロックされた依存15件の公開日も確認し、公開直後のものは以前のバージョンへ固定しました。

## 再現する

このディレクトリで実行します。
開発側にはNode.js・Bun・Rustupのstableツールチェーンと、macOSではXcode Command Line Toolsが必要です。
生成した実行ファイルを使う人に要求する実行環境とは区別します。

```sh
npm ci --ignore-scripts --no-audit --no-fund
node node_modules/scriptc/scripts/install-native.mjs
node build.mjs --rust
node verify.mjs
node inspect-artifacts.mjs
```

測定はリポジトリのルートで`cargo xtask measure --rust`を実行します。
その後、この実験ディレクトリで`node report-rust.mjs --xtask`を実行すると、新しい測定のレポートを生成します。
Cargoとrustcにはリポジトリで固定したRustupの環境を使います。
過去の測定はPython 3.9.6で行いました。現在の再測定はRust製の開発用タスクで行い、過去の結果を上書きしません。
`verify.mjs`は既存のscriptc生成物も比較するため、初めて再現する場合は先に`node build.mjs`でBun・scriptcを生成します。
scriptcの導入スクリプトは、取得済みの対象OS用パッケージからネイティブコンパイラーと設定を配置します。

`build.mjs --rust`はBunとRustのアプリを各3回ビルドします。
依存の取得を含む初回ビルドや、何も変更しないCargoの終了時間を比べず、Rust側のソースの更新時刻を変えてアプリを再コンパイルします。
他のビルドやダウンロードが終わってから計測します。

他環境向けの確認には、対象用標準ライブラリをRustupで用意したうえで`node build-rust-cross.mjs`を使います。
対象はmacOS x64、Linux arm64/x64、Windows x64です。
Windows向けには`.cargo/config.toml`でCランタイムの静的リンクを指定しています。
OS用の標準ライブラリと、最後に実行ファイルを結合するリンカー・SDKは別に必要です。
今回Linux・Windowsのリンカー環境の大規模構築は行っていません。
Bunの他環境向け生成は`node build.mjs --cross`からも再現できます。

## 動作と時間の確認

`verify.mjs`は、別の空ディレクトリからPATHに追加の実行環境を含めず起動します。
資料本文10件、既存Markdown文書17件の58リンク、境界入力の5リンク、失敗時の診断と終了状態が3候補で一致することを確認します。
RustとBunのJSONはキーの並び順ではなく、解析後の内容を比較します。
本文出力は元の資料の後に改行を1つ加えます。

起動時間の測定対象は、バージョン、ヘルプ、同梱テンプレート1件の表示、既存2文書のリンク確認です。
リンク確認の入力は`docs/README.md`と`docs/products/sdev/product.md`だけで、合計18リンクです。
17文書58リンクは動作検証の対象であり、起動時間の測定対象とは区別します。

各操作を5回準備実行した後、各60回の新しいプロセスを交互に順序を変えて起動し、標準出力を捨てて終了までの時間を測ります。
20回ずつの3組の中央値も記録します。
通常のOSやプロセス生成の時間とウォームキャッシュを含み、コールドキャッシュやCLI内部だけの時間ではありません。

内部時間は比較用の`profile`操作で、同じ2文書を読み取り、Markdown解析、パス解決・存在確認・結果の作成の3段階を測ります。
通常の`check`操作では時計を呼びません。
内部時間からライブラリの読み込み、起動、CLIの選択処理、JSONの直列化と出力を除きます。
内部時間も各5回の準備と各60回の交互起動で確認します。

## 結果と判断の境界

過去の数値と判断は`results/comparison-rust.md`、入力のハッシュ・バイナリのハッシュ・測定値は`results/startup-rust.json`へ保存しています。
現在の再測定は`results/startup-rust-xtask.json`、`--xtask`付きのレポートは`results/comparison-rust-xtask.md`へ分けます。
準備実行と測定の回数・中央値・分位点・処理段階の区別は維持し、実行順を決める乱数の方式と測定側の実装が変わったことを新しい結果へ記録します。
内部時間と起動時間、生成可否と実動作、公表される最低OS条件と確認済みの環境を分けます。
今回の実動作・時間の確認対象はmacOS 26.5.1 arm64です。
RustはmacOS arm64/x64を生成でき、Linuxは対象向けのリンク環境、WindowsはMSVCのlink.exe不足で停止しました。
Bunは初期対象5組を生成できました。他OSやmacOS x64の実動作は未検証です。

起動速度だけを最優先にせず、エージェントが主に実行すること、配布サイズ、配布の実現性、将来の拡張を合わせて判断します。
Rustの小さな配布ファイルは今回確認できた利点です。
ローカルサーバーとWeb画面を将来選ぶ場合、BunはHTTPサーバー・HTMLからの資産の変換とバンドル・同梱を標準機能でまとめられます。
RustでもHTTPライブラリとファイル同梱で実現でき、ブラウザー側のTypeScript等の変換が必要ならその工程を別途組み合わせます。
根拠となる公式資料と未決の範囲は比較用Designに記載しています。今回可視化は実装していません。

以前のBun・scriptc比較は`results/comparison.md`と`results/startup.json`に参考として残し、今回の再測定値と混ぜません。
scriptc 0.2.8はMarkedに`--dynamic`が必要で、純粋なネイティブ化は互換性診断で停止しました。
当時の公開後待機設定により0.3系は未検証で、0.2.8の結果から一般化しません。
同じTypeScriptから両方を生成できるため、scriptcで小さな配布ファイルを狙い、困った場合にBunへ切り替える案も候補として残します。
0.3系の依存のネイティブ化は今回の動的エンジン同梱とは異なり、製品での採用や切り替えは未決です。
Rust追加後のTypeScript側の変更でも動的エンジン同梱構成を再ビルドし、同じ動作を確認しています。scriptcの起動時間は再測定していません。

依存、生成物、計測結果はGitの無視対象とし、仕様文書へ一時ログを置きません。
比較を終え、製品にはRustを採用しています。製品の公開操作は作業名から読むhelpとtemplateに整理しており、この実験のファイル名による参照やリンク検査・profileとは異なります。
測定値は比較時点の入力とバイナリについての記録です。以後に文書を更新して再実行した結果とは区別します。
正式な配布物の署名、各OSの実機確認、公開リリースは別の作業です。
