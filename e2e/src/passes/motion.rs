//! Reduced motion: emulates `prefers-reduced-motion` so a component's reduced arm is exercised.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use serde::Deserialize;

/// Emulate `prefers-reduced-motion: reduce` (or clear it), and the light scheme:
/// for a dark page use [`crate::browser::emulate_media`]. `forced-colors` is unwired (todo 50).
pub async fn set_reduced_motion(page: &Page, reduced: bool) -> Result<()> {
    crate::browser::emulate_media(page, crate::Scheme::Light, Some(reduced)).await
}

/// The page must report `prefers-reduced-motion: reduce`, proving the emulation took.
pub async fn assert_reduced_motion_matches(page: &Page) -> Result<()> {
    let matches: bool = page
        .evaluate("matchMedia('(prefers-reduced-motion: reduce)').matches")
        .await?
        .into_value()?;
    if !matches {
        bail!(
            "the page does not match (prefers-reduced-motion: reduce); the emulation did nothing"
        );
    }
    Ok(())
}

/// One script scroll [`spy_scrolls`] saw.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Scroll {
    /// What the browser would have animated: a `smooth` option, or none and a
    /// computed `scroll-behavior: smooth`.
    pub smooth: bool,
    /// Landed half-way, as a smooth scroll replaced mid-way does.
    pub cut: bool,
}

/// Records every `scrollTo`/`scroll`/`scrollBy` on `selector` and whether it was smooth (the suite's
/// Chromium scrolls instantly, todo 687). `cut_short` lands every other smooth one half-way.
pub async fn spy_scrolls(page: &Page, selector: &str, cut_short: bool) -> Result<()> {
    page.evaluate(format!(
        r#"(() => {{
            if (window.__scrollSpy) throw new Error('spy_scrolls runs once per page');
            window.__scrollSpy = [];
            const selector = {};
            let cut = false;
            for (const name of ['scrollTo', 'scroll', 'scrollBy']) {{
                const original = Element.prototype[name];
                Element.prototype[name] = function (...args) {{
                    if (!this.matches(selector)) return original.apply(this, args);
                    const numbers = typeof args[0] === 'number';
                    const options = numbers ? {{ left: args[0], top: args[1] }} : (args[0] ?? {{}});
                    const behavior = options.behavior ?? 'auto';
                    const smooth = behavior === 'smooth'
                        || (behavior === 'auto' && getComputedStyle(this).scrollBehavior === 'smooth');
                    const shortened = {cut_short} && smooth && name !== 'scrollBy' && (cut = !cut);
                    window.__scrollSpy.push({{ smooth, cut: shortened }});
                    if (!shortened) return original.apply(this, args);
                    const half = (from, to) => to === undefined ? from : from + (to - from) / 2;
                    original.call(this, {{
                        left: half(this.scrollLeft, options.left),
                        top: half(this.scrollTop, options.top),
                        behavior: 'instant',
                    }});
                }};
            }}
        }})()"#,
        serde_json::to_string(selector)?
    ))
    .await?;
    Ok(())
}

/// Every scroll [`spy_scrolls`] saw since it was installed, oldest first.
pub async fn scrolls(page: &Page) -> Result<Vec<Scroll>> {
    let seen: Option<Vec<Scroll>> = page
        .evaluate("window.__scrollSpy ?? null")
        .await?
        .into_value()?;
    seen.ok_or_else(|| anyhow::anyhow!("no scroll spy on the page; call spy_scrolls first"))
}

/// WCAG 2.3.3: no transition or animation under `selector`, pseudo-elements included
/// (`Skeleton` and the oval `Loader` animate only their `::after`).
pub async fn assert_still(page: &Page, selector: &str) -> Result<()> {
    let moving: Option<Vec<String>> = page
        .evaluate(format!(
            r#"(() => {{
                const root = document.querySelector({});
                if (!root) return null;
                const moving = s => s.split(',').some(d => parseFloat(d) > 0);
                const out = [];
                for (const el of [root, ...root.querySelectorAll('*')]) {{
                    for (const pseudo of [null, '::before', '::after']) {{
                        const s = getComputedStyle(el, pseudo);
                        if (pseudo && (s.content === 'none' || s.content === 'normal')) continue;
                        const name = `${{el.tagName.toLowerCase()}}.${{el.className}}${{pseudo || ''}}`;
                        if (moving(s.transitionDuration))
                            out.push(`${{name}}: transition ${{s.transitionProperty}} ${{s.transitionDuration}}`);
                        if (s.animationName !== 'none' && moving(s.animationDuration))
                            out.push(`${{name}}: animation ${{s.animationName}} ${{s.animationDuration}}`);
                    }}
                }}
                return out;
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    let Some(moving) = moving else {
        bail!("no element matches {selector}");
    };
    if !moving.is_empty() {
        bail!(
            "{selector} still animates under reduced motion:\n  {}",
            moving.join("\n  ")
        );
    }
    Ok(())
}
