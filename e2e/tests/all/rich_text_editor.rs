//! `RichTextEditor` on the web: every edit goes through the model, the caret follows it,
//! and a controlled parent's late echo neither resets the doc nor moves the caret.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::ImeSetCompositionParams;
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, BACKSPACE, CTRL, ENTER, Key};
use e2e::{Fixture, Viewport};

const KEY_B: Key = Key {
    key: "b",
    code: "KeyB",
    vk: 66,
    text: None,
};

const KEY_Z: Key = Key {
    key: "z",
    code: "KeyZ",
    vk: 90,
    text: None,
};

const EDITOR: &str = "document.querySelector('[role=textbox]')";

async fn eval<T: serde::de::DeserializeOwned>(page: &Page, script: &str) -> T {
    page.evaluate(script).await.unwrap().into_value().unwrap()
}

async fn out(page: &Page) -> String {
    settle(page).await;
    eval(page, "document.getElementById('out').textContent").await
}

async fn settle(page: &Page) {
    page.evaluate("new Promise(r => setTimeout(r, 150))")
        .await
        .unwrap();
}

#[test]
fn typing_enter_backspace_undo_and_shortcuts_edit_the_model() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        settle(page).await;

        let placeholder: String = eval(
            page,
            &format!("getComputedStyle({EDITOR}, '::before').content"),
        )
        .await;
        assert_eq!(placeholder, "\"Write something\"");

        // Char by char: the caret must follow every model change, echoes included.
        keyboard::type_text(page, "hello").await.unwrap();
        assert_eq!(out(page).await, "hello\n");
        let changes: String = eval(page, "document.getElementById('changes').textContent").await;
        assert_eq!(changes, "5");

        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "wörld😀").await.unwrap();
        assert_eq!(out(page).await, "hello\n\nwörld😀\n");

        keyboard::press(page, BACKSPACE).await.unwrap();
        keyboard::press(page, BACKSPACE).await.unwrap();
        assert_eq!(out(page).await, "hello\n\nwörl\n");

        keyboard::press_with(page, KEY_Z, CTRL).await.unwrap();
        assert_ne!(out(page).await, "hello\n\nwörl\n");

        // Markdown typing shortcut, through the model's recognizers.
        page.evaluate(format!(
            "(() => {{ const e = {EDITOR}; getSelection().selectAllChildren(e); getSelection().collapseToEnd(); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "# Title").await.unwrap();
        let markdown = out(page).await;
        assert!(markdown.ends_with("# Title\n"), "{markdown:?}");
        let heading: bool = eval(page, &format!("!!{EDITOR}.querySelector('h1')")).await;
        assert!(heading);

        // Ctrl+B on a selection: the keymap, not the browser, marks it.
        page.evaluate(format!(
            "(() => {{ const h = {EDITOR}.querySelector('h1'); getSelection().selectAllChildren(h); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        keyboard::press_with(page, KEY_B, CTRL).await.unwrap();
        let markdown = out(page).await;
        assert!(markdown.ends_with("# **Title**\n"), "{markdown:?}");

        // A foreign value resets the editor.
        page.evaluate("document.getElementById('replace').click()")
            .await
            .unwrap();
        assert_eq!(out(page).await, "");
        let paragraphs: u32 = eval(
            page,
            &format!("{EDITOR}.querySelectorAll('[data-key]').length"),
        )
        .await;
        assert_eq!(paragraphs, 1);
    });
}

#[test]
fn toolbar_buttons_keep_the_caret_and_composition_lands_in_the_model() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        settle(page).await;

        keyboard::type_text(page, "ab").await.unwrap();
        settle(page).await;
        page.evaluate(
            "document.querySelector('[role=toolbar] button[aria-label=\"Bulleted list\"]').click()",
        )
        .await
        .unwrap();
        assert_eq!(out(page).await, "- ab\n");
        let pressed: String = eval(
            page,
            "document.querySelector('[role=toolbar] button[aria-label=\"Bulleted list\"]').getAttribute('aria-pressed')",
        )
        .await;
        assert_eq!(pressed, "true");

        keyboard::type_text(page, "c").await.unwrap();
        assert_eq!(out(page).await, "- abc\n");

        page.execute(ImeSetCompositionParams::new("に", 1, 1))
            .await
            .unwrap();
        page.execute(ImeSetCompositionParams::new("にほ", 2, 2))
            .await
            .unwrap();
        keyboard::insert_text(page, "日本").await.unwrap();
        settle(page).await;
        keyboard::type_text(page, "d").await.unwrap();
        assert_eq!(out(page).await, "- abc日本d\n");

        // A code block is source with fences while the caret is in it, `CodeBlock` after.
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        page.evaluate(
            "document.querySelector('[role=toolbar] button[aria-label=\"Code block\"]').click()",
        )
        .await
        .unwrap();
        keyboard::type_text(page, "let x = 1;").await.unwrap();
        settle(page).await;
        let source: bool = eval(
            page,
            &format!("!!{EDITOR}.querySelector('[data-code=source] pre')"),
        )
        .await;
        assert!(source);
        assert!(out(page).await.ends_with("```\nlet x = 1;\n```\n"));
        page.evaluate("document.getElementById('replace').focus()")
            .await
            .unwrap();
        settle(page).await;
        let view: bool = eval(
            page,
            &format!("!!{EDITOR}.querySelector('[data-code=view]')"),
        )
        .await;
        assert!(view);
    });
}
