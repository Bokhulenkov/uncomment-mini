use crate::rules::preservation::{is_documentation, should_preserve};
use std::collections::HashSet;
use tree_sitter::Node;

pub struct CommentInfo {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_row: usize,
    pub end_row: usize,
    pub should_preserve: bool,
}

impl CommentInfo {
    pub fn content<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start_byte..self.end_byte]
    }
}

pub struct CommentVisitor<'a> {
    source: &'a str,
    comments: Vec<CommentInfo>,
    extended: HashSet<usize>,
    is_swift: bool,
}

impl<'a> CommentVisitor<'a> {
    pub fn new(source: &'a str, is_swift: bool) -> Self {
        Self {
            source,
            comments: Vec::new(),
            extended: HashSet::new(),
            is_swift,
        }
    }

    pub fn visit_node(&mut self, node: Node) {
        if matches!(node.kind(), "comment" | "multiline_comment") {
            let start = node.start_byte();
            let line_start = self.source[..start].rfind('\n').map_or(0, |pos| pos + 1);
            let content = &self.source[start..node.end_byte()];
            self.comments.push(CommentInfo {
                start_byte: start,
                end_byte: node.end_byte(),
                start_row: node.start_position().row,
                end_row: node.end_position().row,
                should_preserve: should_preserve(content, &self.source[line_start..start], self.is_swift),
            });
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child);
        }
    }

    pub fn get_comments_to_remove(&self) -> impl Iterator<Item = &CommentInfo> {
        self.comments.iter().filter(|comment| !comment.should_preserve)
    }

    pub fn extend_keep_blocks(&mut self) {
        let mut indices: Vec<usize> = (0..self.comments.len())
            .filter(|&i| self.is_standalone_single_line(&self.comments[i]))
            .collect();
        indices.sort_by_key(|&i| self.comments[i].start_byte);

        let mut run_start = 0;
        while run_start < indices.len() {
            let mut run_end = run_start;
            while run_end + 1 < indices.len()
                && self.comments[indices[run_end + 1]].start_row == self.comments[indices[run_end]].start_row + 1
            {
                run_end += 1;
            }

            let has_keep = indices[run_start..=run_end]
                .iter()
                .any(|&i| self.comments[i].content(self.source).contains("~keep"));
            if has_keep {
                for &i in &indices[run_start..=run_end] {
                    if !self.comments[i].content(self.source).contains("~keep") {
                        self.extended.insert(i);
                    }
                    self.comments[i].should_preserve = true;
                }
            }

            run_start = run_end + 1;
        }
    }

    pub fn extend_keep_above(&mut self) {
        let mut indices: Vec<usize> = (0..self.comments.len()).collect();
        indices.sort_by_key(|&i| (self.comments[i].start_byte, self.comments[i].end_byte));

        for position in 0..indices.len() {
            let marker = indices[position];
            if !self.is_keep_marker_line(&self.comments[marker]) {
                continue;
            }

            let mut previous_end = self.comments[marker].end_byte;
            for &next in &indices[position + 1..] {
                if !Self::gap_is_adjacent(
                    &self.source[previous_end.min(self.comments[next].start_byte)..self.comments[next].start_byte],
                ) {
                    break;
                }
                if !self.comments[next].should_preserve {
                    self.extended.insert(next);
                }
                self.comments[next].should_preserve = true;
                previous_end = previous_end.max(self.comments[next].end_byte);
            }
        }
    }

    fn is_keep_marker_line(&self, comment: &CommentInfo) -> bool {
        let content = comment.content(self.source);
        !is_documentation(content) && self.is_standalone(comment) && content.contains("~keep")
    }

    fn gap_is_adjacent(gap: &str) -> bool {
        gap.bytes().all(|byte| byte.is_ascii_whitespace()) && gap.bytes().filter(|&b| b == b'\n').count() <= 1
    }

    #[must_use]
    pub fn redundant_keep_markers(&self) -> Vec<(usize, usize)> {
        let mut ranges: Vec<(usize, usize)> = Vec::new();
        for (index, comment) in self.comments.iter().enumerate() {
            let content = comment.content(self.source);
            if !comment.should_preserve || !is_documentation(content) {
                continue;
            }
            if self.run_members(index).any(|member| self.extended.contains(&member)) {
                continue;
            }
            if !content.contains("~keep") {
                continue;
            }
            for (offset, _) in content.match_indices("~keep") {
                let start = comment.start_byte + offset;
                if Self::is_marker_occurrence(content, offset) {
                    ranges.push(Self::widen_marker(self.source, start, start + "~keep".len()));
                }
            }
        }

        ranges.sort_unstable();
        ranges.dedup();
        ranges
    }

    fn run_members(&self, index: usize) -> impl Iterator<Item = usize> + '_ {
        let anchor = &self.comments[index];
        let (low, high) = (anchor.start_row.saturating_sub(1), anchor.end_row + 1);
        (0..self.comments.len())
            .filter(move |&other| self.comments[other].start_row >= low && self.comments[other].start_row <= high)
    }

    fn is_marker_occurrence(content: &str, offset: usize) -> bool {
        let end = offset + "~keep".len();
        let preceded_by_word = content[..offset]
            .chars()
            .next_back()
            .is_some_and(|c| !c.is_whitespace());
        let followed_by_word = content[end..].chars().next().is_some_and(|c| !c.is_whitespace());
        if preceded_by_word || followed_by_word {
            return false;
        }

        let line_start = content[..offset].rfind('\n').map_or(0, |pos| pos + 1);
        let line_end = content[offset..].find('\n').map_or(content.len(), |pos| offset + pos);
        let line = &content[line_start..line_end];
        if line.contains('`') {
            return false;
        }

        let body = line.trim_start();
        let body_offset = line.len() - body.len();
        let text = body.trim_start_matches(['/', '!', '*', '#']);
        let text_start = line_start + body_offset + (body.len() - text.len());
        if text_start > offset {
            return false;
        }
        let before = &content[text_start..offset];
        !before.contains("//") && !before.contains('#')
    }

    fn widen_marker(source: &str, start: usize, end: usize) -> (usize, usize) {
        if source[end..].starts_with(' ') {
            return (start, end + 1);
        }
        if source[..start].ends_with(' ') {
            return (start - 1, end);
        }
        (start, end)
    }

    fn is_standalone(&self, comment: &CommentInfo) -> bool {
        let line_start = self.source[..comment.start_byte].rfind('\n').map_or(0, |pos| pos + 1);
        self.source[line_start..comment.start_byte]
            .bytes()
            .all(|byte| byte.is_ascii_whitespace())
    }

    fn is_standalone_single_line(&self, comment: &CommentInfo) -> bool {
        comment.start_row == comment.end_row && self.is_standalone(comment)
    }
}
