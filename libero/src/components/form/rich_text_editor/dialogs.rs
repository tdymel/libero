//! The editor's own dialogs: the link editor and the shortcut list, plus command labels.

use dioxus::prelude::*;

use super::model::{BlockKind, Builtin, CommandName, EditorState, Href, Keymap, MarkKind};
use crate::{
    components::{
        buttons::Button,
        form::{FieldStatus, TextField},
        layout::Flex,
        overlay::{Dialog, Shortcut},
    },
    hooks::current_localization,
    localization::RichTextEditorLabels,
};

/// What the link dialog opens with: the link at the caret, if any.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LinkArgs {
    pub href: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LinkChoice {
    Set(String),
    Remove,
}

/// The URL field; Save refuses an href the model would refuse, as a field error.
#[component]
pub(crate) fn LinkDialog(args: LinkArgs, onchoose: EventHandler<LinkChoice>) -> Element {
    let words = current_localization().rich_text_editor;
    let on_link = args.href.is_some();
    let mut url = use_signal(|| args.href.clone().unwrap_or_default());
    let mut refused = use_signal(|| false);
    let mut save = move || match Href::parse(&url.peek()) {
        Ok(href) => onchoose.call(LinkChoice::Set(href.as_str().to_string())),
        Err(_) => refused.set(true),
    };
    rsx! {
        Dialog { title: words.link,
            form {
                onsubmit: move |event: FormEvent| {
                    event.prevent_default();
                    save();
                },
                Flex { direction: "column", gap: "md",
                    TextField {
                        label: words.link_url,
                        r#type: "url",
                        "data-autofocus": "",
                        value: url(),
                        status: match refused() {
                            true => FieldStatus::from(words.link_unsafe),
                            false => FieldStatus::Valid,
                        },
                        oninput: move |text| {
                            url.set(text);
                            refused.set(false);
                        },
                    }
                    Flex { gap: "sm", justify: "flex-end",
                        if on_link {
                            Button {
                                variant: "subtle",
                                onclick: move |_| onchoose.call(LinkChoice::Remove),
                                "{words.link_remove}"
                            }
                        }
                        Button { r#type: "submit", "{words.link_save}" }
                    }
                }
            }
        }
    }
}

/// The text a toolbar or shortcut list names `name` by; `None` for plain editing keys.
pub(crate) fn command_label(name: &CommandName, words: &RichTextEditorLabels) -> Option<String> {
    use Builtin as B;
    let Some(builtin) = Builtin::ALL.iter().find(|b| b.name() == name.as_str()) else {
        return Some(name.to_string());
    };
    let heading = |level: u8| Some(words.heading.replace("{level}", &level.to_string()));
    let word = match builtin {
        B::Bold => words.bold,
        B::Italic => words.italic,
        B::Underline => words.underline,
        B::Strike => words.strike,
        B::Code => words.code,
        B::Unlink => words.unlink,
        B::Paragraph => words.paragraph,
        B::Heading1 => return heading(1),
        B::Heading2 => return heading(2),
        B::Heading3 => return heading(3),
        B::Heading4 => return heading(4),
        B::Heading5 => return heading(5),
        B::Heading6 => return heading(6),
        B::CodeBlock => words.code_block,
        B::BulletList => words.bullet_list,
        B::OrderedList => words.ordered_list,
        B::Quote => words.quote,
        B::Indent => words.indent,
        B::Outdent => words.outdent,
        B::Rule => words.rule,
        B::HardBreak => words.hard_break,
        B::Undo => words.undo,
        B::Redo => words.redo,
        B::Link => words.link,
        B::Shortcuts => words.shortcuts,
        _ => return None,
    };
    Some(word.to_string())
}

/// The keymap as `ShortcutHelp` rows, in binding order.
pub(crate) fn shortcut_rows(keymap: &Keymap, words: &RichTextEditorLabels) -> Vec<Shortcut> {
    keymap
        .bindings()
        .iter()
        .filter_map(|(chord, name)| {
            let label = command_label(name, words)?;
            Some(Shortcut::new(chord.to_string().to_lowercase(), label))
        })
        .collect()
}

/// What a screen reader hears after `name` ran from the keyboard: the toggle's new state.
pub(crate) fn announcement(
    name: &CommandName,
    state: &EditorState,
    words: &RichTextEditorLabels,
) -> Option<String> {
    use Builtin as B;
    let builtin = Builtin::ALL.iter().find(|b| b.name() == name.as_str())?;
    let on = match builtin {
        B::Bold => state.is_active(MarkKind::Bold),
        B::Italic => state.is_active(MarkKind::Italic),
        B::Underline => state.is_active(MarkKind::Underline),
        B::Strike => state.is_active(MarkKind::Strike),
        B::Code => state.is_active(MarkKind::Code),
        B::BulletList => state.list_kind() == Some(false),
        B::OrderedList => state.list_kind() == Some(true),
        B::Quote => state.in_quote(),
        B::CodeBlock => state.block_kind().is_code(),
        B::Paragraph
        | B::Heading1
        | B::Heading2
        | B::Heading3
        | B::Heading4
        | B::Heading5
        | B::Heading6 => {
            const HEADINGS: [Builtin; 6] = [
                B::Heading1,
                B::Heading2,
                B::Heading3,
                B::Heading4,
                B::Heading5,
                B::Heading6,
            ];
            let kind = match state.block_kind() {
                BlockKind::Heading { level } => level
                    .checked_sub(1)
                    .and_then(|index| HEADINGS.get(usize::from(index)))
                    .copied()
                    .unwrap_or(B::Paragraph),
                _ => B::Paragraph,
            };
            return command_label(&kind.into(), words);
        }
        _ => return None,
    };
    let label = command_label(name, words)?;
    let template = if on { words.on } else { words.off };
    Some(template.replace("{name}", &label))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::chord_keys;
    use crate::localization::Localization;

    #[test]
    fn every_default_row_is_a_chord_shortcut_help_shows() {
        let words = RichTextEditorLabels::ENGLISH;
        let rows = shortcut_rows(&Keymap::default(), &words);
        assert!(
            rows.iter()
                .any(|row| *row == Shortcut::new("mod+k", "Link"))
        );
        assert!(
            rows.iter()
                .any(|row| *row == Shortcut::new("mod+alt+2", "Heading 2"))
        );
        assert_eq!(rows.len(), Keymap::default().bindings().len() - 4);
        let help = Localization::ENGLISH.shortcut_help;
        for (chord, _) in Keymap::default().bindings() {
            let chord = chord.to_string().to_lowercase();
            assert!(chord_keys(&chord, false, &help).is_some(), "{chord}");
        }
    }

    #[test]
    fn a_toggle_says_its_new_state() {
        use crate::components::form::rich_text_editor::model::{Doc, Position, Selection};
        let words = RichTextEditorLabels::ENGLISH;
        let mut state = EditorState::new(Doc::from_markdown("**ab**"));
        let key = state.doc.blocks[0].key;
        state.select(Selection::range(
            Position::new(key, 0),
            Position::new(key, 2),
        ));
        let said = announcement(&Builtin::Bold.into(), &state, &words);
        assert_eq!(said.as_deref(), Some("Bold on"));
        let said = announcement(&Builtin::Heading2.into(), &state, &words);
        assert_eq!(said.as_deref(), Some("Paragraph"));
        assert_eq!(
            announcement(&Builtin::SplitBlock.into(), &state, &words),
            None
        );
    }
}
