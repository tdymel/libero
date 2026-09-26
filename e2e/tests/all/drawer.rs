//! `Drawer`: the overlay archetype, docked to an edge. It escaped todo 327 by geometry
//! alone; `contrast_covers` makes that an assertion.

use anyhow::Result;
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually};

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
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_backdrop_click_closes_a_drawer,
    "/drawer",
    backdrop_closes,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_button_in_a_drawer_closes_it_and_focus_returns,
    "/drawer",
    close_button_closes,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(a_right_drawer_hugs_the_right_edge, "/drawer", hugs_the_end);

/// Android's Back closes the drawer, and the app stays (1289).
async fn back_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.platform() != Platform::Android {
        return Ok(());
    }
    d.click(TRIGGER).await?;
    eventually(d, "the drawer to open", async |d| d.exists(DIALOG).await).await?;
    d.press_back().await?;
    eventually(d, "Back to close the drawer", async |d| {
        Ok(!d.exists(DIALOG).await?)
    })
    .await?;
    d.click(TRIGGER).await?;
    eventually(d, "the app to stay and open it again", async |d| {
        d.exists(DIALOG).await
    })
    .await
}

e2e::scenario!(
    android_back_closes_a_drawer,
    "/drawer",
    back_closes,
    native: skip("1275: no Back key off Android"),
    desktop: skip("1275: no Back key off Android")
);

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
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
        }
    });
}
