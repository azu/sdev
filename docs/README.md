# 文書と作業の入口

作業に合う行を選び、ガイドライン、前提となる仕様、テンプレートの順に読みます。
この入口と配下の文書だけで、仕様の作成から実装・変更まで進められます。

## 作業別の参照先

| 作業 | 読むガイドライン | 前提となる仕様 | テンプレート |
| --- | --- | --- | --- |
| プロダクトの目的・利用者・価値を決める | [Productを書く](guidelines/product.md) | 対象の既存Productがあれば読む | [product.md](templates/product.md) |
| 機能の振る舞い・受け入れ条件を決める | [Requirementsを書く](guidelines/requirements.md)、[シナリオ](guidelines/scenarios.md) | 対象Product、変更なら既存Requirements・Design | [requirements.md](templates/requirements.md) |
| 要件を実現する方法を決める | [Designを書く](guidelines/design.md) | 対象Product・Requirements、既存Design | [design.md](templates/design.md) |
| 実装・検証・既存文書の変更を行う | [文書の責務と更新](guidelines/documents.md)、[実装と検証](guidelines/agent.md#実装と検証) | 対象Product・Requirements・Design | 変更対象の文書のテンプレート |

共通の判断基準は[文書の責務と更新](guidelines/documents.md)、対話や調査の進め方は[エージェントの作業原則](guidelines/agent.md)に記述しています。
ガイドラインに判断基準と作業の進め方を、テンプレートに文書の構造をまとめています。

## sdev自身の仕様

| 文書 | 参照先 |
| --- | --- |
| Product | [sdev](products/sdev/product.md) |
| Requirements | [ガイドラインとテンプレートの参照](products/sdev/features/guideline-reference/requirements.md) |
| Design | [ガイドライン参照の設計](products/sdev/features/guideline-reference/design.md) |

文書の配置と関係は[文書の責務と更新](guidelines/documents.md)に従います。
