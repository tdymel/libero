//! Probe 0b of the editor epic (1159): a `contenteditable`'s input events as Rust sees them,
//! on Chromium and in the Android WebView. Prints a report to `editor-probe-<arm>.txt` in the
//! run's artifacts; it asserts only what an editor would build on.

use std::fmt::Write as _;

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::{ImeSetCompositionParams, InsertTextParams};
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard;

/// Records every input event at the window, before (capture) and after (bubble) dioxus's
/// root listener; the gap is the handler's round trip. Also stamps each Rust echo of the selection.
const RECORDER: &str = r#"(() => {
    const out = document.querySelector('#selection');
    const fresh = !window.__probe;
    window.__probe?.observer.disconnect();
    window.__probe = { log: [], trip: [], echo: [] };
    window.__probe.observer = new MutationObserver(() => window.__probe.echo.push([out.textContent, performance.now()]));
    window.__probe.observer.observe(out, { childList: true, characterData: true, subtree: true });
    if (!fresh) return true;
    for (const t of ['keydown', 'beforeinput', 'input', 'compositionstart', 'compositionupdate', 'compositionend']) {
        window.addEventListener(t, e => {
            window.__probe.log.push([t, e.inputType ?? e.key ?? '', e.data ?? '', !!e.isComposing].join('|'));
            if (t === 'beforeinput') window.__probe.t0 = performance.now();
        }, true);
    }
    window.addEventListener('beforeinput', e => {
        window.__probe.trip.push([performance.now() - window.__probe.t0, e.defaultPrevented, e.cancelable]);
    });
    return true;
})()"#;

async fn js<T: serde::de::DeserializeOwned>(page: &Page, expression: &str) -> Result<T> {
    let json: String = page
        .evaluate(format!("JSON.stringify({expression})"))
        .await?
        .into_value()?;
    Ok(serde_json::from_str(&json)?)
}

async fn reset(page: &Page) -> Result<()> {
    js::<bool>(
        page,
        "(window.__probe.log = [], window.__probe.trip = [], true)",
    )
    .await?;
    Ok(())
}

async fn text(page: &Page) -> Result<String> {
    js(page, "document.querySelector('#editor').innerText").await
}

async fn rust_log(page: &Page) -> Result<Vec<String>> {
    let log: String = js(page, "document.querySelector('#log').textContent").await?;
    Ok(log.lines().map(str::to_string).collect())
}

/// Lets the last render land.
async fn settle<D: Driver>(d: &mut D) {
    for _ in 0..20 {
        d.idle().await;
    }
}

/// One step's events: the page's order, Rust's order, and the editor's text after it.
async fn section<D: Driver>(
    d: &mut D,
    page: &Page,
    report: &mut String,
    title: &str,
) -> Result<()> {
    settle(d).await;
    let page_log: Vec<String> = js(page, "window.__probe.log").await?;
    let trips: Vec<(f64, bool, bool)> = js(page, "window.__probe.trip").await?;
    writeln!(report, "## {title}")?;
    writeln!(report, "page: {}", page_log.join(" ; "))?;
    writeln!(report, "rust: {}", rust_log(page).await?.join(" ; "))?;
    let trips: Vec<String> = trips
        .iter()
        .map(|(ms, prevented, cancelable)| {
            format!(
                "{ms:.1}ms{}{}",
                if *prevented { " prevented" } else { "" },
                if *cancelable { "" } else { " uncancelable" }
            )
        })
        .collect();
    writeln!(report, "beforeinput round trips: {}", trips.join(", "))?;
    writeln!(report, "text: {:?}\n", text(page).await?)?;
    reset(page).await?;
    // The fixture's Rust log keeps growing; clear it through a fresh route instead of here.
    Ok(())
}

async fn focus_editor<D: Driver>(d: &mut D, page: &Page) -> Result<()> {
    d.click("#editor").await?;
    eventually(d, "the editor to take focus", async |d| {
        d.is_focused("#editor").await
    })
    .await?;
    js::<bool>(page, RECORDER).await?;
    Ok(())
}

/// Event order and `inputType` for typed text, Enter, Backspace and an IME composition.
async fn observe<D: Driver>(d: &mut D, page: &Page, report: &mut String) -> Result<()> {
    focus_editor(d, page).await?;
    keyboard::type_text(page, "ab").await?;
    section(d, page, report, "observe: CDP keys ab").await?;
    d.type_text("cd").await?;
    section(
        d,
        page,
        report,
        "observe: driver type_text cd (adb input text on Android)",
    )
    .await?;
    d.press(keyboard::ENTER).await?;
    section(d, page, report, "observe: driver Enter").await?;
    d.press(keyboard::BACKSPACE).await?;
    section(d, page, report, "observe: driver Backspace").await?;
    keyboard::press(page, keyboard::ENTER).await?;
    section(d, page, report, "observe: CDP Enter").await?;
    keyboard::press(page, keyboard::BACKSPACE).await?;
    section(d, page, report, "observe: CDP Backspace").await?;
    page.execute(ImeSetCompositionParams::new("n", 1, 1))
        .await?;
    page.execute(ImeSetCompositionParams::new("ni", 2, 2))
        .await?;
    page.execute(InsertTextParams::new("你")).await?;
    section(
        d,
        page,
        report,
        "observe: CDP IME composition n, ni, commit 你",
    )
    .await?;
    Ok(())
}

/// Rust's `prevent_default` on `beforeinput`: `x` and Enter must not reach the DOM.
async fn cancel<D: Driver>(d: &mut D, page: &Page, report: &mut String) -> Result<String> {
    focus_editor(d, page).await?;
    keyboard::type_text(page, "axb").await?;
    keyboard::press(page, keyboard::ENTER).await?;
    section(d, page, report, "cancel: CDP axb Enter").await?;
    d.type_text("xc").await?;
    d.press(keyboard::ENTER).await?;
    section(d, page, report, "cancel: driver xc Enter").await?;
    text(page).await
}

/// Typing at 12 ms a key (1051's pace); on the model route Rust owns the text.
async fn fast<D: Driver>(
    d: &mut D,
    page: &Page,
    report: &mut String,
    route: &str,
) -> Result<String> {
    focus_editor(d, page).await?;
    const TEXT: &str = "bookkeeper committee";
    d.type_burst(TEXT, 12).await?;
    settle(d).await;
    let typed = text(page).await?;
    let befores = rust_log(page)
        .await?
        .iter()
        .filter(|l| l.starts_with("beforeinput"))
        .count();
    writeln!(
        report,
        "## fast {route}: 12 ms a key\ntext: {typed:?}, Rust saw {befores} beforeinput for {} keys",
        TEXT.len()
    )?;
    let trips: Vec<(f64, bool, bool)> = js(page, "window.__probe.trip").await?;
    let mean = trips.iter().map(|t| t.0).sum::<f64>() / trips.len().max(1) as f64;
    let max = trips.iter().map(|t| t.0).fold(0.0, f64::max);
    writeln!(
        report,
        "beforeinput round trip mean {mean:.1} ms, max {max:.1} ms\n"
    )?;
    reset(page).await?;
    Ok(typed)
}

/// Selection push (a `selectionchange` listener sending over the eval channel, echoed by a render)
/// against pull (an eval per key release, timed in Rust).
async fn selection<D: Driver>(d: &mut D, page: &Page, report: &mut String) -> Result<()> {
    for _ in 0..6 {
        keyboard::press(page, keyboard::ARROW_LEFT).await?;
        tokio_sleep(150).await;
    }
    settle(d).await;
    let push: Vec<f64> = js(
        page,
        "window.__probe.echo.map(([t, at]) => at - Number(t.split(':')[3]))",
    )
    .await?;
    let pulls: String = js(page, "document.querySelector('#pulls').textContent").await?;
    writeln!(
        report,
        "## selection\npush (selectionchange -> Rust render), ms: {}",
        push.iter()
            .map(|m| format!("{m:.1}"))
            .collect::<Vec<_>>()
            .join(", ")
    )?;
    writeln!(report, "pull (eval round trip from Rust), us: {pulls}\n")?;
    Ok(())
}

async fn tokio_sleep(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

fn write_report(arm: &str, report: &str) {
    let dir = std::env::var_os("E2E_ARTIFACTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let path = dir.join(format!("editor-ime-probe-{arm}.txt"));
    std::fs::write(&path, report).unwrap();
    eprintln!("editor probe report: {}", path.display());
}

#[test]
#[ignore = "probe 1159 0b: a measurement, run by hand with --ignored"]
fn web() {
    e2e::browser::block_on(async {
        let mut report = String::from("# editor probe: web\n\n");
        let mut d = e2e::driver::Web::open("/editor-ime-probe/observe")
            .await
            .unwrap();
        let page = d.fixture.page.clone();
        observe(&mut d, &page, &mut report).await.unwrap();
        d.finish("observe").await.unwrap();
        let mut d = e2e::driver::Web::open("/editor-ime-probe/cancel")
            .await
            .unwrap();
        let page = d.fixture.page.clone();
        let cancelled = cancel(&mut d, &page, &mut report).await.unwrap();
        d.finish("cancel").await.unwrap();
        let mut d = e2e::driver::Web::open("/editor-ime-probe/model")
            .await
            .unwrap();
        let page = d.fixture.page.clone();
        let typed = fast(&mut d, &page, &mut report, "model").await.unwrap();
        d.finish("model").await.unwrap();
        let mut d = e2e::driver::Web::open("/editor-ime-probe/observe")
            .await
            .unwrap();
        let page = d.fixture.page.clone();
        fast(&mut d, &page, &mut report, "observe").await.unwrap();
        selection(&mut d, &page, &mut report).await.unwrap();
        d.finish("observe").await.unwrap();
        write_report("web", &report);
        assert_eq!(cancelled, "abc");
        assert_eq!(typed, "bookkeeper committee");
    });
}

#[cfg(feature = "android")]
#[test]
fn android() {
    e2e::android::block_on(async {
        let mut report = String::from("# editor probe: android\n\n");
        let mut d = e2e::driver::Android::open("/editor-ime-probe/observe")
            .await
            .unwrap();
        let page = d.page().clone();
        observe(&mut d, &page, &mut report).await.unwrap();
        let mut d = e2e::driver::Android::open("/editor-ime-probe/cancel")
            .await
            .unwrap();
        let cancelled = cancel(&mut d, &page, &mut report).await.unwrap();
        let mut d = e2e::driver::Android::open("/editor-ime-probe/model")
            .await
            .unwrap();
        let typed = fast(&mut d, &page, &mut report, "model").await.unwrap();
        let mut d = e2e::driver::Android::open("/editor-ime-probe/observe")
            .await
            .unwrap();
        let observed = fast(&mut d, &page, &mut report, "observe").await.unwrap();
        selection(&mut d, &page, &mut report).await.unwrap();
        write_report("android", &report);
        d.finish("editor probe").await.unwrap();
        assert_eq!(cancelled, "abc");
        assert_eq!(typed, "bookkeeper committee");
        assert_eq!(observed, "bookkeeper committee");
    });
}
