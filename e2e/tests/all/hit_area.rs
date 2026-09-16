//! Todos 505, 566, 495: an `ActionIcon` under 24px takes presses in an
//! invisible 24x24 box round it, its drawn box unchanged, and a slot button
//! never grows its field's frame.

use e2e::browser::block_on;
use e2e::passes::{motion, pointer};
use e2e::{Fixture, Viewport, wait};
use serde::Deserialize;

/// `HIT(el)`: the points 11.5px out from `el`'s centre (sides and corners)
/// that do not land on `el`, plus its drawn size.
const PROBE: &str = "const HIT = (el) => {
    el.scrollIntoView({ block: 'center' });
    const r = el.getBoundingClientRect();
    const cx = r.x + r.width / 2, cy = r.y + r.height / 2;
    const misses = [];
    for (const dx of [-11.5, 0, 11.5]) for (const dy of [-11.5, 0, 11.5]) {
        const at = document.elementFromPoint(cx + dx, cy + dy);
        if (!at || at.closest('button, a') !== el) misses.push([dx, dy, at ? at.outerHTML.slice(0, 60) : null]);
    }
    return { width: r.width, height: r.height, misses };
};";

#[derive(Debug, Deserialize)]
struct Hit {
    width: f64,
    height: f64,
    misses: Vec<serde_json::Value>,
}

#[test]
fn small_action_icons_take_presses_in_a_24px_box() {
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        // Selector, drawn size.
        let cases = [
            ("#dialog button[aria-label=Close]", 20.0),
            ("#burger-xs", 16.0),
            ("#burger-sm", 22.0),
            ("#icon-xs", 16.0),
            ("#icon-sm", 20.0),
            ("[data-case=password-md] [data-slot=trailing] button", 20.0),
            ("[data-case=select-md] button[aria-label=Clear]", 20.0),
            ("[data-case=color-xs] [data-slot=trailing] button", 16.0),
            // Todo 645: the pill no longer clips its x's box.
            ("[data-case=tags] button[aria-label='Remove rust']", 14.4),
        ];
        for (selector, drawn) in cases {
            let hit: Hit = page
                .evaluate(format!(
                    "(() => {{ {PROBE} return HIT(document.querySelector({selector:?})); }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            println!("{selector}: {hit:?}");
            assert!(
                (hit.width - drawn).abs() < 0.5 && (hit.height - drawn).abs() < 0.5,
                "{selector} is drawn {}x{}, not {drawn}",
                hit.width,
                hit.height
            );
            assert!(
                hit.misses.is_empty(),
                "{selector}: points of its 24px box land elsewhere: {:?}",
                hit.misses
            );
        }

        fixture.console.assert_clean("probing hit areas").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The ripple grows as a `clip-path` inside the box: `overflow: hidden` on the
/// button would clip the hit area as well.
#[test]
fn the_ripple_stays_inside_an_unclipped_button() {
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, false).await.unwrap();

        let (x, y): (f64, f64) = page
            .evaluate(
                "(() => { const el = document.querySelector('#icon-sm'); \
                 el.scrollIntoView({ block: 'center' }); const r = el.getBoundingClientRect(); \
                 return [r.x + r.width / 2, r.y + r.height / 2]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let at = pointer::Point { x, y };
        pointer::drag(page, at, at, 1).await.unwrap();
        // A busy suite can start the animation late, so wait for the circle to grow.
        wait::for_js_true(
            page,
            "(() => { const el = document.querySelector('#icon-sm'); \
             const after = getComputedStyle(el, '::after'); \
             return getComputedStyle(el).overflow === 'visible' \
             && after.animationName.startsWith('lsx-ripple-clip-') \
             && after.clipPath.startsWith('circle(') && !after.clipPath.startsWith('circle(0%'); })()",
            "the ripple's clip circle to grow on an unclipped button",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("a ripple").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The two steppers sit 2px apart: each box stops at the gap's middle, so
/// neither takes the other's presses, and each is still 24px wide.
#[test]
fn the_stepper_boxes_meet_without_overlapping() {
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        for size in ["xs", "md", "xl"] {
            let report: Vec<(String, bool)> = page
                .evaluate(format!(
                    "(() => {{
                        const f = document.querySelector('[data-case=number-{size}]');
                        f.scrollIntoView({{ block: 'center' }});
                        const minus = f.querySelector('button[aria-label=Decrease]');
                        const plus = f.querySelector('button[aria-label=Increase]');
                        const m = minus.getBoundingClientRect(), p = plus.getBoundingClientRect();
                        const mid = (m.right + p.left) / 2, cy = m.y + m.height / 2;
                        const on = (x, y, el) => document.elementFromPoint(x, y)?.closest('button') === el;
                        // Whole pixels: Chromium hit-tests a half pixel on the boundary as the next one.
                        return [
                            ['minus 1px left of the middle', on(mid - 1, cy, minus)],
                            ['plus 1px right of the middle', on(mid + 1, cy, plus)],
                            ['minus 23px left of the middle', on(mid - 23, cy, minus)],
                            ['plus 23px right of the middle', on(mid + 23, cy, plus)],
                            ['minus 11.5px above', on(m.x + m.width / 2, cy - 11.5, minus)],
                            ['plus 11.5px below', on(p.x + p.width / 2, cy + 11.5, plus)],
                        ];
                    }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            for (what, ok) in report {
                assert!(ok, "{size}: {what}");
            }
        }

        fixture.console.assert_clean("stepper hit areas").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 495: at one size, a field with a slot button is as tall as a bare one.
#[test]
fn a_slot_button_does_not_grow_the_frame() {
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        for size in ["xs", "md", "xl"] {
            let heights: Vec<(String, f64)> = page
                .evaluate(format!(
                    "[...document.querySelectorAll('[data-size={size}] [data-case]')].map(c =>
                        [c.dataset.case, c.querySelector('[data-frame]').getBoundingClientRect().height])"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            println!("{size}: {heights:?}");
            let text = heights[0].1;
            for (case, height) in &heights {
                assert!(
                    (height - text).abs() < 0.5,
                    "{case} is {height}px tall, the bare text field {text}px"
                );
            }
        }

        fixture.console.assert_clean("frame heights").unwrap();
        fixture.close().await.unwrap();
    });
}
