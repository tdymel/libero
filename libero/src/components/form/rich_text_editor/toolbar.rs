//! The editor's toolbar buttons and how many of them a narrow bar moves into its More menu.

use dioxus::prelude::*;
use pictogram_core::Svg as SvgData;
use pictogram_icons_lucide as lucide;

use super::model::{Builtin, CommandName, EditorState};
use crate::{context::IconSlot, localization::RichTextEditorLabels};

/// A button of yours in the editor's toolbar, after the block buttons; it runs the command
/// `command` like a built-in button, and its tooltip shows the keymap's chord for it.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{RichTextEditor, rich_text::{Commands, RichTextTool}};
/// # fn app() -> Element {
/// let mut commands = Commands::default();
/// commands.register("shout", |state| state.insert_text("!"));
/// rsx! {
///     RichTextEditor {
///         label: "Notes",
///         commands,
///         tools: vec![RichTextTool::new("shout", "Shout", rsx! { "!" })],
///     }
/// }
/// # }
/// ```
#[derive(Clone)]
pub struct RichTextTool {
    pub(super) command: CommandName,
    pub(super) label: String,
    pub(super) icon: Element,
    pub(super) active: Option<fn(&EditorState) -> bool>,
}

impl RichTextTool {
    /// `label` names the icon-only button for screen readers and its tooltip.
    pub fn new(command: impl Into<CommandName>, label: impl Into<String>, icon: Element) -> Self {
        Self {
            command: command.into(),
            label: label.into(),
            icon,
            active: None,
        }
    }

    /// Makes it a toggle: pressed (`aria-pressed`) while `active` holds for the state.
    pub fn active(mut self, active: fn(&EditorState) -> bool) -> Self {
        self.active = Some(active);
        self
    }
}

impl PartialEq for RichTextTool {
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
            && self.label == other.label
            && self.icon == other.icon
            && match (self.active, other.active) {
                (Some(a), Some(b)) => std::ptr::fn_addr_eq(a, b),
                (a, b) => a.is_none() && b.is_none(),
            }
    }
}

impl std::fmt::Debug for RichTextTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RichTextTool")
            .field("command", &self.command)
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}

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
    /// The code block's language button, `0.0` while the caret is outside one.
    pub language: f64,
}

impl Default for Metrics {
    /// The default theme at a 16px root, until measured.
    fn default() -> Self {
        Self {
            icon: 42.0,
            gap: 4.0,
            block_type: 136.0,
            language: 0.0,
        }
    }
}

/// How many of [`OVERFLOW`] leave a bar `width` px wide next to `extra` caller tools,
/// which never move; below the smallest bar it wraps.
pub(super) fn hidden(width: f64, metrics: Metrics, extra: usize) -> usize {
    (0..=OVERFLOW.len())
        .find(|&count| needed(count, metrics, extra) <= width)
        .unwrap_or(OVERFLOW.len())
}

fn needed(hidden: usize, metrics: Metrics, extra: usize) -> f64 {
    let Metrics {
        icon,
        gap,
        block_type,
        language,
    } = metrics;
    let language = if language > 0.0 { language + gap } else { 0.0 };
    let icons = 12 - hidden + usize::from(hidden > 0) + extra;
    let history = OVERFLOW[..hidden]
        .iter()
        .filter(|builtin| matches!(builtin, Builtin::Undo | Builtin::Redo))
        .count()
        < 2;
    let separators = 1 + usize::from(history) + usize::from(extra > 0);
    icons as f64 * (icon + gap)
        + block_type
        + gap
        + language
        + separators as f64 * (SEPARATOR + gap)
        - gap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_bar_hides_nothing_and_a_phone_keeps_bold_italic_and_link() {
        let metrics = Metrics::default();
        assert_eq!(hidden(800.0, metrics, 0), 0);
        assert_eq!(hidden(342.0, metrics, 0), 9);
        assert_eq!(hidden(100.0, metrics, 0), OVERFLOW.len());
    }

    #[test]
    fn hiding_more_never_needs_more_room() {
        let metrics = Metrics::default();
        // One hidden button only trades places with the More trigger.
        assert_eq!(needed(1, metrics, 0), needed(0, metrics, 0));
        for count in 2..=OVERFLOW.len() {
            assert!(
                needed(count, metrics, 0) < needed(count - 1, metrics, 0),
                "{count}"
            );
        }
    }

    #[test]
    fn larger_icons_or_a_longer_label_hide_more() {
        let width = 600.0;
        let base = hidden(width, Metrics::default(), 0);
        let big = Metrics {
            icon: 50.0,
            ..Metrics::default()
        };
        let long = Metrics {
            block_type: 200.0,
            ..Metrics::default()
        };
        assert!(hidden(width, big, 0) > base);
        assert!(hidden(width, long, 0) > base);
    }

    #[test]
    fn caller_tools_hide_more_builtins() {
        let metrics = Metrics::default();
        assert!(hidden(600.0, metrics, 2) > hidden(600.0, metrics, 0));
    }

    /// Todo 1350: the language button of a code block takes room as well.
    #[test]
    fn the_language_button_hides_more() {
        let width = 600.0;
        let coded = Metrics {
            language: 90.0,
            ..Metrics::default()
        };
        assert!(hidden(width, coded, 0) > hidden(width, Metrics::default(), 0));
    }
}
