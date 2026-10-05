//! `Table` column resize (1156-4d): a header grip drags the width within its
//! limits; the column menu's Widen, Narrow and Reset width do it without a drag,
//! and say the new width.

use anyhow::{Result, bail};
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;

const WIDTHS: &str = "#widths";
const NAME: &str = "th[aria-label=Name]";

async fn width_near<D: Driver>(d: &mut D, want: f64, after: &str) -> Result<()> {
    eventually(d, after, async |d| {
        Ok((d.rect(NAME).await?.width - want).abs() <= 2.0)
    })
    .await
}

async fn a_drag_resizes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let grip = format!("{NAME} [data-resize-handle]");
    width_near(d, 150.0, "the declared width").await?;
    if d.exists("th[aria-label=Origin] [data-resize-handle]")
        .await?
    {
        bail!("Origin opted out, yet has a grip");
    }
    d.drag(&grip, 60.0, 0.0).await?;
    eventually_text(d, WIDTHS, "Name=210", "a 60px drag").await?;
    width_near(d, 210.0, "the drag's end").await?;
    // Clamped to the column's 100px floor; inside the window, where Blitz sees the release.
    d.drag(&grip, -150.0, 0.0).await?;
    eventually_text(d, WIDTHS, "Name=100", "a drag past the floor").await?;
    // The header's text and menu button may keep it wider than 100px: auto layout.
    eventually(d, "the narrowed column", async |d| {
        Ok(d.rect(NAME).await?.width < 140.0)
    })
    .await
}

e2e::scenario!(
    a_header_grip_drags_the_width_within_its_limits,
    "/table-resize",
    a_drag_resizes
);

/// Todo 2015: a finger 18px inside the header's end edge, past the old 8px grip, still resizes.
#[test]
fn a_touch_in_the_grip_lane_resizes_the_column() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    use e2e::browser::{Fixture, Viewport, block_on};
    use e2e::passes::pointer::{Point, touch_drag};
    use e2e::wait;
    block_on(async {
        let fixture = Fixture::open("/table-resize", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        // The 24px lane is coarse-only: measure the edge once the page took the touch screen.
        page.execute(SetTouchEmulationEnabledParams::new(true))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(pointer: coarse)').matches",
            "a coarse pointer",
        )
        .await
        .unwrap();
        let edge: (f64, f64) = page
            .evaluate(format!(
                "(r => [r.right, r.y + r.height / 2])(document.querySelector('{NAME}').getBoundingClientRect())"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let from = Point {
            x: edge.0 - 18.0,
            y: edge.1,
        };
        let to = Point {
            x: from.x + 60.0,
            y: from.y,
        };
        touch_drag(page, from, to, 8).await.unwrap();
        wait::for_js_true(
            page,
            "parseFloat(document.querySelector('#widths').textContent.replace('Name=', '')) >= 175",
            "a touch drag in the grip lane to widen Name by about 60px",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a touch resize").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1451: the grip is a separator Tab reaches; the arrows step it, Home and End go to the limits.
async fn the_keys_resize<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let grip = format!("{NAME} [data-resize-handle]");
    width_near(d, 150.0, "the declared width").await?;
    if d.attr(&grip, "role").await?.as_deref() != Some("separator")
        || d.attr(&grip, "aria-label").await?.as_deref() != Some("Resize Name")
    {
        bail!("the grip is no named separator");
    }
    d.focus(&grip).await?;
    eventually_focused(d, &grip, "focus").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_text(d, WIDTHS, "Name=160", "ArrowRight").await?;
    d.press_shift(keyboard::ARROW_LEFT).await?;
    eventually_text(d, WIDTHS, "Name=110", "Shift+ArrowLeft").await?;
    d.press(keyboard::END).await?;
    eventually_text(d, WIDTHS, "Name=300", "End").await?;
    width_near(d, 300.0, "End").await?;
    eventually(d, "aria-valuenow to follow", async |d| {
        Ok(d.attr(&grip, "aria-valuenow").await?.as_deref() == Some("300"))
    })
    .await?;
    d.press(keyboard::HOME).await?;
    eventually_text(d, WIDTHS, "Name=100", "Home").await
}

e2e::scenario!(
    the_keys_resize_a_column_from_its_grip,
    "/table-resize",
    the_keys_resize
);

/// Clicks the open menu's entry reading `label`, once it shows (todo 1502).
async fn pick<D: Driver>(d: &mut D, label: &str) -> Result<()> {
    let mut found = None;
    eventually(d, &format!("the {label} entry"), async |d| {
        for index in 0..12 {
            let item = format!("[role=menuitem][data-menu-index=\"{index}\"]");
            if d.exists(&item).await? && d.text(&item).await? == label {
                found = Some(item);
                return Ok(true);
            }
        }
        Ok(false)
    })
    .await?;
    d.click(&found.expect("the wait held")).await
}

async fn the_menu_resizes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    width_near(d, 150.0, "the declared width").await?;
    d.click("button[aria-label=\"Name column options\"]")
        .await?;
    eventually(d, "the column menu to open", async |d| {
        d.exists("[role=menuitem]").await
    })
    .await?;
    pick(d, "Widen column").await?;
    eventually_text(d, WIDTHS, "Name=200", "Widen").await?;
    width_near(d, 200.0, "Widen").await?;
    // The open menu hides the change: it is said (todo 1415).
    eventually_text(d, "[role=status]", "Name: 200 px", "the Widen announcement").await?;
    // Widen and Narrow keep the menu open, to step again.
    pick(d, "Narrow column").await?;
    eventually_text(d, WIDTHS, "Name=150", "Narrow").await?;
    // Two quick picks at one spot: on Blitz the second is a double click (todo 1413).
    pick(d, "Narrow column").await?;
    eventually_text(d, WIDTHS, "Name=100", "Narrow to the floor").await?;
    pick(d, "Reset width").await?;
    eventually_text(d, WIDTHS, "", "Reset width").await?;
    eventually_text(
        d,
        "[role=status]",
        "Name: width reset",
        "the reset announcement",
    )
    .await?;
    width_near(d, 150.0, "the reset").await
}

e2e::scenario!(
    the_column_menu_widens_narrows_and_resets_a_width,
    "/table-resize",
    the_menu_resizes
);

/// Todo 2017: in an overflowing table, Widen shows on the column, not only in the state.
async fn an_overflowing_column_widens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let before = d.rect(NAME).await?.width;
    d.click("button[aria-label=\"Name column options\"]")
        .await?;
    pick(d, "Widen column").await?;
    eventually(d, "a set width", async |d| {
        Ok(d.text(WIDTHS).await?.starts_with("Name="))
    })
    .await?;
    let want = before + 50.0;
    eventually(
        d,
        &format!("Name at about {want}px, from {before}"),
        async |d| Ok((d.rect(NAME).await?.width - want).abs() <= 2.0),
    )
    .await
}

e2e::scenario!(
    an_overflowing_tables_column_widens_from_its_menu,
    "/table-resize/wide",
    an_overflowing_column_widens
);
