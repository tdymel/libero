//! `Cascader`: the keyboard walks levels; a disabled branch (Asia) neither takes the cursor
//! nor opens (406). Focus stays on the trigger; the cursor is its `aria-activedescendant`.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text};
use e2e::passes::{keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const TRIGGER: &str = "[role=combobox]";
const SEARCH: &str = "input[role=combobox]";

/// `<cursor row's label>|<open columns>|<cursor row's aria-selected>`, or
/// `closed`.
const CURSOR: &str = "(() => { const t = document.querySelector('[role=combobox]'); \
     if (t.getAttribute('aria-expanded') !== 'true') return 'closed'; \
     const row = document.getElementById(t.getAttribute('aria-activedescendant')); \
     const label = row && row.querySelector('[data-slot=label]'); \
     return `${label && label.textContent}|${document.querySelectorAll('[role=listbox]').length}|${row && row.getAttribute('aria-selected')}`; })()";

/// Todo 1014: a tap opening a searchable dropdown lands focus in its search
/// box, and on a phone raises the keyboard.
pub async fn a_tap_focuses_the_search<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    eventually_focused(d, SEARCH, "a tap on the trigger").await?;
    if d.platform() == Platform::Android {
        eventually(d, "the soft keyboard", async |d| {
            d.soft_keyboard_shown().await
        })
        .await?;
    }
    Ok(())
}

e2e::scenario!(
    a_tap_on_a_searchable_cascader_focuses_its_search_box,
    "/cascader/search",
    a_tap_focuses_the_search
);

const ROOTS: &str = "[data-slot=column]";
const LAST: &str = "[data-slot=column]:last-child";
const BACK: &str = "[data-slot=drill-back]";
const EUROPE: &str = "[role=option][id$='-option-0-0']";
const FRANCE: &str = "[role=option][id$='-option-1-0']";
const LYON: &str = "[role=option][id$='-option-2-1']";

/// Whether the viewport is below `sm` (48rem), where the list drills in (todo 1084).
async fn is_narrow<D: Driver>(d: &mut D) -> Result<bool> {
    Ok(d.viewport().await?.0 < 768.0)
}

/// Drawn and placed: the popover stays `visibility:hidden` until measured, and a tap before that misses.
async fn shown<D: Driver>(d: &mut D, selector: &str) -> Result<bool> {
    Ok(d.exists(selector).await?
        && d.style(selector, "display").await? != "none"
        && d.style(selector, "visibility").await? == "visible")
}

/// Todo 1084: narrow, a tap on a branch shows only its children under a back header;
/// back returns to the roots; a leaf commits and closes. Wide, the columns stay side by side.
pub async fn taps_walk_the_levels<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let narrow = is_narrow(d).await?;
    // The emulator is a phone: its run must take the drill-in path.
    anyhow::ensure!(
        d.platform() != Platform::Android || narrow,
        "a wide Android viewport"
    );
    d.click(TRIGGER).await?;
    eventually(d, "the roots", async |d| shown(d, EUROPE).await).await?;
    d.click(EUROPE).await?;
    eventually(d, "Europe's children", async |d| shown(d, FRANCE).await).await?;
    let roots_shown = shown(d, ROOTS).await?;
    let back_shown = shown(d, BACK).await?;
    if !narrow {
        anyhow::ensure!(
            roots_shown && !back_shown,
            "wide: roots {roots_shown}, back {back_shown}"
        );
    } else {
        anyhow::ensure!(
            !roots_shown && back_shown,
            "narrow: roots {roots_shown}, back {back_shown}"
        );
        eventually_text(d, &format!("{BACK} [data-slot=title]"), "Europe", "the tap").await?;
        let name = d.attr(BACK, "aria-label").await?;
        anyhow::ensure!(
            name.as_deref() == Some("Back to Europe"),
            "back's name {name:?}"
        );
        // The popup takes the trigger's width at least, not one 220px column.
        let (column, trigger) = (d.rect(LAST).await?, d.rect(TRIGGER).await?);
        anyhow::ensure!(
            column.width + 2.0 >= trigger.width,
            "{column:?} vs {trigger:?}"
        );

        d.click(BACK).await?;
        eventually(d, "back to the roots", async |d| {
            Ok(!d.exists(BACK).await? && shown(d, EUROPE).await?)
        })
        .await?;
        anyhow::ensure!(!d.is_focused(BACK).await?, "back took focus");
        d.click(EUROPE).await?;
        eventually(d, "Europe's children again", async |d| {
            shown(d, FRANCE).await
        })
        .await?;
    }
    d.click(FRANCE).await?;
    eventually(d, "France's children", async |d| shown(d, LYON).await).await?;
    d.click(LYON).await?;
    eventually(d, "Lyon committed and closed", async |d| {
        Ok(d.text("#picked").await? == "lyon"
            && d.attr(TRIGGER, "aria-expanded").await?.as_deref() == Some("false"))
    })
    .await
}

e2e::scenario!(
    taps_walk_the_cascader_levels,
    "/cascader",
    taps_walk_the_levels
);

const ASIA: &str = "[role=option][id$='-option-0-1']";

/// Todo 1132: a disabled row is dimmed and half transparent, with `aria-disabled` kept.
pub async fn a_disabled_row_looks_disabled<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    eventually(d, "the roots", async |d| shown(d, EUROPE).await).await?;
    let disabled = d.attr(ASIA, "aria-disabled").await?;
    anyhow::ensure!(
        disabled.as_deref() == Some("true"),
        "aria-disabled {disabled:?}"
    );
    let (asia, europe) = (
        d.style(ASIA, "opacity").await?,
        d.style(EUROPE, "opacity").await?,
    );
    anyhow::ensure!(
        asia.parse::<f32>()? < europe.parse::<f32>()?,
        "opacity {asia} vs {europe}"
    );
    let (asia, europe) = (
        d.style(&format!("{ASIA} [data-slot=label]"), "color")
            .await?,
        d.style(&format!("{EUROPE} [data-slot=label]"), "color")
            .await?,
    );
    anyhow::ensure!(asia != europe, "label colour {asia} as an enabled row's");
    Ok(())
}

e2e::scenario!(
    a_disabled_cascader_row_looks_disabled,
    "/cascader",
    a_disabled_row_looks_disabled
);

/// Android's Back closes the sheet, keeps the value, and the app stays (1300).
async fn back_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.platform() != Platform::Android {
        return Ok(());
    }
    d.click(TRIGGER).await?;
    eventually(d, "the roots", async |d| shown(d, EUROPE).await).await?;
    d.press_back().await?;
    eventually(d, "Back to close the sheet", async |d| {
        Ok(d.attr(TRIGGER, "aria-expanded").await?.as_deref() != Some("true"))
    })
    .await?;
    eventually_text(d, "#picked", "", "Back").await?;
    // The app stayed and still opens the sheet.
    d.click(TRIGGER).await?;
    eventually(d, "the roots again", async |d| shown(d, EUROPE).await).await
}

e2e::scenario!(
    android_back_closes_a_cascader,
    "/cascader",
    back_closes,
    native: skip("1275: no Back key off Android"),
    desktop: skip("1275: no Back key off Android")
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
        let roots = e2e::ax::snapshot(page, "[id$='-listbox-0']").await;
        assert!(
            roots
                .as_ref()
                .map_or(true, |tree| !tree.contains("Oceania")),
            "the hidden roots are in the tree: {roots:?}"
        );
        fixture
            .console
            .assert_clean("a phone's cascader tree")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("cascader", "/cascader")
        .focusable(TRIGGER)
        // The shown column: narrow, the earlier ones are `display:none` (todo 1084).
        .targets("[data-slot=column]:last-child [role=option]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            "[role=listbox]",
        )
        .state(
            "drilled",
            &[Step::Press(keyboard::ARROW_RIGHT)],
            // Of type: narrow, the back header comes first.
            "div[data-slot=column]:nth-of-type(2) [role=listbox]",
        )
        .run();
}

/// Todo 484: while the search box is open it is the combobox, so the role-less trigger
/// may not keep `aria-expanded` or `aria-required` (axe `aria-allowed-attr`).
#[test]
fn a_searchable_field_meets_the_baseline() {
    Suite::new("cascader_search", "/cascader/search")
        .focusable(TRIGGER)
        .targets("[role=option]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            "[role=listbox]",
        )
        .run();
}

/// Todo 482: a search matching nothing shows "No results" and says it.
#[test]
fn a_search_matching_nothing_is_shown_and_said() {
    crate::select::search_matching_nothing("/cascader/search", TRIGGER);
}

/// The open search box carries the field's label, captions and states.
#[test]
fn the_open_search_box_is_announced_as_the_field() {
    block_on(async {
        let fixture = Fixture::open("/cascader/search", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === document.querySelector({SEARCH:?})"),
            "the search box to take focus",
        )
        .await
        .unwrap();

        let wiring: Vec<Option<String>> = page
            .evaluate(format!(
                "(() => {{ const s = document.querySelector({SEARCH:?}); \
                 const t = document.getElementById('lsx-1'); \
                 return ['aria-labelledby', 'aria-describedby', 'aria-invalid', 'aria-required'] \
                 .map(name => s.getAttribute(name)) \
                 .concat(['aria-expanded', 'aria-required', 'role'].map(name => t.getAttribute(name))); }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            wiring,
            [
                Some("lsx-1-label".to_string()),
                Some("lsx-1-description lsx-1-helper lsx-1-status".to_string()),
                Some("true".to_string()),
                Some("true".to_string()),
                None,
                None,
                None,
            ],
            "the search box's labelledby, describedby, invalid, required; \
             the trigger's expanded, required, role"
        );

        fixture.close().await.unwrap();
    });
}

/// Todo 483: the label focuses the trigger it names by id.
#[test]
fn a_click_on_the_label_focuses_the_trigger() {
    crate::select::label_click_focuses("/cascader", TRIGGER);
}

/// Down and Up skip the disabled root, Right opens a level, Left closes one,
/// Enter on a leaf commits the whole path and closes the list.
#[test]
fn the_keyboard_walks_the_levels() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let at = viewport.name();
            let fixture = Fixture::open("/cascader", viewport).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 10).await.unwrap();

            let steps = [
                (
                    keyboard::ARROW_DOWN,
                    "Europe|1|true",
                    "ArrowDown to open on the first root",
                ),
                (
                    keyboard::ARROW_DOWN,
                    "Oceania|1|true",
                    "ArrowDown to skip disabled Asia",
                ),
                (
                    keyboard::ARROW_UP,
                    "Europe|1|true",
                    "ArrowUp to skip disabled Asia",
                ),
                (
                    keyboard::END,
                    "Oceania|1|true",
                    "End to the last enabled root",
                ),
                (keyboard::HOME, "Europe|1|true", "Home to the first root"),
                (
                    keyboard::ARROW_RIGHT,
                    "France|2|true",
                    "ArrowRight into Europe",
                ),
                (
                    keyboard::ARROW_DOWN,
                    "Germany|2|true",
                    "ArrowDown within the second level",
                ),
                (
                    keyboard::ARROW_UP,
                    "France|2|true",
                    "ArrowUp within the second level",
                ),
                (
                    keyboard::ARROW_RIGHT,
                    "Paris|3|true",
                    "ArrowRight into France",
                ),
                (keyboard::ARROW_DOWN, "Lyon|3|true", "ArrowDown to Lyon"),
                (
                    keyboard::ARROW_LEFT,
                    "France|2|true",
                    "ArrowLeft back out of France",
                ),
                (
                    keyboard::ENTER,
                    "Paris|3|true",
                    "Enter on a branch to open it",
                ),
                (
                    keyboard::ARROW_DOWN,
                    "Lyon|3|true",
                    "ArrowDown to Lyon again",
                ),
                (
                    keyboard::ENTER,
                    "closed",
                    "Enter on a leaf to commit and close",
                ),
            ];
            for (key, want, what) in steps {
                keyboard::press(page, key).await.unwrap();
                expect(page, want, what, at).await;
            }

            wait::for_js_true(
                page,
                "document.querySelector('#picked').textContent === 'lyon' \
                 && document.querySelector('[role=combobox]').textContent.includes('Europe / France / Lyon') \
                 && document.activeElement === document.querySelector('[role=combobox]')",
                "Lyon committed with its path shown, focus on the trigger",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            // Todo 538: Enter on the committed leaf confirms, it does not clear.
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            expect(page, "Lyon|3|true", "ArrowDown to reopen on Lyon", at).await;
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            expect(page, "closed", "Enter on the committed leaf", at).await;
            wait::for_js_true(
                page,
                "document.querySelector('#picked').textContent === 'lyon'",
                "Lyon still committed",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            fixture
                .console
                .assert_clean(&format!("the cascader keys at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// Todo 509: Ctrl/Meta chords and Alt+ArrowRight/End leave the walk alone;
/// Alt+ArrowDown opens without moving the cursor, Alt+ArrowUp closes (APG).
#[test]
fn modifier_chords_leave_the_walk_alone() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        let caret_keys = [
            keyboard::ARROW_DOWN,
            keyboard::ARROW_UP,
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_LEFT,
            keyboard::HOME,
            keyboard::END,
        ];
        let not_alt = [("Ctrl", keyboard::CTRL), ("Meta", keyboard::META)];
        let alt = [("Alt", keyboard::ALT)];

        keyboard::assert_chords_ignored_with(page, &not_alt, &caret_keys, CURSOR)
            .await
            .unwrap_or_else(|e| panic!("closed: {e}"));
        keyboard::assert_chords_ignored_with(page, &alt, &caret_keys[1..], CURSOR)
            .await
            .unwrap_or_else(|e| panic!("closed: {e}"));

        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        expect(page, "France|2|true", "ArrowRight into Europe", "desktop").await;
        keyboard::assert_chords_ignored_with(page, &not_alt, &caret_keys, CURSOR)
            .await
            .unwrap_or_else(|e| panic!("open: {e}"));
        keyboard::assert_chords_ignored_with(page, &alt, &caret_keys[2..], CURSOR)
            .await
            .unwrap_or_else(|e| panic!("open: {e}"));

        keyboard::press_with(page, keyboard::ARROW_DOWN, keyboard::ALT)
            .await
            .unwrap();
        page.evaluate("new Promise(r => setTimeout(() => r(1), 60))")
            .await
            .unwrap();
        expect(
            page,
            "France|2|true",
            "Alt+ArrowDown to keep the cursor",
            "desktop",
        )
        .await;
        keyboard::press_with(page, keyboard::ARROW_UP, keyboard::ALT)
            .await
            .unwrap();
        expect(page, "closed", "Alt+ArrowUp to close", "desktop").await;
        keyboard::press_with(page, keyboard::ARROW_DOWN, keyboard::ALT)
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[role=combobox]').getAttribute('aria-expanded') === 'true'",
            "Alt+ArrowDown to open",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("cascader chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 485, 519 (APG select-only): Tab, Alt+ArrowUp and Space on a leaf
/// commit its path, then close.
#[test]
fn tab_commits_the_highlighted_leaf() {
    leaving_commits_lyon(keyboard::TAB, 0);
}

#[test]
fn alt_arrow_up_commits_the_highlighted_leaf() {
    leaving_commits_lyon(keyboard::ARROW_UP, keyboard::ALT);
}

#[test]
fn space_commits_the_highlighted_leaf() {
    leaving_commits_lyon(keyboard::SPACE, 0);
}

fn leaving_commits_lyon(key: keyboard::Key, modifiers: i64) {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        for step in [
            keyboard::ARROW_DOWN,
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_DOWN,
        ] {
            keyboard::press(page, step).await.unwrap();
        }
        expect(page, "Lyon|3|true", "the walk to Lyon", "desktop").await;
        keyboard::press_with(page, key, modifiers).await.unwrap();
        expect(page, "closed", &format!("{} to close", key.key), "desktop").await;
        wait::for_js_true(
            page,
            "document.querySelector('#picked').textContent === 'lyon'",
            &format!("{} to commit Lyon", key.key),
        )
        .await
        .unwrap();
        fixture.console.assert_clean(key.key).unwrap();
        fixture.close().await.unwrap();
    });
}

/// A branch is no pick without `any_level`: Tab on France only closes.
#[test]
fn tab_on_a_branch_commits_nothing() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        expect(page, "France|2|true", "ArrowRight into Europe", "desktop").await;
        keyboard::press(page, keyboard::TAB).await.unwrap();
        expect(page, "closed", "Tab to close", "desktop").await;
        page.evaluate("new Promise(r => setTimeout(() => r(1), 100))")
            .await
            .unwrap();
        let picked: String = page
            .evaluate("document.querySelector('#picked').textContent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(picked, "", "Tab on a branch committed it");
        fixture.console.assert_clean("Tab on a branch").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 632: with `any_level` a branch is a pick, so Tab and Alt+ArrowUp on
/// France commit it, then close.
#[test]
fn leaving_commits_the_highlighted_branch_with_any_level() {
    block_on(async {
        for (key, modifiers) in [(keyboard::TAB, 0), (keyboard::ARROW_UP, keyboard::ALT)] {
            let fixture = Fixture::open("/cascader/any-level", Viewport::Desktop)
                .await
                .unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
            expect(page, "France|2|true", "ArrowRight into Europe", "desktop").await;
            keyboard::press_with(page, key, modifiers).await.unwrap();
            expect(page, "closed", &format!("{} to close", key.key), "desktop").await;
            wait::for_js_true(
                page,
                "document.querySelector('#picked').textContent === 'france' \
                 && document.querySelector('[role=combobox]').textContent.includes('Europe / France')",
                &format!("{} to commit France", key.key),
            )
            .await
            .unwrap();
            fixture.console.assert_clean(key.key).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// `"paths"`: Space commits the highlighted row; the search box takes Tab to
/// commit as well.
#[test]
fn space_and_tab_commit_a_path_row() {
    block_on(async {
        for (route, key) in [
            ("/cascader/paths", keyboard::SPACE),
            ("/cascader/search", keyboard::TAB),
        ] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            wait::for_visible(page, "[role=option]").await.unwrap();
            if route == "/cascader/search" {
                keyboard::type_text(page, "lyon").await.unwrap();
                wait::for_js_true(
                    page,
                    "document.querySelectorAll('[role=option]').length === 1",
                    "the query to leave Lyon alone",
                )
                .await
                .unwrap();
                keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            } else {
                keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            }
            // Lyon's row, whoever holds the active descendant.
            let label = "(() => { const a = document.activeElement.getAttribute('aria-activedescendant'); \
                 const row = a && document.getElementById(a); \
                 const l = row && row.querySelector('[data-slot=label]'); return l ? l.textContent : ''; })()";
            wait::for_js_true(
                page,
                &format!("{label} === 'Europe / France / Lyon'"),
                &format!("{route}: ArrowDown to Lyon"),
            )
            .await
            .unwrap();
            keyboard::press(page, key).await.unwrap();
            wait::for_js_true(
                page,
                "!document.querySelector('[role=listbox]') \
                 && document.getElementById('lsx-1').textContent.includes('Europe / France / Lyon')",
                &format!("{route}: {} to commit Lyon", key.key),
            )
            .await
            .unwrap();
            fixture.console.assert_clean(route).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// APG select-only: on a closed list Home and End open on the first and last
/// enabled root, Space like Enter on the first.
#[test]
fn home_end_and_space_open_a_closed_list() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        for (key, want, what) in [
            (
                keyboard::HOME,
                "Europe|1|true",
                "Home to open on the first root",
            ),
            (
                keyboard::END,
                "Oceania|1|true",
                "End to open on the last root",
            ),
            (
                keyboard::SPACE,
                "Europe|1|true",
                "Space to open on the first root",
            ),
        ] {
            keyboard::press(page, key).await.unwrap();
            expect(page, want, what, "desktop").await;
            keyboard::press(page, keyboard::ESCAPE).await.unwrap();
            expect(page, "closed", "Escape to close", "desktop").await;
        }
        fixture
            .console
            .assert_clean("cascader Home/End/Space")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// In `"paths"` the opening keys arm a whole-path row. They armed a root, which
/// is no row there, so the list opened with nothing highlighted.
#[test]
fn the_paths_layout_opens_on_a_row() {
    block_on(async {
        let fixture = Fixture::open("/cascader/paths", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        for (key, want, what) in [
            (
                keyboard::ARROW_DOWN,
                "Europe / France / Paris|1|true",
                "ArrowDown to open on the first path",
            ),
            (
                keyboard::END,
                "Oceania / Australia / Sydney|1|true",
                "End to open on the last enabled path",
            ),
        ] {
            keyboard::press(page, key).await.unwrap();
            expect(page, want, what, "desktop").await;
            keyboard::press(page, keyboard::ESCAPE).await.unwrap();
            expect(page, "closed", "Escape to close", "desktop").await;
        }
        fixture.console.assert_clean("cascader paths").unwrap();
        fixture.close().await.unwrap();
    });
}

/// APG typeahead: a letter opens a closed list on the root it starts, moves
/// within the cursor's column once open, and skips disabled Asia.
#[test]
fn typing_moves_to_the_row_it_starts() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        for (key, want, what) in [
            (
                letter("o", "KeyO", 79),
                "Oceania|1|true",
                "o to open on Oceania",
            ),
            (letter("e", "KeyE", 69), "Europe|1|true", "e to Europe"),
            (
                letter("a", "KeyA", 65),
                "Europe|1|true",
                "a to pass over disabled Asia",
            ),
        ] {
            keyboard::press(page, key).await.unwrap();
            expect(page, want, what, "desktop").await;
            // Past the typeahead's reset, so the next letter starts afresh.
            page.evaluate("new Promise(r => setTimeout(() => r(1), 600))")
                .await
                .unwrap();
        }
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        expect(page, "France|2|true", "ArrowRight into Europe", "desktop").await;
        keyboard::press(page, letter("g", "KeyG", 71))
            .await
            .unwrap();
        expect(page, "Germany|2|true", "g to Germany", "desktop").await;
        fixture.console.assert_clean("cascader typeahead").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A click on disabled Asia leaves the cursor on Europe, so ArrowDown lands on Oceania.
/// A click on Oceania is the positive control that a click opens a root.
#[test]
fn a_disabled_branch_does_not_open() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect(page, "Europe|1|true", "ArrowDown to open", "desktop").await;

        const ASIA: &str = "[role=option][id$='-option-0-1']";
        wait::for_visible(page, ASIA).await.unwrap();
        let disabled: Option<String> = page
            .evaluate(format!(
                "document.querySelector({ASIA:?}).getAttribute('aria-disabled')"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            disabled.as_deref(),
            Some("true"),
            "Asia's row is aria-disabled"
        );

        pointer::click(page, ASIA).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect(
            page,
            "Oceania|1|true",
            "ArrowDown from Europe after the click on Asia",
            "desktop",
        )
        .await;

        pointer::click(page, "[role=option][id$='-option-0-2']")
            .await
            .unwrap();
        expect(
            page,
            "Australia|2|true",
            "a click on Oceania to open it",
            "desktop",
        )
        .await;

        fixture
            .console
            .assert_clean("clicking a disabled branch")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 642: drilled to Lyon, the marked rows read as a path: one selected row per
/// column, none drawing a start bar (todo 1073).
#[test]
fn the_drilled_rows_read_as_a_path() {
    block_on(async {
        let fixture = Fixture::open("/cascader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect(page, "Lyon|3|true", "ArrowDown to Lyon", "desktop").await;

        // Per column: each selected row's label and whether it draws the bar.
        let marked: Vec<Vec<(String, bool)>> = page
            .evaluate(
                "[...document.querySelectorAll('[role=listbox]')].map(list => \
                 [...list.querySelectorAll('[role=option][aria-selected=true]')].map(row => \
                 [row.querySelector('[data-slot=label]').textContent, \
                  getComputedStyle(row).backgroundImage.startsWith('linear-gradient')]))",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let path = ["Europe", "France", "Lyon"].map(|label| vec![(label.to_string(), false)]);
        assert_eq!(marked, path, "one selected row per column, none barred");
        fixture.console.assert_clean("a drilled cascader").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A letter key with its keydown, which typeahead listens to.
const fn letter(key: &'static str, code: &'static str, vk: i64) -> keyboard::Key {
    keyboard::Key {
        key,
        code,
        vk,
        text: Some(key),
    }
}

async fn expect(page: &chromiumoxide::Page, want: &str, what: &str, at: &str) {
    if let Err(e) = wait::for_js_true(page, &format!("{CURSOR} === {want:?}"), what).await {
        let actual: String = page.evaluate(CURSOR).await.unwrap().into_value().unwrap();
        panic!("at {at}: {e}; cursor reads {actual:?}");
    }
}
