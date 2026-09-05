//! Reduced motion.
//!
//! `Emulation.setEmulatedMedia` flips `prefers-reduced-motion` for the page, so
//! a component's reduced arm is exercised rather than assumed. Several
//! components animate (`Collapse`, `Accordion`, `Skeleton`, `Marquee`,
//! `Indicator`) and their reduced arms are otherwise only read in source.

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};

/// Emulate `prefers-reduced-motion: reduce` (or clear it).
///
/// The same hook takes `forced-colors`, which is todo 50's open library-wide
/// gap. Left unwired deliberately: what each component should emit under
/// forced colours is undecided, and a pass asserting nothing would only look
/// like coverage.
pub async fn set_reduced_motion(page: &Page, reduced: bool) -> Result<()> {
    let value = if reduced { "reduce" } else { "no-preference" };
    page.execute(
        SetEmulatedMediaParams::builder()
            .features(vec![MediaFeature::new("prefers-reduced-motion", value)])
            .build(),
    )
    .await?;
    Ok(())
}
