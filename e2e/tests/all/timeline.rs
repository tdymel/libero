//! `Timeline`: list semantics, and a done event that differs by more than colour.

use e2e::browser::block_on;
use e2e::suite::Suite;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("timeline", "/timeline").run();
}

/// Todo 2429: the AX name of a rich title is its plain-text name, not the drawn content.
#[test]
fn a_rich_title_is_named_by_its_plain_text() {
    block_on(async {
        let fixture = Fixture::open("/timeline/rich", Viewport::Desktop)
            .await
            .unwrap();
        let tree = e2e::ax::snapshot(&fixture.page, "#rich").await.unwrap();
        assert!(tree.contains("Shipped by air"), "{tree}");
        assert!(!tree.contains("✈"), "{tree}");
        fixture.close().await.unwrap();
    });
}

/// Each dot bullet's fill and ring: `[background, border]` per event.
const BULLETS: &str = "(() => [...document.querySelectorAll('#left > li > span')] \
    .map(b => { const s = getComputedStyle(b); return [s.backgroundColor, s.borderTopColor]; }))()";

/// WCAG 1.4.11: a pending ring and the rail below it show the event's place
/// at 3:1, on the page and on the ring's own fill (todo 599).
#[test]
fn a_pending_ring_and_rail_part_at_3_to_1() {
    crate::boundary::assert_boundaries(
        "/timeline",
        "const item = document.querySelector('#left > li:nth-child(3)');
         const ring = item.querySelector(':scope > span');
         const line = ['borderLeftStyle', 'borderLeftWidth'].every(p => CSS(item, p, '::before') !== 'none'
             && CSS(item, p, '::before') !== '0px')
             ? CSS(item, 'borderLeftColor', '::before') : CSS(item, 'backgroundColor', '::before');
         const ringColour = CSS(ring, 'borderTopColor');
         return [
             ['pending ring on the page', RATIO(ringColour, PAGE(item))],
             ['pending ring on its fill', RATIO(ringColour, CSS(ring, 'backgroundColor'))],
             ['pending rail on the page', RATIO(line, PAGE(item))],
         ];",
    );
}

/// 1.4.1: a done dot fills, a pending one stays a ring, so the two differ by
/// shape and not by the accent alone.
#[test]
fn a_done_dot_fills_and_a_pending_one_stays_a_ring() {
    block_on(async {
        let fixture = Fixture::open("/timeline", Viewport::Desktop).await.unwrap();

        let bullets: Vec<(String, String)> = fixture
            .page
            .evaluate_bounded(BULLETS)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let [done, current, pending, last] = bullets.as_slice() else {
            panic!("expected four bullets, got {bullets:?}");
        };
        assert_eq!(done.0, done.1, "a done dot is filled with its ring colour");
        assert_eq!(current.0, current.1, "the current dot is filled too");
        assert_ne!(pending.0, pending.1, "a pending dot is a ring: {pending:?}");
        assert_eq!(pending, last);

        // The AX snapshot keeps no `current`, so the one current event is read here.
        let current: Vec<String> = fixture
            .page
            .evaluate_bounded(
                "[...document.querySelectorAll('#left > li')] \
                 .map(li => li.getAttribute('aria-current') || '-')",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(current, ["-", "step", "-", "-"]);

        fixture
            .console
            .assert_clean("the timeline fixture")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Forced colours paint every fill `Canvas`: a done dot and a glyph bullet
/// must still differ from a pending one there (1.4.1).
#[test]
fn done_and_pending_bullets_stay_apart_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/timeline", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute_bounded(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        // `[background, glyph colour]` of the done and a pending bullet, per list.
        let bullets: Vec<Vec<(String, String)>> = page
            .evaluate_bounded(
                "['#left', '#alternate'].map(id => [1, 3].map(n => { \
                 const s = getComputedStyle(document.querySelector(`${id} > li:nth-child(${n}) > span`)); \
                 return [s.backgroundColor, s.color]; }))",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let [dots, glyphs] = bullets.as_slice() else {
            panic!("expected two lists, got {bullets:?}");
        };
        assert_ne!(dots[0].0, dots[1].0, "a done dot fills like a pending one");
        assert_ne!(glyphs[0], glyphs[1], "a done glyph bullet looks pending");
        fixture.close().await.unwrap();
    });
}

/// Per event of each list: which side of the bullet the body and the opposite sit on
/// (1 right, -1 left), the rail's worst offset from its bullets' centres, and how far
/// the opposite's top strays from the body's.
const OPPOSITE_SIDES: &str = "['#start', '#end', '#alternate'].map(id => { \
     const items = [...document.querySelectorAll(id + ' > li')]; \
     const mid = e => { const r = e.getBoundingClientRect(); return (r.left + r.right) / 2; }; \
     const rows = items.map(li => { const b = mid(li.querySelector(':scope > [data-slot=bullet]')); \
       return [Math.sign(mid(li.querySelector(':scope > [data-slot=body]')) - b), \
               Math.sign(mid(li.querySelector(':scope > [data-slot=opposite]')) - b)]; }); \
     const off = Math.max(...items.slice(0, -1).map(li => { \
       const box = li.getBoundingClientRect(); const rail = getComputedStyle(li, '::before'); \
       const b = li.querySelector(':scope > [data-slot=bullet]').getBoundingClientRect(); \
       return Math.abs(box.left + parseFloat(rail.left) + parseFloat(rail.borderLeftWidth) / 2 \
                       - (b.left + b.width / 2)); })); \
     const top = s => li => li.querySelector(':scope > [data-slot=' + s + ']').getBoundingClientRect().top; \
     const dy = Math.max(...items.map(li => Math.abs(top('opposite')(li) - top('body')(li)))); \
     return { rows, off, dy }; })";

/// Todo 4148: the opposite content sits across the rail from the body, and the rail stays
/// under the bullets, in either direction.
#[test]
fn the_opposite_sits_across_the_rail_from_the_body() {
    #[derive(serde::Deserialize)]
    struct Measured {
        rows: Vec<[f64; 2]>,
        off: f64,
        dy: f64,
    }
    block_on(async {
        for (dir, flip) in [("ltr", 1.0), ("rtl", -1.0)] {
            let fixture = Fixture::open("/timeline/opposite", Viewport::Desktop)
                .await
                .unwrap();
            let page = &fixture.page;
            page.evaluate_bounded(format!("document.documentElement.dir = '{dir}'"))
                .await
                .unwrap();
            wait::for_js_true(
                page,
                &format!("getComputedStyle(document.body).direction === '{dir}'"),
                "the page to take its direction",
            )
            .await
            .unwrap();
            let measured: Vec<Measured> = page
                .evaluate_bounded(OPPOSITE_SIDES)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            let want = [
                [[1.0, -1.0], [1.0, -1.0]],
                [[-1.0, 1.0], [-1.0, 1.0]],
                [[1.0, -1.0], [-1.0, 1.0]],
            ];
            for ((measured, want), id) in
                measured.iter().zip(want).zip(["start", "end", "alternate"])
            {
                assert!(
                    measured.off < 2.0,
                    "{dir} {id}: rail {}px off its bullets",
                    measured.off
                );
                assert!(
                    measured.dy < 1.0,
                    "{dir} {id}: opposite {}px off the body's top",
                    measured.dy
                );
                let want: Vec<[f64; 2]> = want.iter().map(|[a, b]| [a * flip, b * flip]).collect();
                assert_eq!(measured.rows, want, "{dir} {id}: body and opposite sides");
            }
            fixture
                .console
                .assert_clean("the opposite fixture")
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 4630: on a phone the rail runs down the start edge and the opposite takes only
/// its text's width, with the body beside it.
#[test]
fn a_phone_sizes_the_opposite_to_its_text() {
    #[derive(serde::Deserialize)]
    struct Measured {
        bullet_start: f64,
        opposite_share: f64,
        body_after_opposite: bool,
        off: f64,
    }
    const PHONE: &str = "(() => { const li = document.querySelector('#start > li'); \
         const box = li.getBoundingClientRect(); \
         const part = s => li.querySelector(':scope > [data-slot=' + s + ']').getBoundingClientRect(); \
         const bullet = part('bullet'), body = part('body'), opposite = part('opposite'); \
         const rail = getComputedStyle(li, '::before'); \
         return { bullet_start: bullet.left - box.left, opposite_share: opposite.width / box.width, \
           body_after_opposite: body.left >= opposite.right - 0.5, \
           off: Math.abs(box.left + parseFloat(rail.left) + parseFloat(rail.borderLeftWidth) / 2 \
                         - (bullet.left + bullet.width / 2)) }; })()";
    block_on(async {
        let fixture = Fixture::open("/timeline/opposite", Viewport::Mobile)
            .await
            .unwrap();
        let measured: Measured = fixture
            .page
            .evaluate_bounded(PHONE)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            measured.bullet_start < 1.0,
            "the bullet is off the start edge"
        );
        assert!(
            measured.opposite_share < 0.3,
            "the opposite takes {} of the row",
            measured.opposite_share
        );
        assert!(
            measured.body_after_opposite,
            "the body overlaps the opposite"
        );
        assert!(measured.off < 2.0, "rail {}px off its bullet", measured.off);
        fixture.close().await.unwrap();
    });
}

/// Per list: the rail's worst offset from its bullets' centres (px), and which
/// side of the first bullet the first event's content sits on (1 right, -1 left).
const SIDES: &str = "['#left', '#right', '#alternate'].map(id => { \
     const items = [...document.querySelectorAll(id + ' > li')]; \
     const bullet = li => li.querySelector(':scope > span').getBoundingClientRect(); \
     const off = Math.max(...items.slice(0, -1).map(li => { \
       const box = li.getBoundingClientRect(); const b = bullet(li); \
       const rail = getComputedStyle(li, '::before'); \
       const x = box.left + parseFloat(rail.left) + parseFloat(rail.borderLeftWidth) / 2; \
       return Math.abs(x - (b.left + b.width / 2)); })); \
     const li = items[0]; const box = li.getBoundingClientRect(); const s = getComputedStyle(li); \
     const content = (box.left + parseFloat(s.paddingLeft) + box.right - parseFloat(s.paddingRight)) / 2; \
     const b = bullet(li); \
     return [off, Math.sign(content - (b.left + b.width / 2))]; })";

/// Todo 747: `Start`, `End` and `Alternate` are logical, so `dir="rtl"`
/// mirrors each with its rail still under the bullets.
#[test]
fn the_sides_mirror_under_rtl() {
    block_on(async {
        // `#left` is `Start`, `#right` is `End`.
        for (dir, sides) in [("ltr", [1.0, -1.0, 1.0]), ("rtl", [-1.0, 1.0, -1.0])] {
            let fixture = Fixture::open("/timeline", Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            page.evaluate_bounded(format!("document.documentElement.dir = '{dir}'"))
                .await
                .unwrap();
            wait::for_js_true(
                page,
                &format!("getComputedStyle(document.body).direction === '{dir}'"),
                "the page to take its direction",
            )
            .await
            .unwrap();
            let measured: Vec<[f64; 2]> = page
                .evaluate_bounded(SIDES)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            for (([off, side], want), id) in
                measured
                    .iter()
                    .zip(sides)
                    .zip(["start", "end", "alternate"])
            {
                assert!(*off < 2.0, "{dir} {id}: rail {off}px off its bullets");
                assert_eq!(*side, want, "{dir} {id}: content on the wrong side");
            }
            fixture.close().await.unwrap();
        }
    });
}
