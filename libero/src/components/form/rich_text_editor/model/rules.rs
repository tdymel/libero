//! Markdown shortcuts: typed syntax turns into structure (`# ` a heading, `**x**` bold).

use super::doc::{BlockKind, ContentKind, Inline, slice_inlines};
use super::mark::Mark;
use super::state::{EditorState, Position};

impl EditorState {
    /// Applies the shortcut the text just typed completes. Never in code blocks.
    pub fn apply_input_rules(&mut self) -> bool {
        if !self.selection.is_collapsed() {
            return false;
        }
        let block = self.block(self.caret().block);
        if block.kind.content() != ContentKind::Inline || block.kind.is_code() {
            return false;
        }
        self.block_rule() || self.inline_rule()
    }

    /// A prefix typed at the start of a paragraph, ended by a space.
    fn block_rule(&mut self) -> bool {
        let at = self.caret();
        let block = self.block(at.block);
        if block.kind != BlockKind::Paragraph {
            return false;
        }
        let before = slice_inlines(block.inlines(), 0, at.offset);
        let [Inline::Text { text, .. }] = before.as_slice() else {
            return false;
        };
        let Some(prefix) = text.strip_suffix(' ') else {
            return false;
        };
        let prefix = prefix.to_string();
        let strip = |state: &mut Self| {
            state.delete_range(Position::new(at.block, 0), at);
        };
        if !prefix.is_empty() && prefix.len() <= 6 && prefix.chars().all(|c| c == '#') {
            strip(self);
            return self.set_kind(at.block, BlockKind::heading(prefix.len() as u8));
        }
        match prefix.as_str() {
            "-" | "*" | "+" => {
                strip(self);
                self.toggle_list(false)
            }
            ">" => {
                strip(self);
                self.toggle_quote()
            }
            "---" | "***" | "___" => {
                strip(self);
                self.insert_rule()
            }
            fence if fence.starts_with("```") && !fence[3..].contains('`') => {
                let language = fence[3..].trim().to_string();
                strip(self);
                self.set_kind(at.block, BlockKind::code(language))
            }
            number => {
                let digits = number.strip_suffix(['.', ')']).unwrap_or("");
                let Ok(start) = digits.parse::<u64>() else {
                    return false;
                };
                if digits.len() > 9 {
                    return false;
                }
                strip(self);
                self.toggle_list(true);
                if let Some(list) =
                    self.nearest(at.block, |kind| matches!(kind, BlockKind::List { .. }))
                {
                    self.doc.get_mut(list).expect("the list").kind = BlockKind::ordered_list(start);
                }
                true
            }
        }
    }

    /// A closing delimiter typed after its opener: `` `x` ``, `**x**`, `*x*`, `~~x~~`.
    fn inline_rule(&mut self) -> bool {
        let at = self.caret();
        let text: Vec<char> = self
            .block(at.block)
            .text()
            .chars()
            .take(at.offset)
            .collect();
        for (delimiter, mark) in [
            ("`", Mark::Code),
            ("**", Mark::Bold),
            ("~~", Mark::Strike),
            ("*", Mark::Italic),
        ] {
            let delimiter: Vec<char> = delimiter.chars().collect();
            let len = delimiter.len();
            if text.len() < 2 * len + 1 || !text.ends_with(&delimiter) {
                continue;
            }
            let close = text.len() - len;
            // `*` must not be the tail of `**`.
            if len == 1 && delimiter[0] == '*' && close > 0 && text[close - 1] == '*' {
                continue;
            }
            let Some(open) = (0..close.saturating_sub(len))
                .rev()
                .find(|start| text[*start..*start + len] == delimiter[..])
            else {
                continue;
            };
            let inner = &text[open + len..close];
            let doubled = len == 1
                && delimiter[0] == '*'
                && (open > 0 && text[open - 1] == '*' || inner.first() == Some(&'*'));
            let edge = |c: Option<&char>| c == Some(&delimiter[0]);
            if inner.is_empty()
                || doubled
                || edge(inner.first())
                || edge(inner.last())
                || (delimiter[0] == '`' && inner.contains(&'`'))
                || inner.first().is_some_and(|c| c.is_whitespace())
                || inner.last().is_some_and(|c| c.is_whitespace())
                || inner.contains(&'\u{fffc}')
                || inner.contains(&'\n')
            {
                continue;
            }
            let kind = mark.kind();
            let block = at.block;
            self.delete_range(Position::new(block, close), Position::new(block, at.offset));
            self.delete_range(Position::new(block, open), Position::new(block, open + len));
            let end = close - len;
            let inlines = self.doc.get_mut(block).expect("a leaf").inlines_mut();
            super::doc::map_marks(inlines, open, end, |marks| marks.add(mark.clone()));
            self.set_caret(Position::new(block, end));
            let mut marks = self.current_marks();
            marks.remove(kind);
            self.stored_marks = Some(marks);
            return true;
        }
        false
    }
}
