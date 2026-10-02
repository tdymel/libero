//! Editor epic 1159 probes on the web: editing events from a `contenteditable` reach Rust and
//! `prevent_default` in `onbeforeinput` cancels them (0e); `CodeBlock` re-highlight cost (0c).
//! Findings: `codebase/platform/editor-input-events`, `codebase/platform/editor-markdown-wasm`.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventType, ImeSetCompositionParams,
};
use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, BACKSPACE, CTRL, ENTER, Key};
use e2e::{Fixture, Viewport, clock, wait};

const KEY_B: Key = Key {
    key: "b",
    code: "KeyB",
    vk: 66,
    text: None,
};

async fn log(page: &Page) -> Vec<String> {
    let text: String = page
        .evaluate("document.getElementById('log').textContent")
        .await
        .unwrap()
        .into_value()
        .unwrap();
    text.lines().map(str::to_owned).collect()
}

async fn editor_text(page: &Page) -> String {
    page.evaluate("document.getElementById('editor').innerText")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

async fn focus_editor(page: &Page) {
    page.evaluate("document.getElementById('editor').focus()")
        .await
        .unwrap();
}

/// Ctrl+V as an editing command: the browser's own paste, `beforeinput` included.
async fn paste(page: &Page, text: &str) {
    let copy = format!(
        "(() => {{ const t = document.createElement('textarea'); t.value = {text:?}; \
         document.body.append(t); t.select(); const ok = document.execCommand('copy'); \
         t.remove(); return ok; }})()"
    );
    page.evaluate(
        EvaluateParams::builder()
            .expression(copy)
            .user_gesture(true)
            .build()
            .unwrap(),
    )
    .await
    .unwrap();
    focus_editor(page).await;
    for kind in [
        DispatchKeyEventType::RawKeyDown,
        DispatchKeyEventType::KeyUp,
    ] {
        page.execute(
            DispatchKeyEventParams::builder()
                .r#type(kind.clone())
                .key("v")
                .code("KeyV")
                .windows_virtual_key_code(86)
                .modifiers(CTRL)
                .commands(if kind == DispatchKeyEventType::RawKeyDown {
                    vec!["paste".to_owned()]
                } else {
                    vec![]
                })
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    }
}

/// A `paste` event built in the page, for the `data_transfer()` path alone.
const SYNTHETIC_PASTE: &str = "(() => { const dt = new DataTransfer(); \
    dt.setData('text/plain', 'synthetic'); \
    document.getElementById('editor').dispatchEvent(new ClipboardEvent('paste', \
    { clipboardData: dt, bubbles: true, cancelable: true })); })()";

/// Typing, Backspace, Enter, Ctrl+B, paste and an IME composition, once passed through and
/// once cancelled. Run with `--nocapture` to see every logged `inputType`.
#[test]
fn editing_events_reach_rust_and_cancel() {
    block_on(async {
        let fixture = Fixture::open("/editor-probe", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        for cancel in [false, true] {
            if cancel {
                page.evaluate(
                    "document.getElementById('cancel').click(); \
                     document.getElementById('editor').textContent = ''",
                )
                .await
                .unwrap();
                clock::settle(page).await.unwrap();
            }
            let before = log(page).await.len();
            focus_editor(page).await;
            keyboard::type_text(page, "ab").await.unwrap();
            keyboard::press(page, BACKSPACE).await.unwrap();
            keyboard::press(page, ENTER).await.unwrap();
            keyboard::type_text(page, "c").await.unwrap();
            page.evaluate(
                "(() => { const s = getSelection(); s.selectAllChildren(document.getElementById('editor')); })()",
            )
            .await
            .unwrap();
            keyboard::press_with(page, KEY_B, CTRL).await.unwrap();
            page.evaluate("getSelection().collapseToEnd()")
                .await
                .unwrap();
            paste(page, "pasted").await;
            page.evaluate(SYNTHETIC_PASTE).await.unwrap();
            focus_editor(page).await;
            page.evaluate("getSelection().selectAllChildren(document.getElementById('editor')); getSelection().collapseToEnd()")
                .await
                .unwrap();
            page.execute(ImeSetCompositionParams::new("に", 1, 1))
                .await
                .unwrap();
            page.execute(ImeSetCompositionParams::new("にほ", 2, 2))
                .await
                .unwrap();
            keyboard::insert_text(page, "日本").await.unwrap();

            const KINDS: [&str; 10] = [
                "beforeinput insertText",
                "beforeinput deleteContentBackward",
                "beforeinput insertParagraph",
                "beforeinput formatBold",
                "beforeinput insertCompositionText",
                "compositionstart",
                "compositionupdate",
                "compositionend",
                "paste Some(\"synthetic\")",
                "document selectionchange",
            ];
            let has = |lines: &[String], prefix: &str| lines.iter().any(|l| l.starts_with(prefix));
            // A miss is reported by kind below.
            let _ = wait::until("every editing event in the log", || async {
                let lines = log(page).await.split_off(before);
                Ok(KINDS.iter().all(|kind| has(&lines, kind)))
            })
            .await;

            let lines = log(page).await.split_off(before);
            let text = editor_text(page).await;
            let bold: bool = page
                .evaluate("!!document.querySelector('#editor b, #editor strong')")
                .await
                .unwrap()
                .into_value()
                .unwrap();
            eprintln!("cancel={cancel} text={text:?} bold={bold}");
            for line in &lines {
                eprintln!("  {line}");
            }

            for kind in KINDS {
                assert!(
                    has(&lines, kind),
                    "cancel={cancel}: no `{kind}` reached Rust"
                );
            }
            if cancel {
                assert!(!bold, "a cancelled formatBold still bolded");
                assert!(
                    !text.contains('a') && !text.contains('c') && !text.contains("pasted"),
                    "a cancelled beforeinput still changed the text: {text:?}"
                );
            } else {
                assert!(
                    text.contains('a') && text.contains('c'),
                    "typed text missing: {text:?}"
                );
                assert!(bold, "Ctrl+B did not bold");
            }
        }
        fixture.close().await.unwrap();
    });
}

/// Median ms from an `input` event on `#source` to the block's DOM update, over `runs` edits.
const MEASURE: &str = "async (lines, runs) => { \
    const unit = ['fn greet(name: &str) -> String {', \
        '    let n = name.len(); // count', '    format!(\"hi {name} {}\", n + 1)', '}', '']; \
    const base = Array.from({ length: lines }, (_, i) => unit[i % unit.length]).join('\\n'); \
    const ta = document.getElementById('source'); \
    const block = document.getElementById('block'); \
    const edit = (value) => new Promise(resolve => { \
        const t0 = performance.now(); \
        const mo = new MutationObserver(() => { mo.disconnect(); resolve(performance.now() - t0); }); \
        mo.observe(block, { subtree: true, childList: true, characterData: true }); \
        ta.value = value; ta.dispatchEvent(new Event('input', { bubbles: true })); }); \
    await edit(base); \
    const times = []; \
    for (let i = 0; i < runs; i++) times.push(await edit(base + 'x'.repeat(i + 1))); \
    times.sort((a, b) => a - b); \
    return times[Math.floor(runs / 2)]; }";

/// Probe 0c: whole-block re-highlight per keystroke, Rust grammar against the plain control.
/// A measurement, no gate: `E2E_RELEASE=1 ... -- editor_probe::highlight --ignored --nocapture`.
#[test]
#[ignore = "measurement only"]
fn highlight_cost_per_keystroke() {
    block_on(async {
        let fixture = Fixture::open("/editor-probe/highlight", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        for lines in [50, 200] {
            let mut medians = Vec::new();
            for rust in [true, false] {
                let set = format!(
                    "(() => {{ const c = document.getElementById('rust'); if (c.checked !== {rust}) c.click(); }})()"
                );
                page.evaluate(set).await.unwrap();
                clock::settle(page).await.unwrap();
                let ms: f64 = page
                    .evaluate(format!("({MEASURE})({lines}, 30)"))
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                medians.push(ms);
            }
            eprintln!(
                "lines={lines} rust={:.2}ms plain={:.2}ms ratio={:.2}",
                medians[0],
                medians[1],
                medians[0] / medians[1]
            );
        }
        fixture.close().await.unwrap();
    });
}
