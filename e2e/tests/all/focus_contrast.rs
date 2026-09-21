//! Todo 53: `--lsx-focus-contrast` set to an undeclared `var()` on `:root` falls back to
//! primary in Chromium (guaranteed-invalid reads as unset), so the ring survives.

use e2e::browser::block_on;
use e2e::passes::{focus, keyboard};
use e2e::{Fixture, Scheme, Viewport};

const PROBE: &str = "#ring-probe";

#[test]
fn an_undeclared_focus_contrast_resolves_to() {
    block_on(async {
        let fixture = Fixture::open("/focus-contrast", Viewport::Desktop)
            .await
            .unwrap();

        keyboard::tab_to(&fixture.page, PROBE, 5).await.unwrap();

        let (outline, colour): (String, String) = fixture
            .page
            .evaluate(
                "(() => { const s = getComputedStyle(document.querySelector('#ring-probe')); \
                 return [s.outline, s.outlineColor]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();

        // A snapshot, not an assertion: todo 53 asks what happens, not what should.
        insta::assert_snapshot!(
            "undeclared_focus_contrast",
            format!("outline: {outline}\noutline-color: {colour}")
        );

        fixture.close().await.unwrap();
    });
}

/// Todo 604: in dark, `--lsx-ink`/`--lsx-surface` for a hex put the stripe on
/// the page's dark end, 1.66:1 on navy. The hex's own twin does not flip.
#[test]
fn a_hex_background_rings_its_link_in_the_hex_twin_in_dark() {
    block_on(async {
        let fixture = Fixture::open_in("/focus-contrast/hex", Viewport::Desktop, Scheme::Dark)
            .await
            .unwrap();

        let ring = focus::assert_focus_ring(&fixture.page, "#hex-link", 5)
            .await
            .unwrap();
        assert_eq!(ring.outline_color, "rgb(255, 255, 255)", "{ring:?}");
        focus::assert_ring_contrast(&ring).unwrap();

        fixture.console.assert_clean("the hex ring").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 630: in light a dark fill's white stripe met the page's white halo,
/// 1.00:1. The fill is now the halo.
#[test]
fn a_dark_fill_is_the_halo_of_the_rings_inside_it_in_light() {
    block_on(async {
        let fixture = Fixture::open("/focus-contrast/hex", Viewport::Desktop)
            .await
            .unwrap();
        let ring = focus::assert_focus_ring(&fixture.page, "#hex-link", 5)
            .await
            .unwrap();
        focus::assert_ring_contrast(&ring).unwrap_or_else(|e| panic!("#hex-link: {e}"));
        fixture.close().await.unwrap();

        let fixture = Fixture::open("/focus-contrast/fills", Viewport::Desktop)
            .await
            .unwrap();
        for target in ["#paper-link", "#filled-alert [data-slot=close]"] {
            let ring = focus::assert_focus_ring(&fixture.page, target, 5)
                .await
                .unwrap();
            focus::assert_ring_contrast(&ring).unwrap_or_else(|e| panic!("{target}: {e}"));
            eprintln!(
                "{target}: outline {} against {}",
                ring.outline_color, ring.against
            );
        }

        fixture.console.assert_clean("the dark fills").unwrap();
        fixture.close().await.unwrap();
    });
}
