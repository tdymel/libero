use super::{check, md, show, state};
use crate::components::form::rich_text_editor::model::{
    Block, BlockKind, Doc, EditorState, Inline, Mark, MarkKind, NodeRegistry, NodeSpec, Position,
    Selection,
};

#[test]
fn typing_inserts_at_the_caret() {
    check("ab|c", |s| s.insert_text("X"), "abX|c");
    check("|", |s| s.insert_text("hi"), "hi|");
}

#[test]
fn typing_replaces_the_selection() {
    check("a|bc|d", |s| s.insert_text("X"), "aX|d");
}

#[test]
fn typing_takes_the_marks_before_the_caret() {
    check(
        "**bold|** plain",
        |s| s.insert_text("er"),
        "**bolder|** plain",
    );
    check(
        "plain |**bold**",
        |s| s.insert_text("x"),
        "plain x|**bold**",
    );
    check("**|bold**", |s| s.insert_text("x"), "**x|bold**");
}

#[test]
fn typing_at_the_end_of_a_link_leaves_the_link() {
    check(
        "[link|](https://a.example)",
        |s| s.insert_text("x"),
        "[link](https://a.example)x|",
    );
    check(
        "[li|nk](https://a.example)",
        |s| s.insert_text("x"),
        "[lix|nk](https://a.example)",
    );
}

#[test]
fn pasted_lines_become_paragraphs() {
    check("a|b", |s| s.insert_text("1\n2\n3"), "a1\n\n2\n\n3|b");
}

#[test]
fn newlines_stay_text_in_code() {
    check(
        "```\na|b\n```",
        |s| s.insert_text("1\n2"),
        "```\na1\n2|b\n```",
    );
}

#[test]
fn enter_splits_the_block() {
    check("ab|cd", EditorState::split_block, "ab\n\n|cd");
    check("# Ti|tle", EditorState::split_block, "# Ti\n\n# |tle");
}

#[test]
fn enter_at_the_end_of_a_heading_starts_a_paragraph() {
    let mut s = state("# Title|");
    s.split_block();
    assert_eq!(*s.block_kind(), BlockKind::Paragraph);
}

#[test]
fn enter_in_a_list_item_starts_a_new_item() {
    check(
        "- one|\n- two",
        EditorState::split_block,
        "- one\n- |\n- two",
    );
    check("- o|ne", EditorState::split_block, "- o\n- |ne");
}

#[test]
fn enter_in_an_empty_item_leaves_the_list() {
    let mut s = state("- one|");
    s.split_block();
    s.split_block();
    s.insert_text("out");
    assert_eq!(show(&s), "- one\n\nout|");
}

#[test]
fn enter_in_an_empty_nested_item_outdents_it() {
    let mut s = state("- a\n  - b|");
    s.split_block();
    s.split_block();
    s.insert_text("c");
    assert_eq!(show(&s), "- a\n  - b\n- c|");
}

#[test]
fn enter_in_an_empty_last_quote_line_leaves_the_quote() {
    let mut s = state("> quoted|");
    s.split_block();
    s.split_block();
    s.insert_text("after");
    assert_eq!(show(&s), "> quoted\n\nafter|");
}

#[test]
fn enter_in_code_inserts_a_newline() {
    check(
        "```rust\nfn|\n```",
        EditorState::split_block,
        "```rust\nfn\n|\n```",
    );
}

#[test]
fn backspace_deletes_one_char() {
    check("ab|c", EditorState::delete_backward, "a|c");
    check("|abc", EditorState::delete_backward, "|abc");
}

#[test]
fn backspace_deletes_a_whole_emoji() {
    let family = "👨\u{200D}👩\u{200D}👧";
    let mut s = state(&format!("a{family}|"));
    s.delete_backward();
    assert_eq!(show(&s), "a|");
    let mut s = state("a👍🏽|");
    s.delete_backward();
    assert_eq!(show(&s), "a|");
    let mut s = state("a🇩🇪🇫🇷|");
    s.delete_backward();
    assert_eq!(show(&s), "a🇩🇪|");
    let mut s = state("|e\u{301}x");
    s.delete_forward();
    assert_eq!(show(&s), "|x");
}

#[test]
fn backspace_at_a_block_start_joins_it_to_the_one_before() {
    check("ab\n\n|cd", EditorState::delete_backward, "ab|cd");
    check("# ab\n\n|cd", EditorState::delete_backward, "# ab|cd");
}

#[test]
fn backspace_at_the_start_of_the_doc_turns_a_heading_into_a_paragraph() {
    let mut s = state("# |Title");
    assert!(s.delete_backward());
    assert_eq!(*s.block_kind(), BlockKind::Paragraph);
    assert!(
        !s.delete_backward(),
        "a paragraph at the start has nothing to join"
    );
}

#[test]
fn backspace_at_an_item_start_lifts_it_out_of_the_list() {
    check(
        "- a\n- |b\n- c",
        EditorState::delete_backward,
        "- a\n\n|b\n\n- c",
    );
    check("- a\n  - |b", EditorState::delete_backward, "- a\n- |b");
}

#[test]
fn backspace_at_a_quote_start_lifts_the_line_out() {
    check("> |a\n>\n> b", EditorState::delete_backward, "|a\n\n> b");
}

#[test]
fn backspace_after_a_rule_deletes_it() {
    check("a\n\n---\n\n|b", EditorState::delete_backward, "a\n\n|b");
}

#[test]
fn delete_at_a_block_end_joins_the_next_one() {
    check("ab|\n\ncd", EditorState::delete_forward, "ab|cd");
    check("ab|", EditorState::delete_forward, "ab|");
    check("a|\n\n---\n\nb", EditorState::delete_forward, "a|\n\nb");
}

#[test]
fn a_cross_block_delete_joins_the_ends_and_drops_the_middle() {
    check(
        "a|b\n\nmiddle\n\n# c|d",
        EditorState::delete_selection,
        "a|d",
    );
    check("x|y\n\n- one\n- tw|o", EditorState::delete_selection, "x|o");
    check(
        "> q|uote\n\nout|side",
        EditorState::delete_selection,
        "> q|side",
    );
}

#[test]
fn deleting_everything_leaves_an_empty_paragraph() {
    let mut s = state("|a\n\n- b\n\n---\n\n> c|");
    s.delete_selection();
    assert_eq!(s.doc.blocks.len(), 1);
    assert_eq!(s.doc.blocks[0].kind, BlockKind::Paragraph);
    assert_eq!(show(&s), "|");
}

#[test]
fn select_all_then_type_replaces_the_doc() {
    let mut s = state("# a\n\n- |b\n\n```\nc\n```");
    s.select_all();
    s.insert_text("new");
    assert_eq!(show(&s), "new|");
}

#[test]
fn toggle_mark_on_a_selection() {
    check("a |bc| d", |s| s.toggle_mark(Mark::Bold), "a |**bc|** d");
    let mut s = state("**a|bc|d**");
    s.toggle_mark(Mark::Bold);
    assert_eq!(super::md(&s.doc), "**a**bc**d**");
    check("a|b**c|**", |s| s.toggle_mark(Mark::Bold), "a|**bc|**");
}

#[test]
fn toggle_mark_across_blocks_skips_code() {
    check(
        "a|b\n\n```\ncode\n```\n\ncd|",
        |s| s.toggle_mark(Mark::Italic),
        "a|*b*\n\n```\ncode\n```\n\n*cd|*",
    );
}

#[test]
fn toggle_mark_at_a_caret_marks_the_next_typing() {
    let mut s = state("a|");
    s.toggle_mark(Mark::Bold);
    assert!(s.is_active(MarkKind::Bold));
    s.insert_text("b");
    s.toggle_mark(Mark::Bold);
    s.insert_text("c");
    assert_eq!(show(&s), "a**b**c|");
}

#[test]
fn moving_the_caret_drops_stored_marks() {
    let mut s = state("ab|");
    s.toggle_mark(Mark::Bold);
    let first = s.doc.first_leaf();
    s.set_caret(Position::new(first, 1));
    s.insert_text("x");
    assert_eq!(show(&s), "ax|b");
}

#[test]
fn is_active_reads_the_whole_selection() {
    let s = state("**|ab|**");
    assert!(s.is_active(MarkKind::Bold));
    let s = state("**|a**b|");
    assert!(!s.is_active(MarkKind::Bold));
}

#[test]
fn set_link_on_a_selection_and_at_a_caret() {
    let mut s = state("see |docs| now");
    assert_eq!(s.set_link("https://docs.example"), Ok(true));
    assert_eq!(show(&s), "see |[docs|](https://docs.example) now");

    let mut s = state("[do|cs](https://old.example)");
    s.set_link("https://new.example").unwrap();
    assert_eq!(show(&s), "[do|cs](https://new.example)");

    let mut s = state("go |");
    s.set_link("https://a.example").unwrap();
    assert_eq!(show(&s), "go [https://a.example|](https://a.example)");
}

#[test]
fn set_link_refuses_unsafe_schemes() {
    let mut s = state("|x|");
    assert!(s.set_link("javascript:alert(1)").is_err());
    assert_eq!(show(&s), "|x|");
}

#[test]
fn remove_link_at_a_caret_removes_the_whole_link() {
    check(
        "a [li|nk](https://a.example) b",
        EditorState::remove_link,
        "a li|nk b",
    );
}

#[test]
fn headings_toggle() {
    check("ab|", |s| s.toggle_heading(2), "## ab|");
    check("## ab|", |s| s.toggle_heading(2), "ab|");
    check("# ab|", |s| s.toggle_heading(3), "### ab|");
}

#[test]
fn code_block_toggle_drops_marks_and_keeps_lines() {
    let mut s = state("**a**|\\\nb");
    s.toggle_code_block();
    assert_eq!(show(&s), "```\na|\nb\n```");
    s.toggle_code_block();
    assert_eq!(show(&s), "a|\\\nb");
}

#[test]
fn lists_wrap_switch_and_unwrap() {
    check("a|", |s| s.toggle_list(false), "- a|");
    check("|a\n\nb|", |s| s.toggle_list(true), "1. |a\n2. b|");
    check("- a|", |s| s.toggle_list(true), "1. a|");
    check("- a|", |s| s.toggle_list(false), "a|");
    check(
        "- a\n- |b\n- c",
        |s| s.toggle_list(false),
        "- a\n\n|b\n\n- c",
    );
}

#[test]
fn unwrapping_the_middle_of_an_ordered_list_keeps_the_numbers() {
    check(
        "1. a\n2. |b\n3. c",
        |s| s.toggle_list(true),
        "1. a\n\n|b\n\n3. c",
    );
}

#[test]
fn indent_nests_under_the_previous_item() {
    check("- a\n- |b", EditorState::indent, "- a\n  - |b");
    check(
        "- a\n  - b\n- |c",
        EditorState::indent,
        "- a\n  - b\n  - |c",
    );
    let mut s = state("- |a");
    assert!(!s.indent(), "the first item has nothing to nest under");
    let mut s = state("|a");
    assert!(!s.indent(), "outside a list Tab is not the editor's");
}

#[test]
fn outdent_takes_the_following_items_along() {
    check(
        "- a\n  - |b\n  - c",
        EditorState::outdent,
        "- a\n- |b\n  - c",
    );
}

#[test]
fn quotes_wrap_and_unwrap() {
    check("a|", EditorState::toggle_quote, "> a|");
    check("|a\n\nb|", EditorState::toggle_quote, "> |a\n>\n> b|");
    check("> a|", EditorState::toggle_quote, "a|");
}

#[test]
fn insert_rule_splits_and_moves_on() {
    check("ab|cd", EditorState::insert_rule, "ab\n\n---\n\n|cd");
    check("ab|", EditorState::insert_rule, "ab\n\n---\n\n|");
    check("|", EditorState::insert_rule, "---\n\n|");
}

#[test]
fn hard_break_and_inline_nodes_count_as_one_char() {
    let mut s = state("a|b");
    s.insert_hard_break();
    assert_eq!(show(&s), "a\\\n|b");
    s.insert_inline(Inline::Node {
        name: "mention".into(),
        attrs: Default::default(),
    });
    assert_eq!(s.caret().offset, 3);
    s.delete_backward();
    s.delete_backward();
    assert_eq!(show(&s), "a|b");
}

#[test]
fn a_custom_atom_block_is_inserted_and_deleted_whole() {
    let mut s = state("ab|");
    let block = custom_atom(&mut s.doc);
    let key = block.key;
    s.insert_block(block);
    assert!(s.doc.get(key).is_some());
    s.delete_backward();
    s.delete_backward();
    assert!(s.doc.get(key).is_none());
    assert_eq!(show(&s), "ab|");
}

fn custom_atom(doc: &mut Doc) -> Block {
    doc.leaf(
        BlockKind::Custom {
            name: "embed".into(),
            attrs: Default::default(),
            content: crate::components::form::rich_text_editor::model::CustomContent::Atom,
        },
        Vec::new(),
    )
}

#[test]
fn keys_survive_edits_and_splits_get_new_ones() {
    let mut s = state("ab|cd");
    let key = s.caret().block;
    s.insert_text("x");
    s.toggle_heading(1);
    assert_eq!(s.caret().block, key);
    s.split_block();
    assert_ne!(s.caret().block, key);
    assert!(s.doc.get(key).is_some());
}

#[test]
fn clamp_repairs_a_stale_selection() {
    let s = state("ab|");
    let gone = Position::new(
        crate::components::form::rich_text_editor::model::NodeKey(999),
        5,
    );
    let s = s.with_selection(Selection::caret(gone));
    assert_eq!(s.caret(), Position::new(s.doc.first_leaf(), 0));
    let first = s.doc.first_leaf();
    let s = s.with_selection(Selection::caret(Position::new(first, 99)));
    assert_eq!(s.caret().offset, 2);
}

#[test]
fn selected_text_joins_leaves() {
    assert_eq!(state("a|b\n\n- c\n\nd|e").selected_text(), "b\nc\nd");
}

#[test]
fn selected_doc_cuts_the_end_leaves_and_keeps_containers_and_atoms() {
    let doc = state("a|b **c**\n\n- d\n- e\n\n---\n\nf|g").selected_doc();
    assert_eq!(md(&doc), "b **c**\n\n- d\n- e\n\n---\n\nf");
    assert_eq!(md(&state("- a\n- b|c|d").selected_doc()), "- c");
}

#[test]
fn plain_text_writes_caller_nodes_through_the_registry() {
    let mut doc = Doc::from_markdown("x");
    let leaf = doc.first_leaf();
    *doc.get_mut(leaf).unwrap().inlines_mut() = vec![
        Inline::text("hi "),
        Inline::Node {
            name: "mention".into(),
            attrs: [("user".to_string(), "al".into())].into_iter().collect(),
        },
    ];
    let mut registry = NodeRegistry::default();
    registry
        .register(
            NodeSpec::inline("mention")
                .markdown(|attrs, _| format!("@{}", attrs["user"].as_str().unwrap_or_default())),
        )
        .unwrap();
    assert_eq!(doc.plain_text_with(&registry), "hi @al");
    assert_eq!(doc.plain_text_with(&NodeRegistry::default()), "hi ");
}
