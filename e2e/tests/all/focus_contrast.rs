//! Todo 53, part one, answered by measurement.
//!
//! Not a pass and not a regression guard: a question the suite happened to be
//! able to answer cheaply. `--lsx-focus-contrast` set to a `var()` whose
//! referent nobody declares - does the ring fall back to primary, or vanish?
//!
//! ## Measured 2026-09-19, Chromium
//!
//! **The ring falls back to primary. It does not vanish.**
//! `outline: rgb(34, 139, 230) solid 2px`.
//!
//! Chromium does this because a custom property whose value contains a `var()`
//! that fails to substitute computes to the **guaranteed-invalid value**, and a
//! custom property holding that is treated as *unset* by everything downstream.
//! So `var(--lsx-focus-contrast, var(--lsx-color-primary-6))` takes its
//! fallback exactly as if nobody had set `--lsx-focus-contrast` at all.
//!
//! Which means, for the fallback chain: **a broken `--lsx-focus-contrast`
//! degrades safely.** It is the "invalid at computed-value time" rule that
//! would have dropped the ring, and that rule applies to a *normal* property
//! consuming a bad `var()`, not to a custom property holding one. The two are
//! easy to conflate and they give opposite answers.
//!
//! Scope of the measurement: `:root`, an undeclared `var()` referent, Chromium.
//! A property set to an invalid non-`var()` value, or set on an intermediate
//! element rather than `:root`, is a different case and was not measured.

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

        // A snapshot, not an assertion.
        //
        // Nobody has decided what *should* happen here - todo 53 asks what
        // *does*. Asserting "a ring survives" would be betting on the guess
        // that produced the question, and if the bet were wrong the answer
        // would arrive as a red suite that reads like the harness is broken
        // rather than like a finding.
        //
        // A snapshot records the observed behaviour where a reviewer sees it,
        // and still fails if it ever changes - which is the actual thing worth
        // guarding, without pretending a decision was made.
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
