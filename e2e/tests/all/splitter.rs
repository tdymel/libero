//! `Splitter`: the divider keeps the keyboard after a mouse drag (431): `use_drag` cancels
//! the pointerdown's focus, so it focuses the divider itself (439c).

use anyhow::{Context, Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text};
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const DIVIDER: &str = "[role=separator]";
const VALUE_NOW: &str = "document.querySelector('[role=separator]').getAttribute('aria-valuenow')";
/// `Change` events so far on `/splitter`.
const CHANGES: &str = "document.getElementById('box').dataset.changes";

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
    // Todo 2406: a floor reached by `Home` restores the size before it too.
    d.press_shift(keyboard::ARROW_RIGHT).await?;
    moved(d, "Shift+ArrowRight to move it to 60", |v| v == 60.0).await?;
    d.press(keyboard::HOME).await?;
    moved(d, "Home to reach the floor", |v| v == 10.0).await?;
    d.double_click(DIVIDER).await?;
    moved(d, "a double-click to restore the size before Home", |v| {
        v == 60.0
    })
    .await?;
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
    rtl_rightwards_shrinks,
    android: skip("963: is_rtl reads false on the WebView"),
    desktop: skip("963: is_rtl reads false on the WebView")
);
e2e::scenario!(
    the_arrows_and_home_end_move_a_focused_divider,
    "/splitter",
    the_keys_move_it
);

const ROW: &str = "#row [role=separator]";
const COLUMN: &str = "#column [role=separator]";

/// Todo 1039, as the Slider's 1020: on Android a vertical swipe over a vertical
/// divider scrolls the page; a mouse still grabs at once. A sideways drag moves
/// it, a tap starts before it ends, and a horizontal divider still drags.
async fn a_swipe_scrolls<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (_, height) = d.viewport().await?;
    d.drag(ROW, 0.0, -250.0).await?;
    if d.platform() == Platform::Android {
        let at = d.rect(ROW).await?;
        ensure!(
            at.y + at.height / 2.0 < height / 2.0 - 100.0,
            "a swipe up over the divider did not scroll: its top at {}",
            at.y
        );
    } else {
        eventually_text(d, "#row-end", "50", "a vertical mouse drag").await?;
    }

    // Blitz's `drag` waits out a double press of the last one itself.
    d.drag(ROW, 40.0, 0.0).await?;
    // `aria-valuenow` (the first divider's) truncates, `#row-end` rounds.
    eventually(d, "a sideways drag to move it and end", async |d| {
        let value = value(d).await?;
        let ended: f64 = d.text("#row-end").await?.parse().unwrap_or_default();
        Ok(value > 55.0 && (ended - value).abs() <= 1.0)
    })
    .await?;

    let log = d.text("#row-log").await?;
    // The sideways drag's pair is the only one: the swipe before it grabbed nothing.
    if d.platform() == Platform::Android {
        ensure!(
            log == "start end ",
            "a swipe over the divider grabbed it: {log:?}"
        );
    }
    d.click(ROW).await?;
    eventually_text(d, "#row-log", &format!("{log}start end "), "a tap").await?;

    d.drag(COLUMN, 0.0, 40.0).await?;
    eventually(d, "a vertical drag on the horizontal divider", async |d| {
        Ok(d.text("#column-end")
            .await?
            .parse::<f64>()
            .is_ok_and(|v| v > 55.0))
    })
    .await
}

e2e::scenario!(
    a_vertical_swipe_over_a_vertical_divider_scrolls_the_page_and_a_sideways_drag_moves_it,
    "/splitter/scroll",
    a_swipe_scrolls
);

/// Todo 1821: the horizontal divider takes ArrowDown/ArrowUp (step 1), Shift for the big
/// step (10), Home/End; ArrowLeft/Right are not its keys. Pane A grows downwards.
async fn the_column_keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    async fn column<D: Driver>(d: &mut D, what: &str, want: f64) -> Result<()> {
        eventually(d, what, async |d| {
            let now = d
                .attr(COLUMN, "aria-valuenow")
                .await?
                .context("no aria-valuenow")?;
            Ok(now.parse::<f64>()? == want)
        })
        .await
    }

    let orientation = d.attr(COLUMN, "aria-orientation").await?;
    ensure!(
        orientation.as_deref() == Some("horizontal"),
        "the column divider's aria-orientation is {orientation:?}"
    );
    d.focus(COLUMN).await?;
    column(d, "the divider to start at 50", 50.0).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    column(d, "ArrowDown to grow pane A by a step", 51.0).await?;
    d.press(keyboard::ARROW_UP).await?;
    column(d, "ArrowUp to shrink it back", 50.0).await?;
    d.press_shift(keyboard::ARROW_DOWN).await?;
    column(d, "Shift+ArrowDown to grow it by the big step", 60.0).await?;
    d.press_shift(keyboard::ARROW_UP).await?;
    column(d, "Shift+ArrowUp to shrink it by the big step", 50.0).await?;
    // Inert keys first: the step after lands on 49 only if they moved nothing.
    d.press(keyboard::ARROW_LEFT).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    d.press(keyboard::ARROW_UP).await?;
    column(d, "ArrowLeft/Right to move nothing", 49.0).await?;
    d.press(keyboard::HOME).await?;
    column(d, "Home to reach the floor", 10.0).await?;
    d.press(keyboard::END).await?;
    column(d, "End to reach the ceiling", 90.0).await
}

e2e::scenario!(
    the_horizontal_divider_takes_up_down_shift_and_home_end,
    "/splitter/scroll",
    the_column_keys
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("splitter", "/splitter").focusable(DIVIDER).run();
}

/// RTL, the minimum-size page and the two scrolling splitters (todo 1773).
#[test]
fn the_other_splitters_meet_the_baseline() {
    for (name, route) in [
        ("splitter-rtl", "/splitter/rtl"),
        ("splitter-min-size", "/splitter/min-size"),
        ("splitter-scroll", "/splitter/scroll"),
    ] {
        Suite::new(name, route).focusable(DIVIDER).run();
    }
}

/// The drag scenario at the phone width; its web arm runs at the desktop one (todo 1826).
#[test]
fn a_drag_leaves_the_divider_focused_on_mobile() {
    block_on(async {
        let fixture = Fixture::open("/splitter", Viewport::Mobile).await.unwrap();
        let mut d = e2e::driver::Web { fixture };
        a_drag_leaves_it_focused(&mut d, "/splitter").await.unwrap();
        d.finish("a splitter drag on mobile").await.unwrap();
    });
}

/// Under RTL pane A is on the right, and ArrowLeft grows it: the arrow keys move the
/// divider the way they point.
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

        // Rightwards shrinking is the scenario's; here the other way grows it (todo 1826).
        keyboard::tab_to(page, DIVIDER, 5).await.unwrap();
        let start = value_now(page).await;
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("Number({VALUE_NOW}) > {start}"),
            "ArrowLeft to grow pane A",
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

/// The drag-free path (WCAG 2.5.7) around the scenario's collapse and restore: a single
/// click only focuses, and a double-click in a pane is not the divider's (todo 1826).
#[test]
fn a_click_only_focuses_and_a_double_click_in_a_pane_does_nothing() {
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
        // The next step's `Change` is the first: the click before it moved nothing.
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} && {CHANGES} === '1'", value_is("51")),
            "ArrowRight after a single click to be the first move",
        )
        .await
        .unwrap();

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
        page.evaluate(format!("document.querySelector({DIVIDER:?}).focus()"))
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} && {CHANGES} === '2'", value_is("50")),
            "ArrowLeft after a double-click in pane A to be the second move",
        )
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

/// Todo 2404: the line is a fill, which forced colours turn `Canvas` unless it is a system colour.
#[test]
fn the_divider_line_shows_in_forced_colours() {
    block_on(async {
        let fixture = Fixture::open("/splitter/content", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        e2e::browser::force_colours(page).await.unwrap();
        let (line, canvas): (String, String) = page
            .evaluate(format!(
                "(() => {{ const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 const bar = document.querySelector({DIVIDER:?}).parentElement; \
                 return [getComputedStyle(bar).backgroundColor, canvas]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(line, canvas, "the divider line takes the page's Canvas");
        fixture.close().await.unwrap();
    });
}

/// Todo 1581: an overflowing pane scrolls inside itself instead of painting over its neighbours.
#[test]
fn an_overflowing_pane_scrolls_inside_itself() {
    block_on(async {
        let fixture = Fixture::open("/splitter/content", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#pane-a").await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const pane = document.getElementById('pane-a').parentElement; \
             pane.scrollTop = 100; \
             return pane.scrollHeight > pane.clientHeight && pane.scrollTop > 0; })()",
            "pane A to scroll its own overflow",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("an overflowing pane").unwrap();
        fixture.close().await.unwrap();
    });
}
