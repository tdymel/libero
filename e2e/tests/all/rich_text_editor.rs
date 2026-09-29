//! `RichTextEditor` on the web: every edit goes through the model, the caret follows it,
//! and a controlled parent's late echo neither resets the doc nor moves the caret.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::ImeSetCompositionParams;
use e2e::browser::{Scheme, block_on};
use e2e::passes::keyboard::{self, BACKSPACE, CTRL, ENTER, Key};
use e2e::passes::{contrast, pointer};
use e2e::{Fixture, Viewport, wait};

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
fn a_caller_toolbar_runs_commands_and_reads_state_through_the_handle() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/handle", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let undo_disabled = "document.getElementById('ext-undo').disabled";
        assert!(eval::<bool>(page, undo_disabled).await);

        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        settle(page).await;
        keyboard::type_text(page, "ab").await.unwrap();
        settle(page).await;
        page.evaluate(format!(
            "(() => {{ getSelection().selectAllChildren({EDITOR}.querySelector('[data-key]')); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        page.evaluate("document.getElementById('ext-bold').click()")
            .await
            .unwrap();
        assert_eq!(out(page).await, "**ab**\n");
        let pressed: String = eval(
            page,
            "document.getElementById('ext-bold').getAttribute('aria-pressed')",
        )
        .await;
        assert_eq!(pressed, "true");
        assert!(!eval::<bool>(page, undo_disabled).await);

        page.evaluate("document.getElementById('ext-undo').click()")
            .await
            .unwrap();
        assert_eq!(out(page).await, "ab\n");
    });
}

const KEY_K: Key = Key {
    key: "k",
    code: "KeyK",
    vk: 75,
    text: None,
};

const SLASH: Key = Key {
    key: "/",
    code: "Slash",
    vk: 191,
    text: None,
};

async fn select_first_leaf(page: &Page) {
    page.evaluate(format!(
        "(() => {{ getSelection().selectAllChildren({EDITOR}.querySelector('[data-key]')); }})()"
    ))
    .await
    .unwrap();
    settle(page).await;
}

#[test]
fn link_dialog_shortcut_help_block_menu_and_announcements() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        settle(page).await;
        keyboard::type_text(page, "ab").await.unwrap();
        settle(page).await;

        // Mod+B from the keyboard is announced.
        select_first_leaf(page).await;
        keyboard::press_with(page, KEY_B, CTRL).await.unwrap();
        settle(page).await;
        let said: String = eval(page, "document.querySelector('[role=status]').textContent").await;
        assert_eq!(said, "Bold on");
        keyboard::press_with(page, KEY_B, CTRL).await.unwrap();
        assert_eq!(out(page).await, "ab\n");

        // Mod+K: an unsafe scheme is a field error, a safe one links the selection.
        select_first_leaf(page).await;
        keyboard::press_with(page, KEY_K, CTRL).await.unwrap();
        settle(page).await;
        let in_field: bool = eval(
            page,
            "document.activeElement === document.querySelector('[role=dialog] input')",
        )
        .await;
        assert!(in_field);
        keyboard::type_text(page, "javascript:alert(1)")
            .await
            .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        settle(page).await;
        let invalid: String = eval(
            page,
            "document.querySelector('[role=dialog] input').getAttribute('aria-invalid')",
        )
        .await;
        assert_eq!(invalid, "true");
        page.evaluate(
            "(() => { const i = document.querySelector('[role=dialog] input'); i.select(); })()",
        )
        .await
        .unwrap();
        keyboard::type_text(page, "https://example.com")
            .await
            .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        assert_eq!(out(page).await, "[ab](https://example.com)\n");
        let back: bool = eval(page, &format!("document.activeElement === {EDITOR}")).await;
        assert!(back, "focus goes back to the text");

        // Mod+/ lists the keymap.
        keyboard::press_with(page, SLASH, CTRL).await.unwrap();
        settle(page).await;
        let listed: String =
            eval(page, "document.querySelector('[role=dialog]').textContent").await;
        assert!(
            listed.contains("Keyboard shortcuts") && listed.contains("Heading 2"),
            "{listed}"
        );
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        settle(page).await;

        // The block-type menu turns the paragraph into a heading.
        page.evaluate("document.querySelector('[role=toolbar] button[aria-haspopup]').click()")
            .await
            .unwrap();
        settle(page).await;
        page.evaluate(
            "[...document.querySelectorAll('[role=menuitemradio]')].find(i => i.textContent.includes('Heading 2')).click()",
        )
        .await
        .unwrap();
        assert_eq!(out(page).await, "## [ab](https://example.com)\n");
        let trigger: String = eval(
            page,
            "document.querySelector('[role=toolbar] button[aria-haspopup]').textContent",
        )
        .await;
        assert_eq!(trigger, "Heading 2");
    });
}

#[test]
fn node_views_draw_caller_nodes_and_keep_their_content_editable() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/nodes", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        settle(page).await;
        let atom: String = eval(
            page,
            &format!(
                "{EDITOR}.querySelector('[data-atom][contenteditable=false] .mention').textContent"
            ),
        )
        .await;
        assert_eq!(atom, "@ada");

        // The callout's text is its own leaf inside the caller's markup.
        page.evaluate(format!(
            "(() => {{ const e = {EDITOR}; e.focus(); const l = e.querySelector('aside.callout [data-key]'); getSelection().selectAllChildren(l); getSelection().collapseToEnd(); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        keyboard::type_text(page, "!").await.unwrap();
        let text = out(page).await;
        assert!(text.ends_with("careful!"), "{text:?}");
        let note: u32 = eval(
            page,
            &format!("{EDITOR}.querySelectorAll('aside.callout > span').length"),
        )
        .await;
        assert_eq!(note, 1, "the view renders once, its children once");
    });
}

/// A German layout sends "/" as Shift+7.
const SLASH_DE: Key = Key {
    key: "/",
    code: "Digit7",
    vk: 55,
    text: None,
};

#[test]
fn arrow_up_enters_a_rendered_code_block_and_shift_slash_opens_help() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        settle(page).await;
        let view: bool = eval(
            page,
            &format!("!!{EDITOR}.querySelector('[data-code=view]')"),
        )
        .await;
        assert!(view);

        // ArrowUp at the leaf below enters the rendered block at its end.
        page.evaluate(format!(
            "(() => {{ const e = {EDITOR}; e.focus(); const l = [...e.querySelectorAll('[data-key]')].pop(); getSelection().selectAllChildren(l); getSelection().collapseToEnd(); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        settle(page).await;
        keyboard::type_text(page, "Z").await.unwrap();
        let markdown = out(page).await;
        assert_eq!(markdown, "above\n\n```\nlet x = 1;Z\n```\n\nbelow\n");

        keyboard::press_with(page, SLASH_DE, CTRL | keyboard::SHIFT)
            .await
            .unwrap();
        settle(page).await;
        let listed: String = eval(
            page,
            "document.querySelector('[role=dialog]')?.textContent ?? ''",
        )
        .await;
        assert!(listed.contains("Keyboard shortcuts"), "{listed:?}");
    });
}

/// Polls `condition` (a JS expression) until it holds; a background page delays `ResizeObserver`.
async fn until(page: &Page, condition: &str) -> bool {
    eval(
        page,
        &format!(
            "new Promise(r => {{ const end = Date.now() + 5000; const tick = () => {{ if ({condition}) r(true); else if (Date.now() > end) r(false); else setTimeout(tick, 50); }}; tick(); }})"
        ),
    )
    .await
}

const MORE: &str = "document.querySelector('[role=toolbar] [aria-label=\"More formatting\"]')";

#[test]
fn a_narrow_toolbar_keeps_one_row_and_moves_the_rest_into_more() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        assert!(until(page, MORE).await, "no More trigger");
        // [no horizontal overflow, distinct item rows]
        let (overflow, rows): (bool, u32) = eval(
            page,
            "(() => { const t = document.querySelector('[role=toolbar]'); \
               const tops = new Set([...t.querySelectorAll('button')].map(b => Math.round(b.getBoundingClientRect().top))); \
               return [t.scrollWidth > t.clientWidth, tops.size]; })()",
        )
        .await;
        assert!(!overflow, "the toolbar runs out of its column");
        assert_eq!(rows, 1, "the toolbar wraps");

        // A hidden toggle runs from the menu, on the caret's block.
        page.evaluate(format!(
            "(() => {{ const e = {EDITOR}; e.focus(); const l = e.querySelector('[data-key]'); getSelection().selectAllChildren(l); getSelection().collapseToEnd(); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        page.evaluate(format!("{MORE}.click()")).await.unwrap();
        let quote = "[...document.querySelectorAll('[role=menuitemcheckbox]')].find(i => i.textContent.includes('Quote'))";
        assert!(until(page, quote).await, "no Quote in the More menu");
        page.evaluate(format!("{quote}.click()")).await.unwrap();
        let quoted = until(
            page,
            "document.getElementById('out').textContent.startsWith('> above')",
        )
        .await;
        assert!(quoted, "{:?}", out(page).await);
    });
}

/// `[no horizontal overflow, distinct item rows, hidden toggles]` of the toolbar.
const TOOLBAR_ROWS: &str = "(() => { const t = document.querySelector('[role=toolbar]'); \
    const tops = new Set([...t.querySelectorAll('button')].map(b => Math.round(b.getBoundingClientRect().top))); \
    return [t.scrollWidth > t.clientWidth, tops.size, t.querySelectorAll('button').length]; })()";

/// Larger icons than the default theme's: the bar measures them and hides more (1259).
#[test]
fn a_toolbar_with_larger_icons_still_keeps_one_row() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        assert!(until(page, MORE).await, "no More trigger");
        let (_, _, before): (bool, u32, u32) = eval(page, TOOLBAR_ROWS).await;
        page.evaluate(
            "(() => { const s = document.createElement('style'); \
               s.textContent = '[role=toolbar] button:not([aria-haspopup]) { min-width: 56px; }'; \
               document.head.append(s); })()",
        )
        .await
        .unwrap();
        let fewer = format!("{TOOLBAR_ROWS}[2] < {before}");
        assert!(until(page, &fewer).await, "nothing moved into More");
        let (overflow, rows, _): (bool, u32, u32) = eval(page, TOOLBAR_ROWS).await;
        assert!(!overflow, "the toolbar runs out of its column");
        assert_eq!(rows, 1, "the toolbar wraps");
    });
}

/// Dispatches `kind` (`copy` or `cut`) on the editor; `[text/plain, text/markdown, cancelled]`.
async fn clipboard(page: &Page, kind: &str) -> (String, String, bool) {
    eval(
        page,
        &format!(
            "(() => {{ const d = new DataTransfer(); const e = new ClipboardEvent('{kind}', {{ clipboardData: d, bubbles: true, cancelable: true }}); \
               const kept = {EDITOR}.dispatchEvent(e); return [d.getData('text/plain'), d.getData('text/markdown'), !kept]; }})()"
        ),
    )
    .await
}

#[test]
fn copy_writes_plain_text_and_markdown_and_cut_edits_the_model() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!(
            "(() => {{ const e = {EDITOR}; e.focus(); const l = [...e.querySelectorAll('[data-key]')]; \
               const a = l[0].firstChild, b = l[l.length - 1].firstChild; getSelection().setBaseAndExtent(a, 0, b, b.length); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        let (plain, markdown, cancelled) = clipboard(page, "copy").await;
        assert_eq!(plain, "above\nlet x = 1;\nbelow");
        assert_eq!(markdown, "above\n\n```\nlet x = 1;\n```\n\nbelow\n");
        assert!(cancelled, "the browser's own copy ran too");

        // Cut the first paragraph's text: through the model, so `onchange` sees it.
        page.evaluate(format!(
            "(() => {{ const l = {EDITOR}.querySelector('[data-key]'); getSelection().selectAllChildren(l); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        let (plain, _, _) = clipboard(page, "cut").await;
        assert_eq!(plain, "above");
        let cut = until(
            page,
            "document.getElementById('out').textContent.startsWith('```')",
        )
        .await;
        assert!(cut, "{:?}", out(page).await);
    });
}

#[test]
fn node_views_draw_built_in_blocks_and_keep_them_editable() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/builtins", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        settle(page).await;
        // [heading level, h2 inside the view, quote view, rule view as an island]
        let (level, inner, quote, rule): (String, bool, bool, bool) = eval(
            page,
            &format!(
                "(() => {{ const e = {EDITOR}; const h = e.querySelector('header.fancy-heading'); \
                   return [h?.dataset.level ?? '', !!h?.querySelector('h2[data-key]'), \
                   !!e.querySelector('blockquote.fancy-quote p[data-key]'), \
                   e.querySelector('.fancy-rule')?.closest('[data-key]')?.getAttribute('contenteditable') === 'false']; }})()"
            ),
        )
        .await;
        assert_eq!(level, "2");
        assert!(inner, "the heading's text is not inside its view");
        assert!(quote, "the quote's blocks are not inside its view");
        assert!(rule, "the rule's view is not a non-editable island");

        // The caret still finds the text inside a view.
        page.evaluate(format!(
            "(() => {{ const e = {EDITOR}; e.focus(); const h = e.querySelector('h2[data-key]'); getSelection().selectAllChildren(h); getSelection().collapseToEnd(); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        keyboard::type_text(page, "X").await.unwrap();
        let markdown = out(page).await;
        assert!(markdown.starts_with("## TitleX\n"), "{markdown:?}");
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

        // ArrowDown at the leaf above enters the rendered block, which Chrome would skip.
        page.evaluate(format!(
            "(() => {{ const e = {EDITOR}; e.focus(); const l = e.querySelector('li [data-key]'); getSelection().selectAllChildren(l); getSelection().collapseToEnd(); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        settle(page).await;
        keyboard::type_text(page, "Z").await.unwrap();
        let markdown = out(page).await;
        assert!(
            markdown.ends_with("```\nZlet x = 1;\n```\n"),
            "{markdown:?}"
        );
    });
}

const TRAILING: &str = "/rich-text-editor/trailing";

/// Clicks the rendered code block, which puts the caret at its end as source.
async fn enter_code(page: &Page) {
    const VIEW: &str = "[role=textbox] [data-code=view]";
    let rendered: bool = eval(page, &format!("!!document.querySelector('{VIEW}')")).await;
    if rendered {
        pointer::click(page, VIEW).await.unwrap();
    }
    wait::for_js_true(
        page,
        "!!document.querySelector('[role=textbox] [data-code=source]')",
        "the code block as source",
    )
    .await
    .unwrap();
    settle(page).await;
}

#[test]
fn a_trailing_code_block_is_left_by_arrow_down_mod_enter_and_a_click_below() {
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        settle(page).await;

        enter_code(page).await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        settle(page).await;
        keyboard::type_text(page, "after").await.unwrap();
        assert_eq!(
            out(page).await,
            "intro\n\n```rust\nlet x = 1;\n```\n\nafter\n"
        );

        enter_code(page).await;
        keyboard::press_with(page, ENTER, CTRL).await.unwrap();
        settle(page).await;
        keyboard::type_text(page, "mid").await.unwrap();
        let both = "intro\n\n```rust\nlet x = 1;\n```\n\nmid\n\nafter\n";
        assert_eq!(out(page).await, both);

        // Outside a code block Mod+Enter is not the editor's: a caller's send still sees it.
        page.evaluate(
            "window.__free = 0; addEventListener('keydown', e => { if (e.ctrlKey && e.key === 'Enter' && !e.defaultPrevented) __free++; })",
        )
        .await
        .unwrap();
        keyboard::press_with(page, ENTER, CTRL).await.unwrap();
        settle(page).await;
        let free: u32 = eval(page, "window.__free").await;
        assert_eq!(free, 1);
        assert_eq!(out(page).await, both);

        // A fresh doc: a press in the padding under the code block opens a line there.
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        settle(page).await;
        let (x, y): (f64, f64) = eval(
            page,
            &format!(
                "(() => {{ const r = {EDITOR}.getBoundingClientRect(); return [r.left + r.width / 2, r.bottom - 3]; }})()"
            ),
        )
        .await;
        pointer::click_at(page, pointer::Point { x, y })
            .await
            .unwrap();
        settle(page).await;
        keyboard::type_text(page, "end").await.unwrap();
        assert_eq!(
            out(page).await,
            "intro\n\n```rust\nlet x = 1;\n```\n\nend\n"
        );
    });
}

/// The way out without arrows or Ctrl, as on a touch keyboard.
#[test]
fn enter_twice_at_the_end_leaves_a_trailing_code_block() {
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        settle(page).await;

        enter_code(page).await;
        keyboard::press(page, ENTER).await.unwrap();
        settle(page).await;
        keyboard::press(page, ENTER).await.unwrap();
        settle(page).await;
        keyboard::type_text(page, "out").await.unwrap();
        assert_eq!(
            out(page).await,
            "intro\n\n```rust\nlet x = 1;\n```\n\nout\n"
        );
    });
}

#[test]
fn a_typed_fence_sets_the_language_and_the_toolbar_menu_changes_it() {
    const LANGUAGE: &str =
        "document.querySelector('[role=toolbar] button[aria-label^=\"Language:\"]')";
    const PLAIN: &str = "[...document.querySelectorAll('[role=menuitemradio]')].find(i => i.textContent.trim() === 'Plain text')";
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        settle(page).await;

        let shown: bool = eval(page, &format!("!!{LANGUAGE}")).await;
        assert!(!shown, "the language menu shows outside a code block");
        enter_code(page).await;
        page.evaluate(format!("{LANGUAGE}.click()")).await.unwrap();
        wait::for_js_true(page, &format!("!!{PLAIN}"), "the language menu")
            .await
            .unwrap();
        page.evaluate(format!("{PLAIN}.click()")).await.unwrap();
        assert_eq!(out(page).await, "intro\n\n```\nlet x = 1;\n```\n");

        // Down out of the block, then a fence and Enter: a Python block.
        enter_code(page).await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        settle(page).await;
        keyboard::type_text(page, "```py").await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "x = 1").await.unwrap();
        assert_eq!(
            out(page).await,
            "intro\n\n```\nlet x = 1;\n```\n\n```py\nx = 1\n```\n"
        );
    });
}

#[test]
fn the_fence_button_and_mod_shift_l_change_the_language_and_return_to_the_caret() {
    const FENCE: &str = "document.querySelector('[role=textbox] [data-fence] button')";
    const PLAIN: &str = "[...document.querySelectorAll('[role=menuitemradio]')].find(i => i.textContent.trim() === 'Plain text')";
    const IN_TEXT: &str = "document.activeElement?.getAttribute('role') === 'textbox'";
    const KEY_L: Key = Key {
        key: "L",
        code: "KeyL",
        vk: 76,
        text: None,
    };
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        settle(page).await;

        enter_code(page).await;
        let (name, tabindex): (String, String) = eval(
            page,
            &format!("[{FENCE}.getAttribute('aria-label'), {FENCE}.getAttribute('tabindex')]"),
        )
        .await;
        assert_eq!(name, "Code language: Rust");
        assert_eq!(tabindex, "-1");
        page.evaluate(format!("{FENCE}.click()")).await.unwrap();
        wait::for_js_true(page, &format!("!!{PLAIN}"), "the fence's language menu")
            .await
            .unwrap();
        page.evaluate(format!("{PLAIN}.click()")).await.unwrap();
        assert_eq!(out(page).await, "intro\n\n```\nlet x = 1;\n```\n");
        wait::for_js_true(page, IN_TEXT, "focus back in the text")
            .await
            .unwrap();

        // From the keyboard: the chord opens the menu, Escape returns to the caret.
        keyboard::press_with(page, KEY_L, CTRL | keyboard::SHIFT)
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.getAttribute('role') === 'menuitemradio'",
            "focus in the language menu",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(page, IN_TEXT, "focus back in the text after Escape")
            .await
            .unwrap();
        settle(page).await;
        keyboard::type_text(page, "z").await.unwrap();
        assert_eq!(out(page).await, "intro\n\n```\nlet x = 1;z\n```\n");
    });
}

#[test]
fn toolbar_buttons_show_their_name_and_chord_in_a_tooltip() {
    const BOLD: &str = "[role=toolbar] button[aria-label=\"Bold\"]";
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        settle(page).await;
        let shortcut: String = eval(
            page,
            &format!("document.querySelector('{BOLD}').getAttribute('aria-keyshortcuts')"),
        )
        .await;
        assert_eq!(shortcut, "Control+B");
        pointer::hover(page, BOLD).await.unwrap();
        wait::for_js_true(
            page,
            "!!document.querySelector('[role=tooltip]')",
            "a tooltip",
        )
        .await
        .unwrap();
        let text: String = eval(
            page,
            "document.querySelector('[role=tooltip]').textContent.replace(/\\s+/g, ' ').trim()",
        )
        .await;
        assert_eq!(text, "Bold Ctrl + B");
    });
}

/// The contrast of `selector`'s text over its own background, both opaque.
fn hovered_contrast(selector: &str) -> String {
    format!(
        "(() => {{ const s = getComputedStyle(document.querySelector('{selector}')); \
           const lum = c => {{ const [r, g, b] = c.match(/[\\d.]+/g).slice(0, 3).map(v => {{ v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }}); return 0.2126 * r + 0.7152 * g + 0.0722 * b; }}; \
           const [a, b] = [lum(s.color), lum(s.backgroundColor)].sort((x, y) => y - x); return (a + 0.05) / (b + 0.05); }})()"
    )
}

/// The code block rendered and hovered, as source, and a tooltip, in both schemes.
#[test]
fn the_editor_overlays_keep_their_contrast_in_both_schemes() {
    block_on(async {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let fixture = Fixture::open_in(TRAILING, Viewport::Desktop, scheme)
                .await
                .unwrap();
            let page = &fixture.page;
            settle(page).await;
            pointer::hover(page, "[role=textbox] [data-code=view]")
                .await
                .unwrap();
            settle(page).await;
            fixture
                .screenshot(&format!("rte-code-view-{}", scheme.name()))
                .await
                .unwrap();
            contrast::assert_clean(page, "body").await.unwrap();
            enter_code(page).await;
            // The pressed toggle, hovered: its state colours under the tooltip.
            pointer::hover(
                page,
                "[role=toolbar] button[aria-label=\"Code block\"][aria-pressed=true]",
            )
            .await
            .unwrap();
            wait::for_js_true(
                page,
                "!!document.querySelector('[role=tooltip]')",
                "a tooltip",
            )
            .await
            .unwrap();
            settle(page).await;
            fixture
                .screenshot(&format!("rte-code-source-{}", scheme.name()))
                .await
                .unwrap();
            contrast::assert_clean(page, "body").await.unwrap();
            // The text type button names the block's mode; hovered, it keeps its contrast.
            pointer::hover(page, "[role=toolbar] button[aria-label^=\"Text type:\"]")
                .await
                .unwrap();
            settle(page).await;
            fixture
                .screenshot(&format!("rte-block-type-{}", scheme.name()))
                .await
                .unwrap();
            // axe skips hover states: the label over its hover fill, measured here.
            page.evaluate("new Promise(r => setTimeout(r, 300))")
                .await
                .unwrap();
            let ratio: f64 = eval(
                page,
                &hovered_contrast("[role=toolbar] button[aria-label^=\"Text type:\"]"),
            )
            .await;
            assert!(
                ratio >= 4.5,
                "hovered text type button {ratio:.2}:1 in {}",
                scheme.name()
            );
            contrast::assert_clean(page, "body").await.unwrap();
        }
    });
}

#[test]
fn a_mention_list_follows_the_caret_and_takes_keys_through_intercept() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/mentions", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        settle(page).await;
        let overlay = "document.querySelector('[data-overlay]')";
        let active = format!("{EDITOR}.getAttribute('aria-activedescendant')");

        keyboard::type_text(page, "hi @a").await.unwrap();
        settle(page).await;
        let options: u32 = eval(page, "document.querySelectorAll('[role=option]').length").await;
        assert_eq!(options, 2, "ada and alan");
        let wired: bool = eval(
            page,
            &format!(
                "{EDITOR}.getAttribute('aria-controls') === {overlay}.id && {EDITOR}.getAttribute('aria-autocomplete') === 'list'"
            ),
        )
        .await;
        assert!(wired);
        assert_eq!(eval::<String>(page, &active).await, "mention-ada");
        // Placed under the caret's line.
        let placed: String = eval(
            page,
            &format!(
                "(() => {{ const o = {overlay}.getBoundingClientRect(), l = {EDITOR}.querySelector('[data-key]').getBoundingClientRect(); return (getComputedStyle({overlay}).visibility === 'visible' && o.top >= l.bottom - 1 && o.left >= l.left) ? 'ok' : JSON.stringify([{overlay}.getAttribute('style'), o, l]); }})()"
            ),
        )
        .await;
        assert_eq!(placed, "ok");
        // The combobox-like attributes are all allowed on a textbox.
        let axe = contrast::run_full(page, "body").await.unwrap();
        let aria: Vec<&str> = axe
            .violations
            .iter()
            .map(|violation| violation.id.as_str())
            .filter(|id| id.starts_with("aria"))
            .collect();
        assert!(aria.is_empty(), "{aria:?}");

        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        settle(page).await;
        assert_eq!(eval::<String>(page, &active).await, "mention-alan");
        keyboard::press(page, ENTER).await.unwrap();
        assert_eq!(out(page).await, "hi \u{fffc} ");
        let mention: String = eval(
            page,
            &format!("{EDITOR}.querySelector('.mention').textContent"),
        )
        .await;
        assert_eq!(mention, "@alan");
        let gone: bool = eval(
            page,
            &format!("!{overlay} && !{EDITOR}.hasAttribute('aria-activedescendant')"),
        )
        .await;
        assert!(gone);

        // Escape closes the list and leaves the text.
        keyboard::type_text(page, "@g").await.unwrap();
        settle(page).await;
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        settle(page).await;
        assert!(eval::<bool>(page, &format!("!{overlay}")).await);
        assert_eq!(out(page).await, "hi \u{fffc} @g");

        // The caller's toolbar button types the @; a click on an option picks it.
        keyboard::type_text(page, " ").await.unwrap();
        page.evaluate("document.querySelector('[aria-label=\"Mention someone\"]').click()")
            .await
            .unwrap();
        settle(page).await;
        let options: u32 = eval(page, "document.querySelectorAll('[role=option]').length").await;
        assert_eq!(options, 4);
        page.evaluate("document.getElementById('mention-grace').click()")
            .await
            .unwrap();
        assert_eq!(out(page).await, "hi \u{fffc} @g \u{fffc} ");
    });
}

/// Todo 1466: a shown code block scrolls inside itself, the editor stays in its row.
#[test]
fn a_long_code_line_does_not_widen_the_editor() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/narrow", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let fits = "(() => { const row = document.getElementById('row'), r = row.getBoundingClientRect(); \
            const editor = row.firstElementChild.getBoundingClientRect(); \
            const scroll = row.querySelector('[data-code=view] [data-slot=scroll]'); \
            return scroll && editor.right <= r.right + 0.5 && row.scrollWidth <= row.clientWidth \
                && scroll.scrollWidth > scroll.clientWidth ? 'ok' \
                : JSON.stringify([r.width, editor.width, row.scrollWidth, scroll && scroll.scrollWidth]); })()";
        wait::for_js_true(
            page,
            "!!document.querySelector('#row [data-code=view]')",
            "the code block",
        )
        .await
        .unwrap();
        assert_eq!(eval::<String>(page, fits).await, "ok");

        // Todo 1475: so does its source while the caret is in it.
        let code = pointer::centre_of(page, "#row [data-code=view]")
            .await
            .unwrap();
        pointer::click_at(page, code).await.unwrap();
        let source = "(() => { const row = document.getElementById('row'), r = row.getBoundingClientRect(); \
            const editor = row.firstElementChild.getBoundingClientRect(); \
            const pre = row.querySelector('[data-code=source] > pre'); \
            return pre && editor.right <= r.right + 0.5 && row.scrollWidth <= row.clientWidth \
                && pre.scrollWidth > pre.clientWidth ? 'ok' \
                : JSON.stringify([r.width, editor.width, row.scrollWidth, pre && pre.scrollWidth]); })()";
        wait::for_js_true(page, &format!("{source} === 'ok'"), "the source to fit")
            .await
            .unwrap();
        fixture.console.assert_clean("the code block").unwrap();
    });
}

#[test]
fn a_mention_list_near_the_viewport_bottom_flips_above_the_caret() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/mentions", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let leaf = format!("{EDITOR}.querySelector('[data-key]')");
        // The text line 40px above the viewport's bottom edge.
        page.evaluate(format!(
            "(() => {{ document.body.style.paddingTop = innerHeight + 'px'; window.scrollBy(0, {leaf}.getBoundingClientRect().bottom - (innerHeight - 40)); {EDITOR}.focus(); }})()"
        ))
        .await
        .unwrap();
        settle(page).await;
        keyboard::type_text(page, "@").await.unwrap();
        settle(page).await;
        let above: String = eval(
            page,
            &format!(
                "(() => {{ const o = document.querySelector('[data-overlay]').getBoundingClientRect(), l = {leaf}.getBoundingClientRect(); return o.bottom <= l.top + 1 && o.top >= 0 ? 'ok' : JSON.stringify([o, l, innerHeight]); }})()"
            ),
        )
        .await;
        assert_eq!(above, "ok");

        // Scrolled to the middle, the list moves back under the line. Headless Chrome may hold
        // the scroll event until its next frame, so one is sent too.
        page.evaluate(
            "document.body.style.paddingBottom = innerHeight + 'px'; window.scrollBy(0, innerHeight / 2); document.dispatchEvent(new Event('scroll'))",
        )
        .await
        .unwrap();
        settle(page).await;
        let below: String = eval(
            page,
            &format!(
                "(() => {{ const o = document.querySelector('[data-overlay]').getBoundingClientRect(), l = {leaf}.getBoundingClientRect(); return o.top >= l.bottom - 1 ? 'ok' : JSON.stringify([o, l, innerHeight]); }})()"
            ),
        )
        .await;
        assert_eq!(below, "ok");
    });
}
