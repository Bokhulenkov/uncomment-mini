use crate::ast::visitor::CommentVisitor;
use crate::languages::SourceLanguage;
use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::Parser;

pub struct Processor {
    parser: Parser,
}

impl Processor {
    pub fn new() -> Self {
        Self { parser: Parser::new() }
    }

    pub fn process_file(&mut self, path: &Path, language: SourceLanguage) -> Result<bool> {
        let source = std::fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
        let output = self.process_content(&source, language)?;
        if output == source {
            return Ok(false);
        }
        std::fs::write(path, output).with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(true)
    }

    fn process_content(&mut self, source: &str, language: SourceLanguage) -> Result<String> {
        self.parser.set_language(&language.grammar())?;
        let tree = self.parser.parse(source, None).context("Failed to parse source")?;
        let mut visitor = CommentVisitor::new(source, matches!(language, SourceLanguage::Swift));
        visitor.visit_node(tree.root_node());
        visitor.extend_keep_blocks();
        visitor.extend_keep_above();

        let mut ranges: Vec<_> = visitor
            .get_comments_to_remove()
            .map(|comment| {
                let start = comment.start_byte;
                let end = comment.end_byte;
                let line_start = source[..start].rfind('\n').map_or(0, |pos| pos + 1);
                let line_end = source[end..].find('\n').map_or(source.len(), |pos| end + pos + 1);
                if source[line_start..start].bytes().all(|b| b.is_ascii_whitespace())
                    && source[end..line_end].bytes().all(|b| b.is_ascii_whitespace())
                {
                    (line_start, line_end, true)
                } else {
                    (start, end, false)
                }
            })
            .collect();
        ranges.extend(
            visitor
                .redundant_keep_markers()
                .into_iter()
                .map(|(start, end)| (start, end, true)),
        );
        ranges.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));

        let mut output = String::with_capacity(source.len());
        let mut cursor = 0;
        for (start, end, remove_verbatim) in ranges {
            if start < cursor {
                continue;
            }
            output.push_str(&source[cursor..start]);
            if !remove_verbatim {
                // A comment is whitespace: keep tokens and preprocessor lines separated.
                if source[start..end].contains('\n') {
                    output.extend(source[start..end].chars().filter(|c| matches!(c, '\r' | '\n')));
                } else if output.chars().next_back().is_some_and(|c| !c.is_whitespace())
                    && source[end..].chars().next().is_some_and(|c| !c.is_whitespace())
                {
                    output.push(' ');
                }
            }
            cursor = end;
        }
        output.push_str(&source[cursor..]);
        Ok(output)
    }
}
