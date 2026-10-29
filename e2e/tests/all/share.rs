//! `Share` on the web: a press hands the data to `navigator.share`, a
//! cancel stays silent, a refusal is announced, and without a share sheet the
//! link is copied and "Copied" said. Android opens its share sheet over JNI.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_text};
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport, ax, wait};

const BARE: &str = "#bare button";
const LABELLED: &str = "#labelled button";
const FILE: &str = "#file button";
const FILES: &str = "#files button";

#[test]
fn it_meets_the_baseline() {
    Suite::new("share", "/share")
        .focusable(BARE)
        .focusable(LABELLED)
        .targets(BARE)
        .targets(LABELLED)
        .run();
}

/// Replaces `navigator.share` with one that records its data and settles as
/// `answer`: `ok`, or a `DOMException` name. `files` says whether `canShare` takes files.
fn stub_share(answer: &str, files: bool) -> String {
    format!(
        "(() => {{
            window.__shared = [];
            const define = (name, value) => Object.defineProperty(Navigator.prototype, name,
                {{ value, configurable: true, writable: true }});
            define('share', (data) => {{
                window.__shared.push(JSON.stringify({{
                    title: data.title, text: data.text, url: data.url,
                    files: (data.files ?? []).map((file) => `${{file.name}} ${{file.type}} ${{file.size}}`),
                }}));
                if ('{answer}' === 'pending') return new Promise((done) => {{ window.__settle = done; }});
                return '{answer}' === 'ok' ? Promise.resolve()
                    : Promise.reject(new DOMException('stubbed', '{answer}'));
            }});
            define('canShare', (data) => {files} || !data.files);
            return true;
        }})()"
    )
}

/// No `navigator.share`, and a clipboard that records what it was given.
const NO_SHARE_SHEET: &str = "(() => {
    window.__copied = [];
    Object.defineProperty(Navigator.prototype, 'share', { value: undefined, configurable: true, writable: true });
    Object.defineProperty(Navigator.prototype, 'clipboard', { configurable: true,
        get: () => ({ writeText: async (text) => { window.__copied.push(text); } }) });
    return true;
})()";

fn status_is(id: &str, text: &str) -> String {
    format!("document.querySelector('#{id} [role=status]').textContent === '{text}'")
}

fn outcome_is(text: &str) -> String {
    format!("document.querySelector('#outcome').textContent === '{text}'")
}

async fn strings(page: &chromiumoxide::Page, script: &str) -> Vec<String> {
    page.evaluate_bounded(script)
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Enter shares the title, text and url; the sheet is its own announcement, so
/// the status stays empty and focus stays on the button.
#[test]
fn a_press_hands_the_data_to_the_share_sheet() {
    block_on(async {
        let fixture = Fixture::open("/share", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        page.evaluate_bounded(stub_share("ok", true)).await.unwrap();
        keyboard::tab_to(page, BARE, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(page, &outcome_is("Shared"), "the share to settle")
            .await
            .unwrap();
        let shared = strings(page, "window.__shared").await;
        assert_eq!(
            shared,
            [
                r#"{"title":"Libero","text":"Components for Dioxus","url":"https://libero-ui.dev","files":[]}"#
            ]
        );
        assert!(
            page.evaluate_bounded(status_is("bare", ""))
                .await
                .unwrap()
                .into_value::<bool>()
                .unwrap()
        );
        let name: String = page
            .evaluate_bounded("document.activeElement.getAttribute('aria-label')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(name, "Share");

        fixture.console.assert_clean("sharing").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3383: while the sheet is open `data-state` says `pending` (no `aria-busy`: the sheet
/// announces itself, todo 4903); a second press shares nothing, and it goes when the share settles.
#[test]
fn the_button_is_pending_while_the_sheet_is_open() {
    block_on(async {
        let fixture = Fixture::open("/share", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let pending = |on: bool| {
            format!(
                "(() => {{ const b = document.querySelector('{BARE}'); \
                 return !b.hasAttribute('aria-busy') \
                 && (b.dataset.state ?? '').split(' ').includes('pending') === {on}; }})()"
            )
        };

        page.evaluate_bounded(stub_share("pending", true))
            .await
            .unwrap();
        keyboard::tab_to(page, BARE, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(page, &pending(true), "the pending state")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "window.__shared.length === 1",
            "the second press to share nothing",
        )
        .await
        .unwrap();

        page.evaluate_bounded("window.__settle()").await.unwrap();
        wait::for_js_true(page, &outcome_is("Shared"), "the share to settle")
            .await
            .unwrap();
        wait::for_js_true(page, &pending(false), "the pending state to go")
            .await
            .unwrap();
        fixture.console.assert_clean("a pending share").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A closed sheet (`AbortError`) says nothing; a refused one (`NotAllowedError`) says
/// "Share failed" and swaps the glyph for the alert one.
#[test]
fn a_cancel_is_silent_and_a_refusal_is_announced() {
    block_on(async {
        let fixture = Fixture::open("/share", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        let glyph = format!("document.querySelector('{BARE} svg').innerHTML");
        let rest: String = page
            .evaluate_bounded(glyph.clone())
            .await
            .unwrap()
            .into_value()
            .unwrap();

        page.evaluate_bounded(stub_share("AbortError", true))
            .await
            .unwrap();
        pointer::click(page, BARE).await.unwrap();
        wait::for_js_true(page, &outcome_is("Cancelled"), "the cancel to settle")
            .await
            .unwrap();
        assert!(
            page.evaluate_bounded(status_is("bare", ""))
                .await
                .unwrap()
                .into_value::<bool>()
                .unwrap()
        );
        let cancelled: String = page
            .evaluate_bounded(glyph.clone())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(cancelled, rest, "a cancel changed the glyph");

        page.evaluate_bounded(stub_share("NotAllowedError", true))
            .await
            .unwrap();
        pointer::click(page, BARE).await.unwrap();
        wait::for_js_true(
            page,
            &status_is("bare", "Share failed"),
            "the refusal to be announced",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &outcome_is(r#"Failed(\"NotAllowedError: stubbed\")"#),
            "the reason to reach onshare",
        )
        .await
        .unwrap();
        let failed: String = page
            .evaluate_bounded(glyph)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(failed, rest, "a refusal left the share glyph as it was");

        fixture.close().await.unwrap();
    });
}

/// Without `navigator.share` a press copies the url, turns the glyph into a
/// check and says "Copied"; leaving resets it.
#[test]
fn without_a_share_sheet_the_link_is_copied() {
    block_on(async {
        let fixture = Fixture::open("/share", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        page.evaluate_bounded(NO_SHARE_SHEET).await.unwrap();
        keyboard::tab_to(page, LABELLED, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &status_is("labelled", "Copied"),
            "the copy to be announced",
        )
        .await
        .unwrap();
        assert_eq!(
            strings(page, "window.__copied").await,
            ["https://libero-ui.dev"]
        );
        wait::for_js_true(page, &outcome_is("Copied"), "onshare to hear the copy")
            .await
            .unwrap();

        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &status_is("labelled", ""),
            "leaving to reset the status",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("copying the link").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Files go where `canShare` takes them; elsewhere the title still goes, without them.
#[test]
fn files_are_shared_where_can_share_takes_them() {
    block_on(async {
        let fixture = Fixture::open("/share", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        page.evaluate_bounded(stub_share("ok", true)).await.unwrap();
        pointer::click(page, FILE).await.unwrap();
        wait::for_js_true(page, &outcome_is("Shared"), "the file share to settle")
            .await
            .unwrap();
        assert_eq!(
            strings(page, "window.__shared").await,
            [r#"{"title":"Report","files":["report.csv text/csv 8"]}"#]
        );

        page.evaluate_bounded(stub_share("ok", false))
            .await
            .unwrap();
        page.evaluate_bounded("document.querySelector('#outcome').textContent = ''")
            .await
            .unwrap();
        pointer::click(page, FILE).await.unwrap();
        wait::for_js_true(
            page,
            "window.__shared.length === 1",
            "the share without files",
        )
        .await
        .unwrap();
        assert_eq!(
            strings(page, "window.__shared").await,
            [r#"{"title":"Report","files":[]}"#]
        );

        fixture.close().await.unwrap();
    });
}

/// Two files reach `navigator.share` together, in one call.
#[test]
fn several_files_are_shared_in_one_call() {
    block_on(async {
        let fixture = Fixture::open("/share", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        page.evaluate_bounded(stub_share("ok", true)).await.unwrap();
        pointer::click(page, FILES).await.unwrap();
        wait::for_js_true(page, &outcome_is("Shared"), "the share to settle")
            .await
            .unwrap();
        assert_eq!(
            strings(page, "window.__shared").await,
            [r#"{"title":"Reports","files":["a.csv text/csv 8","b.txt text/plain 5"]}"#]
        );

        fixture.console.assert_clean("sharing two files").unwrap();
        fixture.close().await.unwrap();
    });
}

async fn the_share_sheet_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    opens(d, BARE, "android.intent.action.SEND", None).await
}

/// Todo 3408: the chooser names the file, read from libero's own provider.
async fn the_share_sheet_opens_on_a_file<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    opens(d, FILE, "android.intent.action.SEND", Some("report.csv")).await
}

async fn the_share_sheet_opens_on_two_files<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    opens(
        d,
        FILES,
        "android.intent.action.SEND_MULTIPLE",
        Some("a.csv"),
    )
    .await
}

/// The chooser opens over the page for an intent of `action`, which the app logs as it
/// builds it, and shows `name` in its preview.
async fn opens<D: Driver>(d: &mut D, button: &str, action: &str, name: Option<&str>) -> Result<()> {
    d.clear_log().await?;
    d.click(button).await?;
    eventually(d, "the share sheet over the page", async |d| {
        d.exists("html[data-covered]").await
    })
    .await?;
    let intent = format!("libero share intent: {action} ");
    eventually(d, "the share intent in the log", async |d| {
        Ok(!d.log_lines(&intent).await?.is_empty())
    })
    .await?;
    if let Some(name) = name {
        eventually(d, "the file's name in the share sheet", async |d| {
            Ok(d.system_text().await?.contains(name))
        })
        .await?;
    }
    // The covered activity applies no edit until Back closes the share sheet.
    d.press_back().await?;
    eventually_text(d, "#outcome", "Shared", "the share outcome").await
}

e2e::scenario!(
    android_shares_the_link_through_the_share_sheet,
    "/share",
    the_share_sheet_opens,
    android_only("3351: the JNI share sheet is Android's")
);
e2e::scenario!(
    android_shares_a_file_through_the_share_sheet,
    "/share",
    the_share_sheet_opens_on_a_file,
    android_only("3351: the JNI share sheet is Android's")
);
e2e::scenario!(
    android_shares_two_files_through_the_share_sheet,
    "/share",
    the_share_sheet_opens_on_two_files,
    android_only("3361: ACTION_SEND_MULTIPLE is Android's")
);

/// The labelled mode is named by its text, described by `label`, and a disabled one takes no focus.
#[test]
fn the_labelled_button_is_named_by_its_text() {
    block_on(async {
        let fixture = Fixture::open("/share", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        let tree = ax::snapshot(page, "#labelled").await.unwrap();
        assert!(tree.contains(r#"button "Share""#), "{tree}");
        assert_eq!(
            ax::description(page, LABELLED).await.unwrap(),
            "Link to the docs"
        );

        // Todo 3838: soft-disabled, out of the Tab order.
        let soft: bool = page
            .evaluate_bounded(
                "(() => { const b = document.querySelector('#off button'); \
                 return !b.disabled && b.getAttribute('aria-disabled') === 'true' \
                     && b.tabIndex === -1; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(soft, "the disabled Share is a Tab stop");

        fixture.console.assert_clean("labelled share").unwrap();
        fixture.close().await.unwrap();
    });
}
