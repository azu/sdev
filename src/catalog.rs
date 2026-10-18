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
        title: "When exploring a new product or changing its purpose: clarify users, problems, and value through dialogue",
        title_ja: "新しいプロダクトの構想や目的の変更を考えるとき、対話で利用者・課題・提供価値を具体化する",
        source: "docs/guidelines/product.md",
    },
    Topic {
        operation: "help",
        name: "requirements",
        title: "When adding or changing a feature, including vague requests: read existing specifications and interview the user about behavior, scope, and acceptance criteria",
        title_ja: "機能の追加・変更や曖昧な要望を整理するとき、既存仕様の確認と聞き取りで振る舞い・範囲・受け入れ条件を決める",
        source: "docs/guidelines/requirements.md",
    },
    Topic {
        operation: "help",
        name: "design",
        title: "When choosing how to meet requirements: investigate existing architecture, compare alternatives, and explain design decisions",
        title_ja: "要件を実現する方法を決めるとき、既存構成と代替案を調べ、設計と判断理由をまとめる",
        source: "docs/guidelines/design.md",
    },
    Topic {
        operation: "help",
        name: "documents",
        title: "When deciding where information belongs or which documents to update: check responsibilities, placement, links, and template selection",
        title_ja: "情報の置き場所や更新する文書に迷うとき、文書の責務・配置・参照関係・テンプレートの選び方を確認する",
        source: "docs/guidelines/documents.md",
    },
    Topic {
        operation: "help",
        name: "workflow",
        title: "When discussing an unclear idea, researching, implementing, or verifying: clarify purpose and scope through dialogue and proceed within the user's request",
        title_ja: "曖昧なアイデアの相談や調査・実装・検証を進めるとき、対話で目的と範囲を整理し、依頼された範囲で進める",
        source: "docs/guidelines/agent.md",
    },
    Topic {
        operation: "help",
        name: "scenarios",
        title: "When writing or checking acceptance criteria: express observable outcomes for normal, failure, and boundary scenarios",
        title_ja: "受け入れ条件を書く・見直すとき、正常・失敗・境界の条件と観測できる結果をシナリオにする",
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
