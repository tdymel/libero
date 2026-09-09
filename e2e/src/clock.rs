//! A clock the test holds, for the timers it names and no others.

/// Install with `(HELD_CLOCK)([ms, ...])`, then `window.__heldClock.armed(ms)`
/// and `window.__heldClock.fire(ms)`.
///
/// It replaces `window.setTimeout` and `window.clearTimeout` on the page.
/// A call whose delay is in `delays` is kept in `held` under a negative id,
/// which a real timer id never is. Every other call goes to the real clock,
/// so dioxus, animations and anything else on the page run as they would.
/// A held timer runs when the test calls `fire(ms)`, and at no other time.
/// libero's web timer is `window.setTimeout` through web-sys, whose glue looks
/// the method up on the window at call time. So installing this after load
/// is enough.
///
/// **Why not `Emulation.setVirtualTimePolicy`.** Virtual time does not pick out
/// one timer. It stops the page's whole clock: `requestAnimationFrame`, the CSS
/// entry and exit animations, and the tasks dioxus uses to run its effects.
/// A component arms its timer inside one of those effects. So "advance
/// 4.4 s" does not state an order between "the effect armed the timer" and
/// "the timer fired". The policy is also experimental, and headless Chromium
/// supports it unevenly. A held timer has no such question. The test first
/// sees that it was armed (`armed`), then fires it, and each step is a
/// state it can wait on, not a length of time.
///
/// **Why not a fixture that exposes its timer.** libero's `timer()` is a static
/// with no seam a fixture could reach. Adding one is a public API change, for a
/// test.
pub const HELD_CLOCK: &str = r#"(delays) => {
    if (window.__heldClock) return;
    const realSet = window.setTimeout.bind(window);
    const realClear = window.clearTimeout.bind(window);
    const held = new Map();
    let next = -1;
    window.__heldClock = {
        armed: (ms) => [...held.values()].filter((t) => t.ms === ms).length,
        fire: (ms) => {
            const hits = [...held].filter(([, t]) => t.ms === ms);
            if (hits.length !== 1) return hits.length;
            const [id, t] = hits[0];
            held.delete(id);
            t.run();
            return 1;
        },
    };
    window.setTimeout = function (fn, ms, ...args) {
        if (delays.includes(ms) && typeof fn === 'function') {
            const id = next--;
            held.set(id, { ms, run: () => fn(...args) });
            return id;
        }
        return realSet(fn, ms, ...args);
    };
    window.clearTimeout = function (id) {
        if (!held.delete(id)) realClear(id);
    };
}"#;
