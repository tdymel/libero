//! WCAG 1.4.10 Reflow: at 320 CSS px wide the page scrolls in one direction only (todo 1794).
//! 2D content (a data table, a map) is the criterion's own exception, named by the caller.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;

use crate::Viewport;

/// The width 1.4.10 names: 1280 px at 400% zoom.
pub const NARROW: i64 = 320;

/// Narrow a mobile page to 320 px, require no horizontal page scroll outside `exempt`,
/// and widen it back, so the caller carries on at 390.
pub async fn assert_reflows(page: &Page, exempt: &[&str], during: &str) -> Result<()> {
    resize(page, NARROW).await?;
    let offenders = std::cell::RefCell::new(Vec::new());
    // A resize observer may re-render once more: the first clean reading passes.
    let reflowed = crate::wait::until(&format!("{during} to reflow at {NARROW}px"), || async {
        let found = overflowing(page, exempt).await?;
        let clean = found.is_empty();
        *offenders.borrow_mut() = found;
        Ok(clean)
    })
    .await;
    resize(page, Viewport::Mobile.size().0).await?;
    if reflowed.is_err() {
        bail!(
            "{during}: at {NARROW}px wide the page scrolls sideways (WCAG 1.4.10) past {}",
            offenders.borrow().join("; ")
        );
    }
    Ok(())
}

/// Still a mobile device: toggling the emulation's mobile flag changes text layout, and with it
/// the later states' trees. Its layout viewport widens to overflowing content, so the screen tells.
async fn resize(page: &Page, width: i64) -> Result<()> {
    page.execute(SetDeviceMetricsOverrideParams::new(
        width,
        Viewport::Mobile.size().1,
        1.0,
        true,
    ))
    .await?;
    crate::wait::for_js_true(
        page,
        &format!("screen.width === {width}"),
        &format!("the screen to be {width}px wide"),
    )
    .await?;
    crate::clock::settle(page).await
}

/// The outermost-deepest boxes past the right edge that scroll the page: none inside a
/// sideways clipping scroller, a fixed layer or an exempt subtree.
async fn overflowing(page: &Page, exempt: &[&str]) -> Result<Vec<String>> {
    Ok(page
        .evaluate(format!(
            r#"(() => {{
                const exempt = {};
                const doc = document.documentElement;
                const innerWidth = {NARROW};
                if (doc.scrollWidth <= innerWidth) return [];
                const name = el => el.tagName.toLowerCase() + (el.id ? '#' + el.id : '')
                    + (el.getAttribute('role') ? `[role=${{el.getAttribute('role')}}]` : '')
                    + (el.getAttribute('data-slot') ? `[data-slot=${{el.getAttribute('data-slot')}}]` : '');
                const contained = el => {{
                    if (exempt.some(selector => el.closest(selector))) return true;
                    for (let p = el; p && p !== document.body; p = p.parentElement) {{
                        const s = getComputedStyle(p);
                        if (s.position === 'fixed') return true;
                        if (p !== el && (s.overflowX !== 'visible' || /paint|strict|content/.test(s.contain))) return true;
                    }}
                    return false;
                }};
                const past = [...document.body.querySelectorAll('*')].filter(el => {{
                    const r = el.getBoundingClientRect();
                    return r.width > 0 && r.right > innerWidth + 1 && !contained(el);
                }});
                const deepest = past.filter(el => !past.some(other => other !== el && el.contains(other)));
                const exemptPast = exempt.some(selector => [...document.querySelectorAll(selector)]
                    .some(el => el.getBoundingClientRect().right > innerWidth + 1));
                if (!deepest.length)
                    return exemptPast ? [] : [`a page ${{doc.scrollWidth}}px wide with no box past the edge (text or a pseudo-element spilling)`];
                return deepest.slice(0, 5).map(el => `${{name(el)}} reaching ${{Math.round(el.getBoundingClientRect().right)}}px`);
            }})()"#,
            serde_json::to_string(exempt)?
        ))
        .await?
        .into_value()?)
}
