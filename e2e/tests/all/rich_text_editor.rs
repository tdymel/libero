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

/// In the WebView, `adb input text` keys open Gboard composing regions over each word;
/// the model must still hold every letter once, in order, with shortcuts applied.
#[cfg(feature = "android")]
#[test]
fn android() {
    use e2e::driver::{Driver, eventually};

    e2e::android::block_on(async {
        let mut d = e2e::driver::Android::open("/rich-text-editor")
            .await
            .unwrap();
        let page = d.page().clone();
        d.click("[role=textbox]").await.unwrap();
        eventually(&mut d, "the editor to take focus", async |d| {
            d.is_focused("[role=textbox]").await
        })
        .await
        .unwrap();
        // Every input event after dioxus handled it, for the failure message.
        page.evaluate(
            "window.__events = []; for (const t of ['keydown', 'beforeinput', 'compositionstart', 'compositionupdate', 'compositionend']) addEventListener(t, e => __events.push([t, e.inputType ?? e.key ?? '', e.data ?? '', e.ctrlKey ? 'ctrl' : '', e.defaultPrevented ? 'prevented' : ''].join('|'))); document.addEventListener('selectionchange', () => { const s = getSelection(); __events.push(['sel', s.anchorNode?.textContent, s.anchorOffset, s.focusOffset].join('|')); })",
        )
        .await
        .unwrap();
        async fn out_is(d: &mut e2e::driver::Android, page: &Page, want: &str) {
            let mut last = String::new();
            let held = eventually(d, want, async |d| {
                last = d.text("#out").await?;
                Ok(last == want)
            })
            .await;
            let events: Vec<String> =
                eval(page, &format!("[...window.__events, {EDITOR}.innerHTML]")).await;
            assert!(
                held.is_ok(),
                "#out {last:?}, want {want:?}; events {events:#?}"
            );
            eval::<bool>(page, "(window.__events = [], true)").await;
        }

        d.type_text("hello world").await.unwrap();
        out_is(&mut d, &page, "hello world\n").await;
        d.press(ENTER).await.unwrap();
        d.type_text("# Title").await.unwrap();
        out_is(&mut d, &page, "hello world\n\n# Title\n").await;
        d.press(BACKSPACE).await.unwrap();
        out_is(&mut d, &page, "hello world\n\n# Titl\n").await;
        d.press_ctrl(KEY_Z).await.unwrap();
        out_is(&mut d, &page, "hello world\n\n# Title\n").await;
        // The inline rule fires on keys typed into Gboard's composing region.
        d.press(ENTER).await.unwrap();
        d.type_text("a **b** c").await.unwrap();
        out_is(&mut d, &page, "hello world\n\n# Title\n\na **b** c\n").await;
        d.finish("rich text editor").await.unwrap();
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
