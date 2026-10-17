# 文書と作業の入口

作業に合う行を選び、ガイドライン、前提となる仕様、手順、テンプレートの順に読みます。
この入口と配下の文書だけで、仕様の作成から実装・変更まで進められます。

## 作業別の参照先

| 作業 | 最初に読むガイドライン | 前提となる仕様 | 作業手順 | テンプレート |
| --- | --- | --- | --- | --- |
| プロダクトの目的・利用者・価値を決める | [文書の責務と更新](guidelines/documents.md) | 対象の既存Productがあれば読む | [Productを書く](workflows/product.md) | [product.md](templates/product.md) |
| 機能の振る舞い・受け入れ条件を決める | [文書の責務と更新](guidelines/documents.md)、[シナリオ](guidelines/scenarios.md) | 対象Product、変更なら既存Requirements・Design | [Requirementsを書く](workflows/requirements.md) | [requirements.md](templates/requirements.md) |
| 要件を実現する方法を決める | [文書の責務と更新](guidelines/documents.md) | 対象Product・Requirements、既存Design | [Designを書く](workflows/design.md) | [design.md](templates/design.md) |
| 実装・検証・既存文書の変更を行う | [文書の責務と更新](guidelines/documents.md)の変更の扱い | 対象Product・Requirements・Design | [実装と変更](workflows/implementation.md) | 変更対象の文書のテンプレート |

対話や調査の共通の進め方は[エージェントの作業原則](workflows/README.md)に記述しています。
テンプレートは文書の構造、ガイドラインは判断基準、作業手順は読む順序・確認・執筆・見直しを担当します。

## sdev自身の仕様

| 文書 | 参照先 |
| --- | --- |
| Product | [sdev](products/sdev/product.md) |
| Requirements | [ガイドラインとテンプレートの参照](products/sdev/features/guideline-reference/requirements.md) |
| Design | [ガイドライン参照の設計](products/sdev/features/guideline-reference/design.md) |

文書の配置と関係は[文書の責務と更新](guidelines/documents.md)に従います。
[開発基盤の確認記録](reviews/bootstrap.md)は、仕様の決定内容と区別して、実際に確認した結果を記録します。
