//! `ShareButton` on the web: a press hands the data to `navigator.share`, a
//! cancel stays silent, a refusal is announced, and without a share sheet the
//! link is copied and "Copied" said.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, ax, wait};

const BARE: &str = "#bare button";
const LABELLED: &str = "#labelled button";
const FILE: &str = "#file button";

#[test]
fn it_meets_the_baseline() {
    Suite::new("share_button", "/share-button")
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
    page.evaluate(script).await.unwrap().into_value().unwrap()
}

/// Enter shares the title, text and url; the sheet is its own announcement, so
/// the status stays empty and focus stays on the button.
#[test]
fn a_press_hands_the_data_to_the_share_sheet() {
    block_on(async {
        let fixture = Fixture::open("/share-button", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        page.evaluate(stub_share("ok", true)).await.unwrap();
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
            page.evaluate(status_is("bare", ""))
                .await
                .unwrap()
                .into_value::<bool>()
                .unwrap()
        );
        let name: String = page
            .evaluate("document.activeElement.getAttribute('aria-label')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(name, "Share");

        fixture.console.assert_clean("sharing").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A closed sheet (`AbortError`) says nothing; a refused one (`NotAllowedError`) says "Share failed".
#[test]
fn a_cancel_is_silent_and_a_refusal_is_announced() {
    block_on(async {
        let fixture = Fixture::open("/share-button", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        page.evaluate(stub_share("AbortError", true)).await.unwrap();
        pointer::click(page, BARE).await.unwrap();
        wait::for_js_true(page, &outcome_is("Cancelled"), "the cancel to settle")
            .await
            .unwrap();
        assert!(
            page.evaluate(status_is("bare", ""))
                .await
                .unwrap()
                .into_value::<bool>()
                .unwrap()
        );

        page.evaluate(stub_share("NotAllowedError", true))
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

        fixture.close().await.unwrap();
    });
}

/// Without `navigator.share` a press copies the url, turns the glyph into a
/// check and says "Copied"; leaving resets it.
#[test]
fn without_a_share_sheet_the_link_is_copied() {
    block_on(async {
        let fixture = Fixture::open("/share-button", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        page.evaluate(NO_SHARE_SHEET).await.unwrap();
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
        let fixture = Fixture::open("/share-button", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        page.evaluate(stub_share("ok", true)).await.unwrap();
        pointer::click(page, FILE).await.unwrap();
        wait::for_js_true(page, &outcome_is("Shared"), "the file share to settle")
            .await
            .unwrap();
        assert_eq!(
            strings(page, "window.__shared").await,
            [r#"{"title":"Report","files":["report.csv text/csv 8"]}"#]
        );

        page.evaluate(stub_share("ok", false)).await.unwrap();
        page.evaluate("document.querySelector('#outcome').textContent = ''")
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

/// The labelled mode is named by its text, described by `label`, and a disabled one takes no focus.
#[test]
fn the_labelled_button_is_named_by_its_text() {
    block_on(async {
        let fixture = Fixture::open("/share-button", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let tree = ax::snapshot(page, "#labelled").await.unwrap();
        assert!(tree.contains(r#"button "Share""#), "{tree}");
        assert_eq!(
            ax::description(page, LABELLED).await.unwrap(),
            "Link to the docs"
        );

        let unfocusable: bool = page
            .evaluate(
                "(() => { const b = document.querySelector('#off button'); \
                 b.focus(); return b.disabled && document.activeElement !== b; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(unfocusable, "the disabled ShareButton took focus");

        fixture.console.assert_clean("labelled share").unwrap();
        fixture.close().await.unwrap();
    });
}
