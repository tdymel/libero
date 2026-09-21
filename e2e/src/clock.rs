//! A clock the test holds, for the timers it names and no others.

/// Install with `(HELD_CLOCK)([ms, ...])`; timers with those delays wait for
/// `window.__heldClock.armed(ms)` / `.fire(ms)`, all others run on the real clock.
/// Not virtual time: that stops rAF and dioxus effects too, so the arm/fire order is lost.
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
