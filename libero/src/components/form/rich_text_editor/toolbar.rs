//! The editor's toolbar buttons and how many of them a narrow bar moves into its More menu.

use pictogram_core::Svg as SvgData;
use pictogram_icons_lucide as lucide;

use super::model::Builtin;
use crate::{context::IconSlot, localization::RichTextEditorLabels};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Group {
    Marks,
    Blocks,
    History,
}

/// One toolbar button, or one More menu item once the bar is too narrow for it.
#[derive(Clone, Copy)]
pub(super) struct Tool {
    pub builtin: Builtin,
    pub label: &'static str,
    pub slot: IconSlot,
    pub icon: SvgData,
    pub group: Group,
    pub selected: Option<bool>,
    pub disabled: bool,
}

/// The buttons in bar order; block type sits between the marks and the block toggles.
pub(super) fn tools(words: &RichTextEditorLabels) -> [Tool; 12] {
    use {Builtin as B, Group as G, IconSlot as S};
    [
        (
            B::Bold,
            words.bold,
            S::Bold,
            lucide::bold::outlined,
            G::Marks,
        ),
        (
            B::Italic,
            words.italic,
            S::Italic,
            lucide::italic::outlined,
            G::Marks,
        ),
        (
            B::Underline,
            words.underline,
            S::Underline,
            lucide::underline::outlined,
            G::Marks,
        ),
        (
            B::Strike,
            words.strike,
            S::Strikethrough,
            lucide::strikethrough::outlined,
            G::Marks,
        ),
        (
            B::Code,
            words.code,
            S::InlineCode,
            lucide::code::outlined,
            G::Marks,
        ),
        (
            B::Link,
            words.link,
            S::Link,
            lucide::link::outlined,
            G::Marks,
        ),
        (
            B::BulletList,
            words.bullet_list,
            S::BulletList,
            lucide::list::outlined,
            G::Blocks,
        ),
        (
            B::OrderedList,
            words.ordered_list,
            S::OrderedList,
            lucide::list_ordered::outlined,
            G::Blocks,
        ),
        (
            B::Quote,
            words.quote,
            S::Quote,
            lucide::text_quote::outlined,
            G::Blocks,
        ),
        (
            B::CodeBlock,
            words.code_block,
            S::CodeBlock,
            lucide::square_code::outlined,
            G::Blocks,
        ),
        (
            B::Undo,
            words.undo,
            S::Undo,
            lucide::undo_2::outlined,
            G::History,
        ),
        (
            B::Redo,
            words.redo,
            S::Redo,
            lucide::redo_2::outlined,
            G::History,
        ),
    ]
    .map(|(builtin, label, slot, icon, group)| Tool {
        builtin,
        label,
        slot,
        icon,
        group,
        selected: None,
        disabled: false,
    })
}

/// Least needed first: a narrow bar hides a prefix of these. Bold, italic and the
/// block type always stay.
pub(super) const OVERFLOW: [Builtin; 10] = [
    Builtin::CodeBlock,
    Builtin::Quote,
    Builtin::OrderedList,
    Builtin::BulletList,
    Builtin::Code,
    Builtin::Strike,
    Builtin::Underline,
    Builtin::Redo,
    Builtin::Undo,
    Builtin::Link,
];

const SEPARATOR: f64 = 1.0;

/// Rendered px of the bar's parts, measured from Bold, Italic and the block type
/// button, which never overflow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Metrics {
    pub icon: f64,
    pub gap: f64,
    pub block_type: f64,
}

impl Default for Metrics {
    /// The default theme at a 16px root, until measured.
    fn default() -> Self {
        Self {
            icon: 42.0,
            gap: 4.0,
            block_type: 136.0,
        }
    }
}

/// How many of [`OVERFLOW`] leave a bar `width` px wide; below the smallest bar it wraps.
pub(super) fn hidden(width: f64, metrics: Metrics) -> usize {
    (0..=OVERFLOW.len())
        .find(|&count| needed(count, metrics) <= width)
        .unwrap_or(OVERFLOW.len())
}

fn needed(hidden: usize, metrics: Metrics) -> f64 {
    let Metrics {
        icon,
        gap,
        block_type,
    } = metrics;
    let icons = 12 - hidden + usize::from(hidden > 0);
    let history = OVERFLOW[..hidden]
        .iter()
        .filter(|builtin| matches!(builtin, Builtin::Undo | Builtin::Redo))
        .count()
        < 2;
    let separators = 1 + usize::from(history);
    icons as f64 * (icon + gap) + block_type + gap + separators as f64 * (SEPARATOR + gap) - gap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_bar_hides_nothing_and_a_phone_keeps_bold_italic_and_link() {
        let metrics = Metrics::default();
        assert_eq!(hidden(800.0, metrics), 0);
        assert_eq!(hidden(342.0, metrics), 9);
        assert_eq!(hidden(100.0, metrics), OVERFLOW.len());
    }

    #[test]
    fn hiding_more_never_needs_more_room() {
        let metrics = Metrics::default();
        // One hidden button only trades places with the More trigger.
        assert_eq!(needed(1, metrics), needed(0, metrics));
        for count in 2..=OVERFLOW.len() {
            assert!(
                needed(count, metrics) < needed(count - 1, metrics),
                "{count}"
            );
        }
    }

    #[test]
    fn larger_icons_or_a_longer_label_hide_more() {
        let width = 600.0;
        let base = hidden(width, Metrics::default());
        let big = Metrics {
            icon: 50.0,
            ..Metrics::default()
        };
        let long = Metrics {
            block_type: 200.0,
            ..Metrics::default()
        };
        assert!(hidden(width, big) > base);
        assert!(hidden(width, long) > base);
    }
}
