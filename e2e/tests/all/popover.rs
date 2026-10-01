//! `use_popover` with `PopoverOptions::dismiss` (todo 522): Escape anywhere
//! and a press outside close the box; Escape hands focus to the trigger.

use anyhow::{Result, bail};
use e2e::archetypes;
use e2e::browser::block_on;
use e2e::driver::{Driver, Rect, eventually, eventually_focused};
use e2e::passes::{focus, keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const TRIGGER: &str = "#trigger";
const BOX: &str = "#box";
const BOX_TEXT: &str = "#box-text";
const IN_BOX: &str = "#in-box";
const BLANK: &str = "#blank";
const AFTER: &str = "#after";

// Placement, on `/popover/place/*`: a 220px `#anchor`, a 150x60 `#floating`, 8px gap.
const ANCHOR: &str = "#anchor";
const FLOATING: &str = "#floating";
const GAP: f64 = 8.0;

fn close_to(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0
}

/// Opens the box, then waits until `holds(anchor, box)`.
async fn placed<D: Driver>(
    d: &mut D,
    what: &str,
    holds: impl Fn(Rect, Rect, (f64, f64)) -> bool,
) -> Result<()> {
    d.click(ANCHOR).await?;
    let viewport = d.viewport().await?;
    let settled = eventually(d, what, async |d| {
        if !d.exists(FLOATING).await? {
            return Ok(false);
        }
        Ok(holds(
            d.rect(ANCHOR).await?,
            d.rect(FLOATING).await?,
            viewport,
        ))
    })
    .await;
    if settled.is_err() && d.exists(FLOATING).await? {
        let (a, f) = (d.rect(ANCHOR).await?, d.rect(FLOATING).await?);
        bail!("{:?}: {what}: anchor {a:?}, box {f:?}", d.platform());
    }
    settled
}

async fn lands_below<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    placed(d, "the box below the anchor at its start", |a, f, _| {
        close_to(f.x, a.x) && close_to(f.y, a.y + a.height + GAP)
    })
    .await
}

async fn flips_above<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    placed(
        d,
        "the box above an anchor at the viewport bottom",
        |a, f, _| close_to(f.y + f.height + GAP, a.y),
    )
    .await
}

async fn shifts_inside<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    placed(d, "the box inside the viewport", |_, f, (vw, _)| {
        f.x >= 0.0 && f.x + f.width <= vw
    })
    .await
}

async fn matches_width<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    placed(d, "the box as wide as the anchor", |a, f, _| {
        close_to(f.width, a.width)
    })
    .await
}

/// Todo 1002: a page scroll re-places the open box, through the WebView's
/// scroll script on Android.
async fn follows_a_scroll<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    lands_below(d, route).await?;
    let before = d.rect(ANCHOR).await?;
    d.click("#scroll-by").await?;
    eventually(d, "the page to scroll the anchor up", async |d| {
        Ok(close_to(d.rect(ANCHOR).await?.y, before.y - 120.0))
    })
    .await?;
    eventually(d, "the box to follow its anchor", async |d| {
        let (a, f) = (d.rect(ANCHOR).await?, d.rect(FLOATING).await?);
        Ok(close_to(f.x, a.x) && close_to(f.y, a.y + a.height + GAP))
    })
    .await
}

e2e::scenario!(
    it_follows_its_anchor_when_the_page_scrolls,
    "/popover/place/scroll",
    follows_a_scroll,
    native: skip("1002: Blitz runs no document eval to scroll the page")
);
e2e::scenario!(
    it_lands_below_the_anchor_at_its_start,
    "/popover/place/below",
    lands_below
);
e2e::scenario!(
    it_flips_above_an_anchor_at_the_viewport_bottom,
    "/popover/place/flip",
    flips_above
);
e2e::scenario!(
    it_shifts_back_inside_the_viewport,
    "/popover/place/shift",
    shifts_inside
);
e2e::scenario!(
    a_matched_width_is_the_anchors,
    "/popover/place/match",
    matches_width
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("popover", "/popover")
        .focusable(TRIGGER)
        .state("open", &[Step::Click(TRIGGER)], BOX)
        .run();
}

async fn open(page: &chromiumoxide::Page) {
    pointer::click(page, TRIGGER).await.unwrap();
    wait::for_visible(page, BOX).await.unwrap();
}

/// A press on nothing focusable, or on another control, closes the box; a
/// press on the box's own text does not.
#[test]
fn a_press_outside_closes_it() {
    block_on(async {
        let fixture = Fixture::open("/popover", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        open(page).await;
        pointer::click(page, BLANK).await.unwrap();
        wait::for_hidden(page, BOX).await.unwrap();

        open(page).await;
        pointer::click(page, BOX_TEXT).await.unwrap();
        pointer::click(page, IN_BOX).await.unwrap();
        crate::settle::painted(page).await.unwrap();
        assert!(
            wait::is_visible(page, BOX).await.unwrap(),
            "a press inside the box closed it"
        );
        pointer::click(page, AFTER).await.unwrap();
        wait::for_hidden(page, BOX).await.unwrap();
        focus::assert_focused(page, AFTER, "a press on another control")
            .await
            .unwrap();

        // The trigger's own click still toggles it shut.
        open(page).await;
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_hidden(page, BOX).await.unwrap();

        fixture.console.assert_clean("pressing outside").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Open means placed too: an unplaced box sits hidden at 0,0, and a tap
/// aimed at it lands outside.
async fn shown<D: Driver>(d: &mut D, open: bool, what: &str) -> Result<()> {
    eventually(d, what, async |d| {
        // A box removed between the two reads fails the second.
        let placed = d.exists(BOX).await?
            && d.style(BOX, "visibility").await.unwrap_or_default() == "visible";
        Ok(placed == open)
    })
    .await
}

/// Todo 1019: the same by touch, where the WebView cannot tell where focus went.
async fn a_tap_outside_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    shown(d, true, "the box to open").await?;
    d.click(BLANK).await?;
    shown(d, false, "a tap on nothing to close it").await?;

    d.click(TRIGGER).await?;
    shown(d, true, "the box to reopen").await?;
    d.click(BOX_TEXT).await?;
    d.settle().await?;
    if !d.exists(BOX).await? {
        bail!("{:?}: a tap on the box's text closed it", d.platform());
    }
    d.click(TRIGGER).await?;
    shown(d, false, "the trigger to toggle it shut").await
}

e2e::scenario!(
    a_tap_outside_closes_it_and_one_inside_does_not,
    "/popover",
    a_tap_outside_closes
);

/// Todo 1425: a field's portaled dropdown, opened from inside the box, counts as
/// inside it: Arrow Down into it keeps the box open, Escape closes one at a time.
async fn a_field_dropdown_stays_inside<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const FIELD: &str = "#box-field";
    const DROPDOWN: &str = "[role=dialog]:not(#box)";
    // The calendar's one tab stop, as Blitz sets no `:focus-within` on a silent move.
    const ENTERED: &str = "[role=dialog]:not(#box) [data-slot=day][tabindex=\"0\"]";
    d.click(TRIGGER).await?;
    shown(d, true, "the box to open").await?;
    d.click(FIELD).await?;
    eventually(d, "the field's dropdown", async |d| {
        d.exists(DROPDOWN).await
    })
    .await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_focused(d, ENTERED, "Arrow Down into the dropdown").await?;
    // A close on that focus would land a task later, long before Escape's round trip.
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "Escape to close only the dropdown", async |d| {
        Ok(!d.exists(DROPDOWN).await? && d.focused_id().await? == "box-field")
    })
    .await?;
    if !d.exists(BOX).await? {
        bail!(
            "{:?}: focus in the field's dropdown or its Escape closed the box",
            d.platform()
        );
    }
    d.press(keyboard::ESCAPE).await?;
    shown(d, false, "a second Escape to close the box").await
}

e2e::scenario!(
    a_field_dropdown_opened_inside_counts_as_inside,
    "/popover/field",
    a_field_dropdown_stays_inside,
    android: skip("1425: no synchronous containment on the WebView"),
    desktop: skip("1425: no synchronous containment on the WebView")
);

/// Android's Back closes the box, and the app stays (1289).
async fn back_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    archetypes::back_closes(d, TRIGGER, async |d| {
        Ok(d.exists(BOX).await?
            && d.style(BOX, "visibility").await.unwrap_or_default() == "visible")
    })
    .await
}

e2e::scenario!(
    android_back_closes_a_popover,
    "/popover",
    back_closes,
    android_only("1275: no Back key off Android")
);

/// Escape closes it with focus on the trigger or inside the box, and focus
/// ends on the trigger.
#[test]
fn escape_closes_it_and_returns_focus_to_the_trigger() {
    block_on(async {
        let fixture = Fixture::open("/popover", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        open(page).await;
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, BOX).await.unwrap();
        focus::wait_for_focus(page, TRIGGER, "Escape on the trigger")
            .await
            .unwrap();

        open(page).await;
        pointer::click(page, IN_BOX).await.unwrap();
        focus::wait_for_focus(page, IN_BOX, "a click on the box's control")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, BOX).await.unwrap();
        focus::wait_for_focus(page, TRIGGER, "Escape inside the box")
            .await
            .unwrap();

        fixture.console.assert_clean("Escape").unwrap();
        fixture.close().await.unwrap();
    });
}
