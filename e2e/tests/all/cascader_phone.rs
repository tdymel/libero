//! `Cascader` below `sm`: the drill-in list, the bottom sheet, and the same two in a
//! narrow native window.

use anyhow::Result;
use e2e::archetypes;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_text};
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

use super::cascader::{BACK, EUROPE, FRANCE, LAST, TRIGGER, expect, shown, taps_walk_the_levels};

/// Android's Back closes the sheet, keeps the value, and the app stays (1300).
async fn back_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    archetypes::back_closes(d, TRIGGER, async |d| {
        Ok(d.attr(TRIGGER, "aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    eventually_text(d, "#picked", "", "Back").await
}

e2e::scenario!(
    android_back_closes_a_cascader,
    "/cascader",
    back_closes,
    android_only("1275: no Back key off Android")
);

/// The same walk in a 390px browser, below `sm`.
#[test]
fn a_phone_drills_into_one_level_at_a_time() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Mobile).await.unwrap();
        let mut driver = e2e::driver::Web { fixture };
        taps_walk_the_levels(&mut driver, "/cascader")
            .await
            .unwrap();
        driver.finish("a phone's cascader").await.unwrap();
    });
}

/// Todos 1546 and 1658 (WCAG 2.4.11): opening scrolls a low field, helper and error
/// included, above the sheet.
#[test]
fn a_phone_scrolls_a_low_trigger_above_the_sheet() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
    const SHEET: &str = "[data-state~=bordered]:has([role=listbox])";
    block_on(async {
        let fixture = Fixture::open("/cascader/low", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(SetDeviceMetricsOverrideParams::new(320, 568, 1.0, true))
            .await
            .unwrap();
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        let clear = format!(
            "(() => {{ const sheet = document.querySelector({SHEET:?}); \
             if (!sheet || getComputedStyle(sheet).visibility !== 'visible') return false; \
             return document.querySelector('#low-field').getBoundingClientRect().bottom \
             <= sheet.getBoundingClientRect().top; }})()"
        );
        wait::for_js_true(page, &clear, "the field to clear the sheet")
            .await
            .unwrap();
        fixture.console.assert_clean("a low cascader").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1111: below `sm` the dropdown is a sheet on the screen's foot, edge to edge.
#[test]
fn a_phone_shows_the_dropdown_as_a_bottom_sheet() {
    const SHEET: &str = "[data-state~=bordered]:has([role=listbox])";
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Mobile).await.unwrap();
        let mut d = e2e::driver::Web { fixture };
        d.click(TRIGGER).await.unwrap();
        eventually(&mut d, "the roots", async |d| shown(d, EUROPE).await)
            .await
            .unwrap();
        let (width, height) = d.viewport().await.unwrap();
        let sheet = d.rect(SHEET).await.unwrap();
        assert!(
            sheet.x.abs() < 1.0 && (sheet.width - width).abs() < 1.0,
            "not edge to edge: {sheet:?} in {width}"
        );
        assert!(
            (sheet.y + sheet.height - height).abs() < 1.0,
            "not at the foot: {sheet:?} in {height}"
        );
        assert!(sheet.height <= height * 0.7 + 1.0, "too tall: {sheet:?}");
        d.finish("a phone's cascader sheet").await.unwrap();
    });
}

/// The same walk in a 390px Blitz window: stylo matches the narrow rule too.
#[cfg(feature = "native")]
#[test]
// Named `native_*`: the gate's native tier filters on `::native`.
fn native_narrow_window_drills_in() {
    let mut driver = e2e::driver::Native::open("/cascader");
    driver.page.resize(390, 844);
    e2e::futures::executor::block_on(taps_walk_the_levels(&mut driver, "/cascader")).unwrap();
}

/// Todo 1124: stylo applies the sheet's `!important` rules too: edge to edge, at the foot.
#[cfg(feature = "native")]
#[test]
fn native_narrow_window_shows_a_bottom_sheet() {
    let mut driver = e2e::driver::Native::open("/cascader");
    driver.page.resize(390, 844);
    e2e::futures::executor::block_on(async {
        driver.click(TRIGGER).await.unwrap();
        eventually(&mut driver, "the roots", async |d| shown(d, EUROPE).await)
            .await
            .unwrap();
    });
    let (x, y, width, height) = driver.page.rect("[data-state~=bordered]");
    assert!(x.abs() < 1.0 && (width - 390.0).abs() < 1.0, "{x} {width}");
    assert!((y + height - 844.0).abs() < 1.0, "{y} {height}");
    assert!(height <= 844.0 * 0.7 + 1.0, "too tall: {height}");
}

/// Todo 1084: `any_level` on a phone, the first row picks the parent a tap only drilled into.
#[test]
fn a_phone_picks_a_branch_from_its_first_row() {
    const PICK: &str = "[data-slot=pick-parent]";
    block_on(async {
        let fixture = Fixture::open("/cascader/any-level", Viewport::Mobile)
            .await
            .unwrap();
        let mut d = e2e::driver::Web { fixture };
        d.click(TRIGGER).await.unwrap();
        eventually(&mut d, "the roots", async |d| shown(d, EUROPE).await)
            .await
            .unwrap();
        d.click(EUROPE).await.unwrap();
        d.click(FRANCE).await.unwrap();
        eventually(&mut d, "France picked and opened", async |d| {
            Ok(d.text("#picked").await? == "france" && shown(d, PICK).await?)
        })
        .await
        .unwrap();
        eventually_text(
            &mut d,
            &format!("{LAST} {PICK}"),
            "Select France",
            "the tap",
        )
        .await
        .unwrap();
        d.click(BACK).await.unwrap();
        eventually(&mut d, "Select Europe", async |d| {
            Ok(d.text(&format!("{LAST} {PICK}")).await? == "Select Europe")
        })
        .await
        .unwrap();
        d.click(&format!("{LAST} {PICK}")).await.unwrap();
        eventually(&mut d, "Europe committed and closed", async |d| {
            Ok(d.text("#picked").await? == "europe"
                && d.attr(TRIGGER, "aria-expanded").await?.as_deref() == Some("false"))
        })
        .await
        .unwrap();

        d.finish("a phone's any-level cascader").await.unwrap();
    });
}

/// Todo 1084: the hidden columns leave the tree, and the shown one keeps its name
/// from its hidden parent row (accname follows `aria-labelledby` into hidden content).
#[test]
fn a_phone_names_the_one_column_by_its_parent() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        expect(page, "France|2|true", "ArrowRight into Europe", "mobile").await;
        let tree = e2e::ax::snapshot(page, TRIGGER).await.unwrap();
        assert!(tree.contains("listbox \"Europe\""), "{tree}");
        assert!(tree.contains("France"), "{tree}");
        // Only "no node" passes: any other snapshot error is no proof (todo 1770).
        const ROOTS: &str = "[id$='-listbox-0']";
        wait::for_selector(page, ROOTS).await.unwrap();
        match e2e::ax::snapshot(page, ROOTS).await {
            Ok(tree) => assert!(
                !tree.contains("Oceania"),
                "the hidden roots are in the tree: {tree}"
            ),
            Err(e) => assert!(
                e.to_string().starts_with("no accessibility node"),
                "the roots' snapshot failed: {e:#}"
            ),
        }
        fixture
            .console
            .assert_clean("a phone's cascader tree")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
