pub struct Topic {
    pub operation: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub title_ja: &'static str,
    pub source: &'static str,
}

pub const TOPICS: &[Topic] = &[
    Topic {
        operation: "help",
        name: "product",
        title: "Define the product purpose, users, and value",
        title_ja: "プロダクトの目的・利用者・価値を整理する",
        source: "docs/guidelines/product.md",
    },
    Topic {
        operation: "help",
        name: "requirements",
        title: "Specify behavior and acceptance criteria",
        title_ja: "機能の振る舞い・受け入れ条件を決める",
        source: "docs/guidelines/requirements.md",
    },
    Topic {
        operation: "help",
        name: "design",
        title: "Design how to meet the requirements",
        title_ja: "要件を実現する方法を設計する",
        source: "docs/guidelines/design.md",
    },
    Topic {
        operation: "help",
        name: "documents",
        title: "Understand document responsibilities and updates",
        title_ja: "文書の責務・配置・更新を確認する",
        source: "docs/guidelines/documents.md",
    },
    Topic {
        operation: "help",
        name: "workflow",
        title: "Read the dialogue, research, implementation, and verification workflow",
        title_ja: "対話・調査・実装・検証の進め方を読む",
        source: "docs/guidelines/agent.md",
    },
    Topic {
        operation: "help",
        name: "scenarios",
        title: "Write acceptance scenarios",
        title_ja: "受け入れ条件のシナリオを書く",
        source: "docs/guidelines/scenarios.md",
    },
    Topic {
        operation: "template",
        name: "product",
        title: "Product template",
        title_ja: "Productのテンプレート",
        source: "docs/templates/product.md",
    },
    Topic {
        operation: "template",
        name: "requirements",
        title: "Requirements template",
        title_ja: "Requirementsのテンプレート",
        source: "docs/templates/requirements.md",
    },
    Topic {
        operation: "template",
        name: "design",
        title: "Design template",
        title_ja: "Designのテンプレート",
        source: "docs/templates/design.md",
    },
];
