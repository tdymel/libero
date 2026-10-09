//! `max_length` and the empty check (todo 2715).

use super::super::{Doc, Editor, EditorState, Record};
use super::{check, show, state};

#[test]
fn plain_len_counts_the_plain_text_chars() {
    for markdown in [
        "",
        "a",
        "héllo\n\nwörld😀",
        "- one\n- two\n\n> q\n\n```\nx\ny\n```\n",
    ] {
        let doc = Doc::from_markdown(markdown);
        assert_eq!(
            doc.plain_len(),
            doc.plain_text().chars().count(),
            "{markdown:?}"
        );
    }
}

#[test]
fn a_doc_with_only_blank_text_is_empty() {
    assert!(Doc::new().is_empty());
    assert!(Doc::from_markdown("  \n\n   ").is_empty());
    assert!(!Doc::from_markdown("a").is_empty());
    assert!(!Doc::from_markdown("---").is_empty());
    assert!(!Doc::from_markdown("```\ncode\n```").is_empty());
}

#[test]
fn typing_is_cut_to_the_room_left() {
    let mut editor = Editor::new(Doc::new());
    editor.set_max_chars(Some(5));
    assert!(editor.type_text("abc"));
    assert!(editor.type_text("defgh"));
    assert_eq!(editor.doc().plain_text(), "abcde");
    assert!(!editor.type_text("f"));
    assert_eq!(editor.doc().plain_text(), "abcde");
}

#[test]
fn a_replaced_selection_makes_room() {
    let mut editor = Editor::new(Doc::from_markdown("abcde"));
    editor.set_max_chars(Some(5));
    assert!(editor.apply(Record::Skip, |state| state.select_all()));
    assert!(editor.type_text("xyz"));
    assert_eq!(editor.doc().plain_text(), "xyz");
}

#[test]
fn a_split_counts_as_one_char_and_a_full_doc_refuses_it() {
    let mut editor = Editor::new(Doc::new());
    editor.set_max_chars(Some(3));
    editor.type_text("ab");
    assert!(editor.apply(Record::Step, |state| state.split_block()));
    assert_eq!(editor.doc().plain_text(), "ab\n");
    assert!(!editor.apply(Record::Step, |state| state.split_block()));
    assert!(!editor.type_text("c"));
}

#[test]
fn a_doc_over_the_limit_can_shrink_but_not_grow() {
    let mut editor = Editor::new(Doc::from_markdown("abcdef"));
    editor.set_max_chars(Some(3));
    assert!(!editor.type_text("x"));
    editor.apply(Record::Skip, |state| state.select_all());
    assert!(editor.type_text("xy"));
    assert_eq!(editor.doc().plain_text(), "xy");
}

#[test]
fn a_paste_is_cut_across_blocks_to_the_limit() {
    let mut state = state("ab|");
    state.paste_within("cd\n\nef\n\ngh", false, Some(6));
    assert_eq!(state.doc.plain_text(), "abcd\ne");
    assert_eq!(show(&state), "abcd\n\ne|");
    let mut unlimited = super::state("ab|");
    unlimited.paste_within("cd\n\nef", false, None);
    assert_eq!(unlimited.doc.plain_text(), "abcd\nef");
}

#[test]
fn cutting_before_the_caret_takes_the_line_break_as_one_char() {
    check("one\n\ntwo|", |state| state.cut_before_caret(5), "on|");
    check("one\n\ntwo|", |state| state.cut_before_caret(4), "one|");
    check("one\n\ntwo|", |state| state.cut_before_caret(3), "one\n\n|");
    check("one\n\ntwo|", |state| state.cut_before_caret(99), "|");
    check("one|", |state| !state.cut_before_caret(0), "one|");
}

/// A send must not take the Enter that turns a typed fence into a code block.
#[test]
fn enter_after_a_typed_fence_opens_it_instead_of_sending() {
    let typed = |text: &str| {
        let mut state = EditorState::default();
        state.insert_text(text);
        state
    };
    assert!(typed("```rust").enter_opens_fence());
    assert!(!typed("``rust").enter_opens_fence());
    assert!(!typed("hello").enter_opens_fence());
}

#[test]
fn text_a_composition_added_is_cut_but_not_the_text_before_it() {
    check("abc|", |state| state.cut_over(2, 1), "ab|");
    check("abc|", |state| !state.cut_over(9, 1), "abc|");
}
