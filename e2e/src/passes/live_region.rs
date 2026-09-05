//! Live regions.
//!
//! ## What this is checked against
//!
//! - **WCAG 4.1.3 Status Messages** - a status can be presented to assistive
//!   technology without receiving focus.
//! - **WAI-ARIA, `aria-live`** - politeness, and `role="status"` implying
//!   `aria-live="polite"` plus `aria-atomic="true"`.
//!   <https://www.w3.org/TR/wai-aria-1.2/#aria-live>
//!
//! A screen reader cannot be run here, so this proves the three things that are
//! observable and that a reader depends on: the region **exists**, it carries
//! the right politeness, and its text **changes** at the moment it should.
//!
//! That last one is the point. The library's status regions are always mounted
//! and start empty (`codebase/components/combobox`), because a region that
//! mounts together with its text is skipped by some readers. An assertion that
//! the region merely exists would hold just as well for one that never says
//! anything.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use serde::Deserialize;

/// A lookup that distinguishes "not there" from "there and empty".
///
/// Returned as a struct rather than an `Option<String>` because a bare JS
/// `null` cannot be deserialised into one: chromiumoxide reports "No value
/// found", which surfaces as a confusing error about the harness instead of a
/// clear one about the page. Any `evaluate` in this crate that might return
/// nothing returns a defined object instead.
#[derive(Debug, Deserialize)]
struct Lookup {
    found: bool,
    value: String,
}

/// The region's current text.
pub async fn text_of(page: &Page, selector: &str) -> Result<String> {
    let found: Lookup = page
        .evaluate(format!(
            r#"(() => {{
                const el = document.querySelector({});
                return {{ found: !!el, value: el ? el.textContent : '' }};
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    if !found.found {
        bail!("no live region at {selector}");
    }
    Ok(found.value)
}

/// Assert the region exists and is polite (or assertive, as given).
pub async fn assert_politeness(page: &Page, selector: &str, expected: &str) -> Result<()> {
    let live: Lookup = page
        .evaluate(format!(
            r#"(() => {{
                const el = document.querySelector({});
                if (!el) return {{ found: false, value: '' }};
                // `role="status"` implies `aria-live="polite"`, and `role="alert"`
                // implies assertive, so an explicit attribute is not required.
                const role = el.getAttribute('role');
                const implied = role === 'status' ? 'polite' : role === 'alert' ? 'assertive' : '';
                return {{ found: true, value: el.getAttribute('aria-live') || implied }};
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;

    if !live.found {
        bail!("no live region at {selector}");
    }
    if live.value != expected {
        bail!(
            "live region {selector} is {:?}, expected {expected:?}",
            live.value
        );
    }
    Ok(())
}
