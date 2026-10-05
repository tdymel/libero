//! `Notifications`: an accessibility tree that changes over time (WCAG 4.1.3, `aria-live`).
//! `Suite` takes the baselines; the timeline is hand-written on a held clock (315 (c)).

use e2e::browser::block_on;
use e2e::clock::HELD_CLOCK;
use e2e::driver::{Driver, eventually};
use e2e::passes::{keyboard, pointer};
use e2e::suite::Step;
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

        // Queried by what the regions are, not by a role the host may not use
        // (`[role=status]` found nothing).
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
/// was pressed. The region names the key.
#[test]
fn f8_focuses_the_newest_notification() {
    const CLEAR: &str = ".clear-all";

    block_on(async {
        let fixture = Fixture::open("/notifications-clear", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let region: String = page
            .evaluate("document.querySelector('[role=region]')?.getAttribute('aria-label') ?? ''")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(region, "Notifications (F8)");

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
