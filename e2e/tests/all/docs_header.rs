//! The docs site's header row (todo 4829): under a finger every control is a 44px box and the
//! row still fits a 320px screen.

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Every visible control of the row: its id or label, size and edges.
const CONTROLS: &str = "[...document.querySelectorAll('#docs-header button, #docs-header a')]
    .filter(el => el.offsetParent !== null)
    .map(el => {
        const r = el.getBoundingClientRect();
        return { name: el.id || el.getAttribute('aria-label') || el.textContent.trim(),
                 width: r.width, height: r.height, left: r.left, right: r.right };
    })";

#[derive(Debug, serde::Deserialize)]
struct Control {
    name: String,
    width: f64,
    height: f64,
    left: f64,
    right: f64,
}

async fn open_coarse_at_320() -> Fixture {
    let fixture = Fixture::open("/docs-header", Viewport::Mobile)
        .await
        .unwrap();
    let page = &fixture.page;
    page.execute_bounded(SetDeviceMetricsOverrideParams::new(320, 800, 1.0, true))
        .await
        .unwrap();
    crate::hit_area::emulate_coarse_pointer(page).await;
    page.reload_bounded().await.unwrap();
    wait::for_visible(page, "#docs-header").await.unwrap();
    wait::for_js_true(
        page,
        "document.querySelectorAll('#docs-header button').length >= 5",
        "the header buttons",
    )
    .await
    .unwrap();
    fixture
}

const SHELL: &str = include_str!("../../../docs/src/shell.rs");
const COPY: &str = include_str!("../../fixtures/src/docs_header.rs");

/// The header's parts both files hold; `shell_header` adds the router's logo link.
const SHARED: &[&str] = &[
    "first_item_sx",
    "header_sx",
    "burger_sx",
    "site_tools",
    "search_button",
];

/// The item `fn name` from its signature to its closing brace, comments and indent dropped.
fn item(source: &str, name: &str) -> Option<String> {
    let start = source.find(&format!("fn {name}("))?;
    let end = start + source[start..].find("\n}\n")?;
    Some(
        source[start..end]
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// The fixture is a copy (e2e never includes docs source, todo 951): it must not drift.
#[test]
fn the_fixture_copy_matches_the_docs_shell() {
    for name in SHARED {
        let docs = item(SHELL, name).unwrap_or_else(|| panic!("the docs lost `fn {name}`"));
        let copy = item(COPY, name).unwrap_or_else(|| panic!("the copy lost `fn {name}`"));
        assert_eq!(
            copy, docs,
            "`fn {name}` in e2e/fixtures/src/docs_header.rs drifted from docs/src/shell.rs"
        );
    }
}

/// Todo 4829: the burger and the site tools were 28px and 30px boxes that touched each other.
#[test]
fn a_finger_gets_44px_header_controls_that_fit_320px() {
    block_on(async {
        let fixture = open_coarse_at_320().await;
        let page = &fixture.page;

        let controls: Vec<Control> = e2e::js(page, CONTROLS).await;
        let small: Vec<String> = controls
            .iter()
            .filter(|c| c.width < 43.5 || c.height < 43.5)
            .map(|c| format!("{} {}x{}", c.name, c.width, c.height))
            .collect();
        assert!(small.is_empty(), "controls under 44px: {small:?}");

        let overflow: f64 = e2e::js(
            page,
            "Math.max(document.documentElement.scrollWidth, document.querySelector('#docs-header').scrollWidth) - 320",
        )
        .await;
        assert!(overflow <= 0.5, "the header runs {overflow}px past 320px");
        let past: Vec<String> = controls
            .iter()
            .filter(|c| c.left < -0.5 || c.right > 320.5)
            .map(|c| format!("{} {}..{}", c.name, c.left, c.right))
            .collect();
        assert!(past.is_empty(), "controls outside the screen: {past:?}");

        fixture.console.assert_clean("the docs header").unwrap();
        fixture.close().await.unwrap();
    });
}
