//! `Notifications`: an accessibility tree that changes over time (WCAG 4.1.3, `aria-live`).
//! `Suite` takes the baselines; the timeline is hand-written on a held clock (315 (c)).

use e2e::browser::block_on;
use e2e::clock::HELD_CLOCK;
use e2e::driver::{Driver, eventually};
use e2e::passes::{keyboard, pointer};
use e2e::suite::Step;
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport, js, wait};

const TRIGGER: &str = "#notify";

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

/// The shown baseline, polite then assertive (315 (b)); states accumulate on one page. The
/// 20x20 close button presses in a 24x24 box (505, 566), so plain `targets` holds.
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

/// Counts live-region insertions carrying `text`, each one announcement; a re-render that
/// re-sets it counts again. Also empty lists, none allowed at rest (447).
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

/// Records in `window.__longReal` (once set to a list) each real timeout or interval of 1 s
/// or more, with its caller. Installed before the held clock, so a held delay never reaches
/// it. A close that bypasses timers altogether (a frame loop reading the time) goes unseen.
const LONG_REAL_TIMERS: &str = r#"(() => {
    const set = window.setTimeout, every = window.setInterval;
    const note = (kind, ms) => {
        if (!window.__longReal || !(ms >= 1000)) return;
        const caller = (new Error().stack || '').split('\n').slice(3, 6).join(' <- ');
        window.__longReal.push(`${kind}(${ms}) from ${caller}`);
    };
    window.setTimeout = function (fn, ms, ...args) { note('setTimeout', ms); return set.call(window, fn, ms, ...args); };
    window.setInterval = function (fn, ms, ...args) { note('setInterval', ms); return every.call(window, fn, ms, ...args); };
    return true;
})()"#;

/// The timeline (315 (a), (c)): appears, is announced once without taking focus, closes
/// when the test fires the held clock. No step waits on a duration.
#[test]
fn a_timed_notification_appears_is_announced_once_and_closes_itself() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let at = viewport.name();
            let fixture = Fixture::open("/notifications", viewport).await.unwrap();
            let page = &fixture.page;
            let item = format!(
                "[...document.querySelectorAll('[aria-live] li')]\
                 .filter(li => li.textContent.includes({}))",
                serde_json::to_string(TIMED_MESSAGE).unwrap()
            );

            let _: bool = js(page, LONG_REAL_TIMERS).await;
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
            // The probe sees a real timer of its own through the held clock's wrapper.
            let seen: usize = js(
                page,
                "(window.__longReal = [], clearTimeout(setTimeout(() => {}, 1001)), \
                 window.__longReal.splice(0).length)",
            )
            .await;
            assert_eq!(
                seen, 1,
                "at {at}: the long real timer probe saw {seen} timers"
            );
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

            // Armed. A timeout here means the timer no longer goes through
            // `window.setTimeout`, and the fire below would prove nothing.
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

            // Control on the held clock: no long real timer was armed since the press, so
            // none can close it behind the held one (todo 1638, was a 4.8 s real wait).
            let long_real: Vec<String> = js(page, "window.__longReal").await;
            assert!(
                long_real.is_empty(),
                "at {at}: real timers of 1 s or more armed past the held clock, so it does \
                 not drive everything that could close the notification: {long_real:#?}"
            );

            // Announced once, without taking focus (WCAG 4.1.3).
            e2e::passes::focus::assert_focused(page, TIMED_TRIGGER, "showing a notification")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            let announced: usize = js(page, "window.__announced").await;
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
                 .some(el => el.textContent.trim().length > 0)",
            )
            .await;
            assert!(
                !spoken,
                "at {at}: a live region still speaks after the removal"
            );
            let empty: usize = js(page, EMPTY_LISTS).await;
            assert_eq!(
                empty, 0,
                "at {at}: {empty} empty list(s) left after the removal"
            );
            let announced: usize = js(page, "window.__announced").await;
            assert_eq!(announced, 1, "at {at}: closing announced the text again");
            e2e::passes::focus::assert_focused(page, TIMED_TRIGGER, "a notification closing")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));

            fixture
                .console
                .assert_clean(&format!("the notification timeline at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// Todo 4000: `pause_scope: Hovered` pauses only the notification under the pointer or holding
/// focus, and `z_index` sets the stacks' layer.
#[test]
fn a_hovered_scope_pauses_only_that_notification() {
    const FIRST: &str = "[aria-live] li:first-child";

    block_on(async {
        let fixture = Fixture::open("/notifications-pause-hovered", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        e2e::clock::hold(page, &[AUTO_CLOSE_MS]).await.unwrap();
        let armed = async |count: usize, what: &str| {
            e2e::clock::until_armed(page, AUTO_CLOSE_MS, count, what)
                .await
                .unwrap();
        };

        pointer::click(page, TIMED_TRIGGER).await.unwrap();
        wait::for_visible(page, FIRST).await.unwrap();
        armed(2, "both auto-closes to arm").await;
        let layer: String = js(
            page,
            "getComputedStyle(document.querySelector('[aria-live]').parentElement).zIndex",
        )
        .await;
        assert_eq!(layer, "1234", "the host's z_index on its stack");

        pointer::hover(page, FIRST).await.unwrap();
        armed(1, "the pointer to pause the hovered one only").await;
        pointer::hover(page, TIMED_TRIGGER).await.unwrap();
        armed(2, "the pointer leaving to resume it").await;

        keyboard::tab_to(page, CLOSE, 5).await.unwrap();
        armed(1, "focus to pause the focused one only").await;

        fixture
            .console
            .assert_clean("pausing the hovered notification")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// WCAG 2.2.1: a timed notification under the pointer or holding focus is paused (no auto-close
/// armed) and gets its full time back on leaving; one removed under the pointer, which never
/// sees `mouseleave`, leaves the store unpaused (todo 1767).
#[test]
fn hover_and_focus_pause_a_timed_notification() {
    const ITEM: &str = "[aria-live] li";

    block_on(async {
        let fixture = Fixture::open("/notifications", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        e2e::clock::hold(page, &[AUTO_CLOSE_MS, EXIT_MS])
            .await
            .unwrap();
        let armed = async |count: usize, what: &str| {
            e2e::clock::until_armed(page, AUTO_CLOSE_MS, count, what)
                .await
                .unwrap();
        };

        pointer::click(page, TIMED_TRIGGER).await.unwrap();
        wait::for_visible(page, ITEM).await.unwrap();
        armed(1, "the auto-close to arm").await;

        pointer::hover(page, ITEM).await.unwrap();
        armed(0, "the pointer on the item to pause it").await;
        pointer::hover(page, TIMED_TRIGGER).await.unwrap();
        armed(1, "the pointer leaving to resume it").await;

        keyboard::tab_to(page, CLOSE, 5).await.unwrap();
        armed(0, "focus on the close button to pause it").await;
        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        e2e::passes::focus::assert_focused(page, TIMED_TRIGGER, "Shift+Tab out")
            .await
            .unwrap();
        armed(1, "focus leaving to resume it").await;

        // Closed under the pointer: the next one must arm with the pointer elsewhere.
        pointer::hover(page, CLOSE).await.unwrap();
        armed(0, "the pointer on the close button to pause it").await;
        pointer::click(page, CLOSE).await.unwrap();
        e2e::clock::until_armed(page, EXIT_MS, 1, "the exit to arm")
            .await
            .unwrap();
        e2e::clock::fire(page, EXIT_MS).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!document.querySelector({ITEM:?})"),
            "the item to be removed",
        )
        .await
        .unwrap();
        pointer::click(page, TIMED_TRIGGER).await.unwrap();
        wait::for_visible(page, ITEM).await.unwrap();
        armed(
            1,
            "a new item to arm: the removed one left the store paused",
        )
        .await;

        fixture
            .console
            .assert_clean("pausing on hover and focus")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The live regions mount before they speak (some readers skip one mounting with its text),
/// with no empty list meanwhile: that reads "list, 0 items" (447).
#[test]
fn the_live_regions_are_mounted_and_silent_before_anything_happens() {
    block_on(async {
        let fixture = Fixture::open("/notifications", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let regions: usize = page
            .evaluate_bounded("document.querySelectorAll('[aria-live], [role=status]').length")
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
            .evaluate_bounded(
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

        // Queried by what the regions are, not by a role the host may not use
        // (`[role=status]` found nothing).
        let polite: usize = page
            .evaluate_bounded(
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
            .evaluate_bounded(EMPTY_LISTS)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(empty, 0, "{empty} empty list(s) in the page at rest");

        fixture.close().await.unwrap();
    });
}

/// Closing a focused notification moves focus to the next close button, else the previous,
/// else back where it came from (423), never to `<body>`.
#[test]
fn closing_one_hands_focus_on_and_back_out_of_an_empty_stack() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
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
        })
        .await;
    });
}

/// The snackbar's action sits before its close button in Tab order, takes no focus on showing,
/// and pressing it runs the handler once and closes the snackbar, focus back on the trigger.
#[test]
fn the_snackbar_action_is_tabbable_runs_its_handler_and_closes_it() {
    const ACTION: &str = "[aria-live] li [data-slot=actions] button";

    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let at = viewport.name();
            let fixture = Fixture::open("/notifications-snackbar", viewport)
                .await
                .unwrap();
            let page = &fixture.page;
            e2e::clock::hold(page, &[EXIT_MS]).await.unwrap();

            keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_visible(page, ACTION).await.unwrap();
            e2e::passes::focus::assert_focused(page, TRIGGER, "showing a snackbar")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));

            keyboard::press(page, keyboard::TAB).await.unwrap();
            e2e::passes::focus::assert_focused(page, ACTION, "Tab from the trigger")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            keyboard::press(page, keyboard::TAB).await.unwrap();
            e2e::passes::focus::assert_focused(page, CLOSE, "Tab past the action")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();

            keyboard::press(page, keyboard::ENTER).await.unwrap();
            e2e::clock::until_armed(page, EXIT_MS, 1, "the snackbar to start leaving")
                .await
                .unwrap();
            e2e::clock::fire(page, EXIT_MS).await.unwrap();
            wait::for_js_true(
                page,
                "!document.querySelector('[aria-live] li')",
                "the snackbar to be removed",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));
            let undone: String = js(page, "document.querySelector('#undone').textContent").await;
            assert_eq!(undone, "1", "at {at}: Undo ran {undone} times");
            e2e::passes::focus::assert_focused(page, TRIGGER, "the snackbar closing")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));

            fixture
                .console
                .assert_clean(&format!("the snackbar action at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// A `placement` change leaves a shown notification in its stack, announced once; only a
/// new one goes to the new stack (577).
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

/// Todo 5069: under RTL an end stack sits off its edge by the offset, inside the host, as a
/// start one does; the physical translate pushed both past the edge.
#[test]
fn under_rtl_a_stack_keeps_its_offset_inside_the_host() {
    // The anchored edge's gap, and whether the stack fits the host, for the notification
    // with `text`. The stack is the live region's parent.
    const GAPS: &str = "(text) => {
        const li = [...document.querySelectorAll('[aria-live] li')]
            .find(li => li.textContent.includes(text));
        if (!li) return null;
        const host = li.closest('[data-notifications-host]').getBoundingClientRect();
        const stack = li.closest('[aria-live]').parentElement.getBoundingClientRect();
        return [stack.left - host.left, host.right - stack.right];
    }";

    block_on(async {
        let fixture = Fixture::open("/notifications-host", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let _: bool = js(page, "(document.documentElement.dir = 'rtl', true)").await;

        // Bottom end: the left edge under RTL.
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_js_true(page, &format!("!!({GAPS})('Message 1')"), "the end stack")
            .await
            .unwrap();
        let (left, right): (f64, f64) = js(page, format!("({GAPS})('Message 1')")).await;
        assert!(
            (4.0..40.0).contains(&left),
            "end stack {left}px off the left edge"
        );
        assert!(
            right > left,
            "end stack sits right of centre: {right}px from the right"
        );

        // Top start: the right edge under RTL.
        pointer::click(page, "#move").await.unwrap();
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_js_true(page, &format!("!!({GAPS})('Message 2')"), "the start stack")
            .await
            .unwrap();
        let (left, right): (f64, f64) = js(page, format!("({GAPS})('Message 2')")).await;
        assert!(
            (4.0..40.0).contains(&right),
            "start stack {right}px off the right edge"
        );
        assert!(
            left > right,
            "start stack sits left of centre: {left}px from the left"
        );

        fixture.console.assert_clean("RTL stacks").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A contained host unmounting with focus inside a notification sends focus
/// back where it came from, not to `<body>` (todo 577).
#[test]
fn a_contained_host_unmounting_hands_focus_back_out() {
    const DROP: &str = ".drop-host";

    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
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
                    js(page, "document.activeElement.outerHTML.slice(0, 80)").await;
                panic!("at {at}: {e}; focus is on {active}");
            }

            fixture
                .console
                .assert_clean(&format!("unmounting the host at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// The same with the opener inside the host: it goes too, so focus moves to
/// the focusable before the host (todo 589).
#[test]
fn a_contained_host_unmounting_with_its_opener_focuses_the_control_before_it() {
    const DROP: &str = ".drop-host";

    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let at = viewport.name();
            let fixture = Fixture::open("/notifications-host-inside", viewport)
                .await
                .unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            keyboard::tab_to(page, DROP, 5)
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            let landed = wait::for_js_true(
                page,
                &format!(
                    "!document.querySelector({DROP:?}) && \
                     document.activeElement === document.querySelector('#before')"
                ),
                "the host's unmount to focus the control before it",
            )
            .await;
            if let Err(e) = landed {
                let active: String =
                    js(page, "document.activeElement.outerHTML.slice(0, 80)").await;
                panic!("at {at}: {e}; focus is on {active}");
            }

            fixture
                .console
                .assert_clean(&format!("unmounting the host at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// `clear()` with focus inside a notification sends it back where it came
/// from, as closing the last one does (todo 440). Before, focus fell to `<body>`.
#[test]
fn clearing_with_focus_inside_hands_focus_back_out() {
    const CLEAR: &str = ".clear-all";

    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
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
        })
        .await;
    });
}

const F8: keyboard::Key = keyboard::Key {
    key: "F8",
    code: "F8",
    vk: 119,
    text: None,
};

/// Todo 2216: Android hears the key on another thread than the effect that registers the
/// host; F8 must still find it.
async fn f8_reaches_the_newest<D: Driver>(d: &mut D, _route: &str) -> anyhow::Result<()> {
    d.click(TRIGGER).await?;
    eventually(d, "two notifications", async |d| {
        Ok(d.evaluate("document.querySelectorAll('.clear-all').length")
            .await?
            == 2)
    })
    .await?;
    d.press(F8).await?;
    eventually(d, "F8 to focus the newest notification", async |d| {
        let probe =
            "document.activeElement?.closest('li')?.textContent.includes('Second') ?? false";
        Ok(d.evaluate(probe).await? == true)
    })
    .await
}

e2e::scenario!(
    f8_focuses_the_newest_notification_on_the_event_thread,
    "/notifications-clear",
    f8_reaches_the_newest,
    android_only("2216: the split event thread is Android's; the web test above covers F8")
);

/// Todo 575: F8 focuses the newest notification, and closing it returns focus to where F8
/// was pressed. The region names the key as its shortcut (todo 5072).
#[test]
fn f8_focuses_the_newest_notification() {
    const CLEAR: &str = ".clear-all";

    block_on(async {
        let fixture = Fixture::open("/notifications-clear", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let region: String = page
            .evaluate_bounded(
                "(r => r ? `${r.getAttribute('aria-label')} ${r.getAttribute('aria-keyshortcuts')}` : '')\
                 (document.querySelector('[role=region]'))",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(region, "Notifications F8");

        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelectorAll({CLEAR:?}).length === 2"),
            "two notifications",
        )
        .await
        .unwrap();

        keyboard::press(page, F8).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.matches('li .clear-all') && \
             document.activeElement.closest('li').textContent.includes('Second') && \
             document.activeElement.matches(':focus-visible')",
            "F8 to focus the newest notification's button, visibly",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelectorAll({CLEAR:?}).length === 0 && \
                 document.activeElement === document.querySelector({TRIGGER:?})"
            ),
            "closing to hand focus back where F8 was pressed",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("F8").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 5520: a mouse press on a toast, then F8 in, then a close with no key or press inside
/// returns focus with a ring: the press counted only until focus came by keyboard.
#[test]
fn f8_after_a_mouse_press_returns_focus_with_a_ring() {
    block_on(async {
        let fixture = Fixture::open("/notifications-clear", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_visible(page, "[aria-live] li").await.unwrap();
        // Its padding corner: on the toast, off its button.
        let [x, y]: [f64; 2] = js(
            page,
            "(r => [r.left + 4, r.top + 4])(document.querySelector('[aria-live] li').getBoundingClientRect())",
        )
        .await;
        pointer::click_at(page, pointer::Point { x, y })
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.matches('[aria-live] li') ?? false",
            "the press to focus the toast",
        )
        .await
        .unwrap();
        pointer::click(page, TRIGGER).await.unwrap();

        keyboard::press(page, F8).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.matches('li .clear-all') ?? false",
            "F8 to focus the newest notification",
        )
        .await
        .unwrap();
        // A programmatic close, as an auto-close: no keydown inside clears the flag.
        let _: bool = js(page, "(document.activeElement.click(), true)").await;
        wait::for_js_true(
            page,
            &format!(
                "!document.querySelector('[aria-live] li') && \
                 document.activeElement === document.querySelector({TRIGGER:?})"
            ),
            "the close to return focus to the trigger",
        )
        .await
        .unwrap();
        let ring: bool = js(page, "document.activeElement.matches(':focus-visible')").await;
        assert!(ring, "focus came back without a ring after F8");

        fixture.console.assert_clean("F8 after a press").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 670: with the app's host and a contained one both showing, F8 goes to
/// the newest notification across both, and only one host moves focus.
#[test]
fn f8_focuses_the_newest_notification_across_hosts() {
    block_on(async {
        let fixture = Fixture::open("/notifications-two-hosts", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let _: bool = js(
            page,
            "(document.addEventListener('focusin', e => { \
               if (e.target.closest('li')) window.__moves = (window.__moves ?? 0) + 1; \
             }, true), true)",
        )
        .await;

        for (clicks, newest) in [
            (&["#notify-contained", "#notify-app"][..], "app 1"),
            (&["#notify-contained"][..], "contained 2"),
        ] {
            for trigger in clicks {
                pointer::click(page, trigger).await.unwrap();
            }
            wait::for_js_true(
                page,
                &format!(
                    "[...document.querySelectorAll('li')].some(li => li.textContent.includes({newest:?}))"
                ),
                newest,
            )
            .await
            .unwrap();
            let _: bool = js(page, "(window.__moves = 0, true)").await;

            keyboard::press(page, F8).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "document.activeElement?.matches('li .inside') && \
                     document.activeElement.closest('li').textContent.includes({newest:?})"
                ),
                &format!("F8 to focus {newest}"),
            )
            .await
            .unwrap();
            // Settled: a second host acting would have moved focus again by now.
            crate::settle::painted(page).await.unwrap();
            let (moves, still): (u32, bool) = js(
                page,
                format!(
                    "[window.__moves, document.activeElement.closest('li')?.textContent.includes({newest:?}) ?? false]"
                ),
            )
            .await;
            assert_eq!(moves, 1, "F8 moved focus {moves} times for {newest}");
            assert!(still, "focus left {newest} after F8");
        }

        fixture.console.assert_clean("F8 across hosts").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 5441: a finger tap on a toast's body is no hover. Its compat `mouseenter` paused every
/// timer until a tap elsewhere.
#[test]
fn a_tap_on_a_timed_notification_leaves_it_armed() {
    const ITEM: &str = "[aria-live] li";

    block_on(async {
        let fixture = Fixture::open("/notifications", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        e2e::clock::hold(page, &[AUTO_CLOSE_MS, EXIT_MS])
            .await
            .unwrap();

        pointer::tap(page, TIMED_TRIGGER).await.unwrap();
        wait::for_visible(page, ITEM).await.unwrap();
        e2e::clock::until_armed(page, AUTO_CLOSE_MS, 1, "the auto-close to arm")
            .await
            .unwrap();
        pointer::tap(page, ITEM).await.unwrap();
        e2e::clock::settle(page).await.unwrap();
        let armed = e2e::clock::armed(page, AUTO_CLOSE_MS).await.unwrap();
        assert_eq!(armed, 1, "a tap on the message paused the auto-close");
        e2e::passes::focus::assert_focused(page, TIMED_TRIGGER, "a tap on the message")
            .await
            .unwrap();
        e2e::clock::fire(page, AUTO_CLOSE_MS).await.unwrap();
        e2e::clock::until_armed(page, EXIT_MS, 1, "the tapped notification to close on time")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("tapping a timed notification")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 5444: a pointer close still returns focus to the trigger (423), but leaves the page
/// where it is and shows no ring. A keyboard close is covered above.
#[test]
fn a_pointer_close_returns_focus_without_a_scroll_or_a_ring() {
    block_on(async {
        for (viewport, touch) in [(Viewport::Desktop, false), (Viewport::Mobile, true)] {
            let at = viewport.name();
            let fixture = Fixture::open("/notifications-snackbar", viewport)
                .await
                .unwrap();
            let page = &fixture.page;
            let press = async |selector: &str| match touch {
                true => pointer::tap(page, selector).await,
                false => pointer::click(page, selector).await,
            };

            press(TRIGGER).await.unwrap();
            wait::for_visible(page, CLOSE).await.unwrap();
            let scrolled: f64 = js(
                page,
                "(() => { const s = document.createElement('div'); s.style.height = '3000px'; \
                 document.body.append(s); const e = document.scrollingElement; \
                 e.scrollTop = 2000; return e.scrollTop; })()",
            )
            .await;
            assert!(scrolled > 0.0, "at {at}: the page did not scroll");

            press(CLOSE).await.unwrap();
            wait::for_js_true(
                page,
                "!document.querySelector('[aria-live] li')",
                "the snackbar to be removed",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));
            e2e::passes::focus::assert_focused(page, TRIGGER, "a pointer close")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            let (top, ring): (f64, bool) = js(
                page,
                "[document.scrollingElement.scrollTop, document.activeElement.matches(':focus-visible')]",
            )
            .await;
            assert_eq!(top, scrolled, "at {at}: the close scrolled the page");
            assert!(!ring, "at {at}: the returned focus shows a ring");

            fixture
                .console
                .assert_clean(&format!("a pointer close at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todos 5442, 5445: a viewport stack clears the notch, the status bar and the home indicator,
/// and a docked `NavigationBar` instead of the last when one publishes its height.
#[test]
fn a_viewport_stack_clears_the_safe_area_and_a_navigation_bar() {
    use chromiumoxide::cdp::browser_protocol::emulation::{
        SafeAreaInsets, SetSafeAreaInsetsOverrideParams,
    };
    const ITEM: &str = "[aria-live] li";

    block_on(async {
        let fixture = Fixture::open("/notifications", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        let insets = SafeAreaInsets {
            top: Some(47),
            bottom: Some(34),
            left: Some(20),
            right: Some(30),
            ..Default::default()
        };
        page.execute_bounded(SetSafeAreaInsetsOverrideParams::new(insets))
            .await
            .unwrap();

        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_visible(page, ITEM).await.unwrap();
        // The entry slides in: measure where it lands.
        e2e::clock::finish_animations(page, std::time::Duration::from_secs(1))
            .await
            .unwrap();
        let rect = format!(
            "(() => {{ const r = document.querySelector({ITEM:?}).getBoundingClientRect(); \
             return [r.top, r.bottom, r.left, r.right, innerHeight, \
             document.documentElement.clientWidth]; }})()"
        );
        let [top, bottom, left, right, height, width]: [f64; 6] = js(page, &rect).await;
        let bottom_stack = top > height / 2.0;
        // Exact: a fixed `Float` clearing the inset as well would double it (5464).
        match bottom_stack {
            true => assert!(
                (bottom - (height - 46.0)).abs() < 1.0,
                "bottom {bottom} of {height}"
            ),
            false => assert!((top - 59.0).abs() < 1.0, "top {top}"),
        }
        assert!(
            left >= 20.0 - 1.0 && right <= width - 30.0 + 1.0,
            "{left}..{right} of {width}"
        );

        if bottom_stack {
            let _: bool = js(
                page,
                "(document.documentElement.style.setProperty('--lsx-navigation-bar-height', '100px'), true)",
            )
            .await;
            let [_, bottom, ..]: [f64; 6] = js(page, &rect).await;
            assert!(
                bottom <= height - 112.0 + 1.0,
                "bottom {bottom} over a 100px bar"
            );
        }

        fixture
            .console
            .assert_clean("the stack in a safe area")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
