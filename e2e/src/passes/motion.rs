//! Reduced motion.
//!
//! `Emulation.setEmulatedMedia` flips `prefers-reduced-motion` for the page, so
//! a component's reduced arm is exercised rather than assumed. Several
//! components animate (`Collapse`, `Accordion`, `Skeleton`, `Marquee`,
//! `Indicator`) and their reduced arms are otherwise only read in source.

use anyhow::{Result, bail};
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

/// The page must report `prefers-reduced-motion: reduce`.
///
/// Called after [`set_reduced_motion`], so a test that believes it runs
/// reduced proves it does. Without it an emulation that silently did nothing
/// leaves every reduced-motion test measuring the animated arm.
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

/// Nothing under `selector`, itself included, may run a transition or an
/// animation: every computed `transition-duration` and `animation-duration`
/// must be zero, on each element and on its rendered `::before`/`::after`.
///
/// **WCAG 2.3.3 Animation from Interactions**: under reduced motion, motion
/// triggered by an interaction can be switched off. A reduced arm that is
/// declared but lost - moved to another selector, outranked, or never
/// emitted - leaves the component animating, and nothing else notices.
///
/// The pseudo-elements are read because `Skeleton` and the oval `Loader`
/// animate only their `::after`: an element-only read saw both as still.
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
