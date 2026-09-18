//! The shadow scale (813): every surface casts a shadow in both schemes, and
//! the dark page takes its own, deeper scale. `E2E_ELEVATION_SHOTS=1` also
//! writes a screenshot per scheme, with and without the dialog open.

use e2e::browser::{Scheme, block_on};
use e2e::passes::pointer;
use e2e::{Fixture, Viewport, wait};

const SHADOW: &str = "getComputedStyle(document.getElementById('card')).boxShadow";

#[test]
fn every_surface_casts_a_shadow_in_both_schemes() {
    block_on(async {
        let mut shadows = Vec::new();
        for scheme in [Scheme::Light, Scheme::Dark] {
            let fixture = Fixture::open_in("/elevation", Viewport::Desktop, scheme)
                .await
                .unwrap();
            let page = &fixture.page;
            wait::for_visible(page, "[role=menu]").await.unwrap();

            let shadow: String = page.evaluate(SHADOW).await.unwrap().into_value().unwrap();
            assert_ne!(
                shadow,
                "none",
                "{}: the card casts no shadow",
                scheme.name()
            );
            shadows.push(shadow);

            let shots = std::env::var_os("E2E_ELEVATION_SHOTS").is_some();
            if shots {
                fixture
                    .screenshot(&format!("elevation-{}", scheme.name()))
                    .await
                    .unwrap();
            }
            pointer::click(page, "#open-dialog").await.unwrap();
            wait::for_visible(page, "[role=dialog]").await.unwrap();
            let dialog: String = page
                .evaluate("getComputedStyle(document.querySelector('[role=dialog]')).boxShadow")
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_ne!(
                dialog,
                "none",
                "{}: the dialog casts no shadow",
                scheme.name()
            );
            if shots {
                fixture
                    .screenshot(&format!("elevation-dialog-{}", scheme.name()))
                    .await
                    .unwrap();
            }

            fixture.console.assert_clean(scheme.name()).unwrap();
            fixture.close().await.unwrap();
        }
        assert_ne!(shadows[0], shadows[1], "the dark page takes its own scale");
    });
}
