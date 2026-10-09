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
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    tab_cycles_through_every_button_in_a_modal,
    "/modal",
    modal_tab_cycles,
    android: skip("959: the trap cannot wrap Tab on a WebView"),
    desktop: skip("959: the trap cannot wrap Tab on a WebView")
);

/// A WebView's trap cannot cycle, but Tab must still move between the dialog's buttons: the
/// modal used to swallow every Tab there, leaving all but the first button out of reach.
async fn modal_tab_moves_on<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d, TRIGGER).await?;
    assert_inside(d, "opening").await?;
    d.focus("#keep").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#discard", "Tab from Keep editing").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, "#keep", "Shift+Tab from Discard").await
}

e2e::scenario!(
    tab_moves_between_the_buttons_of_a_modal,
    "/modal",
    modal_tab_moves_on
);
e2e::scenario!(
    a_backdrop_click_closes_a_modal,
    "/modal",
    modal_backdrop_closes,
    android: skip("958: element identity on the WebView")
);

/// The trap focuses inside on open, so the first Escape is the dialog's, and focus
/// goes back to the trigger (959, 1318).
async fn modal_takes_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d, TRIGGER).await?;
    eventually_focused(d, INSIDE, "opening the dialog").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the first Escape to close the dialog", async |d| {
        Ok(!d.exists(DIALOG).await?)
    })
    .await?;
    eventually_focused(d, TRIGGER, "closing the dialog").await
}

e2e::scenario!(
    a_modal_takes_focus_and_the_first_escape_closes_it,
    "/modal",
    modal_takes_focus
);

const SHEET: &str = "#sheet";

/// Todo 2559: content that is no `Dialog` takes clicks, its text as its buttons; only
/// the backdrop closes. The ping lands after a click-through's close would.
async fn non_dialog_content_takes_clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    eventually(d, "the sheet to open", async |d| d.exists(SHEET).await).await?;
    d.click("#sheet-text").await?;
    d.click("#ping").await?;
    eventually(d, "the ping inside the sheet", async |d| {
        Ok(d.text("#pings").await? == "1")
    })
    .await?;
    ensure!(d.exists(SHEET).await?, "a click on the sheet closed it");
    d.click_at(4.0, 4.0).await?;
    eventually(d, "a backdrop click to close the sheet", async |d| {
        Ok(!d.exists(SHEET).await?)
    })
    .await?;
    eventually_focused(d, TRIGGER, "closing the sheet").await
}

e2e::scenario!(
    content_that_is_no_dialog_takes_clicks,
    "/modal/paper",
    non_dialog_content_takes_clicks
);

/// Todo 2559: with no `Dialog` and nothing focusable, focus lands on the content box, so
/// Escape reaches the modal and focus goes back to the trigger.
async fn non_dialog_content_takes_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    eventually(d, "the sheet to open", async |d| d.exists(SHEET).await).await?;
    eventually_focused(
        d,
        "[data-lsx-scroll-lock] [tabindex='-1']",
        "opening the sheet",
    )
    .await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "Escape to close the sheet", async |d| {
        Ok(!d.exists(SHEET).await?)
    })
    .await?;
    eventually_focused(d, TRIGGER, "closing the sheet").await
}

e2e::scenario!(
    content_that_is_no_dialog_takes_focus_on_its_box,
    "/modal/paper-static",
    non_dialog_content_takes_focus
);

const ALERT: &str = "[role=alertdialog]";

/// Todo 2564: an `alertdialog` keeps its role and description and ignores the
/// backdrop; Escape still closes it. The ping lands after the backdrop click's close would.
async fn alert_ignores_the_backdrop<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#open-alert").await?;
    eventually(d, "the alert dialog to open", async |d| {
        d.exists(ALERT).await
    })
    .await?;
    ensure!(
        d.attr(ALERT, "aria-describedby").await?.as_deref() == Some("confirm-text"),
        "the alert dialog is not described by its message"
    );
    eventually_focused(d, "[role=alertdialog], [role=alertdialog] *", "opening").await?;
    d.click_at(4.0, 4.0).await?;
    d.click("#ping").await?;
    eventually(d, "the ping after the backdrop click", async |d| {
        Ok(d.text("#pings").await? == "1")
    })
    .await?;
    ensure!(
        d.exists(ALERT).await?,
        "a backdrop click closed the alert dialog"
    );
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "Escape to close the alert dialog", async |d| {
        Ok(!d.exists(ALERT).await?)
    })
    .await?;
    eventually_focused(d, "#open-alert", "closing the alert dialog").await
}

e2e::scenario!(
    an_alertdialog_ignores_the_backdrop,
    "/modal/guarded",
    alert_ignores_the_backdrop,
    android: skip("958: element identity on the WebView")
);

/// Todo 2563: `ondismiss` hears each dismissal and keeps a draft open; with the
/// draft gone, Escape closes.
async fn a_draft_vetoes_dismissal<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d, "#open-draft").await?;
    d.click("#dirty").await?;
    d.press(keyboard::ESCAPE).await?;
    d.click_at(4.0, 4.0).await?;
    eventually(d, "Escape and the backdrop to be refused", async |d| {
        Ok(d.text("#reasons").await? == "Escape,Backdrop")
    })
    .await?;
    ensure!(
        d.exists(DIALOG).await?,
        "a refused dismissal closed the draft"
    );
    d.click("#dirty").await?;
    d.press(keyboard::ESCAPE).await?;
    closed_with_focus_on(d, "#open-draft", "Escape without a draft").await
}

e2e::scenario!(
    ondismiss_keeps_a_draft_open,
    "/modal/guarded",
    a_draft_vetoes_dismissal,
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

const MENU_ITEM: &str = "[role=menuitem]";
const MENU_TRIGGER: &str = "[role=dialog] [aria-haspopup=menu]";

/// Android's Back or Escape, the key that closes the top layer.
#[derive(Clone, Copy, PartialEq)]
enum Dismiss {
    Back,
    Escape,
}

impl Dismiss {
    async fn press<D: Driver>(self, d: &mut D) -> Result<()> {
        match self {
            Dismiss::Back => d.press_back().await,
            Dismiss::Escape => d.press(keyboard::ESCAPE).await,
        }
    }
}

/// A menu in a dialog: the first dismissal closes only the menu, the second the
/// dialog. On web focus also returns to the menu's trigger and Tab stays trapped (1805).
async fn the_menu_closes_before_its_dialog<D: Driver>(d: &mut D, key: Dismiss) -> Result<()> {
    open(d, TRIGGER).await?;
    d.click(MENU_TRIGGER).await?;
    eventually(d, "the menu to open", async |d| d.exists(MENU_ITEM).await).await?;
    key.press(d).await?;
    eventually(d, "the first dismissal to close the menu", async |d| {
        Ok(!d.exists(MENU_ITEM).await?)
    })
    .await?;
    d.settle().await?;
    ensure!(
        d.exists(DIALOG).await?,
        "one dismissal closed the dialog too"
    );
    if key == Dismiss::Escape {
        eventually_focused(d, MENU_TRIGGER, "Escape on the menu").await?;
        for step in 0..4 {
            d.press(keyboard::TAB).await?;
            assert_inside(d, &format!("Tab {step} after the menu closed")).await?;
        }
    }
    key.press(d).await?;
    eventually(d, "a second dismissal to close the dialog", async |d| {
        Ok(!d.exists(DIALOG).await?)
    })
    .await?;
    if key == Dismiss::Escape {
        eventually_focused(d, TRIGGER, "closing the dialog").await?;
    }
    Ok(())
}

/// Android's Back closes the top layer, a menu before its dialog, and the app
/// stays (1275). Only Android has the key.
async fn back_closes_the_top_layer<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d, TRIGGER).await?;
    d.press_back().await?;
    eventually(d, "Back to close the dialog", async |d| {
        Ok(!d.exists(DIALOG).await?)
    })
    .await?;

    the_menu_closes_before_its_dialog(d, Dismiss::Back).await?;
    // Still in the app: the page answers.
    ensure!(
        d.exists(TRIGGER).await?,
        "the app left after the layers closed"
    );
    Ok(())
}

e2e::scenario!(
    android_back_closes_the_top_layer,
    "/modal/menu",
    back_closes_the_top_layer,
    android_only("1275: no Back key off Android")
);

async fn escape_closes_the_menu_first<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    the_menu_closes_before_its_dialog(d, Dismiss::Escape).await
}

e2e::scenario!(
    escape_in_a_modal_menu_closes_only_the_menu,
    "/modal/menu",
    escape_closes_the_menu_first,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);

async fn eval_bool(page: &chromiumoxide::Page, js: &str) -> bool {
    page.evaluate(js).await.unwrap().into_value().unwrap()
}

async fn dialog_count(page: &chromiumoxide::Page) -> i64 {
    page.evaluate("document.querySelectorAll('[role=dialog]').length")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

async fn wait_dialogs(page: &chromiumoxide::Page, n: i64, what: &str) {
    wait::for_js_true(
        page,
        &format!("document.querySelectorAll('[role=dialog]').length === {n}"),
        what,
    )
    .await
    .unwrap();
}

/// Layers close one at a time: Escape and a backdrop click each close only the top
/// modal, focus goes back to the control that opened it, the scroll lock holds until the last.
#[test]
fn nested_modals_close_one_layer_at_a_time() {
    block_on(async {
        let fixture = Fixture::open("/modal/nested", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait_dialogs(page, 1, "the outer dialog").await;
        pointer::click(page, "#open-inner").await.unwrap();
        wait_dialogs(page, 2, "the inner dialog").await;
        wait::for_js_true(
            page,
            "document.activeElement.closest('[role=dialog]')?.querySelector('#inner-close') != null",
            "focus in the inner dialog",
        )
        .await
        .unwrap();
        assert!(
            eval_bool(
                page,
                "getComputedStyle(document.body).overflow === 'hidden'"
            )
            .await,
            "no scroll lock"
        );

        for _ in 0..4 {
            keyboard::press(page, keyboard::TAB).await.unwrap();
            assert!(
                eval_bool(
                    page,
                    "document.activeElement.closest('[role=dialog]')?.querySelector('#inner-close') !== null"
                )
                .await,
                "Tab left the inner dialog"
            );
        }

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait_dialogs(page, 1, "Escape to close the inner only").await;
        focus::wait_for_focus(page, "#open-inner", "Escape on the inner")
            .await
            .unwrap();
        assert!(
            eval_bool(
                page,
                "getComputedStyle(document.body).overflow === 'hidden'"
            )
            .await,
            "the scroll lock dropped with the outer still open"
        );

        pointer::click(page, "#open-inner").await.unwrap();
        wait_dialogs(page, 2, "the inner again").await;
        // Backdrop of the inner: top left corner.
        pointer::click_at(page, pointer::Point { x: 4.0, y: 4.0 })
            .await
            .unwrap();
        wait_dialogs(page, 1, "a backdrop click to close the inner only").await;
        focus::wait_for_focus(page, "#open-inner", "backdrop on the inner")
            .await
            .unwrap();

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait_dialogs(page, 0, "Escape to close the outer").await;
        focus::wait_for_focus(page, TRIGGER, "Escape on the outer")
            .await
            .unwrap();
        assert!(
            eval_bool(
                page,
                "getComputedStyle(document.body).overflow !== 'hidden'"
            )
            .await,
            "the scroll lock stayed after the last close"
        );
        assert_eq!(dialog_count(page).await, 0);
        fixture.console.assert_clean("nested modals").unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1.4.10 / 2.4.11: every control of a dialog taller than the viewport can be reached.
#[test]
fn a_dialog_taller_than_the_viewport_stays_reachable() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/modal/tall", viewport).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_visible(page, DIALOG).await.unwrap();
            let name = viewport.name();
            assert_reachable(
                page,
                "document.querySelector('#accept')",
                &format!("Accept at {name}"),
            )
            .await;
            assert_reachable(
                page,
                "document.querySelector('[role=dialog] button[aria-label]')",
                &format!("the close button at {name}"),
            )
            .await;
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// Todo 2560: with a classic scrollbar, a press on the track of a tall dialog's scroller
/// leaves the modal open, the thumb drags, and the wheel beside the dialog scrolls.
#[test]
fn the_scrollbar_and_the_wheel_beside_a_tall_dialog_scroll_it() {
    use chromiumoxide::cdp::browser_protocol::input::{
        DispatchMouseEventParams, DispatchMouseEventType,
    };
    const SCROLLER: &str = "(() => { let e = document.querySelector('[role=dialog]'); \
         while (e && getComputedStyle(e).overflowY !== 'auto') e = e.parentElement; return e; })()";
    block_on(async {
        let fixture = Fixture::open_with_scrollbars("/modal/tall", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, DIALOG).await.unwrap();
        let geometry: Vec<f64> = page
            .evaluate(format!(
                "(() => {{ const s = {SCROLLER}; const r = s.getBoundingClientRect(); \
                 const d = document.querySelector('[role=dialog]').getBoundingClientRect(); \
                 return [s.offsetWidth - s.clientWidth, s.scrollHeight - s.clientHeight, \
                 r.right, r.bottom, d.left]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let [gutter, overflow, right, bottom, dialog_left] = geometry[..] else {
            panic!("geometry: {geometry:?}");
        };
        assert!(gutter > 0.0, "no classic scrollbar to test with");
        assert!(
            overflow > 0.0,
            "the dialog fits the viewport, nothing to scroll"
        );
        let scroll_top = async || -> f64 {
            page.evaluate(format!("{SCROLLER}.scrollTop"))
                .await
                .unwrap()
                .into_value()
                .unwrap()
        };

        // Headless Chrome neither pages nor drags on CDP presses: the hit test is the
        // proof that the bar is live, and the press must not close.
        let track = pointer::Point {
            x: right - gutter / 2.0,
            y: bottom - 40.0,
        };
        assert!(
            eval_bool(
                page,
                &format!(
                    "document.elementFromPoint({}, {}) === {SCROLLER}",
                    track.x, track.y
                )
            )
            .await,
            "the scrollbar does not take the pointer"
        );
        pointer::click_at(page, track).await.unwrap();

        // Also lets a close the press spawned land first: the scroller reads need the dialog.
        let before = scroll_top().await;
        let beside = DispatchMouseEventParams::builder()
            .r#type(DispatchMouseEventType::MouseWheel)
            .x(dialog_left / 2.0)
            .y(bottom / 2.0)
            .delta_x(0.0)
            .delta_y(120.0)
            .build()
            .unwrap();
        page.execute(beside).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "!!document.querySelector('[role=dialog]') && {SCROLLER}.scrollTop > {before}"
            ),
            "the wheel beside the dialog to scroll",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("scrolling a tall dialog")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Scrolls the element `element` (a JS expression) into view the way a user
/// can, and fails if it is off screen or only an `overflow: hidden` box or the
/// locked page moved to show it (todo 1307).
pub async fn assert_reachable(page: &chromiumoxide::Page, element: &str, what: &str) {
    let probe: String = page
        .evaluate(format!(
            "(() => {{ const a = {element}; if (!a) return JSON.stringify({{missing: true}}); \
             a.scrollIntoView({{block: 'nearest'}}); const r = a.getBoundingClientRect(); \
             const locked = getComputedStyle(document.body).overflow === 'hidden'; const stuck = []; \
             for (let el = a.parentElement; el; el = el.parentElement) {{ \
               if (el.scrollTop <= 0) continue; \
               const s = getComputedStyle(el).overflowY; \
               const page = el === document.documentElement || el === document.body; \
               if (page ? locked : !/auto|scroll/.test(s)) stuck.push(el.tagName + ' ' + s); }} \
             return JSON.stringify({{top: r.top, bottom: r.bottom, vh: innerHeight, stuck}}); }})()"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    let probe: serde_json::Value = serde_json::from_str(&probe).unwrap();
    assert!(probe["missing"].is_null(), "{what}: not found");
    let at = |key: &str| probe[key].as_f64().unwrap();
    assert!(
        at("top") >= -0.5 && at("bottom") <= at("vh") + 0.5,
        "{what} cannot be scrolled on screen: {probe}"
    );
    assert!(
        probe["stuck"]
            .as_array()
            .is_some_and(|stuck| stuck.is_empty()),
        "{what} is shown only by scrolling a box the user cannot: {probe}"
    );
}

/// Todo 1304: the only control removes itself, focus falls to `<body>`, and
/// Escape still closes the dialog and hands focus back.
#[test]
fn escape_closes_after_the_focused_control_is_removed() {
    block_on(async {
        let fixture = Fixture::open("/modal/self-removing", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        focus::wait_for_focus(page, "#remove-me", "opening")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#remove-me')",
            "the button gone",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=dialog]')",
            "Escape with focus on the body to close the dialog",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, TRIGGER, "Escape")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("the self-removing modal")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2297: the last control removes itself, so focus falls to `<body>` and the next
/// Tab is a `<body>` press: it comes back to the dialog, not the page behind.
#[test]
fn tab_stays_in_after_the_focused_control_is_removed() {
    block_on(async {
        let fixture = Fixture::open("/modal/last-removing", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        focus::wait_for_focus(page, "#stay", "opening")
            .await
            .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::wait_for_focus(page, "#remove-me", "Tab")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#remove-me')",
            "the button gone",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[role=dialog]').contains(document.activeElement)",
            "Tab after the removal to stay in the dialog",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("the last-removing modal")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1305: with a classic scrollbar, opening and closing does not shift the page.
#[test]
fn the_scroll_lock_keeps_the_scrollbar_gutter() {
    block_on(async {
        const GUTTER: &str = "innerWidth - document.documentElement.clientWidth";
        let hidden = Fixture::open("/modal/scrollbar", Viewport::Desktop)
            .await
            .unwrap();
        let gutter: f64 = hidden
            .page
            .evaluate(GUTTER)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(gutter, 0.0, "the harness shows scrollbars by default");
        hidden.close().await.unwrap();

        let fixture = Fixture::open_with_scrollbars("/modal/scrollbar", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        // The page's own padding, which the lock adds to rather than replaces.
        page.evaluate("document.documentElement.style.paddingRight = '8px'")
            .await
            .unwrap();
        const WIDTH: &str = "document.querySelector('#ruler').getBoundingClientRect().width";
        let width = async || -> f64 { page.evaluate(WIDTH).await.unwrap().into_value().unwrap() };
        let before = width().await;
        let gutter: f64 = page.evaluate(GUTTER).await.unwrap().into_value().unwrap();
        assert!(gutter > 0.0, "no classic scrollbar to test with");
        let (w, _) = Viewport::Desktop.size();
        let (strip_x, backdrop_x) = (w as f64 - 2.0, w as f64 - gutter - 20.0);
        let undimmed = pixel(page, backdrop_x, 600.0).await;

        keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "getComputedStyle(document.body).overflow === 'hidden' \
                 && !!document.elementFromPoint({backdrop_x}, 600)?.closest('[data-lsx-scroll-lock]')"
            ),
            "the scroll lock and the backdrop",
        )
        .await
        .unwrap();
        let open = width().await;
        assert_eq!(open, before, "the page shifted on open (gutter {gutter}px)");
        // Todo 1316: the backdrop dims the scrollbar's old strip too. Compared by
        // paint: hit testing skips a `scrollbar-gutter`. Control: the backdrop dims at all.
        let undimmed = undimmed.as_slice();
        let painted = wait::until(
            "the backdrop to dim the page and the strip",
            || async move {
                let backdrop = pixel(page, backdrop_x, 600.0).await;
                Ok(backdrop != undimmed && pixel(page, strip_x, 600.0).await == backdrop)
            },
        )
        .await;
        if painted.is_err() {
            let backdrop = pixel(page, backdrop_x, 600.0).await;
            assert!(backdrop != undimmed, "the backdrop does not dim the page");
            panic!("the {gutter}px scrollbar strip is not dimmed by the backdrop");
        }

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "getComputedStyle(document.body).overflow !== 'hidden'",
            "the lock to lift",
        )
        .await
        .unwrap();
        assert_eq!(width().await, before, "the page shifted on close");
        let padding: String = page
            .evaluate("document.documentElement.style.paddingRight")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(padding, "8px", "the page's own padding after close");
        fixture.console.assert_clean("the scrollbar modal").unwrap();
        fixture.close().await.unwrap();
    });
}

/// One CSS px of the viewport as PNG bytes: equal bytes are equal pixels.
async fn pixel(page: &chromiumoxide::Page, x: f64, y: f64) -> Vec<u8> {
    use chromiumoxide::cdp::browser_protocol::page::{
        CaptureScreenshotFormat, CaptureScreenshotParams, Viewport as Clip,
    };
    let clip = Clip {
        x,
        y,
        width: 1.0,
        height: 1.0,
        scale: 1.0,
    };
    let params = CaptureScreenshotParams::builder()
        .format(CaptureScreenshotFormat::Png)
        .clip(clip)
        .build();
    page.screenshot(params).await.unwrap()
}

/// Todo 1306: the caller of `use_modal` unmounting while open settles the
/// opening as a dismissal and hands focus back.
#[test]
fn unmounting_the_owner_settles_and_returns_focus() {
    block_on(async {
        let fixture = Fixture::open("/modal/owner", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, "#leave").await.unwrap();
        pointer::click(page, "#leave").await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=dialog]') && document.querySelector('#result').textContent === 'None'",
            "the owner gone, its opening dismissed",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, TRIGGER, "the owner unmounting")
            .await
            .unwrap();
        fixture.console.assert_clean("the dropped owner").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
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
        })
        .await;
    });
}
