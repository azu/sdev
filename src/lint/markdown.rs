use github_slugger::Slugger;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::collections::BTreeSet;

pub struct Reference {
    pub target: String,
    pub line: usize,
    pub column: usize,
}
pub struct Markdown {
    pub references: Vec<Reference>,
    pub anchors: BTreeSet<String>,
}

pub fn parse(body: &str) -> Markdown {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_GFM
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
    let mut result = Markdown {
        references: Vec::new(),
        anchors: BTreeSet::new(),
    };
    let mut slugger = Slugger::default();
    let mut heading = None::<String>;
    let mut metadata = false;
    let mut starts = vec![0];
    for (i, byte) in body.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(i + 1);
        }
    }
    for (event, range) in Parser::new_ext(body, options).into_offset_iter() {
        match event {
            Event::Start(Tag::MetadataBlock(_)) => metadata = true,
            Event::End(TagEnd::MetadataBlock(_)) => metadata = false,
            _ if metadata => {}
            Event::Start(Tag::Heading { .. }) => heading = Some(String::new()),
            Event::End(TagEnd::Heading(_)) => {
                if let Some(text) = heading.take() {
                    result.anchors.insert(slugger.slug(text.trim()));
                }
            }
            Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
                let line = starts.partition_point(|start| *start <= range.start);
                let column = body[starts[line - 1]..range.start].chars().count() + 1;
                result.references.push(Reference {
                    target: dest_url.into_string(),
                    line,
                    column,
                });
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some(heading) = &mut heading {
                    heading.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(heading) = &mut heading {
                    heading.push(' ');
                }
            }
            _ => {}
        }
    }
    result
}
