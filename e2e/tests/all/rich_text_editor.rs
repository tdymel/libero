//! `RichTextEditor` on the web: every edit goes through the model, the caret follows it,
//! and a controlled parent's late echo neither resets the doc nor moves the caret.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::ImeSetCompositionParams;
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, BACKSPACE, CTRL, ENTER, Key};
use e2e::passes::{live_region, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, js, wait};

pub(super) const KEY_B: Key = Key {
    key: "b",
    code: "KeyB",
    vk: 66,
    text: None,
};

pub(super) const KEY_Z: Key = Key {
    key: "z",
    code: "KeyZ",
    vk: 90,
    text: None,
};

pub(super) const EDITOR: &str = "document.querySelector('[role=textbox]')";

const OUT: &str = "document.getElementById('out').textContent";

pub(super) async fn out(page: &Page) -> String {
    js(page, OUT).await
}

/// Waits for `#out` to pass `check`; fails with what it read last.
pub(super) async fn out_where(page: &Page, what: &str, check: impl Fn(&str) -> bool) {
    let last = std::cell::RefCell::new(String::new());
    let held = wait::until(what, || async {
        let now: String = page.evaluate_bounded(OUT).await?.into_value()?;
        let passes = check(&now);
        *last.borrow_mut() = now;
        Ok(passes)
    })
    .await;
    assert!(held.is_ok(), "{what}: #out reads {:?}", last.borrow());
}

pub(super) async fn out_eq(page: &Page, want: &str) {
    out_where(page, &format!("#out to read {want:?}"), |now| now == want).await;
}

/// Waits out the editor's caret sync after an edit: it replies on a 0 ms timer, queued
/// before this one. Until then the editor ignores selection moves.
pub(super) const SYNCED: &str = "new Promise(r => setTimeout(r))";

/// Runs `script`, which moves the DOM selection, and returns once the model has it: the
/// editor's `selectionchange` listener, added before ours, hands it over before ours runs.
pub(super) async fn select(page: &Page, script: &str) {
    page.evaluate_bounded(format!(
        "{SYNCED}.then(() => new Promise((r, fail) => {{ \
           document.addEventListener('selectionchange', () => r(true), {{ once: true }}); \
           setTimeout(() => fail(new Error('the selection did not move')), 5000); {script}; }}))"
    ))
    .await
    .unwrap();
}

/// Waits for `check`, a script returning 'ok' or what it saw instead, to return 'ok'.
pub(super) async fn holds(page: &Page, check: &str, what: &str) {
    if wait::for_js_true(page, &format!("{check} === 'ok'"), what)
        .await
        .is_err()
    {
        panic!("{what}: {}", js::<String>(page, check).await);
    }
}

/// A real press on the element `expression` finds: a JS `.click()` skips the `mousedown`
/// that would take focus from the caret (todo 1795).
pub(super) async fn press(page: &Page, expression: &str) {
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
pub(super) async fn editor_status(page: &Page) {
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
pub(super) async fn announced(page: &Page, message: &str) {
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

/// Todo 1610 (WCAG 2.1.2): Tab indents a list item, Escape then Tab leaves the editor,
/// and the way out is in the text's description.
#[test]
fn escape_then_tab_leaves_a_list() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_bounded(format!("{EDITOR}.focus()"))
            .await
            .unwrap();
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
        page.evaluate_bounded(format!("{EDITOR}.focus()"))
            .await
            .unwrap();

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
        page.evaluate_bounded("document.getElementById('replace').click()")
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

pub(super) async fn select_first_leaf(page: &Page) {
    select(
        page,
        &format!("getSelection().selectAllChildren({EDITOR}.querySelector('[data-key]'))"),
    )
    .await;
}

/// Polls `condition` (a JS expression) until it holds; a background page delays `ResizeObserver`.
pub(super) async fn until(page: &Page, condition: &str) -> bool {
    js(
        page,
        &format!(
            "new Promise(r => {{ const end = Date.now() + 5000; const tick = () => {{ if ({condition}) r(true); else if (Date.now() > end) r(false); else setTimeout(tick, 50); }}; tick(); }})"
        ),
    )
    .await
}

/// `[no horizontal overflow, distinct item rows, hidden toggles]` of the toolbar.
pub(super) const TOOLBAR_ROWS: &str = "(() => { const t = document.querySelector('[role=toolbar]'); \
    const tops = new Set([...t.querySelectorAll('button')].map(b => Math.round(b.getBoundingClientRect().top))); \
    return [t.scrollWidth > t.clientWidth, tops.size, t.querySelectorAll('button').length]; })()";

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

/// Pastes `text` as plain text into the editor.
async fn paste_text(page: &Page, text: &str) {
    page.evaluate_bounded(format!(
        "(() => {{ const d = new DataTransfer(); d.setData('text/plain', {text:?}); \
           {EDITOR}.dispatchEvent(new ClipboardEvent('paste', {{ clipboardData: d, bubbles: true, cancelable: true }})); }})()"
    ))
    .await
    .unwrap();
}

/// Todo 2716: a typed URL, a typed `[text](url)` and a pasted URL become links.
#[test]
fn typed_and_pasted_urls_and_typed_markdown_links_become_links() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_bounded(format!("{EDITOR}.focus()"))
            .await
            .unwrap();

        keyboard::type_text(page, "see https://a.example ok")
            .await
            .unwrap();
        out_eq(page, "see [https://a.example](https://a.example) ok\n").await;
        let href: String = js(
            page,
            &format!("{EDITOR}.querySelector('a')?.getAttribute('href') ?? ''"),
        )
        .await;
        assert_eq!(href, "https://a.example");

        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "[docs](https://b.example)")
            .await
            .unwrap();
        out_where(page, "the typed Markdown link", |now| {
            now.ends_with("\n\n[docs](https://b.example)\n")
        })
        .await;
        let text: String = js(
            page,
            &format!("[...{EDITOR}.querySelectorAll('a')].map(a => a.textContent).join('|')"),
        )
        .await;
        assert_eq!(text, "https://a.example|docs");

        keyboard::press(page, ENTER).await.unwrap();
        paste_text(page, "https://c.example").await;
        out_where(page, "the pasted URL", |now| {
            now.ends_with("\n\n[https://c.example](https://c.example)\n")
        })
        .await;

        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "word").await.unwrap();
        select(
            page,
            &format!(
                "const l = [...{EDITOR}.querySelectorAll('[data-key]')]; const t = l[l.length - 1].firstChild; \
                 getSelection().setBaseAndExtent(t, 0, t, t.length)"
            ),
        )
        .await;
        paste_text(page, "https://d.example").await;
        out_where(page, "the URL over a selection", |now| {
            now.ends_with("\n\n[word](https://d.example)\n")
        })
        .await;

        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "https://e.example")
            .await
            .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        out_where(page, "Enter after a URL", |now| {
            now.contains("[https://e.example](https://e.example)")
        })
        .await;
        fixture.close().await.unwrap();
    });
}

/// Todo 2725: Backspace right after a link rule turns the link back to text.
#[test]
fn backspace_right_after_a_made_link_turns_it_back_to_text() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let links = format!("{EDITOR}.querySelectorAll('a').length");
        page.evaluate_bounded(format!("{EDITOR}.focus()"))
            .await
            .unwrap();

        keyboard::type_text(page, "https://a.example ")
            .await
            .unwrap();
        wait::for_js_true(page, &format!("{links} === 1"), "the autolink")
            .await
            .unwrap();
        keyboard::press(page, keyboard::BACKSPACE).await.unwrap();
        wait::for_js_true(page, &format!("{links} === 0"), "the autolink undone")
            .await
            .unwrap();

        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "[docs](https://b.example)")
            .await
            .unwrap();
        wait::for_js_true(page, &format!("{links} === 1"), "the typed link")
            .await
            .unwrap();
        keyboard::press(page, keyboard::BACKSPACE).await.unwrap();
        wait::for_js_true(page, &format!("{links} === 0"), "the typed link undone")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
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
fn toolbar_buttons_keep_the_caret_and_composition_lands_in_the_model() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_bounded(format!("{EDITOR}.focus()"))
            .await
            .unwrap();

        keyboard::type_text(page, "ab").await.unwrap();
        out_eq(page, "ab\n").await;
        // The pointer stays on the button, its tooltip open through the composition (todo 2044).
        pointer::click(page, "[role=toolbar] button[aria-label=\"Bulleted list\"]")
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

        // Of the DOM moves probed, only focus leaving and coming back commits "に" this way
        // (`- abcに日本`), so a failure names the focus moves.
        page.evaluate_bounded(
            "window.__moves = []; for (const type of ['focusin', 'focusout'])
             document.addEventListener(type, (e) => __moves.push(type + ' ' + e.target.outerHTML.slice(0, 80)), true)",
        )
        .await
        .unwrap();
        page.execute_bounded(ImeSetCompositionParams::new("に", 1, 1))
            .await
            .unwrap();
        page.execute_bounded(ImeSetCompositionParams::new("にほ", 2, 2))
            .await
            .unwrap();
        keyboard::insert_text(page, "日本").await.unwrap();
        let composed = wait::until("the composition in the model", || async {
            let now: String = page.evaluate_bounded(OUT).await?.into_value()?;
            Ok(now == "- abc日本\n")
        })
        .await;
        if composed.is_err() {
            let moves: String = js(page, "JSON.stringify(__moves)").await;
            panic!("#out reads {:?}; focus moves: {moves}", out(page).await);
        }
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
        page.evaluate_bounded("document.getElementById('replace').focus()")
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

pub(super) const TRAILING: &str = "/rich-text-editor/trailing";

/// Waits for the caret to have left the code block, which then renders as a view.
pub(super) async fn left_code(page: &Page) {
    wait::for_js_true(
        page,
        "!document.querySelector('[role=textbox] [data-code=source]')",
        "the caret out of the code block",
    )
    .await
    .unwrap();
}

/// A condition on `#out`, for `wait::for_js_true`.
pub(super) fn out_is(want: &str) -> String {
    format!("document.getElementById('out').textContent === {want:?}")
}

const DELETE: Key = Key {
    key: "Delete",
    code: "Delete",
    vk: 46,
    text: None,
};

/// Dispatches an `insertReplacementText` of `data` over the first `word` in the editor's
/// text nodes, as a spelling menu does; `true` if the editor cancelled it.
async fn replace_word(page: &Page, word: &str, data: &str) -> bool {
    js(
        page,
        &format!(
            "(() => {{ const w = document.createTreeWalker({EDITOR}, NodeFilter.SHOW_TEXT), nodes = []; let all = ''; \
             while (w.nextNode()) {{ nodes.push([w.currentNode, all.length]); all += w.currentNode.data; }} \
             const i = all.indexOf({word:?}), j = i + {word:?}.length; \
             const at = (k, end) => nodes.map(([n, s]) => [n, k - s]).find(([n, o]) => o >= 0 && (end ? o <= n.length : o < n.length)); \
             const [sc, so] = at(i, false), [ec, eo] = at(j, true); \
             const range = new StaticRange({{ startContainer: sc, startOffset: so, endContainer: ec, endOffset: eo }}); \
             const e = new InputEvent('beforeinput', {{ inputType: 'insertReplacementText', data: {data:?}, targetRanges: [range], bubbles: true, cancelable: true }}); \
             return !{EDITOR}.dispatchEvent(e); }})()"
        ),
    )
    .await
}

#[test]
fn a_replacement_across_a_bold_run_is_one_undo_step() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_bounded(format!("{EDITOR}.focus()"))
            .await
            .unwrap();
        keyboard::type_text(page, "brwon").await.unwrap();
        out_eq(page, "brwon\n").await;
        select(
            page,
            &format!(
                "const w = document.createTreeWalker({EDITOR}, NodeFilter.SHOW_TEXT); \
                 while (w.nextNode() && !w.currentNode.data.includes('brwon')); \
                 getSelection().setBaseAndExtent(w.currentNode, 2, w.currentNode, 4)"
            ),
        )
        .await;
        keyboard::press_with(page, KEY_B, CTRL).await.unwrap();
        out_eq(page, "br**wo**n\n").await;

        // Three text nodes: the replacement takes the start's marks.
        assert!(replace_word(page, "brwon", "brown").await);
        out_eq(page, "brown\n").await;
        keyboard::press_with(page, KEY_Z, CTRL).await.unwrap();
        out_eq(page, "br**wo**n\n").await;
        fixture.close().await.unwrap();
    });
}

#[test]
fn ctrl_backspace_and_ctrl_delete_take_a_word_and_replacements_apply_at_their_range() {
    block_on(async {
        const CODE: &str = "\n\n```rust\nlet x = 1;\n```\n";
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, &format!("!!{EDITOR}"), "the editor")
            .await
            .unwrap();
        // The caret starts before "intro"; moving it from script would race the model.
        page.evaluate_bounded(format!("{EDITOR}.focus()"))
            .await
            .unwrap();
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

        // Without a target range and with a collapsed caret the word is unknown: nothing is typed.
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

        // With a target range it replaces there, away from the caret.
        assert!(replace_word(page, "one", "won").await);
        out_eq(page, &format!("^ won z{CODE}")).await;
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

        page.evaluate_bounded(
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

/// Todo 2071: a key the editor acts on, pressed while the input after a press still waits
/// for the model, runs after that input.
#[test]
fn a_key_behind_held_input_runs_after_it() {
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
        out_eq(page, "above\n\n```\nlet x = 1;\n```\n\nbelow\n").await;
        // One task: the press's caret moves to `ab|ove` unreported, as Chrome queues its
        // `selectionchange` behind input, and the typed key is still held when Enter comes.
        page.evaluate_bounded(format!(
            "(() => {{ const r = {EDITOR}; \
               const leaf = [...r.querySelectorAll('[data-key]')].find(l => l.textContent === 'above'); \
               document.getSelection().collapse(leaf.firstChild, 2); \
               r.dispatchEvent(new InputEvent('beforeinput', {{ inputType: 'insertText', data: 'x', bubbles: true, cancelable: true }})); \
               r.dispatchEvent(new KeyboardEvent('keydown', {{ key: 'Enter', code: 'Enter', bubbles: true, cancelable: true }})); }})()"
        ))
        .await
        .unwrap();
        out_eq(page, "abx\n\nove\n\n```\nlet x = 1;\n```\n\nbelow\n").await;
        fixture.close().await.unwrap();
    });
}

/// Todo 2072: a press while the model's caret is still syncing moves the model's caret
/// with no key after it, so a toolbar button then acts at the press.
#[test]
fn a_press_while_syncing_moves_the_model_caret() {
    const BULLETS: &str = "[role=toolbar] button[aria-label=\"Bulleted list\"]";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, BULLETS).await.unwrap();
        let at = point_in(page, "below", 2).await;
        pointer::click_at(page, at).await.unwrap();
        keyboard::type_text(page, "x").await.unwrap();
        out_eq(page, "above\n\n```\nlet x = 1;\n```\n\nbexlow\n").await;

        // The editor's synced reply is a 0 ms timer: hold it while the press lands.
        page.evaluate_bounded(
            "(() => { const real = window.setTimeout; window.__realTimeout = real; window.__synced = []; \
               window.setTimeout = (f, ms, ...a) => ms ? real(f, ms, ...a) : (window.__synced.push(f), 0); })()",
        )
        .await
        .unwrap();
        keyboard::type_text(page, "y").await.unwrap();
        out_eq(page, "above\n\n```\nlet x = 1;\n```\n\nbexylow\n").await;
        wait::for_js_true(page, "window.__synced.length > 0", "the synced reply held")
            .await
            .unwrap();
        // The editor's listener came first, so its report is out once ours sees the press.
        // A selectionchange from its own caret may still be queued: that one does not count.
        page.evaluate_bounded(
            "document.addEventListener('selectionchange', () => { if (document.getSelection().anchorNode?.textContent === 'above') window.__moved = true; })",
        )
        .await
        .unwrap();
        let at = point_in(page, "above", 2).await;
        pointer::click_at(page, at).await.unwrap();
        wait::for_js_true(
            page,
            "window.__moved === true",
            "the press to move the selection",
        )
        .await
        .unwrap();
        page.evaluate_bounded(
            "(() => { window.setTimeout = window.__realTimeout; window.__synced.forEach(f => f()); })()",
        )
        .await
        .unwrap();

        pointer::click(page, BULLETS).await.unwrap();
        out_eq(page, "- above\n\n```\nlet x = 1;\n```\n\nbexylow\n").await;
        fixture.close().await.unwrap();
    });
}

/// Todo 2624: a toolbar press right after a press in the text reports that press at once,
/// not with its late `selectionchange`; one after a key does not.
#[test]
fn a_toolbar_press_reports_the_fresh_press_at_once() {
    const BULLETS: &str = "[role=toolbar] button[aria-label=\"Bulleted list\"]";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, BULLETS).await.unwrap();
        let reads: Vec<u32> = page
            .evaluate_bounded(format!(
                "(() => {{ const root = document.querySelector('[data-lsx-rich-text]'); \
                   const toolbar = document.querySelector({BULLETS:?}); \
                   const real = document.getSelection; let reads = 0; \
                   document.getSelection = function () {{ reads += 1; return real.call(document); }}; \
                   const press = (el) => el.dispatchEvent(new MouseEvent('mousedown', {{ bubbles: true, button: 0 }})); \
                   press(root); const editor = reads; press(toolbar); const fresh = reads - editor; \
                   press(root); root.dispatchEvent(new KeyboardEvent('keydown', {{ key: 'Shift', bubbles: true }})); \
                   const before = reads; press(toolbar); const stale = reads - before; \
                   delete document.getSelection; return [fresh, stale]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            reads[0] > 0,
            "the toolbar press read no selection: {reads:?}"
        );
        assert_eq!(reads[1], 0, "a press after a key reported: {reads:?}");
        fixture.close().await.unwrap();
    });
}
