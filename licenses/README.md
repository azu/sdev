# 配布する通知

`rust-1.96.0-COPYRIGHT-library.html`は、Rustupで導入したRust 1.96.0の`share/doc/rust/COPYRIGHT-library.html`を内容を変えずに配置したものです。
配布準備で`THIRD-PARTY-NOTICES.html`として実行ファイルと一緒に用意します。
通知を表示するCLI操作や、実行時に通知ファイルを読む処理はありません。
製品CLIの実行時依存に外部crateはなく、Markdownリンクの変換に使うpulldown-cmarkはビルド時だけの依存です。
Rustのバージョンを更新するときは、この通知も対応する配布物のものへ更新してください。

sdev自身のライセンスは未指定です。
この通知をsdev全体のライセンスとして扱わず、公開前にプロジェクトのライセンスを決めてルートのLICENSEへ配置してください。
LICENSEがある場合は、配布準備で通知と一緒にコピーします。
