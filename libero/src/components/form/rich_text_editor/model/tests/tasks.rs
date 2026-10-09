use super::{check, md, show, state};
use crate::components::form::rich_text_editor::model::{
    BlockKind, Builtin, Chord, Doc, Editor, EditorState, KeyPress, Keymap,
};
use crate::hooks::UndoHistory;

fn typed(before: &str, text: &str) -> String {
    let mut editor = Editor::with_history(UndoHistory::new(state(before)));
    for c in text.chars() {
        editor.type_text(&c.to_string());
    }
    show(editor.state())
}

#[test]
fn task_items_read_and_write_markdown() {
    for markdown in [
        "- [ ] open\n- [x] done\n- plain",
        "1. [x] first\n2. [ ] second",
        "- [ ] a\n  - [x] nested",
        "- [ ]",
        "- [x] two\n\n  paragraphs",
    ] {
        assert_eq!(md(&Doc::from_markdown(markdown)), markdown);
    }
    assert_eq!(md(&Doc::from_markdown("- [X] upper")), "- [x] upper");
    let doc = Doc::from_markdown("- [x] a\n- b");
    let items = doc.blocks[0].children();
    assert_eq!(items[0].kind, BlockKind::task_item(true));
    assert_eq!(items[1].kind, BlockKind::list_item());
}

#[test]
fn a_box_without_its_space_or_outside_a_list_stays_text() {
    let doc = Doc::from_markdown("- [ ]a\n\n[ ] para");
    assert_eq!(doc.blocks[0].children()[0].kind, BlockKind::list_item());
    assert_eq!(doc.blocks[0].children()[0].children()[0].text(), "[ ]a");
    assert_eq!(doc.blocks[1].text(), "[ ] para");
    // A plain item whose text starts like a box is escaped, so it reads back plain.
    let back = Doc::from_markdown(&doc.to_markdown());
    assert_eq!(back.rekeyed(), doc.rekeyed());
}

#[test]
fn typing_a_box_at_an_item_start_makes_a_task() {
    assert_eq!(typed("|", "- [ ] "), "- [ ] |");
    assert_eq!(typed("- |", "[x] "), "- [x] |");
    assert_eq!(typed("1. |", "[X] "), "1. [x] |");
    assert_eq!(typed("- [ ] |", "[x] "), "- [x] |");
}

#[test]
fn a_box_typed_elsewhere_stays_text() {
    let plain = typed("|", "[ ] ");
    assert_eq!(state(&plain).doc.blocks[0].kind, BlockKind::Paragraph);
    let later = typed("- a|", " [ ] ");
    assert_eq!(
        state(&later).doc.blocks[0].children()[0].kind,
        BlockKind::list_item()
    );
}

#[test]
fn enter_after_a_task_makes_an_open_task_and_twice_leaves_the_list() {
    check("- [x] a|", EditorState::split_block, "- [x] a\n- [ ] |");
    check("- [x] a|b", EditorState::split_block, "- [x] a\n- [ ] |b");
    check("- [ ] a\n- [ ] |", EditorState::split_block, "- [ ] a\n\n|");
    check("- a|", EditorState::split_block, "- a\n- |");
}

#[test]
fn toggle_task_list_converts_items_both_ways() {
    check("- a|", EditorState::toggle_task_list, "- [ ] a|");
    check("- [ ] a|", EditorState::toggle_task_list, "- a|");
    check("a|", EditorState::toggle_task_list, "- [ ] a|");
    // Mixed: the plain items join the tasks, a checked one stays checked.
    check(
        "- [x] |a\n- b|",
        EditorState::toggle_task_list,
        "- [x] |a\n- [ ] b|",
    );
    check(
        "- [x] |a\n  - [ ] b|",
        EditorState::toggle_task_list,
        "- |a\n  - b|",
    );
}

#[test]
fn toggle_task_checks_the_selected_tasks() {
    check("- [ ] a|", EditorState::toggle_task, "- [x] a|");
    check("- [x] a|", EditorState::toggle_task, "- [ ] a|");
    check(
        "- [x] |a\n- [ ] b|\n- c",
        EditorState::toggle_task,
        "- [x] |a\n- [x] b|\n- c",
    );
    let mut plain = state("- a|");
    assert!(!plain.toggle_task());
    let mut outside = state("a|");
    assert!(!outside.toggle_task());
}

#[test]
fn task_state_reads_the_caret_item() {
    assert_eq!(state("- [x] a|").task_state(), Some(true));
    assert_eq!(state("- [ ] a\n  - b|").task_state(), None);
    assert_eq!(state("- [ ] a|\n  - b").task_state(), Some(false));
    assert_eq!(state("a|").task_state(), None);
}

#[test]
fn task_commands_have_their_default_chords() {
    let keymap = Keymap::default();
    let shifted_nine = KeyPress {
        ctrl: true,
        shift: true,
        code: "Digit9".into(),
        ..KeyPress::new("(")
    };
    assert_eq!(
        keymap.command_for(&shifted_nine, false),
        Some(&Builtin::TaskList.into())
    );
    let chord = Chord::parse("Mod+Alt+Enter").unwrap();
    let press = KeyPress {
        ctrl: true,
        alt: true,
        ..KeyPress::new(chord.key.clone())
    };
    assert_eq!(
        keymap.command_for(&press, false),
        Some(&Builtin::ToggleTask.into())
    );
}
