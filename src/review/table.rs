use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub(super) struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub highlighted_rows: Vec<usize>,
}

pub(super) struct Row {
    pub line: usize,
    pub end: usize,
    pub text: String,
    pub context: String,
}

pub(super) fn describe(headers: &[String], cells: &[String]) -> String {
    cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            format!(
                "{}: {cell}",
                headers.get(i).map(String::as_str).unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

pub(super) fn parse(text: &str) -> Option<(Table, Vec<Row>)> {
    let mut table = Table {
        headers: vec![],
        rows: vec![],
        highlighted_rows: vec![],
    };
    let mut selections = Vec::new();
    let mut in_table = false;
    let mut in_cell = false;
    let mut cell = String::new();
    let mut cells = Vec::new();
    let mut line = 1;
    let mut end = 1;
    for (event, range) in Parser::new_ext(
        text,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    )
    .into_offset_iter()
    {
        match event {
            Event::Start(Tag::Table(_)) => in_table = true,
            Event::End(TagEnd::Table) => break,
            Event::Start(Tag::TableHead | Tag::TableRow) if in_table => {
                cells.clear();
                line = text[..range.start].bytes().filter(|b| *b == b'\n').count() + 1;
                end = line
                    + text[range]
                        .trim_end()
                        .bytes()
                        .filter(|b| *b == b'\n')
                        .count();
            }
            Event::Start(Tag::TableCell) if in_table => {
                in_cell = true;
                cell.clear();
            }
            Event::End(TagEnd::TableCell) if in_table => {
                in_cell = false;
                cells.push(cell.split_whitespace().collect::<Vec<_>>().join(" "));
            }
            Event::Text(value)
            | Event::Code(value)
            | Event::Html(value)
            | Event::InlineHtml(value)
                if in_cell =>
            {
                cell.push_str(&value)
            }
            Event::SoftBreak | Event::HardBreak if in_cell => cell.push(' '),
            Event::Start(Tag::Image { .. }) if in_cell => cell.push_str("[image: "),
            Event::End(TagEnd::Image) if in_cell => cell.push(']'),
            Event::End(TagEnd::TableHead | TagEnd::TableRow) if in_table => {
                let is_header = table.headers.is_empty();
                if is_header {
                    table.headers = cells.clone();
                } else {
                    table.rows.push(cells.clone());
                }
                selections.push(Row {
                    line,
                    end,
                    text: cells.join(" | "),
                    context: describe(&table.headers, &cells),
                });
            }
            _ => (),
        }
    }
    (!table.headers.is_empty()).then_some((table, selections))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rows_keep_headers_escaped_pipes_and_actual_source_lines() {
        let (table, rows) = parse("| **条件** | 方法 |\n| --- | --- |\n| 登録 | `a\u{5c}|b`を使う |\n| 読む | ~~旧~~新 |\n").unwrap();
        assert_eq!(table.headers, ["条件", "方法"]);
        assert_eq!(table.rows[0], ["登録", "a|bを使う"]);
        assert_eq!(table.rows[1], ["読む", "旧新"]);
        assert_eq!(rows[1].line, 3);
        assert_eq!(rows[1].end, 3);
        assert_eq!(rows[1].text, "登録 | a|bを使う");
        assert_eq!(rows[1].context, "条件: 登録 | 方法: a|bを使う");
    }
}
