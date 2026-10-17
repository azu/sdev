# 配布する通知

`rust-1.96.0-COPYRIGHT-library.html`は、Rustupで導入したRust 1.96.0の`share/doc/rust/COPYRIGHT-library.html`を内容を変えずに配置したものです。
配布準備で`THIRD-PARTY-NOTICES.html`として実行ファイルと一緒に用意します。
Rustのバージョンを更新するときは、この通知も対応する配布物のものへ更新してください。

製品が使う外部crateの通知は、配布準備で`THIRD-PARTY-CRATES.html`へまとめます。
Cargo.lockで解決された対象ターゲットの通常の依存をCargoから取得し、そのパッケージに含まれるLICENSE・COPYING・NOTICE・UNLICENSEの本文とライセンスの宣言を載せます。
開発用タスクだけの依存や、ビルド時だけの依存は含めません。
通知資料が見つからなければ配布準備を失敗にし、必要な通知を省略しません。
通知を表示するCLI操作や、実行時に通知ファイルを読む処理はありません。

sdev自身のライセンスは未指定です。
これらの通知をsdev全体のライセンスとして扱わず、公開前にプロジェクトのライセンスを決めてルートのLICENSEへ配置してください。
LICENSEがある場合は、配布準備で通知と一緒にコピーします。
