//! `Modal`: the overlay archetype. `contrast_covers` guards todo 327: the scroll lock made
//! axe judge the whole dialog off-screen, so the open state checked nothing.

use anyhow::{Result, ensure};
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::suite::Step;
use e2e::wait;
use e2e::{
    Fixture, Suite, Viewport,
    passes::{focus, keyboard, pointer},
};

const TRIGGER: &str = "#open-modal";
const DIALOG: &str = "[role=dialog]";
const INSIDE: &str = "[role=dialog], [role=dialog] *";

/// Clicks `trigger` until a dialog shows.
pub async fn open<D: Driver>(d: &mut D, trigger: &str) -> Result<()> {
    d.click(trigger).await?;
    eventually(
        d,
        &format!("a click on {trigger} to open a dialog"),
        async |d| d.exists(DIALOG).await,
    )
    .await
}

async fn assert_inside<D: Driver>(d: &mut D, step: &str) -> Result<()> {
    eventually_focused(d, INSIDE, step).await
}

/// Opening moves focus in; Tab and Shift+Tab stay inside.
pub async fn tab_stays_inside<D: Driver>(d: &mut D, trigger: &str) -> Result<()> {
    open(d, trigger).await?;
    assert_inside(d, "opening").await?;
    for step in 0..4 {
        d.press(keyboard::TAB).await?;
        assert_inside(d, &format!("Tab {step}")).await?;
    }
    for step in 0..4 {
        d.press_shift(keyboard::TAB).await?;
        assert_inside(d, &format!("Shift+Tab {step}")).await?;
    }
    Ok(())
}

/// A click on the top left corner, backdrop for a centred dialog and an end
/// drawer, closes it and hands focus back to `trigger`.
pub async fn a_backdrop_click_closes<D: Driver>(d: &mut D, trigger: &str) -> Result<()> {
    open(d, trigger).await?;
    assert_inside(d, "opening").await?;
    d.click_at(4.0, 4.0).await?;
    closed_with_focus_on(d, trigger, "a backdrop click").await
}

pub async fn closed_with_focus_on<D: Driver>(d: &mut D, trigger: &str, after: &str) -> Result<()> {
    eventually(d, &format!("{after} to close the dialog"), async |d| {
        Ok(!d.exists(DIALOG).await?)
    })
    .await?;
    eventually_focused(d, trigger, after).await
}

async fn modal_traps_tab<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    tab_stays_inside(d, TRIGGER).await
}

async fn modal_tab_cycles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d, TRIGGER).await?;
    assert_inside(d, "opening").await?;
    d.focus("#keep").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#discard", "Tab from Keep editing").await?;
    d.press(keyboard::TAB).await?;
    assert_inside(d, "Tab from Discard").await?;
    ensure!(!d.is_focused("#discard").await?, "Tab did not wrap");
    Ok(())
}

async fn modal_backdrop_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    a_backdrop_click_closes(d, TRIGGER).await
}

e2e::scenario!(
    a_modal_moves_focus_in_and_traps_tab,
    "/modal",
    modal_traps_tab,
    android: skip("958: element identity on the WebView")
);
e2e::scenario!(
    tab_cycles_through_every_button_in_a_modal,
    "/modal",
    modal_tab_cycles,
    android: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_backdrop_click_closes_a_modal,
    "/modal",
    modal_backdrop_closes,
    android: skip("958: element identity on the WebView")
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("modal", "/modal")
        .focusable(TRIGGER)
        .contrast_covers(DIALOG)
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ENTER)],
            DIALOG,
        )
        .run();
}

/// APG: a dialog with nothing focusable takes focus itself, not leaving it on the trigger
/// behind the backdrop.
#[test]
fn a_dialog_with_nothing_focusable_takes_focus_itself() {
    block_on(async {
        let fixture = Fixture::open("/modal/static", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.matches('[role=dialog]') === true",
            "focus on the dialog",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::assert_focused(page, DIALOG, "Tab in a dialog with nothing to focus")
            .await
            .unwrap();

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=dialog]')",
            "Escape to close the dialog",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, TRIGGER, "Escape")
            .await
            .unwrap();
        fixture.console.assert_clean("the static modal").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A click on the dialog's text used to drop focus to `<body>`, outside the
/// trap: Escape then did nothing and Tab reached the page behind.
#[test]
fn a_click_on_the_dialog_text_keeps_escape_and_the_trap() {
    block_on(async {
        let fixture = Fixture::open("/modal", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, DIALOG).await.unwrap();

        pointer::click(page, "#modal-text").await.unwrap();
        for _ in 0..4 {
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            let inside: bool = page
                .evaluate(
                    "document.querySelector('[role=dialog]').contains(document.activeElement)",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                inside,
                "Shift+Tab after a click on the text left the dialog"
            );
        }

        pointer::click(page, "#modal-text").await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=dialog]')",
            "Escape after a click on the text to close the dialog",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, TRIGGER, "Escape")
            .await
            .unwrap();
        fixture.console.assert_clean("the clicked modal").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/modal", viewport).await.unwrap();

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
                .assert_clean(&format!("the modal contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
