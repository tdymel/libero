//! What a dismissed overlay leaves behind (WCAG 4.1.2, 1.3.1): a closing popover must not stay
//! announced or tabbable. For the first animated popover (`codebase/use-popover`).

use anyhow::{Result, bail};
use chromiumoxide::Page;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Residue {
    present: bool,
    visible: bool,
    inert: bool,
    aria_hidden: bool,
    tabbables: usize,
}

/// After dismissal, the box must be gone, `inert`, `aria-hidden` or `visibility: hidden`.
/// `opacity: 0` and a zero-size box still expose their contents and can hold focus.
pub async fn assert_gone_from_at(page: &Page, selector: &str) -> Result<()> {
    let residue: Residue = page
        .evaluate(format!(
            r#"(() => {{
                const el = document.querySelector({});
                if (!el) return {{ present: false, visible: false, inert: false, aria_hidden: false, tabbables: 0 }};
                const s = getComputedStyle(el);
                const focusable = 'a[href],button,input,select,textarea,[tabindex]:not([tabindex="-1"])';
                return {{
                    present: true,
                    visible: s.visibility !== 'hidden' && s.display !== 'none',
                    inert: el.hasAttribute('inert') || !!el.closest('[inert]'),
                    aria_hidden: el.getAttribute('aria-hidden') === 'true',
                    tabbables: el.querySelectorAll(focusable).length,
                }};
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;

    if !residue.present {
        return Ok(());
    }
    if residue.inert || residue.aria_hidden || !residue.visible {
        return Ok(());
    }
    bail!(
        "after dismissal {selector} is still in the accessibility tree: visible, not inert, \
         not aria-hidden, and holding {} tabbable element(s). A screen reader still reads a \
         box the user has closed.",
        residue.tabbables
    );
}
