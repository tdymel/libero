//! `RichTextEditor` code blocks, caller node views, task boxes and the stored view on the web.

use super::rich_text_editor::{
    EDITOR, KEY_Z, SYNCED, TOOLBAR_ROWS, TRAILING, announced, editor_status, holds, left_code, out,
    out_eq, out_where, press, select,
};
use chromiumoxide::Page;
use e2e::browser::{Scheme, block_on};
use e2e::passes::keyboard::{self, CTRL, ENTER, Key};
use e2e::passes::{contrast, pointer};
use e2e::{Fixture, Viewport, js, wait};

/// No transition or animation still running, as a screenshot or a colour read needs.
const STILL: &str = "document.getAnimations().every(a => a.playState !== 'running' \
    || a.effect?.getComputedTiming().iterations === Infinity)";

/// Axe over `body` once nothing moves; a bubble that remounted since the wait restarts its fade (3451).
async fn clean_when_still(page: &Page) {
    let mut outcome = Ok(());
    for _ in 0..3 {
        wait::for_js_true(page, STILL, "the transitions to end")
            .await
            .unwrap();
        outcome = contrast::assert_clean_except(page, "body", contrast::LINE_NUMBERS).await;
        if outcome.is_ok() {
            return;
        }
    }
    outcome.unwrap();
}

#[test]
fn node_views_draw_caller_nodes_and_keep_their_content_editable() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/nodes", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let mention = "[role=textbox] [data-atom][contenteditable=false] .mention";
        wait::for_selector(page, mention).await.unwrap();
        let atom: String = js(
            page,
            &format!("document.querySelector('{mention}').textContent"),
        )
        .await;
        assert_eq!(atom, "@ada");

        // The callout's text is its own leaf inside the caller's markup.
        select(
            page,
            &format!(
                "const e = {EDITOR}; e.focus(); const l = e.querySelector('aside.callout [data-key]'); getSelection().selectAllChildren(l); getSelection().collapseToEnd()"
            ),
        )
        .await;
        keyboard::type_text(page, "!").await.unwrap();
        out_where(page, "the typed key in the callout", |now| {
            now.ends_with("careful!")
        })
        .await;
        let note: u32 = js(
            page,
            &format!("{EDITOR}.querySelectorAll('aside.callout > span').length"),
        )
        .await;
        assert_eq!(note, 1, "the view renders once, its children once");
        fixture.close().await.unwrap();
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
        wait::for_selector(page, "[role=textbox] [data-code=view]")
            .await
            .unwrap();

        // ArrowUp at the leaf below enters the rendered block at its end.
        select(
            page,
            &format!(
                "const e = {EDITOR}; e.focus(); const l = [...e.querySelectorAll('[data-key]')].pop(); getSelection().selectAllChildren(l); getSelection().collapseToEnd()"
            ),
        )
        .await;
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        wait::for_selector(page, "[role=textbox] [data-code=source]")
            .await
            .unwrap();
        keyboard::type_text(page, "Z").await.unwrap();
        out_eq(page, "above\n\n```\nlet x = 1;Z\n```\n\nbelow\n").await;

        keyboard::press_with(page, SLASH_DE, CTRL | keyboard::SHIFT)
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "!!document.querySelector('[role=dialog]')?.textContent.includes('Keyboard shortcuts')",
            "the keymap listed",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn node_views_draw_built_in_blocks_and_keep_them_editable() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/builtins", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "[role=textbox] .fancy-rule")
            .await
            .unwrap();
        // [heading level, h2 inside the view, quote view, rule view as an island]
        let (level, inner, quote, rule): (String, bool, bool, bool) = js(
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
        select(
            page,
            &format!(
                "const e = {EDITOR}; e.focus(); const h = e.querySelector('h2[data-key]'); getSelection().selectAllChildren(h); getSelection().collapseToEnd()"
            ),
        )
        .await;
        keyboard::type_text(page, "X").await.unwrap();
        out_where(page, "the typed key in the heading", |now| {
            now.starts_with("## TitleX\n")
        })
        .await;
        fixture.close().await.unwrap();
    });
}

/// Todo 2753: a task's box is a named checkbox outside the text. A click toggles its own item
/// in one undo step and says so, the caret stays put, and the arrows pass over the box.
#[test]
fn a_task_box_click_toggles_its_item_and_the_caret_skips_it() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        editor_status(page).await;
        keyboard::type_text(page, "- [ ] one").await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "two").await.unwrap();
        out_eq(page, "- [ ] one\n- [ ] two\n").await;

        let first = format!("{EDITOR}.querySelector('[data-task]')");
        holds(
            page,
            &format!(
                "(() => {{ const b = {first}; const got = [b.getAttribute('role'), b.getAttribute('aria-checked'), \
                   document.getElementById(b.getAttribute('aria-labelledby'))?.textContent, \
                   b.getAttribute('contenteditable'), String(b.hasAttribute('tabindex'))].join(); \
                   return got === 'checkbox,false,one,false,false' ? 'ok' : got; }})()"
            ),
            "the box is an untabbable checkbox island named by its item's text",
        )
        .await;

        press(page, &first).await;
        out_eq(page, "- [x] one\n- [ ] two\n").await;
        announced(page, "Task done on").await;
        let checked: String = js(page, &format!("{first}.getAttribute('aria-checked')")).await;
        assert_eq!(checked, "true");
        // One undo step takes the click back; a second click checks it again.
        keyboard::press_with(page, KEY_Z, CTRL).await.unwrap();
        out_eq(page, "- [ ] one\n- [ ] two\n").await;
        press(page, &first).await;
        out_eq(page, "- [x] one\n- [ ] two\n").await;
        // The caret stayed at the end of "two".
        keyboard::type_text(page, "s").await.unwrap();
        out_eq(page, "- [x] one\n- [ ] twos\n").await;

        // From the start of "two", one more ArrowLeft lands at the end of "one", not in the box.
        select(
            page,
            &format!(
                "getSelection().collapse({EDITOR}.querySelectorAll('[data-key]')[1].firstChild, 0)"
            ),
        )
        .await;
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        holds(
            page,
            &format!(
                "(() => {{ const s = getSelection(); const leaf = {EDITOR}.querySelector('[data-key]'); \
                   return leaf.contains(s.anchorNode) && s.anchorOffset === 3 ? 'ok' : `${{s.anchorNode?.textContent}}@${{s.anchorOffset}}`; }})()"
            ),
            "the caret at the end of \"one\"",
        )
        .await;
        page.evaluate(SYNCED).await.unwrap();
        keyboard::type_text(page, "s").await.unwrap();
        out_eq(page, "- [x] ones\n- [ ] twos\n").await;

        // The toolbar's Task list button shows the item as a task and turns it back.
        let button = "document.querySelector('[role=toolbar] [aria-label=\"Task list\"]')";
        holds(
            page,
            &format!("(() => {{ const b = {button}; return b?.getAttribute('aria-pressed') === 'true' ? 'ok' : String(b?.outerHTML); }})()"),
            "the Task list button pressed in a task",
        )
        .await;
        press(page, button).await;
        out_eq(page, "- ones\n- [ ] twos\n").await;
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_source_mode_code_block_stays_left_to_right_on_a_right_to_left_page() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/rtl-code", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        enter_code(page).await;
        let direction: String = js(
            page,
            "getComputedStyle(document.querySelector('[role=textbox] [data-code=source] pre')).direction",
        )
        .await;
        assert_eq!(direction, "ltr");
        fixture.close().await.unwrap();
    });
}

/// Clicks the rendered code block, which puts the caret at its end as source.
async fn enter_code(page: &Page) {
    const VIEW: &str = "[role=textbox] [data-code=view]";
    wait::for_selector(page, "[role=textbox] [data-code]")
        .await
        .unwrap();
    let rendered: bool = js(page, &format!("!!document.querySelector('{VIEW}')")).await;
    if rendered {
        pointer::click(page, VIEW).await.unwrap();
    }
    wait::for_selector(page, "[role=textbox] [data-code=source]")
        .await
        .unwrap();
    page.evaluate(SYNCED).await.unwrap();
}

#[test]
fn a_trailing_code_block_is_left_by_arrow_down_mod_enter_and_a_click_below() {
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        enter_code(page).await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        left_code(page).await;
        keyboard::type_text(page, "after").await.unwrap();
        out_eq(page, "intro\n\n```rust\nlet x = 1;\n```\n\nafter\n").await;

        enter_code(page).await;
        keyboard::press_with(page, ENTER, CTRL).await.unwrap();
        left_code(page).await;
        keyboard::type_text(page, "mid").await.unwrap();
        let both = "intro\n\n```rust\nlet x = 1;\n```\n\nmid\n\nafter\n";
        out_eq(page, both).await;

        // Outside a code block Mod+Enter is not the editor's: a caller's send still sees it.
        page.evaluate(
            "window.__free = 0; addEventListener('keydown', e => { if (e.ctrlKey && e.key === 'Enter' && !e.defaultPrevented) __free++; })",
        )
        .await
        .unwrap();
        keyboard::press_with(page, ENTER, CTRL).await.unwrap();
        // The editor handles a key within it; past its caret sync it has done all it would.
        page.evaluate(SYNCED).await.unwrap();
        let free: u32 = js(page, "window.__free").await;
        assert_eq!(free, 1);
        assert_eq!(out(page).await, both);

        // A fresh doc: a press in the padding under the code block opens a line there.
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "[role=textbox] [data-code=view]")
            .await
            .unwrap();
        let (x, y): (f64, f64) = js(
            page,
            &format!(
                "(() => {{ const r = {EDITOR}.getBoundingClientRect(); return [r.left + r.width / 2, r.bottom - 3]; }})()"
            ),
        )
        .await;
        pointer::click_at(page, pointer::Point { x, y })
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{EDITOR}.querySelectorAll('[data-key]').length === 2"),
            "a line under the code block",
        )
        .await
        .unwrap();
        keyboard::type_text(page, "end").await.unwrap();
        out_eq(page, "intro\n\n```rust\nlet x = 1;\n```\n\nend\n").await;
        fixture.close().await.unwrap();
    });
}

/// The way out without arrows or Ctrl, as on a touch keyboard.
#[test]
fn enter_twice_at_the_end_leaves_a_trailing_code_block() {
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Mobile).await.unwrap();
        let page = &fixture.page;

        enter_code(page).await;
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        left_code(page).await;
        keyboard::type_text(page, "out").await.unwrap();
        out_eq(page, "intro\n\n```rust\nlet x = 1;\n```\n\nout\n").await;
        fixture.close().await.unwrap();
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
        wait::for_selector(page, "[role=toolbar] button[aria-label=\"Code block\"]")
            .await
            .unwrap();

        let shown: bool = js(page, &format!("!!{LANGUAGE}")).await;
        assert!(!shown, "the language menu shows outside a code block");
        enter_code(page).await;
        press(page, LANGUAGE).await;
        wait::for_js_true(page, &format!("!!{PLAIN}"), "the language menu")
            .await
            .unwrap();
        press(page, PLAIN).await;
        out_eq(page, "intro\n\n```\nlet x = 1;\n```\n").await;

        // Down out of the block, then a fence and Enter: a Python block.
        enter_code(page).await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        left_code(page).await;
        keyboard::type_text(page, "```py").await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "x = 1").await.unwrap();
        out_eq(page, "intro\n\n```\nlet x = 1;\n```\n\n```py\nx = 1\n```\n").await;
        fixture.close().await.unwrap();
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

        enter_code(page).await;
        let (name, tabindex): (String, String) = js(
            page,
            &format!("[{FENCE}.getAttribute('aria-label'), {FENCE}.getAttribute('tabindex')]"),
        )
        .await;
        assert_eq!(
            name, "rust, Code language",
            "the name starts with the visible text"
        );
        assert_eq!(tabindex, "-1");
        press(page, FENCE).await;
        wait::for_js_true(page, &format!("!!{PLAIN}"), "the fence's language menu")
            .await
            .unwrap();
        press(page, PLAIN).await;
        out_eq(page, "intro\n\n```\nlet x = 1;\n```\n").await;
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
        page.evaluate(SYNCED).await.unwrap();
        keyboard::type_text(page, "z").await.unwrap();
        out_eq(page, "intro\n\n```\nlet x = 1;z\n```\n").await;
        fixture.close().await.unwrap();
    });
}

/// The contrast of `selector`'s text over its own background, both opaque.
fn hovered_contrast(selector: &str) -> String {
    format!(
        "(() => {{ {colour} const s = getComputedStyle(document.querySelector('{selector}')); \
           return CONTRAST(RGBA(s.color), RGBA(s.backgroundColor)); }})()",
        colour = contrast::COLOUR_JS,
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
            let still = || wait::for_js_true(page, STILL, "the transitions to end");
            wait::for_selector(page, "[role=textbox] [data-code=view]")
                .await
                .unwrap();
            pointer::hover(page, "[role=textbox] [data-code=view]")
                .await
                .unwrap();
            still().await.unwrap();
            fixture
                .screenshot(&format!("rte-code-view-{}", scheme.name()))
                .await
                .unwrap();
            contrast::assert_clean_except(page, "body", contrast::LINE_NUMBERS)
                .await
                .unwrap();
            enter_code(page).await;
            // The pressed toggle, hovered: its state colours under the tooltip.
            pointer::hover(
                page,
                "[role=toolbar] button[aria-label=\"Code block\"][aria-pressed=true]",
            )
            .await
            .unwrap();
            // Shown once measured; its fade-in then runs as a transition.
            wait::for_visible(page, "[role=tooltip]").await.unwrap();
            still().await.unwrap();
            fixture
                .screenshot(&format!("rte-code-source-{}", scheme.name()))
                .await
                .unwrap();
            clean_when_still(page).await;
            // The text type button names the block's mode; hovered, it keeps its contrast.
            pointer::hover(page, "[role=toolbar] button[aria-label^=\"Text type:\"]")
                .await
                .unwrap();
            wait::for_hidden(page, "[role=tooltip]").await.unwrap();
            still().await.unwrap();
            fixture
                .screenshot(&format!("rte-block-type-{}", scheme.name()))
                .await
                .unwrap();
            // axe skips hover states: the label over its hover fill, measured here.
            let ratio: f64 = js(
                page,
                &hovered_contrast("[role=toolbar] button[aria-label^=\"Text type:\"]"),
            )
            .await;
            assert!(
                ratio >= 4.5,
                "hovered text type button {ratio:.2}:1 in {}",
                scheme.name()
            );
            contrast::assert_clean_except(page, "body", contrast::LINE_NUMBERS)
                .await
                .unwrap();
            fixture.close().await.unwrap();
        }
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
        assert_eq!(js::<String>(page, fits).await, "ok");

        // Todo 1475: so does its source while the caret is in it. The toolbar first measures
        // itself into one row: the block moves up, and a click aimed before that missed it (1626).
        wait::for_js_true(
            page,
            &format!("(([overflow, rows]) => !overflow && rows === 1)({TOOLBAR_ROWS})"),
            "the toolbar in one row",
        )
        .await
        .unwrap();
        pointer::click(page, "#row [data-code=view]").await.unwrap();
        let source = "(() => { const row = document.getElementById('row'), r = row.getBoundingClientRect(); \
            const editor = row.firstElementChild.getBoundingClientRect(); \
            const pre = row.querySelector('[data-code=source] > pre'); \
            return pre && editor.right <= r.right + 0.5 && row.scrollWidth <= row.clientWidth \
                && pre.scrollWidth > pre.clientWidth ? 'ok' \
                : JSON.stringify([r.width, editor.width, row.scrollWidth, pre && pre.scrollWidth]); })()";
        holds(page, source, "the source to fit").await;
        fixture.console.assert_clean("the code block").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_view_shows_a_doc_read_only_and_follows_an_editor() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/view", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let static_view = "#static [data-code=view] pre, #static a[href='https://example.com']";
        wait::for_js_true(
            page,
            &format!("document.querySelectorAll(\"{static_view}\").length === 2"),
            "the code block and the link of the stored doc",
        )
        .await
        .unwrap();
        let texts: Vec<String> = js(
            page,
            "['h1', 'strong', 'em', 'li', 'blockquote'].map(tag => document.querySelector('#static ' + tag).textContent)",
        )
        .await;
        assert_eq!(
            texts,
            ["Release notes", "bold", "italic", "First item", "A quote"]
        );

        let editing: u32 = js(
            page,
            "document.querySelectorAll('#static [contenteditable=true], #static [role=textbox], #live [contenteditable=true], #live [role=textbox]').length",
        )
        .await;
        assert_eq!(editing, 0, "a view is not an editing surface");

        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "ab").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#live p')?.textContent === 'ab'",
            "the view to follow the editor",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}
