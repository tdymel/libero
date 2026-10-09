use super::{show, state};
use crate::components::form::rich_text_editor::model::{Builtin, Commands, Editor};
use crate::hooks::UndoHistory;

fn editor(markdown: &str) -> Editor {
    Editor::with_history(UndoHistory::new(state(markdown)))
}

/// Types `typed` one char at a time into `before` and checks the result.
#[track_caller]
fn typing(before: &str, typed: &str, after: &str) {
    let mut editor = editor(before);
    for c in typed.chars() {
        editor.type_text(&c.to_string());
    }
    assert_eq!(show(editor.state()), after, "from {before:?}");
}

#[track_caller]
fn pasting(before: &str, text: &str, after: &str) {
    let mut state = state(before);
    state.paste(text, false);
    assert_eq!(show(&state), after, "from {before:?}");
}

#[test]
fn a_url_followed_by_a_space_becomes_a_link() {
    typing(
        "see https://a.example/x?y=1|",
        " ",
        "see [https://a.example/x?y=1](https://a.example/x?y=1) |",
    );
    typing(
        "mail mailto:me@a.example|",
        " ",
        "mail [mailto:me@a.example](mailto:me@a.example) |",
    );
    typing(
        "http://a.example|",
        " ",
        "[http://a.example](http://a.example) |",
    );
}

#[test]
fn text_typed_after_an_autolink_is_plain() {
    typing(
        "https://a.example|",
        " more",
        "[https://a.example](https://a.example) more|",
    );
}

#[test]
fn the_punctuation_that_ends_a_sentence_stays_outside_the_link() {
    typing(
        "go to https://a.example.|",
        " ",
        "go to [https://a.example](https://a.example). |",
    );
    typing("(https://a.example)|", " ", "(https://a.example) |");
    typing(
        "https://a.example/(x)|",
        " ",
        "[https://a.example/(x)](https://a.example/\\(x\\)) |",
    );
}

#[test]
fn the_scheme_may_be_written_in_capitals() {
    typing(
        "HTTPS://A.example|",
        " ",
        "[HTTPS://A.example](https://A.example) |",
    );
}

#[test]
fn only_the_allowed_schemes_link() {
    for word in [
        "javascript:alert(1)",
        "ftp://a.example",
        "tel:+4912345",
        "www.a.example",
        "a.example",
        "https://",
        "https:///x",
        "mailto:",
        "mailto:nobody",
        "xhttps://a.example",
    ] {
        let mut editor = editor(&format!("{word}|"));
        editor.type_text(" ");
        let marked = editor.doc().blocks[0]
            .inlines()
            .iter()
            .any(|inline| matches!(inline, crate::components::form::rich_text_editor::model::Inline::Text { marks, .. } if !marks.is_empty()));
        assert!(!marked, "{word:?} became a link");
    }
}

#[test]
fn only_a_whole_word_links() {
    typing(
        "a https://a.example|",
        " ",
        "a [https://a.example](https://a.example) |",
    );
    typing("see(https://a.example|", " ", "see(https://a.example |");
}

#[test]
fn code_and_links_are_left_alone() {
    typing(
        "```\nhttps://a.example|\n```",
        " ",
        "```\nhttps://a.example |\n```",
    );
    typing("`https://a.example|`", " ", "`https://a.example |`");
    typing(
        "[https://a.example|](https://b.example)",
        " ",
        "[https://a.example](https://b.example) |",
    );
}

#[test]
fn enter_after_a_url_links_it_and_splits() {
    let mut editor = editor("see https://a.example|");
    assert!(editor.run(&Commands::builtin(), Builtin::SplitBlock));
    assert_eq!(
        show(editor.state()),
        "see [https://a.example](https://a.example)\n\n|"
    );
    assert!(editor.undo());
    assert_eq!(show(editor.state()), "see https://a.example|");
}

#[test]
fn undo_brings_the_plain_url_back() {
    let mut editor = editor("https://a.example|");
    editor.type_text(" ");
    assert!(editor.undo());
    assert_eq!(show(editor.state()), "https://a.example |");
}

#[test]
fn a_typed_markdown_link_becomes_a_link() {
    typing(
        "a [text](https://a.example|",
        ")",
        "a [text|](https://a.example)",
    );
    typing(
        "[**bold** text](https://a.example/(x)|",
        ")",
        "[**bold** text|](https://a.example/\\(x\\))",
    );
    typing(
        "[text](https://a.example \"The title\"|",
        ")",
        "[text|](https://a.example \"The title\")",
    );
}

#[test]
fn text_typed_after_a_markdown_link_is_plain() {
    typing(
        "[text](https://a.example|",
        ") more",
        "[text](https://a.example) more|",
    );
}

#[test]
fn a_typed_markdown_link_drops_its_syntax() {
    let mut editor = editor("[text](https://a.example|");
    editor.type_text(")");
    assert_eq!(editor.doc().blocks[0].text(), "text");
    assert!(editor.undo());
    assert_eq!(editor.doc().blocks[0].text(), "[text](https://a.example)");
}

#[test]
fn an_unsafe_empty_or_image_link_stays_text() {
    for typed in [
        "[x](javascript:alert(1)",
        "[](https://a.example",
        "![x](https://a.example",
        "[x] (https://a.example",
        "[x](",
    ] {
        let mut editor = editor(&format!("{typed}|"));
        editor.type_text(")");
        assert_eq!(
            editor.doc().blocks[0].text(),
            format!("{typed})"),
            "{typed:?}"
        );
    }
}

#[test]
fn a_bare_url_pasted_at_the_caret_is_a_link() {
    pasting(
        "see |",
        "https://a.example/x",
        "see [https://a.example/x|](https://a.example/x)",
    );
    pasting(
        "|",
        "  mailto:me@a.example\n",
        "[mailto:me@a.example|](mailto:me@a.example)",
    );
}

#[test]
fn a_bare_url_pasted_over_a_selection_links_it() {
    pasting(
        "a |bc| d",
        "https://a.example",
        "a |[bc|](https://a.example) d",
    );
}

#[test]
fn other_pastes_stay_text() {
    for text in [
        "see https://a.example",
        "https://a.example and more",
        "https://a.example\nhttps://b.example",
        "javascript:alert(1)",
        "www.a.example",
    ] {
        let mut state = state("|");
        state.paste(text, false);
        let linked = state.doc.to_markdown().contains("](");
        assert!(!linked, "{text:?} made a link");
    }
}

#[test]
fn a_url_pasted_into_code_is_raw() {
    pasting(
        "```\n|\n```",
        "https://a.example",
        "```\nhttps://a.example|\n```",
    );
}
