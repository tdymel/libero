//! ArrowLeft/ArrowRight swap under `dir="rtl"` (115), one test per family. `dir` is set on
//! `<html>`, as an RTL app does, so portaled popups inherit it.

use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, Key};
use e2e::{Fixture, Viewport, wait};

async fn open_rtl(route: &str) -> Fixture {
    open_in(route, "rtl").await
}

/// Opens `route` with `dir` set on `<html>`, as an app sets it.
pub(crate) async fn open_in(route: &str, dir: &str) -> Fixture {
    let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
    fixture
        .page
        .evaluate(format!("document.documentElement.dir = '{dir}'"))
        .await
        .unwrap();
    wait::for_js_true(
        &fixture.page,
        &format!("getComputedStyle(document.body).direction === '{dir}'"),
        "the page to take its direction",
    )
    .await
    .unwrap();
    fixture
}

/// Presses `key`, then waits for `condition` (JS) to hold.
async fn press_until(page: &Page, key: Key, condition: &str, what: &str) {
    keyboard::press(page, key).await.unwrap();
    wait::for_js_true(page, condition, what)
        .await
        .unwrap_or_else(|e| panic!("{what}: {e}"));
}

/// The roving strips (`Tabs`, `SegmentedControl`, `RadioGroup`).
#[test]
fn a_tab_strip_steps_forward_on_arrow_left() {
    block_on(async {
        let fixture = open_rtl("/tabs").await;
        let page = &fixture.page;
        keyboard::tab_to(page, "[role=tab]", 10).await.unwrap();
        let selected = |n: usize| {
            format!(
                "document.querySelector('[role=tablist] > [role=tab]:nth-child({n})')\
                 .getAttribute('aria-selected') === 'true'"
            )
        };
        press_until(
            page,
            keyboard::ARROW_LEFT,
            &selected(2),
            "ArrowLeft to the second tab",
        )
        .await;
        press_until(
            page,
            keyboard::ARROW_RIGHT,
            &selected(1),
            "ArrowRight back to the first",
        )
        .await;
        fixture.console.assert_clean("RTL tabs").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `Menubar` along the bar.
#[test]
fn a_menubar_steps_forward_on_arrow_left() {
    block_on(async {
        let fixture = open_rtl("/menubar-docs").await;
        let page = &fixture.page;
        let focused = |n: usize| {
            format!("document.activeElement?.getAttribute('data-menubar-index') === '{n}'")
        };
        keyboard::tab_to(page, "[role=menubar] [data-menubar-index=\"0\"]", 10)
            .await
            .unwrap();
        press_until(page, keyboard::ARROW_LEFT, &focused(1), "ArrowLeft to Edit").await;
        press_until(
            page,
            keyboard::ARROW_RIGHT,
            &focused(0),
            "ArrowRight back to File",
        )
        .await;
        fixture.console.assert_clean("RTL menubar").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `Menu`: ArrowLeft opens a submenu, drawn on the left of its parent, and
/// ArrowRight closes it back onto its item.
#[test]
fn a_submenu_opens_on_the_left_on_arrow_left() {
    block_on(async {
        let fixture = open_rtl("/menubar-docs").await;
        let page = &fixture.page;
        // Centred, so either side has room and no collision flip decides it.
        page.evaluate(
            "document.querySelector('[role=menubar]').parentElement.style.marginInline = 'auto'",
        )
        .await
        .unwrap();
        keyboard::tab_to(page, "[role=menubar] [data-menubar-index=\"0\"]", 10)
            .await
            .unwrap();
        let on = |label: &str| format!("document.activeElement?.textContent.trim() === '{label}'");
        press_until(
            page,
            keyboard::ARROW_DOWN,
            &on("New"),
            "ArrowDown to open File",
        )
        .await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        press_until(
            page,
            keyboard::ARROW_DOWN,
            &on("Open recent"),
            "down to Open recent",
        )
        .await;
        press_until(
            page,
            keyboard::ARROW_LEFT,
            &on("notes.md"),
            "ArrowLeft into the submenu",
        )
        .await;
        wait::for_js_true(
            page,
            "(() => { const [parent, child] = document.querySelectorAll('[role=menu]'); \
             return child.getBoundingClientRect().right <= parent.getBoundingClientRect().left + 1; })()",
            "the submenu to sit left of its parent",
        )
        .await
        .unwrap();
        press_until(
            page,
            keyboard::ARROW_RIGHT,
            &on("Open recent"),
            "ArrowRight to close it",
        )
        .await;
        fixture.console.assert_clean("RTL submenu").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `Tree`: ArrowLeft opens a row, ArrowRight closes it.
#[test]
fn a_tree_opens_on_arrow_left() {
    block_on(async {
        let fixture = open_rtl("/tree").await;
        let page = &fixture.page;
        keyboard::tab_to(page, "[data-tree-id='src']", 10)
            .await
            .unwrap();
        let expanded = |state: &str| {
            format!(
                "document.querySelector(\"[data-tree-id='src']\")\
                 .getAttribute('aria-expanded') === '{state}'"
            )
        };
        press_until(
            page,
            keyboard::ARROW_LEFT,
            &expanded("true"),
            "ArrowLeft to open src",
        )
        .await;
        press_until(
            page,
            keyboard::ARROW_RIGHT,
            &expanded("false"),
            "ArrowRight to close src",
        )
        .await;
        fixture.console.assert_clean("RTL tree").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The day grid: ArrowLeft is tomorrow.
#[test]
fn a_calendar_steps_to_tomorrow_on_arrow_left() {
    block_on(async {
        let fixture = open_rtl("/calendar").await;
        let page = &fixture.page;
        keyboard::tab_to(
            page,
            "[role=grid] [data-date='2026-03-18']:not([data-outside])",
            10,
        )
        .await
        .unwrap();
        let on = |date: &str| format!("document.activeElement?.dataset.date === '{date}'");
        press_until(
            page,
            keyboard::ARROW_LEFT,
            &on("2026-03-19"),
            "ArrowLeft to the 19th",
        )
        .await;
        press_until(
            page,
            keyboard::ARROW_RIGHT,
            &on("2026-03-18"),
            "ArrowRight to the 18th",
        )
        .await;
        fixture.console.assert_clean("RTL calendar").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Chip cursors (`TagsField`, `Select`, `FileField`): ArrowRight leaves the
/// input for the last chip, which sits to its right.
#[test]
fn a_chip_cursor_walks_back_on_arrow_right() {
    block_on(async {
        let fixture = open_rtl("/tags-field/cursor").await;
        let page = &fixture.page;
        keyboard::tab_to(page, "#cursor", 10).await.unwrap();
        let on = |label: &str| {
            format!("document.activeElement?.getAttribute('aria-label') === 'Remove {label}'")
        };
        press_until(
            page,
            keyboard::ARROW_RIGHT,
            &on("css"),
            "ArrowRight to the last chip",
        )
        .await;
        press_until(
            page,
            keyboard::ARROW_RIGHT,
            &on("wasm"),
            "ArrowRight to the one before",
        )
        .await;
        press_until(
            page,
            keyboard::ARROW_LEFT,
            &on("css"),
            "ArrowLeft forward again",
        )
        .await;
        fixture.console.assert_clean("RTL chips").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `Carousel`'s dots (and `Lightbox`'s strip): ArrowLeft is the next slide.
#[test]
fn carousel_dots_step_forward_on_arrow_left() {
    block_on(async {
        let fixture = open_rtl("/carousel").await;
        let page = &fixture.page;
        let dots = "[aria-roledescription=carousel] button[id*='-indicator-']";
        keyboard::tab_to(page, dots, 20).await.unwrap();
        let on = |n: usize| {
            format!(
                "[...document.querySelectorAll(\"{dots}\")].indexOf(document.activeElement) === {n}"
            )
        };
        press_until(
            page,
            keyboard::ARROW_LEFT,
            &on(1),
            "ArrowLeft to the second dot",
        )
        .await;
        press_until(
            page,
            keyboard::ARROW_RIGHT,
            &on(0),
            "ArrowRight back to the first",
        )
        .await;
        fixture.console.assert_clean("RTL carousel dots").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Which way a glyph points on screen: the x of its `(vx, vy)` after every transform up to
/// the root (the `ChevronRight` glyph draws `(1, 0)`, `ChevronDown` `(0, 1)`).
async fn screen_x(page: &Page, selector: &str, vx: f64, vy: f64) -> Option<f64> {
    page.evaluate(format!(
        "(() => {{ let el = document.querySelector({selector:?}); \
         if (!el) return null; \
         let m = new DOMMatrix(); \
         for (; el; el = el.parentElement) {{ \
           const t = getComputedStyle(el).transform; \
           if (t && t !== 'none') m = new DOMMatrix(t).multiply(m); }} \
         return m.a * {vx} + m.c * {vy}; }})()"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

/// A chevron that means "previous", "next" or "has children" points the way
/// the layout goes, so under RTL it mirrors (the Scroller and Menu rule).
#[test]
fn directional_chevrons_mirror() {
    block_on(async {
        let mut wrong = Vec::new();
        // Route, glyph, the vector it draws, and +1 for forward, -1 for back.
        let cases: [(&str, &str, (f64, f64), f64); 7] = [
            (
                "/carousel",
                "[aria-label='Next slide'] svg",
                (1.0, 0.0),
                1.0,
            ),
            (
                "/carousel",
                "[aria-label='Previous slide'] svg",
                (-1.0, 0.0),
                -1.0,
            ),
            (
                "/pagination",
                "[aria-label='Go to previous page'] svg",
                (-1.0, 0.0),
                -1.0,
            ),
            (
                "/pagination",
                "[aria-label='Go to next page'] svg",
                (1.0, 0.0),
                1.0,
            ),
            ("/calendar", "[aria-label^='Next'] svg", (1.0, 0.0), 1.0),
            (
                "/tree/default",
                "[data-tree-id='src'] [data-tree-chevron] svg",
                (1.0, 0.0),
                1.0,
            ),
            ("/cascader", "[data-slot='branch'] svg", (0.0, 1.0), 1.0),
        ];
        for (route, glyph, (vx, vy), forward) in cases {
            let fixture = open_rtl(route).await;
            let page = &fixture.page;
            if route == "/cascader" {
                keyboard::tab_to(page, "[role=combobox]", 10).await.unwrap();
                keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
                wait::for_visible(page, glyph).await.unwrap();
            }
            // The tree chevron turns with a transition when `dir` flips.
            wait::for_js_true(
                page,
                "document.getAnimations().every(a => a.playState !== 'running')",
                "transitions to end",
            )
            .await
            .unwrap();
            // Under RTL forward is towards -x.
            let x = screen_x(page, glyph, vx, vy).await;
            if x.is_none_or(|x| x * forward > -0.5) {
                wrong.push(format!("{route} {glyph}: x {x:?}"));
            }
            fixture.close().await.unwrap();
        }
        assert!(
            wrong.is_empty(),
            "chevrons not mirrored under RTL: {wrong:#?}"
        );
    });
}
