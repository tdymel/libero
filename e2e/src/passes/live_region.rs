//! Live regions (WCAG 4.1.3, <https://www.w3.org/TR/wai-aria-1.2/#aria-live>): the region exists,
//! has the right politeness, and its text changes when it should (`codebase/components/combobox`).

use anyhow::{Result, bail};
use chromiumoxide::Page;
use serde::Deserialize;

/// A lookup that distinguishes "not there" from "there and empty".
/// A struct, since a bare JS `null` fails to deserialise as "No value found".
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
