//! `RichTextEditor` on the web: every edit goes through the model, the caret follows it,
//! and a controlled parent's late echo neither resets the doc nor moves the caret.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::ImeSetCompositionParams;
use e2e::browser::{Scheme, block_on};
use e2e::passes::keyboard::{self, BACKSPACE, CTRL, ENTER, Key};
use e2e::passes::{contrast, live_region, pointer};
use e2e::{Fixture, Viewport, js, wait};

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

const OUT: &str = "document.getElementById('out').textContent";

async fn out(page: &Page) -> String {
    js(page, OUT).await
}

/// Waits for `#out` to pass `check`; fails with what it read last.
async fn out_where(page: &Page, what: &str, check: impl Fn(&str) -> bool) {
    let last = std::cell::RefCell::new(String::new());
    let held = wait::until(what, || async {
        let now: String = page.evaluate(OUT).await?.into_value()?;
        let passes = check(&now);
        *last.borrow_mut() = now;
        Ok(passes)
    })
    .await;
    assert!(held.is_ok(), "{what}: #out reads {:?}", last.borrow());
}

async fn out_eq(page: &Page, want: &str) {
    out_where(page, &format!("#out to read {want:?}"), |now| now == want).await;
}

/// Waits out the editor's caret sync after an edit: it replies on a 0 ms timer, queued
/// before this one. Until then the editor ignores selection moves.
const SYNCED: &str = "new Promise(r => setTimeout(r))";

/// Runs `script`, which moves the DOM selection, and returns once the model has it: the
/// editor's `selectionchange` listener, added before ours, hands it over before ours runs.
async fn select(page: &Page, script: &str) {
    page.evaluate(format!(
        "{SYNCED}.then(() => new Promise((r, fail) => {{ \
           document.addEventListener('selectionchange', () => r(true), {{ once: true }}); \
           setTimeout(() => fail(new Error('the selection did not move')), 5000); {script}; }}))"
    ))
    .await
    .unwrap();
}

/// Waits for `check`, a script returning 'ok' or what it saw instead, to return 'ok'.
async fn holds(page: &Page, check: &str, what: &str) {
    if wait::for_js_true(page, &format!("{check} === 'ok'"), what)
        .await
        .is_err()
    {
        panic!("{what}: {}", js::<String>(page, check).await);
    }
}

/// A real press on the element `expression` finds: a JS `.click()` skips the `mousedown`
/// that would take focus from the caret (todo 1795).
async fn press(page: &Page, expression: &str) {
    let marked: bool = js(
        page,
        format!(
            "(() => {{ document.querySelectorAll('[data-e2e-press]').forEach(e => e.removeAttribute('data-e2e-press')); \
               const el = {expression}; if (!el) return false; el.setAttribute('data-e2e-press', ''); return true; }})()"
        ),
    )
    .await;
    assert!(marked, "nothing to press at {expression}");
    pointer::click(page, "[data-e2e-press]").await.unwrap();
}

/// The editor's own live region, polite and present before any message (todo 1800).
const STATUS: &str = "#e2e-editor-status";

/// Marks the status region next to the editor's text as [`STATUS`] and checks it is polite.
async fn editor_status(page: &Page) {
    let marked: bool = js(
        page,
        format!(
            "(() => {{ let p = {EDITOR}; while (p && !p.querySelector('[role=status]')) p = p.parentElement; \
               const s = p?.querySelector('[role=status]'); if (!s) return false; s.id = 'e2e-editor-status'; return true; }})()"
        ),
    )
    .await;
    assert!(marked, "no live region beside the editor");
    live_region::assert_politeness(page, STATUS, "polite")
        .await
        .unwrap();
}

/// Waits for the editor's live region to say `message`.
async fn announced(page: &Page, message: &str) {
    let last = std::cell::RefCell::new(String::new());
    let said = wait::until(&format!("{message:?} announced"), || async {
        let now = live_region::text_of(page, STATUS).await?;
        let found = now == message;
        *last.borrow_mut() = now;
        Ok(found)
    })
    .await;
    assert!(
        said.is_ok(),
        "{message:?} not announced; the region says {:?}",
        last.borrow()
    );
}

/// No transition or animation still running, as a screenshot or a colour read needs.
const STILL: &str = "document.getAnimations().every(a => a.playState !== 'running' \
    || a.effect?.getComputedTiming().iterations === Infinity)";

/// Todo 1610 (WCAG 2.1.2): Tab indents a list item, Escape then Tab leaves the editor,
/// and the way out is in the text's description.
#[test]
fn escape_then_tab_leaves_a_list() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        let hint: String = js(
            page,
            &format!(
                "{EDITOR}.getAttribute('aria-describedby').split(' ').map(id => document.getElementById(id)?.textContent).join(' ')"
            ),
        )
        .await;
        assert!(hint.contains("Press Escape, then Tab"), "{hint}");

        keyboard::type_text(page, "- a").await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "b").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!!{EDITOR}.querySelector('ul ul')"),
            "Tab to nest the item",
        )
        .await
        .unwrap();
        holds(
            page,
            &format!("(document.activeElement === {EDITOR} ? 'ok' : 'focus left on Tab')"),
            "Tab in a list keeps focus",
        )
        .await;

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        holds(
            page,
            &format!("(document.activeElement !== {EDITOR} ? 'ok' : 'focus stayed')"),
            "Escape then Tab to leave",
        )
        .await;
        let nested: usize = js(page, &format!("{EDITOR}.querySelectorAll('ul ul').length")).await;
        assert_eq!(nested, 1, "the leaving Tab indented again");
        fixture.close().await.unwrap();
    });
}

#[test]
fn typing_enter_backspace_undo_and_shortcuts_edit_the_model() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();

        let placeholder: String = js(
            page,
            &format!("getComputedStyle({EDITOR}, '::before').content"),
        )
        .await;
        assert_eq!(placeholder, "\"Write something\"");

        // Char by char: the caret must follow every model change, echoes included.
        keyboard::type_text(page, "hello").await.unwrap();
        out_eq(page, "hello\n").await;
        let changes: String = js(page, "document.getElementById('changes').textContent").await;
        assert_eq!(changes, "5");

        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "wörld😀").await.unwrap();
        out_eq(page, "hello\n\nwörld😀\n").await;

        keyboard::press(page, BACKSPACE).await.unwrap();
        keyboard::press(page, BACKSPACE).await.unwrap();
        out_eq(page, "hello\n\nwörl\n").await;

        keyboard::press_with(page, KEY_Z, CTRL).await.unwrap();
        // Undo takes back the last deletion only, not some other change (todo 1757).
        out_eq(page, "hello\n\nwörld\n").await;

        // Markdown typing shortcut, through the model's recognizers.
        select(
            page,
            &format!(
                "const e = {EDITOR}; getSelection().selectAllChildren(e); getSelection().collapseToEnd()"
            ),
        )
        .await;
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "# Title").await.unwrap();
        out_where(page, "a heading", |now| now.ends_with("# Title\n")).await;
        let heading: bool = js(page, &format!("!!{EDITOR}.querySelector('h1')")).await;
        assert!(heading);

        // Ctrl+B on a selection: the keymap, not the browser, marks it.
        select(
            page,
            &format!("getSelection().selectAllChildren({EDITOR}.querySelector('h1'))"),
        )
        .await;
        keyboard::press_with(page, KEY_B, CTRL).await.unwrap();
        out_where(page, "Ctrl+B to bold the heading", |now| {
            now.ends_with("# **Title**\n")
        })
        .await;

        // A foreign value resets the editor.
        page.evaluate("document.getElementById('replace').click()")
            .await
            .unwrap();
        out_eq(page, "").await;
        let paragraphs: u32 = js(
            page,
            &format!("{EDITOR}.querySelectorAll('[data-key]').length"),
        )
        .await;
        assert_eq!(paragraphs, 1);
        fixture.close().await.unwrap();
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
        assert!(js::<bool>(page, undo_disabled).await);

        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "ab").await.unwrap();
        out_eq(page, "ab\n").await;
        select_first_leaf(page).await;
        pointer::click(page, "#ext-bold").await.unwrap();
        out_eq(page, "**ab**\n").await;
        wait::for_js_true(
            page,
            &format!(
                "document.getElementById('ext-bold').getAttribute('aria-pressed') === 'true' && !{undo_disabled}"
            ),
            "the handle's state: bold, undo enabled",
        )
        .await
        .unwrap();

        pointer::click(page, "#ext-undo").await.unwrap();
        out_eq(page, "ab\n").await;
        fixture.close().await.unwrap();
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
    select(
        page,
        &format!("getSelection().selectAllChildren({EDITOR}.querySelector('[data-key]'))"),
    )
    .await;
}

#[test]
fn link_dialog_shortcut_help_block_menu_and_announcements() {
    const DIALOG: &str = "document.querySelector('[role=dialog]')";
    const HEADING: &str = "[...document.querySelectorAll('[role=menuitemradio]')].find(i => i.textContent.includes('Heading 2'))";
    const TRIGGER: &str = "document.querySelector('[role=toolbar] button[aria-haspopup]')";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "ab").await.unwrap();
        out_eq(page, "ab\n").await;

        // Mod+B from the keyboard is announced, on and off.
        editor_status(page).await;
        select_first_leaf(page).await;
        keyboard::press_with(page, KEY_B, CTRL).await.unwrap();
        announced(page, "Bold on").await;
        keyboard::press_with(page, KEY_B, CTRL).await.unwrap();
        out_eq(page, "ab\n").await;
        announced(page, "Bold off").await;

        // Mod+K: an unsafe scheme is a field error, a safe one links the selection.
        select_first_leaf(page).await;
        keyboard::press_with(page, KEY_K, CTRL).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement === document.querySelector('[role=dialog] input')",
            "focus in the link field",
        )
        .await
        .unwrap();
        keyboard::type_text(page, "javascript:alert(1)")
            .await
            .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[role=dialog] input').getAttribute('aria-invalid') === 'true'",
            "the unsafe link to be a field error",
        )
        .await
        .unwrap();
        page.evaluate(
            "(() => { const i = document.querySelector('[role=dialog] input'); i.select(); })()",
        )
        .await
        .unwrap();
        keyboard::type_text(page, "https://example.com")
            .await
            .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        out_eq(page, "[ab](https://example.com)\n").await;
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "focus back in the text",
        )
        .await
        .unwrap();

        // Mod+/ lists the keymap.
        keyboard::press_with(page, SLASH, CTRL).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "!!{DIALOG}?.textContent.includes('Keyboard shortcuts') && {DIALOG}.textContent.includes('Heading 2')"
            ),
            "the keymap listed",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "the keymap closed, focus back in the text",
        )
        .await
        .unwrap();

        // The block-type menu turns the paragraph into a heading.
        press(page, TRIGGER).await;
        wait::for_js_true(page, &format!("!!{HEADING}"), "the block menu")
            .await
            .unwrap();
        press(page, HEADING).await;
        out_eq(page, "## [ab](https://example.com)\n").await;
        wait::for_js_true(
            page,
            &format!("{TRIGGER}.textContent === 'Heading 2'"),
            "the menu trigger to name the heading",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
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

/// Polls `condition` (a JS expression) until it holds; a background page delays `ResizeObserver`.
async fn until(page: &Page, condition: &str) -> bool {
    js(
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
        let (overflow, rows): (bool, u32) = js(
            page,
            "(() => { const t = document.querySelector('[role=toolbar]'); \
               const tops = new Set([...t.querySelectorAll('button')].map(b => Math.round(b.getBoundingClientRect().top))); \
               return [t.scrollWidth > t.clientWidth, tops.size]; })()",
        )
        .await;
        assert!(!overflow, "the toolbar runs out of its column");
        assert_eq!(rows, 1, "the toolbar wraps");

        // A hidden toggle runs from the menu, on the caret's block.
        select(
            page,
            &format!(
                "const e = {EDITOR}; e.focus(); const l = e.querySelector('[data-key]'); getSelection().selectAllChildren(l); getSelection().collapseToEnd()"
            ),
        )
        .await;
        press(page, MORE).await;
        let quote = "[...document.querySelectorAll('[role=menuitemcheckbox]')].find(i => i.textContent.includes('Quote'))";
        assert!(until(page, quote).await, "no Quote in the More menu");
        press(page, quote).await;
        let quoted = until(
            page,
            "document.getElementById('out').textContent.startsWith('> above')",
        )
        .await;
        assert!(quoted, "{:?}", out(page).await);
        fixture.close().await.unwrap();
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
        let (_, _, before): (bool, u32, u32) = js(page, TOOLBAR_ROWS).await;
        page.evaluate(
            "(() => { const s = document.createElement('style'); \
               s.textContent = '[role=toolbar] button:not([aria-haspopup]) { min-width: 56px; }'; \
               document.head.append(s); })()",
        )
        .await
        .unwrap();
        let fewer = format!("{TOOLBAR_ROWS}[2] < {before}");
        assert!(until(page, &fewer).await, "nothing moved into More");
        let (overflow, rows, _): (bool, u32, u32) = js(page, TOOLBAR_ROWS).await;
        assert!(!overflow, "the toolbar runs out of its column");
        assert_eq!(rows, 1, "the toolbar wraps");
        fixture.close().await.unwrap();
    });
}

/// Dispatches `kind` (`copy` or `cut`) on the editor; `[text/plain, text/markdown, cancelled]`.
async fn clipboard(page: &Page, kind: &str) -> (String, String, bool) {
    js(
        page,
        &format!(
            "(() => {{ const d = new DataTransfer(); const e = new ClipboardEvent('{kind}', {{ clipboardData: d, bubbles: true, cancelable: true }}); \
               const kept = {EDITOR}.dispatchEvent(e); return [d.getData('text/plain'), d.getData('text/markdown'), !kept]; }})()"
        ),
    )
    .await
}

/// Both flavours carry the Markdown, so a code block keeps its fence (todo 1258).
#[test]
fn copy_writes_markdown_and_cut_edits_the_model() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        select(
            page,
            &format!(
                "const e = {EDITOR}; e.focus(); const l = [...e.querySelectorAll('[data-key]')]; \
                 const a = l[0].firstChild, b = l[l.length - 1].firstChild; getSelection().setBaseAndExtent(a, 0, b, b.length)"
            ),
        )
        .await;
        let (plain, markdown, cancelled) = clipboard(page, "copy").await;
        assert_eq!(plain, "above\n\n```\nlet x = 1;\n```\n\nbelow");
        assert_eq!(markdown, plain);
        assert!(cancelled, "the browser's own copy ran too");

        // Cut the first paragraph's text: through the model, so `onchange` sees it.
        select_first_leaf(page).await;
        let (plain, _, _) = clipboard(page, "cut").await;
        assert_eq!(plain, "above");
        let cut = until(
            page,
            "document.getElementById('out').textContent.startsWith('```')",
        )
        .await;
        assert!(cut, "{:?}", out(page).await);
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
                js(page, &format!("[...window.__events, {EDITOR}.innerHTML]")).await;
            assert!(
                held.is_ok(),
                "#out {last:?}, want {want:?}; events {events:#?}"
            );
            js::<bool>(page, "(window.__events = [], true)").await;
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

        keyboard::type_text(page, "ab").await.unwrap();
        out_eq(page, "ab\n").await;
        pointer::click(page, "[role=toolbar] button[aria-label=\"Bulleted list\"]")
            .await
            .unwrap();
        // Off the button: its tooltip rising mid-composition commits the composed text
        // (`- abcに日本`), todo 2044.
        pointer::move_to(page, pointer::Point { x: 2.0, y: 2.0 })
            .await
            .unwrap();
        out_eq(page, "- ab\n").await;
        wait::for_js_true(
            page,
            "document.querySelector('[role=toolbar] button[aria-label=\"Bulleted list\"]').getAttribute('aria-pressed') === 'true'",
            "the list toggle pressed",
        )
        .await
        .unwrap();

        keyboard::type_text(page, "c").await.unwrap();
        out_eq(page, "- abc\n").await;

        page.execute(ImeSetCompositionParams::new("に", 1, 1))
            .await
            .unwrap();
        page.execute(ImeSetCompositionParams::new("にほ", 2, 2))
            .await
            .unwrap();
        keyboard::insert_text(page, "日本").await.unwrap();
        out_eq(page, "- abc日本\n").await;
        keyboard::type_text(page, "d").await.unwrap();
        out_eq(page, "- abc日本d\n").await;

        // A code block is source with fences while the caret is in it, `CodeBlock` after.
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        pointer::click(page, "[role=toolbar] button[aria-label=\"Code block\"]")
            .await
            .unwrap();
        keyboard::type_text(page, "let x = 1;").await.unwrap();
        out_where(page, "the typed code", |now| {
            now.ends_with("```\nlet x = 1;\n```\n")
        })
        .await;
        wait::for_selector(page, "[role=textbox] [data-code=source] pre")
            .await
            .unwrap();
        page.evaluate("document.getElementById('replace').focus()")
            .await
            .unwrap();
        wait::for_selector(page, "[role=textbox] [data-code=view]")
            .await
            .unwrap();

        // ArrowDown at the leaf above enters the rendered block, which Chrome would skip.
        select(
            page,
            &format!(
                "const e = {EDITOR}; e.focus(); const l = e.querySelector('li [data-key]'); getSelection().selectAllChildren(l); getSelection().collapseToEnd()"
            ),
        )
        .await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_selector(page, "[role=textbox] [data-code=source]")
            .await
            .unwrap();
        keyboard::type_text(page, "Z").await.unwrap();
        out_where(page, "the typed key in the code block", |now| {
            now.ends_with("```\nZlet x = 1;\n```\n")
        })
        .await;
        fixture.close().await.unwrap();
    });
}

const TRAILING: &str = "/rich-text-editor/trailing";

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

/// Waits for the caret to have left the code block, which then renders as a view.
async fn left_code(page: &Page) {
    wait::for_js_true(
        page,
        "!document.querySelector('[role=textbox] [data-code=source]')",
        "the caret out of the code block",
    )
    .await
    .unwrap();
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

#[test]
fn toolbar_buttons_show_their_name_and_chord_in_a_tooltip() {
    const BOLD: &str = "[role=toolbar] button[aria-label=\"Bold\"]";
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_selector(page, BOLD).await.unwrap();
        let shortcut: String = js(
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
        let text: String = js(
            page,
            "document.querySelector('[role=tooltip]').textContent.replace(/\\s+/g, ' ').trim()",
        )
        .await;
        assert_eq!(text, "Bold Ctrl + B");
        fixture.close().await.unwrap();
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
            contrast::assert_clean_except(page, "body", contrast::LINE_NUMBERS)
                .await
                .unwrap();
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

#[test]
fn a_mention_list_follows_the_caret_and_takes_keys_through_intercept() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/mentions", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        let overlay = "document.querySelector('[data-overlay]')";
        let active = format!("{EDITOR}.getAttribute('aria-activedescendant')");
        let options = |count: u32| async move {
            let shown = format!("document.querySelectorAll('[role=option]').length === {count}");
            wait::for_js_true(page, &shown, &format!("{count} options")).await
        };

        editor_status(page).await;
        keyboard::type_text(page, "hi @a").await.unwrap();
        options(2).await.expect("ada and alan");
        // Todo 2052: the list's count is said, and again as typing narrows it.
        announced(page, "2 results").await;
        keyboard::type_text(page, "l").await.unwrap();
        options(1).await.expect("alan");
        announced(page, "1 result").await;
        keyboard::press(page, BACKSPACE).await.unwrap();
        options(2).await.unwrap();
        announced(page, "2 results").await;
        // Placed under the caret's line.
        holds(
            page,
            &format!(
                "(() => {{ const o = {overlay}.getBoundingClientRect(), l = {EDITOR}.querySelector('[data-key]').getBoundingClientRect(); return (getComputedStyle({overlay}).visibility === 'visible' && o.top >= l.bottom - 1 && o.left >= l.left) ? 'ok' : JSON.stringify([{overlay}.getAttribute('style'), o, l]); }})()"
            ),
            "the list under the line",
        )
        .await;
        let wired: bool = js(
            page,
            &format!(
                "{EDITOR}.getAttribute('aria-controls') === {overlay}.id && {EDITOR}.getAttribute('aria-autocomplete') === 'list'"
            ),
        )
        .await;
        assert!(wired);
        assert_eq!(js::<String>(page, &active).await, "mention-ada");
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
        wait::for_js_true(
            page,
            &format!("{active} === 'mention-alan'"),
            "ArrowDown to move to alan",
        )
        .await
        .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        out_eq(page, "hi \u{fffc} ").await;
        let mention: String = js(
            page,
            &format!("{EDITOR}.querySelector('.mention').textContent"),
        )
        .await;
        assert_eq!(mention, "@alan");
        wait::for_js_true(
            page,
            &format!("!{overlay} && !{EDITOR}.hasAttribute('aria-activedescendant')"),
            "the list gone after a pick",
        )
        .await
        .unwrap();

        // Escape closes the list and leaves the text.
        keyboard::type_text(page, "@g").await.unwrap();
        wait::for_js_true(page, &format!("!!{overlay}"), "the list for @g")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(page, &format!("!{overlay}"), "Escape to close the list")
            .await
            .unwrap();
        assert_eq!(out(page).await, "hi \u{fffc} @g");

        // The caller's toolbar button types the @; a click on an option picks it.
        keyboard::type_text(page, " ").await.unwrap();
        pointer::click(page, "[aria-label=\"Mention someone\"]")
            .await
            .unwrap();
        options(4).await.unwrap();
        pointer::click(page, "#mention-grace").await.unwrap();
        out_eq(page, "hi \u{fffc} @g \u{fffc} ").await;
        fixture.close().await.unwrap();
    });
}

/// Todo 2067: the list reaches past a clipping parent's edge and is still hit there.
#[test]
fn a_mention_list_is_not_clipped_by_its_parent() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/mentions", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "@a").await.unwrap();
        wait::for_visible(page, "#mention-alan").await.unwrap();
        holds(
            page,
            "(() => { const o = document.getElementById('mention-alan').getBoundingClientRect(); \
               const pane = document.getElementById('pane').getBoundingClientRect(); \
               const hit = document.elementFromPoint(o.left + o.width / 2, o.top + o.height / 2); \
               return o.bottom > pane.bottom && hit?.id === 'mention-alan' ? 'ok' \
                 : JSON.stringify([hit && hit.tagName + '#' + hit.id, o, pane]); })()",
            "the last option past the pane's edge, unclipped",
        )
        .await;
        fixture.close().await.unwrap();
    });
}

/// Todo 2067: in a modal the portaled list sits above it, and a press on it or an Escape
/// for it leaves the modal open.
#[test]
fn a_mention_list_in_a_modal_counts_as_inside() {
    const DIALOG: &str = "!!document.querySelector('[role=dialog]')";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/mentions-modal", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, "#open-modal").await.unwrap();
        wait::for_selector(page, "[role=dialog] [role=textbox]")
            .await
            .unwrap();
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "@a").await.unwrap();
        wait::for_visible(page, "#mention-alan").await.unwrap();
        pointer::click(page, "#mention-alan").await.unwrap();
        out_eq(page, "\u{fffc} ").await;
        assert!(
            js::<bool>(page, DIALOG).await,
            "a press on the list closed the modal"
        );

        keyboard::type_text(page, "@g").await.unwrap();
        wait::for_visible(page, "#mention-grace").await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[data-overlay]')",
            "Escape to close the list",
        )
        .await
        .unwrap();
        assert!(
            js::<bool>(page, DIALOG).await,
            "the list's Escape closed the modal"
        );
        fixture.close().await.unwrap();
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
        keyboard::type_text(page, "@").await.unwrap();
        wait::for_visible(page, "[data-overlay]").await.unwrap();
        holds(
            page,
            &format!(
                "(() => {{ const o = document.querySelector('[data-overlay]').getBoundingClientRect(), l = {leaf}.getBoundingClientRect(); return o.bottom <= l.top + 1 && o.top >= 0 ? 'ok' : JSON.stringify([o, l, innerHeight]); }})()"
            ),
            "the list above the line",
        )
        .await;

        // Scrolled to the middle, the list moves back under the line. Headless Chrome may hold
        // the scroll event until its next frame, so one is sent too.
        page.evaluate(
            "document.body.style.paddingBottom = innerHeight + 'px'; window.scrollBy(0, innerHeight / 2); document.dispatchEvent(new Event('scroll'))",
        )
        .await
        .unwrap();
        holds(
            page,
            &format!(
                "(() => {{ const o = document.querySelector('[data-overlay]').getBoundingClientRect(), l = {leaf}.getBoundingClientRect(); return o.top >= l.bottom - 1 ? 'ok' : JSON.stringify([o, l, innerHeight]); }})()"
            ),
            "the list back under the line",
        )
        .await;
        fixture.close().await.unwrap();
    });
}

#[test]
fn the_text_is_named_by_its_label_and_a_label_click_focuses_it() {
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, &format!("!!{EDITOR}"), "the editor")
            .await
            .unwrap();
        let tree = e2e::ax::snapshot(page, "[role=textbox]").await.unwrap();
        let first = tree.lines().next().unwrap_or_default();
        assert!(first.contains("textbox \"Snippet\""), "{first}");

        pointer::click(page, "label").await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "the label click to focus the text",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_readonly_editor_is_a_tab_stop_and_a_disabled_one_says_so() {
    const READONLY: &str = "document.querySelector('#readonly [role=textbox]')";
    const DISABLED: &str = "document.querySelector('#disabled [role=textbox]')";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, &format!("!!{DISABLED}"), "both editors")
            .await
            .unwrap();
        // [read-only tabindex, disabled tabindex, disabled aria-disabled]
        let (readonly, disabled, state): (String, Option<String>, String) = js(
            page,
            &format!(
                "[{READONLY}.getAttribute('tabindex'), {DISABLED}.getAttribute('tabindex'), {DISABLED}.getAttribute('aria-disabled')]"
            ),
        )
        .await;
        assert_eq!(readonly, "0");
        assert_eq!(disabled, None);
        assert_eq!(state, "true");

        // The label reaches the read-only text, as Tab does.
        pointer::click(page, "#readonly label").await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {READONLY}"),
            "the read-only text to take focus",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn toolbar_and_menu_actions_are_announced() {
    const BULLETS: &str = "[role=toolbar] button[aria-label=\"Bulleted list\"]";
    const HEADING: &str = "[...document.querySelectorAll('[role=menuitemradio]')].find(i => i.textContent.includes('Heading 3'))";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        editor_status(page).await;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "ab").await.unwrap();
        wait::for_js_true(page, &out_is("ab\n"), "the typed text")
            .await
            .unwrap();

        pointer::click(page, BULLETS).await.unwrap();
        announced(page, "Bulleted list on").await;
        pointer::click(page, BULLETS).await.unwrap();
        announced(page, "Bulleted list off").await;

        pointer::click(page, "[role=toolbar] button[aria-haspopup]")
            .await
            .unwrap();
        wait::for_js_true(page, &format!("!!{HEADING}"), "the block menu")
            .await
            .unwrap();
        press(page, HEADING).await;
        announced(page, "Heading 3").await;
        fixture.close().await.unwrap();
    });
}

/// Todo 1795: the toolbar is one tab stop the arrows walk, Tab leaves it for the text, and
/// the block-type menu runs from the keyboard.
#[test]
fn the_toolbar_and_its_block_menu_work_from_the_keyboard() {
    const ITEMS: &str =
        "[...document.querySelectorAll('[role=toolbar] button')].filter(b => b.offsetParent)";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "ab").await.unwrap();
        out_eq(page, "ab\n").await;
        let at = |index: &str| format!("{ITEMS}.indexOf(document.activeElement) === {index}");

        page.evaluate("document.activeElement.blur()")
            .await
            .unwrap();
        keyboard::tab_to(page, "[role=toolbar] button", 10)
            .await
            .unwrap();
        for (key, index, what) in [
            (keyboard::ARROW_RIGHT, "1", "ArrowRight to the second tool"),
            (
                keyboard::END,
                &format!("{ITEMS}.length - 1"),
                "End to the last tool",
            ),
            (keyboard::HOME, "0", "Home to the first tool"),
        ] {
            keyboard::press(page, key).await.unwrap();
            wait::for_js_true(page, &at(index), what).await.unwrap();
        }
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "Tab from the toolbar to the text",
        )
        .await
        .unwrap();

        // Back on the remembered tool, then right to the block-type trigger.
        keyboard::press_with(page, keyboard::TAB, keyboard::SHIFT)
            .await
            .unwrap();
        wait::for_js_true(page, &at("0"), "Shift+Tab back to the first tool")
            .await
            .unwrap();
        let trigger: usize = js(
            page,
            format!("{ITEMS}.findIndex(b => b.hasAttribute('aria-haspopup'))"),
        )
        .await;
        for step in 1..=trigger {
            keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
            wait::for_js_true(page, &at(&step.to_string()), "ArrowRight along the toolbar")
                .await
                .unwrap();
        }
        keyboard::press(page, ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.getAttribute('role') === 'menuitemradio'",
            "Enter to open the block menu with focus in it",
        )
        .await
        .unwrap();
        let on_heading =
            "document.activeElement?.textContent.includes('Heading 2') ?? false".to_string();
        for _ in 0..8 {
            if js::<bool>(page, &on_heading).await {
                break;
            }
            let before: String = js(page, "document.activeElement.textContent").await;
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            wait::for_js_true(
                page,
                &format!("document.activeElement.textContent !== {before:?}"),
                "ArrowDown to the next block type",
            )
            .await
            .unwrap();
        }
        assert!(js::<bool>(page, &on_heading).await, "no Heading 2 in reach");
        keyboard::press(page, ENTER).await.unwrap();
        out_eq(page, "## ab\n").await;
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "focus back in the text after the pick",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1795: on a phone the overflow tools sit in More, whose menu the arrows walk and
/// Escape closes back onto its trigger.
#[test]
fn the_more_menu_works_from_the_keyboard_on_a_phone() {
    const ACTIVE_ROLE: &str = "document.activeElement?.getAttribute('role')";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        assert!(until(page, MORE).await, "no More trigger");
        keyboard::tab_to(page, "[role=toolbar] button", 10)
            .await
            .unwrap();
        keyboard::press(page, keyboard::END).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {MORE}"),
            "End to the More trigger",
        )
        .await
        .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{ACTIVE_ROLE} === 'menuitemcheckbox'"),
            "Enter to open More with focus in it",
        )
        .await
        .unwrap();
        let first: String = js(page, "document.activeElement.textContent").await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{ACTIVE_ROLE} === 'menuitemcheckbox' && document.activeElement.textContent !== {first:?}"
            ),
            "ArrowDown to the next hidden tool",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {MORE}"),
            "Escape back onto More",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "Tab from the toolbar to the text",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A condition on `#out`, for `wait::for_js_true`.
fn out_is(want: &str) -> String {
    format!("document.getElementById('out').textContent === {want:?}")
}

const DELETE: Key = Key {
    key: "Delete",
    code: "Delete",
    vk: 46,
    text: None,
};

#[test]
fn ctrl_backspace_and_ctrl_delete_take_a_word_and_replacements_are_ignored() {
    block_on(async {
        const CODE: &str = "\n\n```rust\nlet x = 1;\n```\n";
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, &format!("!!{EDITOR}"), "the editor")
            .await
            .unwrap();
        // The caret starts before "intro"; moving it from script would race the model.
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "^").await.unwrap();
        wait::for_js_true(page, &out_is(&format!("^intro{CODE}")), "the typed key")
            .await
            .unwrap();
        keyboard::press_with(page, DELETE, CTRL).await.unwrap();
        wait::for_js_true(
            page,
            &out_is(&format!("^{CODE}")),
            "Ctrl+Delete to take a word",
        )
        .await
        .unwrap();

        keyboard::type_text(page, " one two").await.unwrap();
        keyboard::press_with(page, BACKSPACE, CTRL).await.unwrap();
        wait::for_js_true(
            page,
            &out_is(&format!("^ one {CODE}")),
            "Ctrl+Backspace to take a word",
        )
        .await
        .unwrap();

        // A substitution's target range is unknown: nothing is typed at the caret.
        let cancelled: bool = js(
            page,
            &format!(
                "(() => {{ const e = new InputEvent('beforeinput', {{ inputType: 'insertReplacementText', data: 'X', bubbles: true, cancelable: true }}); return !{EDITOR}.dispatchEvent(e); }})()"
            ),
        )
        .await;
        assert!(cancelled, "the browser would apply the replacement itself");
        keyboard::type_text(page, "z").await.unwrap();
        let typed = until(page, &out_is(&format!("^ one z{CODE}"))).await;
        let now: String = js(page, "document.getElementById('out').textContent").await;
        assert!(typed, "{now:?}");
        fixture.close().await.unwrap();
    });
}

/// The viewport point just past the `at`th character of the leaf reading `word`.
async fn point_in(page: &Page, word: &str, at: usize) -> pointer::Point {
    let (x, y): (f64, f64) = js(
        page,
        &format!(
            "(() => {{ const leaf = [...{EDITOR}.querySelectorAll('[data-key]')].find(l => l.textContent === '{word}'); \
               const r = document.createRange(); r.setStart(leaf.firstChild, {at}); r.setEnd(leaf.firstChild, {at} + 1); \
               const b = r.getBoundingClientRect(); return [b.left + 1, b.top + b.height / 2]; }})()"
        ),
    )
    .await;
    pointer::Point { x, y }
}

/// Todo 2062: the first press into an unfocused editor puts the caret where it lands, even
/// when a key follows before the browser's `selectionchange`; once fresh, once from a field.
#[test]
fn a_first_press_puts_the_caret_where_it_lands() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "[role=textbox] [data-key]")
            .await
            .unwrap();
        let at = point_in(page, "below", 2).await;
        pointer::click_at(page, at).await.unwrap();
        keyboard::type_text(page, "xy").await.unwrap();
        out_eq(page, "above\n\n```\nlet x = 1;\n```\n\nbexylow\n").await;

        page.evaluate(
            "(() => { const i = document.createElement('input'); i.id = 'other'; document.body.prepend(i); i.focus(); })()",
        )
        .await
        .unwrap();
        let at = point_in(page, "above", 2).await;
        pointer::click_at(page, at).await.unwrap();
        keyboard::type_text(page, "z").await.unwrap();
        out_eq(page, "abzove\n\n```\nlet x = 1;\n```\n\nbexylow\n").await;
        fixture.close().await.unwrap();
    });
}
