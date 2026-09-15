//! `Notifications`: the component the framework was not built for.
//!
//! Added deliberately to find the edge. Every other unit is a static page that
//! `Suite` opens, measures and snapshots. A notification exists only after an
//! action and removes itself on a timer, so its accessibility tree is a
//! function of *time* - which is the one axis none of the machinery has.
//!
//! ## What this is checked against
//!
//! - **WCAG 4.1.3 Status Messages** - a status is presented to assistive
//!   technology without taking focus. That second half is the assertion that
//!   matters and the one a sighted check cannot make.
//! - **WAI-ARIA `aria-live`** - politeness, and a region that is mounted before
//!   it has anything to say.
//!   <https://www.w3.org/TR/wai-aria-1.2/#aria-live>
//!
//! ## Where the framework did not reach, and what was done about it
//!
//! 1. **`Suite` cannot express "do this, then measure over time".** Its `state`
//!    steps reach a state and measure it, which is enough for the two AX
//!    baselines below, so those go through `Suite`. The timeline (appears, is
//!    announced once, disappears on auto-close) is hand-written in
//!    [`a_timed_notification_appears_is_announced_once_and_closes_itself`].
//!    That is the right answer rather than a gap to close: bending `Suite` into
//!    a timeline runner would make it worse at the twenty static components it
//!    serves well (todo 315 (c)).
//! 2. **Auto-close is tested by holding the clock, not by sleeping.** See
//!    [`HELD_CLOCK`] for how, and why CDP's virtual time was not used.

use e2e::browser::block_on;
use e2e::clock::HELD_CLOCK;
use e2e::passes::{keyboard, pointer, target_size};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const TRIGGER: &str = "#notify";
const MESSAGE: &str = "Saved to your library";

/// A shown notification's close button, the one undersized control here.
const CLOSE: &str = "[aria-live] li [data-slot=close]";

const ASSERTIVE_TRIGGER: &str = "#notify-assertive";
const TIMED_TRIGGER: &str = "#notify-timed";
const TIMED_MESSAGE: &str = "Draft saved";
/// The fixture's `TIMED_AUTO_CLOSE_MS`. A delay nothing else schedules, so the
/// held clock can take this one timer and no other.
const AUTO_CLOSE_MS: u32 = 4321;
/// `theme.notifications.transition_duration`: the exit, after which the item
/// is removed.
const EXIT_MS: u32 = 200;

/// The shown state's baseline, polite and then assertive (todo 315 (b)).
///
/// The states accumulate on one page, so the "assertive" tree holds both: the
/// polite one in its region and the assertive one in the other. The resting
/// tree is the empty case, the 18 silent regions with no list in them.
///
/// `targets`: the close button is drawn 20x20 but takes presses in a 24x24 box
/// (todos 505, 566), so it meets WCAG 2.5.8 outright. The click handler is on
/// that button, and the card around it dismisses nothing.
#[test]
fn it_meets_the_baseline() {
    Suite::new("notifications", "/notifications")
        .focusable(TRIGGER)
        .targets(CLOSE)
        .state("polite", &[Step::Click(TRIGGER)], "[aria-live=polite] li")
        .state(
            "assertive",
            &[Step::Click(ASSERTIVE_TRIGGER)],
            "[aria-live=assertive] li",
        )
        .run();
}

/// Counts what the live regions say, from before anything is shown.
///
/// A screen reader announces an insertion into a live region, so an insertion
/// that carries `text` is one announcement. A re-render that put the item
/// back, or re-set its text, would count again.
/// Lists with no item in them: none may sit in the page at rest (todo 447).
const EMPTY_LISTS: &str = "[...document.querySelectorAll('ul, ol, [role=list]')]\
     .filter(list => !list.querySelector('li, [role=listitem]')).length";

const ANNOUNCEMENTS: &str = r#"(text) => {
    window.__announced = 0;
    const count = (node) => (node.textContent || '').includes(text);
    const observer = new MutationObserver((records) => {
        for (const r of records) {
            if (r.type === 'characterData' && count(r.target)) window.__announced++;
            for (const node of r.addedNodes) if (count(node)) window.__announced++;
        }
    });
    for (const region of document.querySelectorAll('[aria-live]')) {
        observer.observe(region, { childList: true, subtree: true, characterData: true });
    }
    return document.querySelectorAll('[aria-live]').length;
}"#;

async fn js<T: serde::de::DeserializeOwned>(page: &chromiumoxide::Page, expression: String) -> T {
    page.evaluate(expression.as_str())
        .await
        .unwrap_or_else(|e| panic!("evaluate {expression}: {e}"))
        .into_value()
        .unwrap_or_else(|e| panic!("read {expression}: {e}"))
}

/// The timeline `Suite` cannot express (todo 315 (a) and (c)): the notification
/// appears, is announced once and without taking focus, and closes itself when
/// its auto-close timer fires. Every step waits on a state and none on a
/// duration. The clock moves only when the test fires it.
#[test]
fn a_timed_notification_appears_is_announced_once_and_closes_itself() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/notifications", viewport).await.unwrap();
            let page = &fixture.page;
            let item = format!(
                "[...document.querySelectorAll('[aria-live] li')]\
                 .filter(li => li.textContent.includes({}))",
                serde_json::to_string(TIMED_MESSAGE).unwrap()
            );

            let _: bool = js(
                page,
                format!("(({HELD_CLOCK})([{AUTO_CLOSE_MS}, {EXIT_MS}]), true)"),
            )
            .await;
            let regions: usize = js(
                page,
                format!(
                    "({ANNOUNCEMENTS})({})",
                    serde_json::to_string(TIMED_MESSAGE).unwrap()
                ),
            )
            .await;
            assert!(regions > 0, "at {at}: no live region to observe");

            // Appears.
            keyboard::tab_to(page, TIMED_TRIGGER, 10).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_js_true(
                page,
                &format!("{item}.length === 1"),
                "the timed notification",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));
            let politeness: String = js(
                page,
                format!("{item}[0].closest('[aria-live]').getAttribute('aria-live')"),
            )
            .await;
            assert_eq!(
                politeness, "polite",
                "at {at}: the notification is in the wrong region"
            );

            // Armed. If this times out while the item is shown, either the
            // library no longer arms a timer or its timer no longer goes
            // through `window.setTimeout`. Either way the clock holds nothing,
            // and the fire below would prove nothing.
            wait::for_js_true(
                page,
                &format!("window.__heldClock.armed({AUTO_CLOSE_MS}) === 1"),
                "the held clock to hold the notification's auto-close timer",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));
            let exit_armed: usize = js(page, format!("window.__heldClock.armed({EXIT_MS})")).await;
            assert_eq!(
                exit_armed, 0,
                "at {at}: the exit is armed before the auto-close fired"
            );

            // The control on the clock itself (Ted, 2026-09-20). If libero's
            // timer went round the wrapper - a bound `setTimeout`, a worker, a
            // future `TimerApi` arm - the real delay would still be running
            // underneath, and every assertion after the fire would be about a
            // notification that was closing on its own. So once, at the first
            // viewport, the test spends the real delay and requires the
            // notification to be untouched by it. This costs the suite ~4.5 s;
            // the property is about the timer path, not about the viewport.
            if viewport == Viewport::ALL[0] {
                tokio::time::sleep(std::time::Duration::from_millis(
                    u64::from(AUTO_CLOSE_MS) + 500,
                ))
                .await;
                let shown: usize = js(page, format!("{item}.length")).await;
                assert_eq!(
                    shown, 1,
                    "the notification closed after its real {AUTO_CLOSE_MS}ms, so its timer \
                     did not go through the held clock and nothing here is being driven"
                );
                let still_armed: usize =
                    js(page, format!("window.__heldClock.armed({AUTO_CLOSE_MS})")).await;
                assert_eq!(
                    still_armed, 1,
                    "at {at}: the held auto-close timer vanished"
                );
            }

            // Announced once, without taking focus (WCAG 4.1.3).
            e2e::passes::focus::assert_focused(page, TIMED_TRIGGER, "showing a notification")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            let announced: usize = js(page, "window.__announced".into()).await;
            assert_eq!(
                announced, 1,
                "at {at}: the live regions said the text {announced} times"
            );

            // Auto-close fires: the item starts leaving and arms its exit.
            let fired: usize = js(page, format!("window.__heldClock.fire({AUTO_CLOSE_MS})")).await;
            assert_eq!(fired, 1, "at {at}: fired {fired} auto-close timers");
            wait::for_js_true(
                page,
                &format!("{item}.length === 1 && window.__heldClock.armed({EXIT_MS}) === 1"),
                "the notification to start leaving and arm its exit",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            // The exit fires: it is gone from the DOM, so from the AX tree too.
            let fired: usize = js(page, format!("window.__heldClock.fire({EXIT_MS})")).await;
            assert_eq!(fired, 1, "at {at}: fired {fired} exit timers");
            wait::for_js_true(
                page,
                &format!("{item}.length === 0"),
                "the notification to be removed",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            let spoken: bool = js(
                page,
                "[...document.querySelectorAll('[aria-live]')]\
                 .some(el => el.textContent.trim().length > 0)"
                    .into(),
            )
            .await;
            assert!(
                !spoken,
                "at {at}: a live region still speaks after the removal"
            );
            let empty: usize = js(page, EMPTY_LISTS.into()).await;
            assert_eq!(
                empty, 0,
                "at {at}: {empty} empty list(s) left after the removal"
            );
            let announced: usize = js(page, "window.__announced".into()).await;
            assert_eq!(announced, 1, "at {at}: closing announced the text again");
            e2e::passes::focus::assert_focused(page, TIMED_TRIGGER, "a notification closing")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));

            fixture
                .console
                .assert_clean(&format!("the notification timeline at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// The assertion that only a browser can make: the status is announced **and
/// focus does not move**.
///
/// WCAG 4.1.3 exists precisely because the tempting implementation - focus the
/// new thing so a screen reader reads it - is a worse experience than a live
/// region, and is indistinguishable from it in a screenshot.
#[test]
fn a_notification_is_announced_without_stealing_focus() {
    block_on(async {
        let fixture = Fixture::open("/notifications", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        // The notification arrives in a live region somewhere on the page.
        wait::for_js_true(
            page,
            &format!(
                "[...document.querySelectorAll('[aria-live], [role=status], [role=alert]')]\
                 .some(el => el.textContent.includes({}))",
                serde_json::to_string(MESSAGE).unwrap()
            ),
            "the notification to reach a live region",
        )
        .await
        .expect("a notification should be announced through a live region");

        // And focus is still on the button that asked for it.
        e2e::passes::focus::assert_focused(page, TRIGGER, "showing a notification")
            .await
            .expect("showing a notification must not move focus (WCAG 4.1.3)");

        fixture
            .console
            .assert_clean("showing a notification")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The live regions are mounted before they have anything to say, and hold no
/// empty list meanwhile.
///
/// A region that mounts together with its text is skipped by some screen
/// readers, which is why the library keeps them always-mounted and empty
/// (`codebase/components/notifications`). An assertion that a region exists
/// *while it is speaking* would hold just as well for the broken shape. Only
/// the list inside comes with the first notification: an empty one is read
/// as "list, 0 items" (todo 447).
#[test]
fn the_live_regions_are_mounted_and_silent_before_anything_happens() {
    block_on(async {
        let fixture = Fixture::open("/notifications", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let regions: usize = page
            .evaluate("document.querySelectorAll('[aria-live], [role=status]').length")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            regions > 0,
            "no live region is mounted before the first notification; a region created \
             together with its text is skipped by some screen readers"
        );

        let spoken: bool = page
            .evaluate(
                "[...document.querySelectorAll('[aria-live], [role=status]')]\
                 .some(el => el.textContent.trim().length > 0)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            !spoken,
            "a live region is speaking before anything happened"
        );

        // Queried by what the regions *are* rather than by a role the host may
        // not use: the first version of this assumed `[role=status]`, found
        // nothing, and reported it as a harness error rather than as a wrong
        // assumption.
        let polite: usize = page
            .evaluate(
                "[...document.querySelectorAll('[aria-live], [role=status]')]\
                 .filter(el => (el.getAttribute('aria-live') || \
                   (el.getAttribute('role') === 'status' ? 'polite' : '')) === 'polite').length",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            polite > 0,
            "a notification host should mount at least one polite live region, found none \
             among {regions}"
        );

        let empty: usize = page
            .evaluate(EMPTY_LISTS)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(empty, 0, "{empty} empty list(s) in the page at rest");

        fixture.close().await.unwrap();
    });
}

/// WCAG 2.5.8 for the close button, with the numbers written down (todo 366).
///
/// Drawn 20x20, it takes presses in an invisible 24x24 box (todos 505, 566),
/// so the press target measures 24x24 at both viewports and the baseline
/// declares it with plain `targets`.
///
/// **It asserts the count, not only the verdict.** The defect todo 366 was
/// filed for was not a wrong measurement, it was no measurement: the unit had
/// no target selector at all and reported green. A selector that matches
/// nothing gives that same green, so a rename of `[data-slot=close]` has to
/// turn this red rather than quietly reduce it to a no-op.
/// This pins the exact count.
#[test]
fn its_close_button_takes_presses_in_a_24px_box() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/notifications", viewport).await.unwrap();
            let page = &fixture.page;

            pointer::click(page, TRIGGER).await.unwrap();
            wait::for_visible(page, CLOSE).await.unwrap();

            let measured = target_size::measure_all(page, CLOSE).await.unwrap();
            assert_eq!(
                measured.len(),
                1,
                "at {}: {CLOSE} matched {} element(s), expected exactly one. A selector that \
                 matches nothing measures nothing and reports the same green as one that \
                 measures and passes, which is the defect this test exists for. Found: \
                 {measured:?}",
                viewport.name(),
                measured.len()
            );
            println!("{}: {measured:?}", viewport.name());
            target_size::assert_sizes(CLOSE, &measured).unwrap();

            fixture.close().await.unwrap();
        }
    });
}

/// Closing a focused notification hands focus to the next one's close button,
/// the previous one's if it was the last, and back where it came from once the
/// stack is empty (todo 423). Before, focus fell to `<body>`.
#[test]
fn closing_one_hands_focus_on_and_back_out_of_an_empty_stack() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/notifications", viewport).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
            for _ in 0..3 {
                keyboard::press(page, keyboard::ENTER).await.unwrap();
            }
            wait::for_js_true(
                page,
                &format!("document.querySelectorAll({CLOSE:?}).length === 3"),
                "three notifications",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));
            let _: bool = js(
                page,
                format!("(window.__closes = [...document.querySelectorAll({CLOSE:?})], true)"),
            )
            .await;

            // Entered from the last page control, into the first notification.
            keyboard::tab_to(page, TIMED_TRIGGER, 5).await.unwrap();
            keyboard::press(page, keyboard::TAB).await.unwrap();
            keyboard::press(page, keyboard::TAB).await.unwrap();

            for (closed, lands, on) in [
                (1, "window.__closes[2]", "the next close button"),
                (
                    2,
                    "window.__closes[0]",
                    "the previous close button, after the last",
                ),
                (
                    0,
                    &format!("document.querySelector({TIMED_TRIGGER:?})"),
                    "the control focus came from",
                ),
            ] {
                let focused: bool = js(
                    page,
                    format!("document.activeElement === window.__closes[{closed}]"),
                )
                .await;
                assert!(
                    focused,
                    "at {at}: close button {closed} does not hold focus"
                );
                keyboard::press(page, keyboard::ENTER).await.unwrap();
                wait::for_js_true(
                    page,
                    &format!("document.activeElement === {lands} && document.activeElement.matches(':focus-visible')"),
                    &format!("closing notification {closed} to focus {on}, visibly"),
                )
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            }

            fixture
                .console
                .assert_clean(&format!("closing notifications at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// A host whose `placement` changes leaves a shown notification in its stack:
/// the same element, announced once. Only a new one goes to the new stack.
/// Before, every shown one remounted in the new stack's region (todo 577).
#[test]
fn a_placement_change_leaves_shown_notifications_in_place() {
    block_on(async {
        let fixture = Fixture::open("/notifications-host", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let first = "[...document.querySelectorAll('[aria-live] li')]\
                     .find(li => li.textContent.includes('Message 1'))";
        let second = "[...document.querySelectorAll('[aria-live] li')]\
                      .find(li => li.textContent.includes('Message 2'))";

        let _: usize = js(page, format!("({ANNOUNCEMENTS})('Message 1')")).await;
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_js_true(page, &format!("!!{first}"), "the first notification")
            .await
            .unwrap();
        let _: bool = js(page, format!("(window.__first = {first}, true)")).await;

        pointer::click(page, "#move").await.unwrap();
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_js_true(page, &format!("!!{second}"), "the second notification")
            .await
            .unwrap();

        let (same, apart, announced): (bool, bool, usize) = js(
            page,
            format!(
                "[window.__first === {first}, \
                  window.__first.closest('[aria-live]')?.parentElement !== \
                  {second}.closest('[aria-live]').parentElement, \
                  window.__announced]"
            ),
        )
        .await;
        assert!(same, "the shown notification was remounted");
        assert!(
            apart,
            "the new notification did not go to the new placement"
        );
        assert_eq!(announced, 1, "the shown notification was announced again");

        fixture.console.assert_clean("moving the host").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A contained host unmounting with focus inside a notification sends focus
/// back where it came from, not to `<body>` (todo 577).
#[test]
fn a_contained_host_unmounting_hands_focus_back_out() {
    const DROP: &str = ".drop-host";

    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/notifications-host", viewport)
                .await
                .unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            keyboard::tab_to(page, DROP, 5)
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            // Tab entered the notification from `#move`, the control after `#notify`.
            let landed = wait::for_js_true(
                page,
                &format!(
                    "!document.querySelector({DROP:?}) && \
                     document.activeElement === document.querySelector('#move')"
                ),
                "the host's unmount to focus the control focus came from",
            )
            .await;
            if let Err(e) = landed {
                let active: String =
                    js(page, "document.activeElement.outerHTML.slice(0, 80)".into()).await;
                panic!("at {at}: {e}; focus is on {active}");
            }

            fixture
                .console
                .assert_clean(&format!("unmounting the host at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// `clear()` with focus inside a notification sends it back where it came
/// from, as closing the last one does (todo 440). Before, focus fell to `<body>`.
#[test]
fn clearing_with_focus_inside_hands_focus_back_out() {
    const CLEAR: &str = ".clear-all";

    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/notifications-clear", viewport)
                .await
                .unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_js_true(
                page,
                &format!("document.querySelectorAll({CLEAR:?}).length === 2"),
                "two notifications",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            keyboard::tab_to(page, CLEAR, 5)
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "document.querySelectorAll({CLEAR:?}).length === 0 && \
                     document.activeElement === document.querySelector({TRIGGER:?}) && \
                     document.activeElement.matches(':focus-visible')"
                ),
                "clearing to focus the control focus came from, visibly",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            fixture
                .console
                .assert_clean(&format!("clearing notifications at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
