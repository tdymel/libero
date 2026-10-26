//! Todos 505, 566, 495: an `ActionIcon` under 24px takes presses in an invisible 24x24
//! box, and a slot button never grows its field's frame.

use e2e::browser::block_on;
use e2e::passes::{keyboard, motion, pointer};
use e2e::wait::PageBounded;
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
            // Todo 1069: size words follow `Button`'s heights, 24px and up.
            ("#dialog button[aria-label=Close]", 30.0),
            ("#burger-xs", 16.0),
            ("#burger-sm", 22.0),
            ("#icon-xs", 16.0),
            ("#icon-sm", 20.0),
            // Todo 3077: a zoomable picture under 24px.
            ("#zoom-tiny", 16.0),
            // Todo 3130: a clickable swatch at the smallest size.
            ("#swatch-xs", 16.0),
            ("[data-case=password-md] [data-slot=trailing] button", 30.0),
            ("[data-case=select-md] button[aria-label=Clear]", 30.0),
            ("[data-case=color-xs] [data-slot=trailing] button", 24.0),
            // Todo 645: the pill no longer clips its x's box.
            ("[data-case=tags] button[aria-label='Remove rust']", 14.4),
        ];
        for (selector, drawn) in cases {
            let hit: Hit = page
                .evaluate_bounded(format!(
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

/// WCAG 2.5.8: a switch track under 24px tall takes presses 11.5px above and below its
/// centre, drawn size unchanged.
#[test]
fn a_small_switch_track_takes_presses_in_a_24px_box() {
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#switch-xs ~ [data-slot=track]")
            .await
            .unwrap();

        for (id, drawn) in [("switch-xs", 16.0), ("switch-sm", 18.0)] {
            let hit: Hit = page
                .evaluate_bounded(format!(
                    "(() => {{
                        const el = document.querySelector('#{id} ~ [data-slot=track]');
                        el.scrollIntoView({{ block: 'center' }});
                        const r = el.getBoundingClientRect();
                        const cx = r.x + r.width / 2, cy = r.y + r.height / 2;
                        const misses = [];
                        for (const dx of [-r.width / 2 + 1, 0, r.width / 2 - 1])
                            for (const dy of [-11.5, 0, 11.5]) {{
                                const at = document.elementFromPoint(cx + dx, cy + dy);
                                if (!at || at.closest('[data-slot=track]') !== el)
                                    misses.push([dx, dy, at ? at.outerHTML.slice(0, 60) : null]);
                            }}
                        return {{ width: r.width, height: r.height, misses }};
                    }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                (hit.height - drawn).abs() < 0.5,
                "#{id} track is drawn {}px tall, not {drawn}",
                hit.height
            );
            assert!(
                hit.misses.is_empty(),
                "#{id}: points of its 24px box land elsewhere: {:?}",
                hit.misses
            );
        }

        fixture
            .console
            .assert_clean("probing switch tracks")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2707: on a coarse pointer a small control takes presses 21px out from its centre,
/// its drawn size unchanged. A pagination control's sides are its neighbours' to share.
#[test]
fn a_finger_presses_small_controls_in_a_44px_box() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute_bounded(SetTouchEmulationEnabledParams::new(true))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(pointer: coarse)').matches",
            "a coarse pointer",
        )
        .await
        .unwrap();

        let cases = [
            ("#chip-filter", true),
            ("#chip-action", true),
            ("[data-case=checkbox] [data-slot=box]", true),
            ("[data-case=radio] [data-slot=circle]", true),
            ("[data-case=switch] [data-slot=track]", true),
            ("[data-case=pagination] [aria-current=page]", false),
            ("#coarse-icon-xs", true),
            ("#coarse-icon-sm", true),
            // Todo 3611: a length in rem is read as px.
            ("#coarse-icon-rem", true),
            ("#coarse-burger-xs", true),
            ("#dialog button[aria-label=Close]", true),
            // Todo 3589: the thumb's 24px square grows to 44px.
            ("[data-case=slider] [role=slider]", true),
        ];
        for (selector, sides) in cases {
            let hit: Hit = page
                .evaluate_bounded(format!(
                    "(() => {{
                        const el = document.querySelector({selector:?});
                        el.scrollIntoView({{ block: 'center' }});
                        const r = el.getBoundingClientRect();
                        const cx = r.x + r.width / 2, cy = r.y + r.height / 2;
                        const misses = [];
                        for (const dx of {sides} ? [-21, 0, 21] : [0]) for (const dy of [-21, 0, 21]) {{
                            const at = document.elementFromPoint(cx + dx, cy + dy);
                            if (!at || !el.contains(at)) misses.push([dx, dy, at ? at.outerHTML.slice(0, 60) : null]);
                        }}
                        return {{ width: r.width, height: r.height, misses }};
                    }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            println!("{selector}: {hit:?}");
            assert!(hit.height < 40.0, "{selector} grew its drawn size: {hit:?}");
            assert!(
                hit.misses.is_empty(),
                "{selector}: points of its 44px box land elsewhere: {:?}",
                hit.misses
            );
        }

        fixture.console.assert_clean("coarse hit areas").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4188: under a finger a chip's x takes its whole pill's height and 44px across,
/// and a tap at the pill's bottom edge removes that chip, not the one on the next row.
#[test]
fn a_finger_on_a_chip_x_never_removes_the_next_rows_chip() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute_bounded(SetTouchEmulationEnabledParams::new(true))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(pointer: coarse)').matches",
            "a coarse pointer",
        )
        .await
        .unwrap();

        for case in ["tag-rows", "pick-rows"] {
            let probe: serde_json::Value = page
                .evaluate_bounded(format!(
                    "(() => {{
                        const xs = [...document.querySelectorAll('[data-case={case}] button[aria-label^=Remove]')];
                        xs[0].scrollIntoView({{ block: 'center' }});
                        const misses = [], tops = [];
                        for (const el of xs) {{
                            const r = el.getBoundingClientRect();
                            const pill = el.closest('[data-slot=chip-trailing]').parentElement.getBoundingClientRect();
                            const cx = r.x + r.width / 2, cy = r.y + r.height / 2;
                            tops.push([pill.top, pill.height]);
                            const points = [[-21, 0], [21, 0]];
                            for (let y = pill.top + 1; y < pill.bottom; y += 1) points.push([0, y - cy]);
                            for (const [dx, dy] of points) {{
                                const at = document.elementFromPoint(cx + dx, cy + dy);
                                if (!at || !el.contains(at)) misses.push([el.ariaLabel, dx, dy, at ? at.outerHTML.slice(0, 60) : null]);
                            }}
                        }}
                        const first = xs[0].getBoundingClientRect();
                        const pill = xs[0].closest('[data-slot=chip-trailing]').parentElement.getBoundingClientRect();
                        return {{ misses, tops, tap: {{ x: first.x + first.width / 2, y: pill.bottom - 1 }} }};
                    }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            println!("{case}: {probe}");
            let tops = probe["tops"].as_array().unwrap();
            // One line per pill, one pill per row: else the x is not over the next row's.
            assert!(
                tops.len() == 3
                    && tops[0][0].as_f64() < tops[1][0].as_f64()
                    && tops.iter().all(|top| top[1].as_f64() < Some(32.0)),
                "{case}: the chips do not stack one per row: {tops:?}"
            );
            assert_eq!(
                probe["misses"].as_array().map(Vec::len),
                Some(0),
                "{case}: points of an x's box land elsewhere: {}",
                probe["misses"]
            );

            let tap = &probe["tap"];
            let at = pointer::Point {
                x: tap["x"].as_f64().unwrap(),
                y: tap["y"].as_f64().unwrap(),
            };
            pointer::tap_at(page, at).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "!document.querySelector('[data-case={case}] button[aria-label=\"Remove Kiwi 1\"]')
                        && !!document.querySelector('[data-case={case}] button[aria-label=\"Remove Kiwi 2\"]')"
                ),
                "the tapped chip gone and the next row's kept",
            )
            .await
            .unwrap();
        }

        fixture
            .console
            .assert_clean("chip rows under a finger")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3589: a `md` icon keeps its own box on a coarse pointer, so grouped icons do not
/// take each other's presses.
#[test]
fn a_medium_icon_in_a_group_keeps_its_own_box_under_a_finger() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute_bounded(SetTouchEmulationEnabledParams::new(true))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(pointer: coarse)').matches",
            "a coarse pointer",
        )
        .await
        .unwrap();

        let hit: Hit = page
            .evaluate_bounded(
                "(() => {
                    const first = document.querySelector('#group-first');
                    const second = document.querySelector('#group-second');
                    first.scrollIntoView({ block: 'center' });
                    const f = first.getBoundingClientRect(), s = second.getBoundingClientRect();
                    const misses = [];
                    const own = (el, x, y) => document.elementFromPoint(x, y)?.closest('button') === el;
                    // 3px inside the first icon's end, and 3px above and below the second.
                    if (!own(first, s.x - 3, s.y + s.height / 2)) misses.push('first end taken');
                    if (own(second, s.x + s.width / 2, s.y - 3)) misses.push('second grew up');
                    if (own(second, s.x + s.width / 2, s.bottom + 3)) misses.push('second grew down');
                    return { width: f.width, height: f.height, misses };
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(hit.misses.is_empty(), "{hit:?}");

        fixture.console.assert_clean("grouped md icons").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3691: a compact icon in a group takes a finger across its own width only, and keeps
/// the 44px height, so its box does not cover the neighbours' ends.
#[test]
fn a_compact_icon_in_a_group_keeps_its_width_under_a_finger() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute_bounded(SetTouchEmulationEnabledParams::new(true))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(pointer: coarse)').matches",
            "a coarse pointer",
        )
        .await
        .unwrap();

        let misses: Vec<String> = page
            .evaluate_bounded(
                "(() => {
                    const ids = ['first', 'second', 'third'];
                    const icons = ids.map(id => document.querySelector('#compact-' + id));
                    icons[0].scrollIntoView({ block: 'center' });
                    const own = (el, x, y) => document.elementFromPoint(x, y)?.closest('button') === el;
                    const misses = [];
                    icons.forEach((icon, i) => {
                        const r = icon.getBoundingClientRect(), y = r.y + r.height / 2;
                        if (!own(icon, r.x + 3, y)) misses.push(ids[i] + ' start taken');
                        if (!own(icon, r.right - 3, y)) misses.push(ids[i] + ' end taken');
                        if (!own(icon, r.x + r.width / 2, r.y - 5)) misses.push(ids[i] + ' lost its height');
                    });
                    const stacked = ['first', 'second'].map(id => document.querySelector('#stacked-' + id));
                    stacked.forEach((icon, i) => {
                        const r = icon.getBoundingClientRect(), x = r.x + r.width / 2;
                        if (!own(icon, x, r.y + 3)) misses.push('stacked ' + i + ' top taken');
                        if (!own(icon, x, r.bottom - 3)) misses.push('stacked ' + i + ' bottom taken');
                        if (!own(icon, r.x - 5, r.y + r.height / 2)) misses.push('stacked ' + i + ' lost its width');
                    });
                    return misses;
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(misses.is_empty(), "{misses:?}");

        fixture.console.assert_clean("grouped sm icons").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4187: on a phone, every point of a stacked choice's drawn control and of its 44px
/// height belongs to that row, and a tap on the drawn lower edge of row 1 picks row 1 only.
#[test]
fn a_finger_on_a_stacked_choice_never_lands_on_the_next_row() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        page.execute_bounded(SetTouchEmulationEnabledParams::new(true))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(pointer: coarse)').matches",
            "a coarse pointer",
        )
        .await
        .unwrap();

        for (case, drawn) in [
            ("radio", "circle"),
            ("checkbox", "box"),
            ("switch", "track"),
        ] {
            let group = format!("[data-case=stacked-{case}]");
            let probe: (Vec<String>, pointer::Point) = page
                .evaluate_bounded(format!(
                    "(() => {{
                        const rows = [...document.querySelectorAll('{group} [data-slot={drawn}]')].map(el => el.parentElement);
                        rows[0].scrollIntoView({{ block: 'center' }});
                        const misses = [];
                        rows.forEach((row, i) => {{
                            const r = row.querySelector('[data-slot={drawn}]').getBoundingClientRect();
                            const cx = r.x + r.width / 2, cy = r.y + r.height / 2;
                            for (const dy of [-21, 2 - r.height / 2, 0, r.height / 2 - 2, 21]) {{
                                const at = document.elementFromPoint(cx, cy + dy);
                                if (at?.closest('[data-slot={drawn}]')?.parentElement !== row) misses.push(`row ${{i}} dy ${{dy}}: ${{at?.outerHTML.slice(0, 60)}}`);
                            }}
                        }});
                        const r = rows[0].querySelector('[data-slot={drawn}]').getBoundingClientRect();
                        return [misses, {{ x: r.x + r.width / 2, y: r.bottom - 2 }}];
                    }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(probe.0.is_empty(), "{case}: {:?}", probe.0);

            pointer::tap_at(page, probe.1).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{
                        const inputs = [...document.querySelectorAll('{group} input')];
                        return inputs[0].checked && inputs.slice(1).every(input => !input.checked);
                    }})()"
                ),
                &format!("the first {case} alone picked"),
            )
            .await
            .unwrap();
        }

        fixture.console.assert_clean("stacked choices").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3589: the choice controls' transitions are off under reduced motion, on without it.
#[test]
fn the_choice_controls_skip_their_transitions_under_reduced_motion() {
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let parts = [
            "[data-case=switch] [data-slot=track]",
            "[data-case=switch] [data-slot=thumb]",
            "[data-case=checkbox] [data-slot=box]",
            "[data-case=radio] [data-slot=circle]",
            "[data-case=radio] [data-slot=dot]",
        ];
        let moving = format!(
            "{:?}.filter(s => getComputedStyle(document.querySelector(s)).transitionDuration === '0s')",
            parts
        );

        motion::set_reduced_motion(page, false).await.unwrap();
        let still: Vec<String> = page
            .evaluate_bounded(moving.clone())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            still.is_empty(),
            "no transition without the query: {still:?}"
        );

        motion::set_reduced_motion(page, true).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{moving}.length === {}", parts.len()),
            "every transition to be off under reduced motion",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("reduced motion").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 3090, 3107: a label-less radio or checkbox takes a mouse press 11.5px out from
/// its 18px drawn size.
#[test]
fn a_nameless_radio_and_checkbox_take_presses_in_a_24px_box() {
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        for selector in [
            "[data-case=radio] [data-slot=circle]",
            "[data-case=checkbox] [data-slot=box]",
        ] {
            let hit: Hit = page
                .evaluate_bounded(format!(
                    "(() => {{
                        const el = document.querySelector({selector:?});
                        el.scrollIntoView({{ block: 'center' }});
                        const r = el.getBoundingClientRect();
                        const cx = r.x + r.width / 2, cy = r.y + r.height / 2;
                        const misses = [];
                        for (const dx of [-11.5, 0, 11.5]) for (const dy of [-11.5, 0, 11.5]) {{
                            const at = document.elementFromPoint(cx + dx, cy + dy);
                            if (!at || !el.contains(at)) misses.push([dx, dy, at ? at.outerHTML.slice(0, 60) : null]);
                        }}
                        return {{ width: r.width, height: r.height, misses }};
                    }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(hit.height < 20.0, "{selector} grew its drawn size: {hit:?}");
            assert!(
                hit.misses.is_empty(),
                "{selector}: its 24px box lands elsewhere: {:?}",
                hit.misses
            );
        }

        fixture.console.assert_clean("a radio hit area").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3131: a table row's light box takes a mouse press 11.5px out from its drawn size,
/// and toggles the row's selection.
#[test]
fn a_table_row_checkbox_takes_presses_in_a_24px_box() {
    block_on(async {
        let fixture = Fixture::open("/table/select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let hit: Hit = page
            .evaluate_bounded(
                "(() => {
                    const el = document.querySelector('tbody tr:nth-child(2) [data-slot=control]');
                    el.scrollIntoView({ block: 'center' });
                    const r = el.getBoundingClientRect();
                    const cx = r.x + r.width / 2, cy = r.y + r.height / 2;
                    const misses = [];
                    for (const dx of [-11.5, 0, 11.5]) for (const dy of [-11.5, 0, 11.5]) {
                        const at = document.elementFromPoint(cx + dx, cy + dy);
                        if (!at || !el.contains(at)) misses.push([dx, dy, at ? at.outerHTML.slice(0, 60) : null]);
                    }
                    return { width: r.width, height: r.height, misses };
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(hit.height < 20.0, "the box grew its drawn size: {hit:?}");
        assert!(
            hit.misses.is_empty(),
            "its 24px box lands elsewhere: {:?}",
            hit.misses
        );

        let (x, y): (f64, f64) = page
            .evaluate_bounded(
                "(() => { const r = document.querySelector('tbody tr:nth-child(2) [data-slot=control]')\
                 .getBoundingClientRect(); return [r.x + r.width / 2 - 11.5, r.y + r.height / 2 + 11.5]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let before: String = page
            .evaluate_bounded("document.querySelector('#selection').textContent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let at = pointer::Point { x, y };
        pointer::drag(page, at, at, 1).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector('#selection').textContent !== {before:?}"),
            "a press in the box's corner to toggle the row",
        )
        .await
        .unwrap();
        let clicked: String = page
            .evaluate_bounded("document.querySelector('#clicked').textContent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(clicked.is_empty(), "the corner press also clicked the row");

        page.evaluate_bounded("document.querySelector('tbody tr:nth-child(2) input').focus()")
            .await
            .unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector('#selection').textContent === {before:?}"),
            "Space on the box's input to toggle the row back",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("a table checkbox hit area")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The ripple grows by a transform inside a clipping child span: `overflow: hidden`
/// on the button would clip the hit area as well.
#[test]
fn the_ripple_stays_inside_an_unclipped_button() {
    block_on(async {
        let fixture = Fixture::open("/hit-area", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, false).await.unwrap();

        let (x, y): (f64, f64) = page
            .evaluate_bounded(
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
             const span = el.querySelector(':scope > [data-ripple]'); \
             if (!span) return false; \
             const after = getComputedStyle(span, '::after'); \
             const box = span.getBoundingClientRect(), host = el.getBoundingClientRect(); \
             return getComputedStyle(el).overflow === 'visible' \
             && getComputedStyle(span).overflow === 'hidden' \
             && box.width === host.width && box.height === host.height \
             && after.animationName.startsWith('lsx-ripple-') \
             && after.transform !== 'none' && new DOMMatrix(after.transform).a > 0; })()",
            "the ripple circle to grow inside its span on an unclipped button",
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
                .evaluate_bounded(format!(
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
                .evaluate_bounded(format!(
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
