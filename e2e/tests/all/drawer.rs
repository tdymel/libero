//! `Drawer`: the overlay archetype, docked to an edge. It escaped todo 327 by geometry
//! alone; `contrast_covers` makes that an assertion.

use anyhow::Result;
use e2e::archetypes::{self, Overlay};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};

use crate::modal;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

pub const TRIGGER: &str = "#open-drawer";
const DIALOG: &str = "[role=dialog]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("drawer", "/drawer")
        .focusable(TRIGGER)
        .contrast_covers(DIALOG)
        .targets(TRIGGER)
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ENTER)],
            DIALOG,
        )
        .run();
}

async fn traps_tab<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    modal::tab_stays_inside(d, TRIGGER).await
}

async fn backdrop_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    modal::a_backdrop_click_closes(d, TRIGGER).await
}

async fn close_button_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    modal::open(d, TRIGGER).await?;
    d.click("#drawer-close").await?;
    modal::closed_with_focus_on(d, TRIGGER, "the drawer's Close").await
}

async fn hugs_the_end<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    modal::open(d, TRIGGER).await?;
    let (vw, vh) = d.viewport().await?;
    eventually(d, "the drawer to dock at the right edge", async |d| {
        let r = d.rect(DIALOG).await?;
        Ok((r.x + r.width - vw).abs() <= 1.0 && r.y.abs() <= 1.0 && (r.height - vh).abs() <= 1.0)
    })
    .await
}

e2e::scenario!(
    a_drawer_moves_focus_in_and_traps_tab,
    "/drawer",
    traps_tab,
    android: skip("959: Tab leaves the drawer for the body on the WebView"),
    desktop: skip("959: Shift+Tab from the first control leaves the drawer on the WebView")
);
e2e::scenario!(a_backdrop_click_closes_a_drawer, "/drawer", backdrop_closes);
e2e::scenario!(
    a_button_in_a_drawer_closes_it_and_focus_returns,
    "/drawer",
    close_button_closes
);
e2e::scenario!(a_right_drawer_hugs_the_right_edge, "/drawer", hugs_the_end);

/// Opening moves focus in; the first Escape closes it and hands focus to the trigger.
async fn escape_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    modal::open(d, TRIGGER).await?;
    eventually_focused(d, "[role=dialog], [role=dialog] *", "opening the drawer").await?;
    d.press(keyboard::ESCAPE).await?;
    modal::closed_with_focus_on(d, TRIGGER, "Escape").await
}

e2e::scenario!(
    escape_closes_a_drawer_and_returns_focus,
    "/drawer",
    escape_closes
);

/// Android's Back closes the drawer, and the app stays (1289).
async fn back_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    archetypes::back_closes(d, TRIGGER, async |d| d.exists(DIALOG).await).await
}

e2e::scenario!(
    android_back_closes_a_drawer,
    "/drawer",
    back_closes,
    android_only("1275: no Back key off Android")
);

/// An end drawer docks to the left under `dir="rtl"`.
#[test]
fn an_end_drawer_docks_left_in_rtl() {
    block_on(async {
        let fixture = Fixture::open("/drawer", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.evaluate("document.documentElement.dir = 'rtl'")
            .await
            .unwrap();
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        e2e::wait::for_visible(page, DIALOG).await.unwrap();
        let x: f64 = page
            .evaluate("document.querySelector('[role=dialog]').getBoundingClientRect().x")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(x.abs() <= 1.0, "the end drawer is at x={x} under rtl");
        fixture.close().await.unwrap();
    });
}

/// 1.4.10: a drawer taller than the viewport scrolls; its last control is reachable.
#[test]
fn a_drawer_taller_than_the_viewport_scrolls() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/drawer/tall", viewport).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            e2e::wait::for_visible(page, DIALOG).await.unwrap();
            let reachable: bool = page
                .evaluate(
                    "(() => { const a = document.querySelector('#drawer-close'); a.focus(); \
                     const b = a.getBoundingClientRect(); return b.top >= 0 && b.bottom <= innerHeight; })()",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                reachable,
                "the last control is off screen at {}",
                viewport.name()
            );
            fixture.close().await.unwrap();
        }
    });
}

/// 1.4.10: the widest size is capped by the viewport, not pushed past its edge.
#[test]
fn a_wide_drawer_stays_inside_a_narrow_viewport() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/drawer/wide", viewport).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            e2e::wait::for_visible(page, DIALOG).await.unwrap();
            let (right, width): (f64, f64) = page
                .evaluate(
                    "(() => { const b = document.querySelector('[role=dialog]').getBoundingClientRect(); \
                     return [b.right, innerWidth]; })()",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                right <= width + 1.0,
                "the drawer ends at {right}px in a {width}px viewport at {}",
                viewport.name()
            );
            fixture.close().await.unwrap();
        }
    });
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/drawer", viewport).await.unwrap();

            Overlay {
                trigger: TRIGGER,
                panel: DIALOG,
                traps_focus: true,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the drawer contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}
