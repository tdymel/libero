//! `RichTextEditor` mention lists, send-on-Enter and composing soft keyboards, web and Android.

#[cfg(feature = "android")]
use super::rich_text_editor::KEY_Z;
use super::rich_text_editor::{
    EDITOR, announced, editor_status, holds, left_code, out, out_eq, out_where, select,
};
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, BACKSPACE, CTRL, ENTER};
use e2e::passes::{contrast, pointer};
use e2e::{Fixture, Viewport, js, wait};

const SENT: &str = "document.getElementById('sent').textContent";

/// Waits for `#sent`, the fixture's list of sent messages, to read `want`.
async fn sent_eq(page: &Page, want: &str) {
    let last = std::cell::RefCell::new(String::new());
    let held = wait::until(&format!("#sent to read {want:?}"), || async {
        let now: String = page.evaluate(SENT).await?.into_value()?;
        let passes = now == want;
        *last.borrow_mut() = now;
        Ok(passes)
    })
    .await;
    assert!(
        held.is_ok(),
        "#sent reads {:?}, want {want:?}",
        last.borrow()
    );
}

/// Pastes `text` as plain text into the editor.
async fn paste(page: &Page, text: &str) {
    page.evaluate(format!(
        "(() => {{ const d = new DataTransfer(); d.setData('text/plain', {text:?}); \
           {EDITOR}.dispatchEvent(new ClipboardEvent('paste', {{ clipboardData: d, bubbles: true, cancelable: true }})); }})()"
    ))
    .await
    .unwrap();
}

async fn editor_focused(page: &Page) -> bool {
    js(page, &format!("document.activeElement === {EDITOR}")).await
}

/// Todo 2715: with `onsubmit` Enter sends, Shift+Enter breaks the line, a code block keeps
/// Enter, `max_length` stops typing and cuts a paste, and the handle clears and refocuses.
#[test]
fn enter_sends_a_message_and_the_limit_and_handle_serve_a_chat_box() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/chat", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        let hint: String = js(page, &format!("{EDITOR}.getAttribute('enterkeyhint')")).await;
        assert_eq!(hint, "send");
        // Todo 2743: the send key is named on the text.
        let keys: String = js(page, &format!("{EDITOR}.getAttribute('aria-keyshortcuts')")).await;
        assert_eq!(keys, "Enter");
        editor_status(page).await;

        keyboard::type_text(page, "hello").await.unwrap();
        out_eq(page, "hello\n").await;
        keyboard::press(page, ENTER).await.unwrap();
        sent_eq(page, "hello").await;
        out_eq(page, "").await;
        wait::for_js_true(
            page,
            "document.getElementById('empty').textContent === 'true'",
            "the handle to read the cleared editor as empty",
        )
        .await
        .unwrap();
        assert!(editor_focused(page).await, "the clear took focus away");

        keyboard::type_text(page, "again").await.unwrap();
        out_eq(page, "again\n").await;
        keyboard::press_with(page, ENTER, keyboard::SHIFT)
            .await
            .unwrap();
        keyboard::type_text(page, "b").await.unwrap();
        wait::for_js_true(
            page,
            "document.getElementById('plain').textContent === 'again\\nb'",
            "the handle's plain text across a line break",
        )
        .await
        .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        sent_eq(page, "hello|again\nb").await;

        // max_length is 12: typing stops there and a paste is cut to it.
        keyboard::type_text(page, "123456789012345").await.unwrap();
        out_eq(page, "123456789012\n").await;
        // Todo 2734: meeting the limit is said, so a refused key is not silent.
        announced(page, "Character limit reached").await;
        paste(page, "ZZZ").await;
        select(
            page,
            &format!("getSelection().selectAllChildren({EDITOR}.querySelector('[data-key]'))"),
        )
        .await;
        paste(page, "abcdefghijklmnopqrstuvwxyz").await;
        out_eq(page, "abcdefghijkl\n").await;
        keyboard::press(page, ENTER).await.unwrap();
        sent_eq(page, "hello|again\nb|abcdefghijkl").await;

        // A code block keeps Enter; Mod+Enter leaves it, and then Enter sends.
        keyboard::type_text(page, "```").await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "x").await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "y").await.unwrap();
        out_where(page, "two lines of code", |now| {
            now.ends_with("```\nx\ny\n```\n")
        })
        .await;
        sent_eq(page, "hello|again\nb|abcdefghijkl").await;
        keyboard::press_with(page, ENTER, CTRL).await.unwrap();
        left_code(page).await;
        keyboard::type_text(page, "z").await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        sent_eq(page, "hello|again\nb|abcdefghijkl|x\ny\nz").await;

        // An empty doc sends nothing: Enter opens a line as usual.
        keyboard::press(page, ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{EDITOR}.querySelectorAll('[data-key]').length === 2"),
            "Enter on an empty box to open a line",
        )
        .await
        .unwrap();
        sent_eq(page, "hello|again\nb|abcdefghijkl|x\ny\nz").await;
        fixture.close().await.unwrap();
    });
}

/// `SubmitOn::ModEnter`: Enter stays a new paragraph, Mod+Enter sends; the handle's `clear`
/// and `focus` work from buttons outside the text.
#[test]
fn mod_enter_sends_a_comment_and_the_handle_clears_and_focuses() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/comment", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        let hint: bool = js(page, &format!("{EDITOR}.hasAttribute('enterkeyhint')")).await;
        assert!(!hint, "Enter is a new paragraph here, not send");
        let keys: String = js(page, &format!("{EDITOR}.getAttribute('aria-keyshortcuts')")).await;
        assert_eq!(keys, "Control+Enter");

        keyboard::type_text(page, "note").await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        keyboard::type_text(page, "more").await.unwrap();
        out_eq(page, "note\n\nmore\n").await;
        sent_eq(page, "").await;
        keyboard::press_with(page, ENTER, CTRL).await.unwrap();
        sent_eq(page, "note\nmore").await;
        out_eq(page, "").await;

        keyboard::type_text(page, "x").await.unwrap();
        out_eq(page, "x\n").await;
        pointer::click(page, "#clear").await.unwrap();
        out_eq(page, "").await;
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "focus back in the text",
        )
        .await
        .unwrap();

        page.evaluate("document.activeElement.blur()")
            .await
            .unwrap();
        pointer::click(page, "#focus").await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "the handle's focus",
        )
        .await
        .unwrap();
        keyboard::type_text(page, "y").await.unwrap();
        out_eq(page, "y\n").await;
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

        // Todo 2715: Enter sends, the cleared box takes the next words, `max_length` holds.
        let mut d = e2e::driver::Android::open("/rich-text-editor/chat")
            .await
            .unwrap();
        d.click("[role=textbox]").await.unwrap();
        eventually(&mut d, "the chat box to take focus", async |d| {
            d.is_focused("[role=textbox]").await
        })
        .await
        .unwrap();
        async fn text_is(d: &mut e2e::driver::Android, selector: &str, want: &str) {
            let mut last = String::new();
            let held = eventually(d, want, async |d| {
                last = d.text(selector).await?;
                Ok(last == want)
            })
            .await;
            assert!(held.is_ok(), "{selector} reads {last:?}, want {want:?}");
        }
        d.type_text("hello world").await.unwrap();
        text_is(&mut d, "#out", "hello world\n").await;
        d.press(ENTER).await.unwrap();
        text_is(&mut d, "#sent", "hello world").await;
        text_is(&mut d, "#out", "").await;
        d.type_text("abcdefghijklmno").await.unwrap();
        text_is(&mut d, "#out", "abcdefghijkl\n").await;
        d.finish("rich text editor").await.unwrap();
    });
}

/// Todos 2733, 3144, 1432 on a composing soft keyboard: Enter in a composed word sends it,
/// Shift+Enter there breaks the line, and Enter on an open mention list picks.
#[cfg(feature = "android")]
mod composing_keyboard {
    use super::*;

    #[test]
    fn android() {
        use e2e::android::{composing_ime, restore_ime};
        use e2e::driver::{Android, Driver, eventually};

        const RECORD: &str =
            "window.__composed = 0; addEventListener('compositionstart', () => __composed++)";

        async fn open(route: &str) -> Option<Android> {
            let mut d = Android::open(route).await.unwrap();
            if !composing_ime().await.unwrap() {
                eprintln!("no AOSP keyboard enabled: skipped");
                return None;
            }
            d.page().evaluate(RECORD).await.unwrap();
            d.click("[role=textbox]").await.unwrap();
            eventually(&mut d, "the editor to take focus", async |d| {
                d.is_focused("[role=textbox]").await
            })
            .await
            .unwrap();
            Some(d)
        }
        async fn text_is(d: &mut Android, selector: &str, want: &str) {
            let mut last = String::new();
            let held = eventually(d, want, async |d| {
                last = d.text(selector).await?;
                Ok(last == want)
            })
            .await;
            assert!(held.is_ok(), "{selector} reads {last:?}, want {want:?}");
        }

        e2e::android::block_on(async {
            let Some(mut d) = open("/rich-text-editor/chat-spellcheck").await else {
                return;
            };
            d.type_text("hello wor").await.unwrap();
            d.press(ENTER).await.unwrap();
            text_is(&mut d, "#sent", "hello wor").await;
            text_is(&mut d, "#out", "").await;
            d.type_text("ab").await.unwrap();
            d.press_shift(ENTER).await.unwrap();
            text_is(&mut d, "#out", "ab\\\n\n").await;
            text_is(&mut d, "#sent", "hello wor").await;
            let composed: u32 = js(d.page(), "window.__composed").await;

            let Some(mut d) = open("/rich-text-editor/mentions-spellcheck").await else {
                return;
            };
            d.type_text("hi @gr").await.unwrap();
            eventually(&mut d, "the mention list", async |d| {
                d.exists("#mention-list").await
            })
            .await
            .unwrap();
            // Placed under the caret, above the soft keyboard.
            let above: bool = js(
            d.page(),
            "document.getElementById('mention-list').getBoundingClientRect().bottom <= visualViewport.height",
        )
        .await;
            assert!(above, "the mention list sits under the soft keyboard");
            d.press(ENTER).await.unwrap();
            text_is(&mut d, "#out", "hi \u{fffc} ").await;
            let composed = composed + js::<u32>(d.page(), "window.__composed").await;
            restore_ime().await.unwrap();
            assert!(composed > 0, "the keyboard composed no word");
            d.finish("composing keyboard").await.unwrap();
        });
    }
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
        out_eq(page, "hi \u{fffc} @g ").await;
        pointer::click(page, "[aria-label=\"Mention someone\"]")
            .await
            .unwrap();
        out_eq(page, "hi \u{fffc} @g @").await;
        options(4).await.unwrap();
        // Todo 2102: on a loaded CI runner the click missed the pick; aim at a list that holds still.
        let last = std::cell::RefCell::new(String::new());
        wait::until("the list placed and still", || async {
            let rect: String = page
                .evaluate(format!(
                    "(() => {{ const o = {overlay}; return o && getComputedStyle(o).visibility === 'visible' \
                     ? JSON.stringify(o.getBoundingClientRect()) : ''; }})()"
                ))
                .await?
                .into_value()?;
            let still = !rect.is_empty() && rect == *last.borrow();
            *last.borrow_mut() = rect;
            Ok(still)
        })
        .await
        .unwrap();
        // A click that still misses is retried while the option shows; the pick must follow one.
        let gone = "!document.getElementById('mention-grace')";
        for _ in 0..3 {
            pointer::click(page, "#mention-grace").await.unwrap();
            let picked = wait::for_js_true(page, gone, "the list gone after a pick");
            if tokio::time::timeout(std::time::Duration::from_secs(2), picked)
                .await
                .is_ok()
            {
                break;
            }
        }
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
