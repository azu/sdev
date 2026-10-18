# 文書と作業の入口

[English](README.md)

利用先の指示・規約を先に確認し、作業に合うガイドライン、前提となる仕様、適用するテンプレートの順に読みます。
以下のテンプレートは、利用先で対象文書用のものが指定されていない場合に使うsdevの既定です。
既存文書の変更では、その文書の構造を尊重します。
この入口と配下の文書だけで、仕様の作成から実装・変更まで進められます。

## 作業別の参照先

| 作業 | 読むガイドライン | 前提となる仕様 | テンプレート |
| --- | --- | --- | --- |
| プロダクトの目的・利用者・価値を決める | [Productを書く](guidelines/product.ja.md) | 対象の既存Productがあれば読む | [product.md](templates/product.ja.md) |
| 機能の振る舞い・受け入れ条件を決める | [Requirementsを書く](guidelines/requirements.ja.md)、[シナリオ](guidelines/scenarios.ja.md) | 対象Product、変更なら既存Requirements・Design | [requirements.md](templates/requirements.ja.md) |
| 要件を実現する方法を決める | [Designを書く](guidelines/design.ja.md) | 対象Product・Requirements、既存Design | [design.md](templates/design.ja.md) |
| 実装・検証・既存文書の変更を行う | [文書の責務と更新](guidelines/documents.ja.md)、[実装と検証](guidelines/agent.ja.md#実装と検証) | 対象Product・Requirements・Design | 変更対象の文書のテンプレート |

共通の判断基準は[文書の責務と更新](guidelines/documents.ja.md)、対話や調査の進め方は[エージェントの作業原則](guidelines/agent.ja.md)に記述しています。
ガイドラインに判断基準と作業の進め方を、テンプレートに文書の構造をまとめています。

## sdev自身の仕様

| 文書 | 参照先 |
| --- | --- |
| Product | [sdev](products/sdev/product.ja.md) |
| Requirements | [ガイドラインとテンプレートの参照](products/sdev/features/guideline-reference/requirements.ja.md) |
| Design | [ガイドライン参照の設計](products/sdev/features/guideline-reference/design.ja.md) |
| Requirements | [コマンドの配布とガイドラインの参照](products/sdev/features/command-distribution/requirements.ja.md) |
| Design | [コマンドの配布と資料参照の設計](products/sdev/features/command-distribution/design.ja.md) |
| Requirements | [Markdownのリンク切れと文書の参照方向の検査](products/sdev/features/document-lint/requirements.ja.md) |
| Design | [文書リンクの検査の設計](products/sdev/features/document-lint/design.ja.md) |
| Requirements | [プロジェクトへの導入](products/sdev/features/project-bootstrap/requirements.ja.md) |
| Design | [プロジェクトへの導入の設計](products/sdev/features/project-bootstrap/design.ja.md) |
| Requirements | [既定言語と複数言語の文書](products/sdev/features/localization/requirements.ja.md) |
| Design | [言語と文書の設計](products/sdev/features/localization/design.ja.md) |

文書の配置と関係は[文書の責務と更新](guidelines/documents.ja.md)に従います。

配布物の選択、コマンドからの参照、更新と公開準備は[配布と更新](engineering/guidelines/releasing.ja.md)を参照します。

## sdev自身の開発基盤

Rustの実装・検証を行う場合は、[Rust開発ガイド](engineering/guidelines/rust-development.ja.md)を参照します。
目的と提供範囲は独立した[Rust開発基盤のProduct](products/development-platform/product.ja.md)、参照機能の条件と構成は[共通開発ガイドのRequirements](products/development-platform/features/common-development-guide/requirements.ja.md)と[Design](products/development-platform/features/common-development-guide/design.ja.md)に記述しています。
採用する環境と共通CIの構成は[Rust開発環境と共通CIの設計](engineering/design/rust-development.ja.md)で定義します。
この開発環境はsdev自身が対象であり、sdevを使う他のプロジェクトへ一律に適用しません。
helpの説明を変えるときは、[ローカルでのエージェント評価](../evals/README.ja.md)で、独立したsubagentの判定を通じて聞き取りの応答の質を比較します。
