//! `Splitter`: the divider keeps the keyboard after a mouse drag (todo 431).
//!
//! `use_drag` cancels the pointerdown, and with it the browser's own focus, so
//! the hook focuses the pressed divider itself (todo 439c).

use anyhow::{Context, Result};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const DIVIDER: &str = "[role=separator]";
const VALUE_NOW: &str = "document.querySelector('[role=separator]').getAttribute('aria-valuenow')";

async fn value<D: Driver>(d: &mut D) -> Result<f64> {
    let now = d
        .attr(DIVIDER, "aria-valuenow")
        .await?
        .context("no aria-valuenow")?;
    Ok(now.parse()?)
}

/// Waits until the divider's value passes `holds`, naming `what`.
async fn moved<D: Driver>(d: &mut D, what: &str, holds: impl Fn(f64) -> bool) -> Result<f64> {
    eventually(d, what, async |d| Ok(holds(value(d).await?))).await?;
    value(d).await
}

async fn a_drag_leaves_it_focused<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#before").await?;
    let start = value(d).await?;
    d.drag(DIVIDER, 40.0, 0.0).await?;
    let dragged = moved(d, "the drag to move it right", |v| v > start + 5.0).await?;
    eventually_focused(d, DIVIDER, "a drag").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    moved(d, "ArrowRight to move it", |v| v > dragged).await?;
    Ok(())
}

async fn a_double_click_collapses<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.double_click(DIVIDER).await?;
    moved(d, "a double-click to collapse pane A", |v| v == 10.0).await?;
    d.double_click(DIVIDER).await?;
    moved(d, "a second double-click to restore it", |v| v == 50.0).await?;
    Ok(())
}

async fn rtl_rightwards_shrinks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(DIVIDER).await?;
    let start = value(d).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    let pressed = moved(d, "ArrowRight to shrink pane A", |v| v < start).await?;
    d.drag(DIVIDER, 40.0, 0.0).await?;
    moved(d, "a rightward drag to shrink pane A", |v| {
        v < pressed - 5.0
    })
    .await?;
    Ok(())
}

async fn the_keys_move_it<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(DIVIDER).await?;
    let start = value(d).await?;
    d.press(keyboard::ARROW_LEFT).await?;
    moved(d, "ArrowLeft to move it", |v| v < start).await?;
    d.press(keyboard::HOME).await?;
    let min = moved(d, "Home to reach the floor", |v| v <= 10.0).await?;
    d.press(keyboard::END).await?;
    moved(d, "End to move it past Home", |v| v > min).await?;
    Ok(())
}

e2e::scenario!(
    a_drag_moves_the_divider_and_leaves_it_focused,
    "/splitter",
    a_drag_leaves_it_focused
);
e2e::scenario!(
    a_double_click_collapses_pane_a_and_restores_it,
    "/splitter",
    a_double_click_collapses
);
e2e::scenario!(
    under_rtl_rightwards_shrinks_pane_a,
    "/splitter/rtl",
    rtl_rightwards_shrinks
);
e2e::scenario!(
    the_arrows_and_home_end_move_a_focused_divider,
    "/splitter",
    the_keys_move_it
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("splitter", "/splitter").focusable(DIVIDER).run();
}

/// Focus starts on a button outside, so a divider that never takes focus
/// fails the first assertion rather than passing on focus it already had.
#[test]
fn a_drag_leaves_the_divider_focused_for_the_arrow_keys() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/splitter", viewport).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, "#before", 5).await.unwrap();

            let from = pointer::centre_of(page, DIVIDER).await.unwrap();
            let to = pointer::Point {
                x: from.x + 40.0,
                y: from.y,
            };
            wait::for_js_change(page, VALUE_NOW, "the drag to move the divider", || {
                pointer::drag(page, from, to, 10)
            })
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            wait::for_js_true(
                page,
                &format!("document.activeElement === document.querySelector({DIVIDER:?})"),
                "the divider to hold focus after the drag",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            wait::for_js_change(page, VALUE_NOW, "ArrowRight to move the divider", || {
                keyboard::press(page, keyboard::ARROW_RIGHT)
            })
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            fixture
                .console
                .assert_clean(&format!("a splitter drag at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Under RTL pane A is on the right: the arrow keys and a drag move the
/// divider the way they point, so rightwards shrinks pane A.
#[test]
fn under_rtl_the_divider_moves_the_way_the_arrow_and_pointer_go() {
    block_on(async {
        let fixture = Fixture::open("/splitter/rtl", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let pane_a_right: bool = page
            .evaluate(
                "(() => { const s = document.querySelector('[role=separator]'); \
                 const a = document.getElementById(s.getAttribute('aria-controls')); \
                 return a.getBoundingClientRect().left > s.getBoundingClientRect().left; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(pane_a_right, "pane A is not on the right under RTL");

        keyboard::tab_to(page, DIVIDER, 5).await.unwrap();
        let start = value_now(page).await;
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("Number({VALUE_NOW}) < {start}"),
            "ArrowRight to shrink pane A",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("Number({VALUE_NOW}) > {start}"),
            "ArrowLeft to grow pane A",
        )
        .await
        .unwrap();

        let before = value_now(page).await;
        let from = pointer::centre_of(page, DIVIDER).await.unwrap();
        let to = pointer::Point {
            x: from.x + 40.0,
            y: from.y,
        };
        pointer::drag(page, from, to, 10).await.unwrap();
        wait::for_js_true(
            page,
            &format!("Number({VALUE_NOW}) < {before}"),
            "a drag to the right to shrink pane A",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("an RTL splitter").unwrap();
        fixture.close().await.unwrap();
    });
}

async fn value_now(page: &chromiumoxide::Page) -> f64 {
    let now: String = page
        .evaluate(VALUE_NOW)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    now.parse().unwrap()
}

/// Alt+ArrowLeft is Back: the divider must not swallow a browser chord.
#[test]
fn modifier_chords_are_left_to_the_browser() {
    block_on(async {
        let fixture = Fixture::open("/splitter", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, DIVIDER, 5).await.unwrap();

        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_LEFT,
                keyboard::ARROW_RIGHT,
                keyboard::HOME,
                keyboard::END,
            ],
            VALUE_NOW,
        )
        .await
        .unwrap();

        fixture.close().await.unwrap();
    });
}

/// The drag-free path (WCAG 2.5.7): a single click only focuses, a double-click
/// collapses pane A to the floor and the next one restores it.
#[test]
fn a_double_click_toggles_pane_a_collapsed() {
    block_on(async {
        let fixture = Fixture::open("/splitter", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let value_is = |value: &'static str| format!("{VALUE_NOW} === '{value}'");
        wait::for_js_true(page, &value_is("50"), "the divider to start at 50")
            .await
            .unwrap();

        pointer::click(page, DIVIDER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === document.querySelector({DIVIDER:?})"),
            "a click to focus the divider",
        )
        .await
        .unwrap();
        let now: String = page
            .evaluate(VALUE_NOW)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(now, "50", "a single click moved the divider");

        // Pointer capture sends every `dblclick` to the root: one in a pane is not the divider's.
        let pane_a: String = page
            .evaluate("document.querySelector('[role=separator]').getAttribute('aria-controls')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        pointer::double_click(page, &format!("[id={pane_a:?}]"))
            .await
            .unwrap();
        let now: String = page
            .evaluate(VALUE_NOW)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(now, "50", "a double-click in pane A moved the divider");

        pointer::double_click(page, DIVIDER).await.unwrap();
        wait::for_js_true(page, &value_is("10"), "a double-click to collapse pane A")
            .await
            .unwrap();
        pointer::double_click(page, DIVIDER).await.unwrap();
        wait::for_js_true(page, &value_is("50"), "a double-click to restore pane A")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("a divider double-click")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// WCAG 2.5.8 on its own terms: the default hit area is 24px across the line.
#[test]
fn the_default_hit_area_is_24px_thick() {
    block_on(async {
        let fixture = Fixture::open("/splitter", Viewport::Desktop).await.unwrap();
        let width: f64 = fixture
            .page
            .evaluate(format!(
                "document.querySelector({DIVIDER:?}).getBoundingClientRect().width"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(width, 24.0, "the divider's hit area is {width}px thick");
        fixture.close().await.unwrap();
    });
}

/// A floor raised after the divider moved pulls pane A up to it, and the
/// separator never reports a value outside its own bounds.
#[test]
fn a_raised_min_size_clamps_the_moved_divider() {
    block_on(async {
        let fixture = Fixture::open("/splitter/min-size", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, DIVIDER, 5).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{VALUE_NOW} === '50'"),
            "the divider to start at 50",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::HOME).await.unwrap();
        wait::for_js_true(page, &format!("{VALUE_NOW} === '10'"), "Home to reach 10")
            .await
            .unwrap();

        pointer::click(page, "#raise").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[role=separator]').getAttribute('aria-valuemin') === '40'",
            "the floor to rise to 40",
        )
        .await
        .unwrap();

        let now: String = page
            .evaluate(VALUE_NOW)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(now, "40", "aria-valuenow below the raised aria-valuemin");
        let pane_share: f64 = page
            .evaluate(
                "(() => { const s = document.querySelector('[role=separator]'); \
                 const a = document.getElementById(s.getAttribute('aria-controls')); \
                 return a.getBoundingClientRect().width / a.parentElement.getBoundingClientRect().width * 100; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            (pane_share - 40.0).abs() < 1.0,
            "pane A is drawn at {pane_share:.1}%, under the 40% floor"
        );

        fixture.console.assert_clean("a raised min_size").unwrap();
        fixture.close().await.unwrap();
    });
}
