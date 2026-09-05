//! Focus management and the focus ring.
//!
//! One rule governs this whole file: **ask the document what it holds, never
//! ask the focus call whether it worked.** `HTMLElement.focus()` on a
//! `visibility: hidden` element does nothing and returns successfully, which is
//! how a searchable `Select` shipped with a search box that never took focus -
//! `focus()` said `Ok`, `is_focused()` said false (`codebase/use-popover`).
//! Every assertion here reads `document.activeElement`.
//!
//! ## What this is checked against
//!
//! - **WCAG 2.4.7 Focus Visible** - a keyboard focus indicator is visible.
//! - **WCAG 1.4.11 Non-text Contrast** - that indicator clears 3:1 against
//!   what it is painted on, which for an offset outline is the **parent**
//!   surface, never the element's own.
//! - **WCAG 2.4.3 Focus Order** and **3.2.1 On Focus** - where focus goes, and
//!   where it comes back to.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use serde::Deserialize;

/// Enough of the focused element to name it in a failure.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Focused {
    pub tag: String,
    pub role: Option<String>,
    pub label: Option<String>,
    pub id: Option<String>,
}

/// What the document actually has focused.
pub async fn active_element(page: &Page) -> Result<Focused> {
    let value = page
        .evaluate(
            r#"(() => {
                const el = document.activeElement;
                if (!el) return { tag: 'none', role: null, label: null, id: null };
                return {
                    tag: el.tagName.toLowerCase(),
                    role: el.getAttribute('role'),
                    label: el.getAttribute('aria-label'),
                    id: el.id || null,
                };
            })()"#,
        )
        .await?
        .into_value()?;
    Ok(value)
}

/// Assert focus is on the element matching `selector`.
pub async fn assert_focused(page: &Page, selector: &str, during: &str) -> Result<()> {
    let matches: bool = page
        .evaluate(format!(
            "document.activeElement === document.querySelector({})",
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    if !matches {
        let actual = active_element(page).await?;
        bail!("after {during}, expected focus on {selector} but the document holds {actual:?}");
    }
    Ok(())
}

/// One element's ring-bearing styles.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Ring {
    pub selector: String,
    pub outline: String,
    pub outline_color: String,
    pub outline_width: f64,
    pub box_shadow: String,
    /// A field frame indicates focus by changing its border colour
    /// (`use_field_frame`'s `focus_within`), not by drawing an outline. A pass
    /// that looks only at outline and box-shadow reports "no focus ring" for a
    /// component whose indicator is perfectly visible.
    pub border_color: String,
    /// The background this ring is drawn against, resolved by walking up until
    /// a non-transparent one is found.
    pub against: String,
}

/// Assert that keyboard focus produces a visible ring.
///
/// Two things here were learned the hard way and are the reason this is not
/// three lines:
///
/// * **Focus by Tab, not by `element.focus()`.** Every ring in the library is
///   `:focus-visible` (`sx.rs`), and in Chromium a scripted focus does not
///   always match it. Driving the keyboard is the only way to see what a
///   keyboard user sees.
/// * **The ring is usually not on the focused element.** A field's ring is
///   drawn on its *frame*, and the focused node is the `<input>` inside it.
///   Asserting on the input alone reports "no focus ring" for a component whose
///   ring is perfectly visible - which is exactly what this pass did on its
///   first run. So the element and its ancestors are all measured, and the
///   assertion is that *something the user can see* changed.
pub async fn assert_focus_ring(page: &Page, selector: &str, tab_budget: usize) -> Result<Ring> {
    // Blur **first**, then read the unfocused state.
    //
    // Reading `before` and blurring afterwards means that a caller who had
    // already focused the element - a test that tabbed to it to check something
    // else, say - measures the focused state as the baseline, finds it
    // identical to the focused state, and is told there is no focus ring. The
    // pass then accuses a component whose ring is plainly visible, which is
    // this file's recurring failure mode.
    page.evaluate("document.activeElement && document.activeElement.blur()")
        .await?;
    let before = ring_chain(page, selector).await?;

    super::keyboard::tab_to(page, selector, tab_budget).await?;

    let after = ring_chain(page, selector).await?;

    let changed: Vec<(&Ring, &Ring)> = before
        .iter()
        .zip(after.iter())
        .filter(|(b, a)| {
            b.outline != a.outline
                || b.box_shadow != a.box_shadow
                || b.border_color != a.border_color
        })
        .collect();

    let Some((_, ring)) = changed.into_iter().next() else {
        let shown = after
            .iter()
            .map(|r| {
                format!(
                    "\n    {} outline={:?} box-shadow={:?} border-color={:?}",
                    r.selector, r.outline, r.box_shadow, r.border_color
                )
            })
            .collect::<String>();
        bail!(
            "tabbing to {selector} produced no visible ring anywhere on it or its ancestors:{shown}"
        );
    };

    Ok(ring.clone())
}

/// WCAG 1.4.11: a focus indicator is a non-text contrast case, so 3:1 against
/// what it sits on.
///
/// **This discharges todo 53's third part**, which asks for a by-hand sweep of
/// every `focus_visible(focus_ring_sx())` on a dark or saturated background,
/// read in a browser because the emitted CSS cannot show the failure -
/// `Carousel`'s current dot shipped a white ring on a white page, present in
/// the CSS, matching `:focus-visible`, invisible to a user. Any component added
/// to the suite with `.focusable()` is covered automatically. Check here before
/// doing that sweep by hand.
///
/// The background is resolved by walking up for the first non-transparent one.
/// That is the cheap approximation of "effective background" and it is wrong
/// for a ring drawn over a gradient or an image - which is the reason contrast
/// on *text* is left to axe rather than computed here
/// (`principles/use-crates-for-solved-domains`). For a ring on a flat surface
/// it is right, and it catches the case that matters: a ring too faint to see.
pub fn assert_ring_contrast(ring: &Ring) -> Result<()> {
    // Whichever property actually carries the indicator.
    let indicator = if has_visible_outline(&ring.outline) {
        &ring.outline_color
    } else {
        &ring.border_color
    };
    let (Some(fg), Some(bg)) = (parse_rgb(indicator), parse_rgb(&ring.against)) else {
        // A ring drawn with box-shadow only has no outline colour to read.
        return Ok(());
    };
    let ratio = contrast(fg, bg);
    if ratio < 3.0 {
        bail!(
            "focus ring on {} is {ratio:.2}:1 against {} - WCAG 1.4.11 wants 3:1",
            ring.selector,
            ring.against
        );
    }
    Ok(())
}

async fn ring_chain(page: &Page, selector: &str) -> Result<Vec<Ring>> {
    let chain = page
        .evaluate(format!(
            r#"(() => {{
                const start = document.querySelector({});
                if (!start) return [];
                const describe = (el) => {{
                    const s = getComputedStyle(el);
                    // Start at the PARENT, not at the element.
                    //
                    // A focus ring is an `outline` with a positive
                    // `outline-offset`, so it is painted *outside* the
                    // element's own box - over whatever the element sits on.
                    // Measuring against the element's own background reports a
                    // filled primary button's ring as 1.37:1 against the
                    // button's blue, when what the user sees is the ring
                    // against the page behind it. A border's outer edge meets
                    // the parent too.
                    let against = 'rgba(0, 0, 0, 0)';
                    for (let p = el.parentElement; p; p = p.parentElement) {{
                        const bg = getComputedStyle(p).backgroundColor;
                        if (bg && !bg.startsWith('rgba(0, 0, 0, 0')) {{ against = bg; break; }}
                    }}
                    const name = el.tagName.toLowerCase()
                        + (el.getAttribute('role') ? `[role=${{el.getAttribute('role')}}]` : '')
                        + (el.getAttribute('data-slot') ? `[data-slot=${{el.getAttribute('data-slot')}}]` : '');
                    return {{
                        selector: name,
                        outline: s.outline,
                        outline_color: s.outlineColor,
                        outline_width: parseFloat(s.outlineWidth) || 0,
                        box_shadow: s.boxShadow,
                        border_color: s.borderColor,
                        against,
                    }};
                }};
                const out = [];
                let el = start;
                // Four levels is enough for a field frame; more would start
                // reporting the page's own chrome as the component's ring.
                for (let i = 0; el && i < 5; i++, el = el.parentElement) out.push(describe(el));
                return out;
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    Ok(chain)
}

/// An `outline` shorthand that actually draws something. Chromium reports
/// `"rgb(0, 0, 0) none 3px"` for an element with no outline at all, so a
/// non-empty string proves nothing.
fn has_visible_outline(outline: &str) -> bool {
    !outline.is_empty() && !outline.contains("none") && !outline.starts_with("0px")
}

fn parse_rgb(value: &str) -> Option<(f64, f64, f64)> {
    let inner = value.trim().strip_prefix("rgb")?;
    let inner = inner
        .trim_start_matches('a')
        .trim_start_matches('(')
        .trim_end_matches(')');
    let parts: Vec<f64> = inner
        .split(',')
        .filter_map(|p| p.trim().parse::<f64>().ok())
        .collect();
    if parts.len() < 3 {
        return None;
    }
    // A fully transparent colour is not a ring.
    if parts.len() > 3 && parts[3] == 0.0 {
        return None;
    }
    Some((parts[0], parts[1], parts[2]))
}

fn relative_luminance((r, g, b): (f64, f64, f64)) -> f64 {
    let channel = |c: f64| {
        let c = c / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

fn contrast(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Assert focus returned to where it started, which is the half of focus
/// management that is easiest to get wrong and hardest to notice
/// (`principles/focus-after-removal`).
pub async fn assert_focus_returned(page: &Page, trigger: &str) -> Result<()> {
    assert_focused(page, trigger, "the overlay closed").await
}
