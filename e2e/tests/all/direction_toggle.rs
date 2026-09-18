//! `DirectionToggle`: every press turns the document's `dir`, and the name
//! always says where the next press goes.

use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, ax, wait};

const BUTTON: &str = "#direction";

/// At rest only: a press persists the direction, and `Suite` cannot shield the
/// other units' pages from that (see `SHIELD_STORAGE`).
#[test]
fn it_meets_the_baseline() {
    Suite::new("direction_toggle", "/direction-toggle")
        .focusable(BUTTON)
        .targets(BUTTON)
        .run();
}

/// Every test page shares one browser profile, and a stored `rtl` would
/// restore into every fixture opened after it. Presses write to a stand-in,
/// which also records what the component tried to persist.
const SHIELD_STORAGE: &str = r#"(() => {
    window.__realStorage = window.localStorage;
    window.__stored = [];
    const items = new Map();
    Object.defineProperty(window, 'localStorage', { configurable: true, value: {
        getItem: k => items.has(k) ? items.get(k) : null,
        setItem: (k, v) => { items.set(k, String(v)); window.__stored.push(k + '=' + v); },
        removeItem: k => { items.delete(k); window.__stored.push('-' + k); },
    }});
})()"#;

/// The computed accessible name, from the node's own snapshot line.
async fn name_of(page: &Page) -> String {
    let tree = ax::snapshot(page, BUTTON).await.unwrap();
    let first = tree.lines().next().unwrap_or_default();
    first
        .split_once('"')
        .and_then(|(_, rest)| rest.rsplit_once('"'))
        .map(|(name, _)| name.to_string())
        .unwrap_or_default()
}

async fn wait_for_name(page: &Page, expected: &str) {
    let result = wait::until(&format!("the button to be named {expected:?}"), || async {
        Ok(name_of(page).await == expected)
    })
    .await;
    if let Err(error) = result {
        panic!("{error}; it is named {:?}", name_of(page).await);
    }
}

async fn wait_for_dir(page: &Page, dir: &str) {
    let result = wait::for_js_true(
        page,
        &format!("document.documentElement.dir === '{dir}'"),
        &format!("the document root to be {dir:?}"),
    )
    .await;
    if let Err(error) = result {
        let now: String = page
            .evaluate("document.documentElement.dir")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        panic!("{error}; it is {now:?}");
    }
}

/// Whether the page text sits after the button in reading order: to its right
/// left to right, to its left right to left.
async fn text_follows_button(page: &Page) -> bool {
    page.evaluate(
        "(() => { const t = document.querySelector('#page-text').getBoundingClientRect(); \
         const b = document.querySelector('#direction').getBoundingClientRect(); \
         return document.documentElement.dir === 'rtl' ? t.right <= b.left : t.left >= b.right; })()",
    )
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

/// Enter, Space and a click each turn the direction, and focus stays on the button.
#[test]
fn each_press_turns_the_direction_and_renames_the_button() {
    block_on(async {
        let fixture = Fixture::open("/direction-toggle", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(SHIELD_STORAGE).await.unwrap();

        wait_for_name(page, "Switch to right-to-left text").await;
        wait_for_dir(page, "").await;
        assert!(
            text_follows_button(page).await,
            "the row is out of order at rest"
        );

        keyboard::tab_to(page, BUTTON, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait_for_name(page, "Switch to left-to-right text").await;
        wait_for_dir(page, "rtl").await;
        assert!(
            text_follows_button(page).await,
            "the row did not turn under rtl"
        );
        focus::assert_focused(page, BUTTON, "pressing Enter")
            .await
            .unwrap();

        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait_for_dir(page, "ltr").await;
        wait_for_name(page, "Switch to right-to-left text").await;
        focus::assert_focused(page, BUTTON, "pressing Space")
            .await
            .unwrap();

        pointer::click(page, BUTTON).await.unwrap();
        wait_for_dir(page, "rtl").await;

        let (stored, leaked, pressed): (Vec<String>, Option<String>, Option<String>) = page
            .evaluate(
                "[window.__stored, window.__realStorage.getItem('lsx-direction'), \
                 document.querySelector('#direction').getAttribute('aria-pressed')]",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            stored,
            [
                "lsx-direction=rtl",
                "lsx-direction=ltr",
                "lsx-direction=rtl"
            ],
            "each press persists the direction it turned to"
        );
        assert_eq!(leaked, None, "a press reached the shared localStorage");
        assert_eq!(pressed, None, "an action button carries no pressed state");

        fixture
            .console
            .assert_clean("turning the direction")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 853: `clear` drops the kept choice and the root's `dir`, as if none was made.
#[test]
fn clearing_drops_the_choice_and_the_roots_dir() {
    block_on(async {
        let fixture = Fixture::open("/direction-toggle", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(SHIELD_STORAGE).await.unwrap();
        let kept_is =
            |kept: &str| format!("document.querySelector('#kept').textContent === '{kept}'");

        wait::for_js_true(page, &kept_is("none"), "nothing kept at rest")
            .await
            .unwrap();
        pointer::click(page, BUTTON).await.unwrap();
        wait_for_dir(page, "rtl").await;
        wait::for_js_true(page, &kept_is("rtl"), "the press kept")
            .await
            .unwrap();

        pointer::click(page, "#clear").await.unwrap();
        wait::for_js_true(page, &kept_is("none"), "the choice dropped")
            .await
            .unwrap();
        let (has_dir, stored): (bool, Vec<String>) = page
            .evaluate("[document.documentElement.hasAttribute('dir'), window.__stored]")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!has_dir, "the root kept a `dir`");
        assert_eq!(stored, ["lsx-direction=rtl", "-lsx-direction"]);
        wait_for_name(page, "Switch to right-to-left text").await;

        fixture
            .console
            .assert_clean("clearing the direction")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
