//! WCAG 2.5.8 target size (minimum): 24 by 24 CSS pixels.
//!
//! <https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html>
//!
//! Straight off the box model, so it is objective and cheap. The criterion has
//! exemptions (inline targets in a sentence, targets whose spacing gives them
//! a 24px circle), so this reports the measurement and the caller decides
//! which controls it applies to, rather than the pass failing a whole page.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use serde::Deserialize;

pub const MINIMUM: f64 = 24.0;

#[derive(Debug, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// Measure every element matching `selector`.
pub async fn measure_all(page: &Page, selector: &str) -> Result<Vec<Size>> {
    let sizes = page
        .evaluate(format!(
            r#"[...document.querySelectorAll({})].map(el => {{
                const r = el.getBoundingClientRect();
                return {{ width: r.width, height: r.height }};
            }})"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    Ok(sizes)
}

/// Fail if any match is smaller than 24x24.
pub async fn assert_minimum(page: &Page, selector: &str) -> Result<()> {
    let sizes = measure_all(page, selector).await?;
    if sizes.is_empty() {
        bail!("target size: nothing matched {selector}");
    }
    assert_sizes(selector, &sizes)
}

/// The assertion on its own, for a caller that already has the measurements.
///
/// Split out because a control that only exists in an open state has to be
/// measured where it exists: `[role=option]` matches nothing while the list is
/// closed, and "nothing matched" is not a failure there.
pub fn assert_sizes(selector: &str, sizes: &[Size]) -> Result<()> {
    let small: Vec<_> = sizes
        .iter()
        .filter(|s| s.width < MINIMUM || s.height < MINIMUM)
        .collect();
    if !small.is_empty() {
        bail!("target size below WCAG 2.5.8 minimum ({MINIMUM}px) for {selector}: {small:?}");
    }
    Ok(())
}
