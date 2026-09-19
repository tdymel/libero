//! `ThemeToggle`: every press moves the scheme one step round the cycle,
//! and the name always says where the next press goes (todo 406).

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, ax, wait};

const BUTTON: &str = "#scheme";

/// At rest only: a press persists the scheme, and `Suite` cannot shield the
/// other units' pages from that (see `SHIELD_STORAGE`).
#[test]
fn it_meets_the_baseline() {
    Suite::new("theme_toggle", "/theme-toggle")
        .focusable(BUTTON)
        .targets(BUTTON)
        .dark_snapshot("the label names the next scheme, which follows the platform's")
        .run();
}

const TOGGLE: &str = "#split > button";
const CHEVRON: &str = "#split > div > button";

/// The split button: both halves are tab stops with a ring and 24px, and the
/// picker's menu is checked in the open state. Nothing is picked, so nothing
/// persists.
#[test]
fn the_split_button_meets_the_baseline() {
    Suite::new("theme_toggle_themes", "/theme-toggle/themes")
        .focusable(TOGGLE)
        .focusable(CHEVRON)
        .targets(TOGGLE)
        .targets(CHEVRON)
        .state(
            "open",
            &[Step::TabTo(CHEVRON), Step::Press(keyboard::ENTER)],
            "[role=menu]",
        )
        .dark_snapshot("the label names the next scheme, which follows the platform's")
        .run();
}

/// Escape closes the theme menu and hands focus back to the chevron.
#[test]
fn escape_returns_focus_to_the_chevron() {
    block_on(async {
        let fixture = Fixture::open("/theme-toggle/themes", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, CHEVRON, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, "[role=menuitemradio][aria-checked=true]")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        focus::wait_for_focus(page, CHEVRON, "Escape")
            .await
            .unwrap();

        fixture.console.assert_clean("the theme menu").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Every test page shares one browser profile, and a stored `dark` would
/// restore into every fixture opened after it. Presses write to a stand-in,
/// which also records what the component tried to persist.
const SHIELD_STORAGE: &str = r#"(() => {
    window.__realStorage = window.localStorage;
    window.__stored = [];
    const items = new Map();
    Object.defineProperty(window, 'localStorage', { configurable: true, value: {
        getItem: k => items.has(k) ? items.get(k) : null,
        setItem: (k, v) => { items.set(k, String(v)); window.__stored.push(k + '=' + v); },
        removeItem: k => items.delete(k),
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

/// `None` is following the platform: the attribute is removed.
async fn wait_for_pin(page: &Page, pin: Option<&str>) {
    let expected = match pin {
        Some(scheme) => format!("'{scheme}'"),
        None => "null".to_string(),
    };
    wait::for_js_true(
        page,
        &format!("document.documentElement.getAttribute('data-lsx-theme') === {expected}"),
        &format!("the document root to be pinned to {expected}"),
    )
    .await
    .unwrap();
}

async fn text_colour(page: &Page) -> String {
    page.evaluate("getComputedStyle(document.querySelector('#page-text')).color")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Under a light platform: dark, then light, then back to following it. Enter,
/// Space and a click each take one step, and focus stays on the button.
#[test]
fn each_press_steps_the_cycle_and_renames_the_button() {
    block_on(async {
        let fixture = Fixture::open("/theme-toggle", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(SHIELD_STORAGE).await.unwrap();
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("prefers-color-scheme", "light")])
                .build(),
        )
        .await
        .unwrap();

        wait_for_name(page, "Switch to the dark theme").await;
        wait_for_pin(page, None).await;
        let light_text = text_colour(page).await;

        keyboard::tab_to(page, BUTTON, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait_for_pin(page, Some("dark")).await;
        wait_for_name(page, "Switch to the light theme").await;
        assert_ne!(
            text_colour(page).await,
            light_text,
            "the page text kept its light colour under the dark pin"
        );
        focus::assert_focused(page, BUTTON, "pressing Enter")
            .await
            .unwrap();

        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait_for_pin(page, Some("light")).await;
        wait_for_name(page, "Follow the system theme").await;
        focus::assert_focused(page, BUTTON, "pressing Space")
            .await
            .unwrap();

        pointer::click(page, BUTTON).await.unwrap();
        wait_for_pin(page, None).await;
        wait_for_name(page, "Switch to the dark theme").await;

        let (stored, leaked): (Vec<String>, Option<String>) = page
            .evaluate("[window.__stored, window.__realStorage.getItem('lsx-color-scheme')]")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            stored,
            [
                "lsx-color-scheme=dark",
                "lsx-color-scheme=light",
                "lsx-color-scheme=system"
            ],
            "each press persists the setting it moved to"
        );
        assert_eq!(leaked, None, "a press reached the shared localStorage");

        fixture
            .console
            .assert_clean("cycling the colour scheme")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
