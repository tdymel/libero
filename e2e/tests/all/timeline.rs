//! `Timeline`: list semantics, and a done event that differs by more than colour.

use e2e::browser::block_on;
use e2e::passes::contrast;
use e2e::suite::Suite;
use e2e::{Fixture, Viewport};

#[test]
fn it_meets_the_baseline() {
    Suite::new("timeline", "/timeline")
        .waive(contrast::TODO_297)
        .run();
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
            .evaluate(BULLETS)
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
            .evaluate(
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
