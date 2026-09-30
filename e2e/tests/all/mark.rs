//! `Mark` (todo 53): a link inside draws its focus ring against the `var()` tint, which
//! `background()` reads no contrast twin off; measured against the `Mark` itself.

use e2e::browser::block_on;
use e2e::passes::{contrast, focus};
use e2e::wait;
use e2e::{Fixture, Suite, Viewport};

/// Rings again under the dark scheme, whose tints differ.
#[test]
fn it_meets_the_baseline() {
    LINKS
        .iter()
        .fold(Suite::new("mark", "/mark"), |suite, link| {
            suite.focusable(link)
        })
        .tab_budget(LINKS.len() + 2)
        .run();
}

const LINKS: &[&str] = &[
    "#mark-default",
    "#mark-primary",
    "#mark-secondary",
    "#mark-error",
    "#mark-info",
    "#mark-success",
    // A dark shade: its white twin met the white halo at 1:1 until the fill became the halo (todo 630).
    "#mark-info-6",
];

#[test]
fn a_link_inside_a_mark_has_a_ring_that_clears_three_to_one() {
    block_on(async {
        let fixture = Fixture::open("/mark", Viewport::Desktop).await.unwrap();

        for link in LINKS {
            let ring = focus::assert_focus_ring(&fixture.page, link, LINKS.len() + 2)
                .await
                .unwrap();
            focus::assert_ring_contrast(&ring).unwrap_or_else(|e| panic!("{link}: {e}"));
            eprintln!(
                "{link}: outline {} against {}",
                ring.outline_color, ring.against
            );
        }

        fixture.console.assert_clean("the mark fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1542. Forced colours drop the tint, so the mark paints the system `Mark` pair.
#[test]
fn a_mark_keeps_its_highlight_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/mark", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        const MARK: &str = "(() => { const probe = document.createElement('span'); \
            probe.style.background = 'Mark'; probe.style.color = 'MarkText'; document.body.append(probe); \
            const want = getComputedStyle(probe); const marks = [...document.querySelectorAll('mark')]; \
            const ok = marks.length > 0 && marks.every(m => { const s = getComputedStyle(m); \
            return s.backgroundColor === want.backgroundColor && s.color === want.color; }); \
            probe.remove(); return ok; })()";
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(page, MARK, "every mark in Mark on MarkText")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 762: the link colour read 3.5-4.0:1 on every tint and 1.01:1 on
/// `info.6`. A link now takes the tint's text twin, underlined.
#[test]
fn a_link_inside_a_mark_reads_on_every_tint() {
    block_on(async {
        let fixture = Fixture::open("/mark", Viewport::Desktop).await.unwrap();

        contrast::assert_clean(&fixture.page, "body").await.unwrap();

        fixture.close().await.unwrap();
    });
}
