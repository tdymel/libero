//! What a dismissed overlay leaves behind.
//!
//! **WCAG 4.1.2 Name, Role, Value** and **1.3.1 Info and Relationships**: what
//! is exposed to assistive technology has to match what is presented. A panel
//! the sighted user has dismissed, still announced and still tabbable, does not.
//!
//! Between a popover closing and unmounting it is still in the DOM, still
//! tabbable and still in the accessibility tree, so a screen reader user can
//! reach a box the sighted user has already dismissed. Nothing in the library
//! animates a popover yet, which means nothing exhibits this today - it is the
//! obligation the first animated one inherits (`codebase/use-popover`), and
//! having the assertion already written is what makes that inheritance real
//! rather than a note somebody has to remember.

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

/// After dismissal, the box must be gone or genuinely hidden from AT.
///
/// "Genuinely" means one of `inert`, `aria-hidden`, or `visibility: hidden`.
/// `opacity: 0` and a zero-size box do not count: both still expose their
/// contents to a screen reader and can still hold focus.
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
