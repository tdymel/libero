//! `use_tour`: a hole around each step's target and a modal card beside it. Focus moves into
//! the card, the arrows step once each, Escape and Back end it and focus returns.

use anyhow::{Result, bail};
use e2e::archetypes::{self, Overlay};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, js, passes::keyboard, wait};

use crate::modal;

pub const TRIGGER: &str = "#start-tour";
const CARD: &str = "[data-lsx-tour] [data-slot=card]";
const HIGHLIGHT: &str = "[data-lsx-tour] > [data-slot=highlight]";
const PROGRESS: &str = "[data-lsx-tour] [data-slot=progress]";
const NEXT: &str = "[data-lsx-tour] [data-slot=next]";
/// The fixture writes how the tour last ended: `finished`, or `closed at <index>`.
const ENDED: &str = "#ended";
/// `TourDefaults::padding`.
const PADDING: f64 = 6.0;

async fn open<D: Driver>(d: &mut D) -> Result<()> {
    d.click(TRIGGER).await?;
    eventually_focused(d, CARD, "starting the tour").await
}

/// The hole is `target`'s box grown by the padding on every side.
async fn hole_on<D: Driver>(d: &mut D, target: &str, during: &str) -> Result<()> {
    let settled = eventually(d, &format!("the hole around {target}"), async |d| {
        let (t, h) = (d.rect(target).await?, d.rect(HIGHLIGHT).await?);
        let near = |a: f64, b: f64| (a - b).abs() <= 1.0;
        Ok(near(h.x, t.x - PADDING)
            && near(h.y, t.y - PADDING)
            && near(h.width, t.width + 2.0 * PADDING)
            && near(h.height, t.height + 2.0 * PADDING))
    })
    .await;
    if settled.is_err() {
        bail!(
            "{:?}: after {during}, the hole is {:?}, {target} is {:?}",
            d.platform(),
            d.rect(HIGHLIGHT).await?,
            d.rect(target).await?
        );
    }
    Ok(())
}

async fn focuses_the_card<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    eventually_text(d, PROGRESS, "1 of 3", "starting the tour").await
}

e2e::scenario!(starting_a_tour_focuses_its_card, "/tour", focuses_the_card);

async fn pads_the_target<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    hole_on(d, "#first", "starting the tour").await?;
    // A hole with size draws its own edge (1.4.11).
    let ringed = format!("{HIGHLIGHT}[data-ringed]");
    eventually(d, "the hole's ring", async |d| d.exists(&ringed).await).await
}

e2e::scenario!(
    the_hole_is_the_target_plus_padding,
    "/tour",
    pads_the_target
);

async fn next_moves_the_hole<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    d.click(NEXT).await?;
    eventually_text(d, PROGRESS, "2 of 3", "Next").await?;
    hole_on(d, "#second", "Next").await?;
    eventually_focused(d, CARD, "Next").await
}

e2e::scenario!(
    next_moves_the_hole_to_the_next_target,
    "/tour",
    next_moves_the_hole
);

/// The card's and the trap's keydown both bubble: one press must be one step.
async fn one_arrow_one_step<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(d, "ArrowRight to leave the first step", async |d| {
        Ok(d.text(PROGRESS).await? != "1 of 3")
    })
    .await?;
    let progress = d.text(PROGRESS).await?;
    if progress != "2 of 3" {
        bail!("{:?}: one ArrowRight went to {progress:?}", d.platform());
    }
    d.press(keyboard::ARROW_LEFT).await?;
    eventually_text(d, PROGRESS, "1 of 3", "ArrowLeft").await
}

e2e::scenario!(an_arrow_steps_exactly_once, "/tour", one_arrow_one_step);

async fn escape_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    d.press(keyboard::ESCAPE).await?;
    modal::closed_with_focus_on(d, TRIGGER, "Escape").await?;
    eventually_text(d, ENDED, "closed at 0", "Escape").await
}

e2e::scenario!(
    escape_ends_the_tour_and_returns_focus,
    "/tour",
    escape_closes
);

async fn done_finishes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    for progress in ["2 of 3", "3 of 3"] {
        d.click(NEXT).await?;
        eventually_text(d, PROGRESS, progress, "Next").await?;
    }
    d.click(NEXT).await?;
    modal::closed_with_focus_on(d, TRIGGER, "Done").await?;
    eventually_text(d, ENDED, "finished", "Done").await
}

e2e::scenario!(done_on_the_last_step_finishes, "/tour", done_finishes);

async fn traps_tab<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    modal::tab_stays_inside(d, TRIGGER).await
}

e2e::scenario!(
    tab_stays_in_the_card,
    "/tour",
    traps_tab,
    android: skip("959: Tab leaves a trap for the body on the WebView"),
    desktop: skip("959: Shift+Tab from the first control leaves a trap on the WebView")
);

/// The mask covers the hole too: the highlighted element takes no press.
async fn target_is_inert<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    hole_on(d, "#first", "starting the tour").await?;
    let target = d.rect("#first").await?;
    d.click_at(
        target.x + target.width / 2.0,
        target.y + target.height / 2.0,
    )
    .await?;
    d.settle().await?;
    if !d.exists(CARD).await? {
        bail!("{:?}: a press on the target ended the tour", d.platform());
    }
    eventually_text(d, PROGRESS, "1 of 3", "a press on the target").await
}

e2e::scenario!(a_press_on_the_target_does_nothing, "/tour", target_is_inert);

/// Android's Back ends the tour, and the app stays (1289).
async fn back_closes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    archetypes::back_closes(d, TRIGGER, async |d| d.exists(CARD).await).await
}

e2e::scenario!(
    android_back_ends_a_tour,
    "/tour",
    back_closes,
    android_only("1275: no Back key off Android")
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("tour", "/tour")
        .focusable(TRIGGER)
        .contrast_covers(CARD)
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ENTER)],
            CARD,
        )
        .run();
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/tour", viewport).await.unwrap();
            Overlay {
                trigger: TRIGGER,
                panel: CARD,
                traps_focus: true,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));
            fixture
                .console
                .assert_clean(&format!("the tour contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// Under `dir="rtl"`, ArrowLeft is forward.
#[test]
fn arrow_left_goes_forward_in_rtl() {
    block_on(async {
        let fixture = Fixture::open("/tour", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.evaluate("document.documentElement.dir = 'rtl'")
            .await
            .unwrap();
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, CARD).await.unwrap();
        e2e::passes::focus::wait_for_focus(page, CARD, "starting the tour")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({PROGRESS:?})?.textContent === '2 of 3'"),
            "ArrowLeft under rtl",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// JS: whether the hole is `target`'s box plus the padding.
fn hole_js(target: &str) -> String {
    format!(
        "(() => {{ const t = document.querySelector({target:?}).getBoundingClientRect(); \
         const h = document.querySelector({HIGHLIGHT:?}).getBoundingClientRect(); \
         const near = (a, b) => Math.abs(a - b) <= 1; \
         return near(h.x, t.x - {PADDING}) && near(h.y, t.y - {PADDING}) \
           && near(h.width, t.width + {}) && near(h.height, t.height + {}); }})()",
        2.0 * PADDING,
        2.0 * PADDING
    )
}

/// Starts the tour on `route` from the keyboard and waits for the card's focus.
async fn start_on(route: &str) -> Fixture {
    let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
    let page = &fixture.page;
    keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
    keyboard::press(page, keyboard::ENTER).await.unwrap();
    wait::for_visible(page, CARD).await.unwrap();
    e2e::passes::focus::wait_for_focus(page, CARD, "starting the tour")
        .await
        .unwrap();
    fixture
}

/// A target below one scroller's fold and past another's side edge: both scroll (2222).
#[test]
fn a_nested_target_scrolls_into_view() {
    block_on(async {
        let fixture = start_on("/tour/nested").await;
        let page = &fixture.page;
        let inside = "(() => { const t = document.querySelector('#deep').getBoundingClientRect(); \
             const within = (id) => { const r = document.querySelector(id).getBoundingClientRect(); \
               return t.left >= r.left - 1 && t.right <= r.right + 1 \
                 && t.top >= r.top - 1 && t.bottom <= r.bottom + 1; }; \
             return within('#inner') && within('#outer'); })()";
        wait::for_js_true(page, inside, "#deep to scroll into both scrollers")
            .await
            .unwrap();
        wait::for_js_true(page, &hole_js("#deep"), "the hole to follow the scroll")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The target growing with nothing scrolled or resized moves the hole (2223).
#[test]
fn a_growing_target_grows_the_hole() {
    block_on(async {
        let fixture = start_on("/tour").await;
        let page = &fixture.page;
        wait::for_js_true(page, &hole_js("#first"), "the hole on #first")
            .await
            .unwrap();
        page.evaluate("document.querySelector('#first').style.width = '260px'")
            .await
            .unwrap();
        wait::for_js_true(page, &hole_js("#first"), "the hole to grow with #first")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A target that never mounts: after its tries the card shows in the middle, and the
/// debug build warns, so the console is allowed at close.
#[test]
fn a_missing_target_centres_the_card() {
    centres_the_card("/tour/missing");
}

/// A target mounted under `display: none` measures 0x0: it takes the same middle card.
#[test]
fn a_hidden_target_centres_the_card() {
    centres_the_card("/tour/hidden");
}

fn centres_the_card(route: &str) {
    block_on(async {
        let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, CARD).await.unwrap();
        e2e::passes::focus::wait_for_focus(page, CARD, "starting the tour")
            .await
            .unwrap();
        let centred = format!(
            "(() => {{ const r = document.querySelector({CARD:?}).getBoundingClientRect(); \
             return Math.abs(r.x + r.width / 2 - innerWidth / 2) <= 1 \
               && Math.abs(r.y + r.height / 2 - innerHeight / 2) <= 1; }})()"
        );
        wait::for_js_true(page, &centred, "the card to centre")
            .await
            .unwrap();
        let hole: String = js(
            page,
            &format!("document.querySelector({HIGHLIGHT:?}).style.width"),
        )
        .await;
        assert_eq!(hole, "0px", "{route}: the target left a hole");
        fixture
            .close_allowing("the debug build warns of the missing target")
            .await
            .unwrap();
    });
}

/// A held ArrowRight steps once: its auto-repeats would run on to the last step and finish.
#[test]
fn a_held_arrow_steps_once() {
    block_on(async {
        let fixture = Fixture::open("/tour", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        e2e::passes::focus::wait_for_focus(page, CARD, "starting the tour")
            .await
            .unwrap();
        keyboard::hold(page, keyboard::ARROW_RIGHT, 0, 6)
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({PROGRESS:?})?.textContent !== '1 of 3'"),
            "the held ArrowRight to leave the first step",
        )
        .await
        .unwrap();
        let progress: String = js(
            page,
            &format!("document.querySelector({PROGRESS:?})?.textContent ?? 'closed'"),
        )
        .await;
        assert_eq!(
            progress, "2 of 3",
            "a held ArrowRight stepped more than once"
        );
        fixture.close().await.unwrap();
    });
}

/// At 320x256 CSS px, a 1280x1024 window at 400 % zoom, a long card stays inside the
/// viewport and scrolls to its Next button, centred or placed (1.4.10).
#[test]
fn a_long_card_scrolls_in_a_short_viewport() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
    block_on(async {
        let fixture = Fixture::open("/tour/long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(SetDeviceMetricsOverrideParams::new(320, 256, 1.0, false))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "innerWidth === 320 && innerHeight === 256",
            "the short viewport",
        )
        .await
        .unwrap();
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        for (step, progress) in [("centred", "1 of 2"), ("placed", "2 of 2")] {
            e2e::passes::focus::wait_for_focus(page, CARD, step)
                .await
                .unwrap();
            wait::for_js_true(
                page,
                &format!("document.querySelector({PROGRESS:?})?.textContent === '{progress}'"),
                step,
            )
            .await
            .unwrap();
            // Inside the viewport, taller inside than out, and Next in reach once scrolled down.
            let fits = format!(
                "(() => {{ const c = document.querySelector({CARD:?}); const r = c.getBoundingClientRect(); \
                 if (r.top < -0.5 || r.bottom > innerHeight + 0.5 || c.scrollHeight <= c.clientHeight) return false; \
                 c.scrollTop = c.scrollHeight; \
                 const n = document.querySelector({NEXT:?}).getBoundingClientRect(); \
                 return n.top >= r.top - 0.5 && n.bottom <= r.bottom + 0.5 && n.bottom <= innerHeight + 0.5; }})()"
            );
            if let Err(e) =
                wait::for_js_true(page, &fits, &format!("the {step} card to fit and scroll")).await
            {
                let seen: String = js(
                    page,
                    &format!(
                        "(() => {{ const c = document.querySelector({CARD:?}); const r = c.getBoundingClientRect(); \
                         const n = document.querySelector({NEXT:?}).getBoundingClientRect(); \
                         return JSON.stringify({{ top: r.top, bottom: r.bottom, scroll: c.scrollHeight, client: c.clientHeight, \
                         next: [n.top, n.bottom], max: getComputedStyle(c).maxHeight }}); }})()"
                    ),
                )
                .await;
                panic!("{step}: {e}; the card {seen}");
            }
            if step == "centred" {
                keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
            }
        }
        fixture.close().await.unwrap();
    });
}
