//! Focus management and the focus ring (WCAG 2.4.7, 1.4.11, 2.4.3, 3.2.1).
//! Always read `document.activeElement`: `focus()` on a hidden element succeeds silently (`codebase/use-popover`).

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

/// Wait for focus to land on `selector`, for a repair made in an effect after
/// the render. On timeout it fails as `assert_focused` does, naming the holder.
pub async fn wait_for_focus(page: &Page, selector: &str, during: &str) -> Result<()> {
    let settled = crate::wait::until(&format!("focus on {selector} after {during}"), || async {
        Ok(assert_focused(page, selector, during).await.is_ok())
    })
    .await;
    match settled {
        Ok(()) => Ok(()),
        Err(_) => assert_focused(page, selector, during).await,
    }
}

/// One element's ring-bearing styles.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Ring {
    pub selector: String,
    pub outline: String,
    pub outline_color: String,
    pub outline_width: f64,
    pub box_shadow: String,
    /// A field frame shows focus by its border colour (`use_field_frame`'s `focus_within`).
    pub border_color: String,
    /// A `[data-ring]` overlay: the sibling drawing a field's or checkbox's keyboard ring
    /// (`ring_overlay` in `components/common/focus_ring.rs`).
    pub overlay: bool,
    /// The first non-transparent background walking up, which the ring is drawn against.
    pub against: String,
}

/// Assert that tabbing (scripted focus may miss `:focus-visible`) draws a visible ring
/// on the element, an ancestor or a ring overlay: a field's ring sits on its frame.
pub async fn assert_focus_ring(page: &Page, selector: &str, tab_budget: usize) -> Result<Ring> {
    // Blur first: an already focused element would make `before` the focused state.
    page.evaluate("document.activeElement && document.activeElement.blur()")
        .await?;
    let before = ring_chain(page, selector).await?;

    super::keyboard::tab_to(page, selector, tab_budget).await?;

    let after = ring_chain(page, selector).await?;

    let Some(ring) = pick_ring(&before, &after) else {
        let has_overlay = after.iter().any(|r| r.overlay);
        let shown = after
            .iter()
            .map(|r| {
                format!(
                    "\n    {}{} outline={:?} box-shadow={:?} border-color={:?}",
                    r.selector,
                    if r.overlay { " (ring overlay)" } else { "" },
                    r.outline,
                    r.box_shadow,
                    r.border_color
                )
            })
            .collect::<String>();
        bail!(
            "tabbing to {selector} produced no visible ring anywhere on it, its ancestors or \
             their ring overlays{}:{shown}",
            if has_overlay {
                " (a border change does not count where a [data-ring] overlay exists)"
            } else {
                ""
            }
        );
    };

    Ok(ring.clone())
}

/// The level of a [`ring_chain`] whose styles changed on focus. An outline or shadow wins;
/// a border change counts only without a `[data-ring]` overlay (review 7, E4).
pub fn pick_ring<'a>(before: &[Ring], after: &'a [Ring]) -> Option<&'a Ring> {
    let pairs: Vec<(&Ring, &Ring)> = before.iter().zip(after.iter()).collect();
    let drawn = pairs
        .iter()
        .find(|(b, a)| b.outline != a.outline || b.box_shadow != a.box_shadow);
    let has_overlay = after.iter().any(|r| r.overlay);
    let bordered = pairs
        .iter()
        .find(|(b, a)| b.border_color != a.border_color)
        .filter(|_| !has_overlay);
    drawn.or(bordered).map(|(_, ring)| *ring)
}

/// WCAG 1.4.11: the ring needs 3:1 against the first flat background behind it (todo 53, part 3).
/// Wrong over gradients or images, which is why text contrast is left to axe.
pub fn assert_ring_contrast(ring: &Ring) -> Result<()> {
    // A two-tone ring (`focus_ring_sx`) brings its own surround: stripe against halo, not the page.
    if has_visible_outline(&ring.outline)
        && let Some(halo) = halo_color(&ring.box_shadow)
    {
        let (Some(stripe), Some(halo_rgb)) = (parse_rgb(&ring.outline_color), parse_rgb(halo))
        else {
            bail!(
                "the two-tone focus ring on {} has a tone this pass cannot read \
                 (stripe {:?}, halo {halo:?}), so its contrast is unmeasured",
                ring.selector,
                ring.outline_color
            );
        };
        let ratio = contrast(stripe, halo_rgb);
        if ratio < 3.0 {
            bail!(
                "the two-tone focus ring on {} is {ratio:.2}:1 between its stripe ({}) and its \
                 halo ({halo}) - the pair is what carries the indicator, so it wants 3:1",
                ring.selector,
                ring.outline_color
            );
        }
        return Ok(());
    }

    // Whichever property actually carries the indicator. An inset ring's
    // outline is transparent (forced colours only): its stripe is the deepest inset band.
    let transparent = ring.outline_color.replace(' ', "").ends_with(",0)");
    let indicator = if has_visible_outline(&ring.outline) && !transparent {
        ring.outline_color.as_str()
    } else if let Some(stripe) = inset_stripe_color(&ring.box_shadow) {
        stripe
    } else if ring.box_shadow != "none" {
        shadow_color(&ring.box_shadow)
    } else {
        ring.border_color.as_str()
    };
    // An unreadable colour fails: passing it once left box-shadow rings unmeasured.
    let Some(fg) = parse_rgb(indicator) else {
        bail!(
            "the focus indicator on {} has a colour this pass cannot read ({indicator:?}), \
             so its contrast is unmeasured",
            ring.selector
        );
    };
    let Some(bg) = parse_rgb(&ring.against) else {
        bail!(
            "the surface behind the focus ring on {} cannot be read ({:?})",
            ring.selector,
            ring.against
        );
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

/// The ring-bearing styles of `selector`, its ancestors and their ring overlays.
pub async fn ring_chain(page: &Page, selector: &str) -> Result<Vec<Ring>> {
    let chain = page
        .evaluate(format!(
            r#"(() => {{
                const start = document.querySelector({});
                if (!start) return [];
                const describe = (el, overlay) => {{
                    const s = getComputedStyle(el);
                    // An offset ring is painted over the parent; an overlay's over its owner's parent;
                    // an inset ring (`inset_focus_ring_sx`) over the element itself.
                    let against = 'rgba(0, 0, 0, 0)';
                    const inset = s.outlineStyle !== 'none' && (parseFloat(s.outlineOffset) || 0) + (parseFloat(s.outlineWidth) || 0) <= 0;
                    const from = overlay
                        ? (el.offsetParent || el.parentElement).parentElement
                        : inset ? el : el.parentElement;
                    for (let p = from; p; p = p.parentElement) {{
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
                        overlay,
                        against,
                    }};
                }};
                const out = [];
                let el = start;
                // Five levels reach a field frame without the page's chrome,
                // plus each level's following `[data-ring]` siblings.
                for (let i = 0; el && i < 5; i++, el = el.parentElement) {{
                    out.push(describe(el, false));
                    // A ring drawn on a marked child line (Tree's row, whose `<li>` holds the subtree).
                    if (i === 0)
                        for (const child of el.querySelectorAll(':scope > [data-ring]')) out.push(describe(child, false));
                    for (let sib = el.nextElementSibling; sib; sib = sib.nextElementSibling)
                        if (sib.hasAttribute('data-ring')) out.push(describe(sib, true));
                }}
                return out;
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    Ok(chain)
}

/// An `outline` that draws something: Chromium reports `"rgb(0, 0, 0) none 3px"` for none.
fn has_visible_outline(outline: &str) -> bool {
    !outline.is_empty() && !outline.contains("none") && !outline.starts_with("0px")
}

/// The first shadow split into colour and lengths; the colour's own commas forbid a plain split.
fn first_shadow(shadow: &str) -> Option<(&str, &str)> {
    let start = shadow.find("rgb")?;
    let close = start + shadow[start..].find(')')?;
    let rest = &shadow[close + 1..];
    let end = rest.find(',').unwrap_or(rest.len());
    Some((&shadow[start..=close], rest[..end].trim()))
}

/// The halo colour if the first shadow is `focus_ring_sx`'s: outset, spread only, no offset or blur.
fn halo_color(shadow: &str) -> Option<&str> {
    let (color, lengths) = first_shadow(shadow)?;
    // Chromium puts the keyword after the lengths: `rgb(0, 0, 0) 0px 0px 0px 1px inset`.
    if lengths.contains("inset") {
        return None;
    }
    let px: Vec<f64> = lengths
        .split_whitespace()
        .filter_map(|p| p.trim_end_matches("px").parse::<f64>().ok())
        .collect();
    match px[..] {
        [x, y, blur, spread] if x == 0.0 && y == 0.0 && blur == 0.0 && spread > 0.0 => Some(color),
        _ => None,
    }
}

/// The colour of the deepest inset shadow: `inset_focus_ring_sx` paints its
/// stripe as the deepest band, a halo band over its outer part.
fn inset_stripe_color(shadow: &str) -> Option<&str> {
    let mut rest = shadow;
    let mut deepest: Option<(f64, &str)> = None;
    while let Some(start) = rest.find("rgb") {
        let close = start + rest[start..].find(')')?;
        let color = &rest[start..=close];
        let tail = &rest[close + 1..];
        let end = tail.find(',').unwrap_or(tail.len());
        let lengths = &tail[..end];
        if lengths.contains("inset") {
            let depth = lengths
                .split_whitespace()
                .filter_map(|p| p.trim_end_matches("px").parse::<f64>().ok())
                .fold(0.0, |max: f64, px| max.max(px.abs()));
            if depth > 0.0 && deepest.is_none_or(|(most, _)| depth > most) {
                deepest = Some((depth, color));
            }
        }
        rest = &tail[end..];
    }
    deepest.map(|(_, color)| color)
}

/// The colour of the first shadow, as Chromium computes it:
/// `rgb(34, 139, 230) 0px 0px 0px 2px`.
fn shadow_color(shadow: &str) -> &str {
    shadow
        .find("rgb")
        .and_then(|start| {
            let end = shadow[start..].find(')')?;
            Some(&shadow[start..=start + end])
        })
        .unwrap_or(shadow)
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

/// Assert focus returned to the trigger (`principles/focus-after-removal`).
pub async fn assert_focus_returned(page: &Page, trigger: &str) -> Result<()> {
    assert_focused(page, trigger, "the overlay closed").await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring(outline: &str, outline_color: &str, box_shadow: &str) -> Ring {
        Ring {
            selector: "x".into(),
            outline: outline.into(),
            outline_color: outline_color.into(),
            outline_width: 2.0,
            box_shadow: box_shadow.into(),
            border_color: "rgb(0, 0, 0)".into(),
            overlay: false,
            against: "rgb(255, 255, 255)".into(),
        }
    }

    #[test]
    fn a_shadow_ring_is_measured_by_its_own_colour() {
        let faint = ring(
            "rgb(0, 0, 0) none 0px",
            "rgb(0, 0, 0)",
            "rgb(238, 238, 238) 0px 0px 0px 2px",
        );
        assert!(assert_ring_contrast(&faint).is_err());
        let strong = ring(
            "rgb(0, 0, 0) none 0px",
            "rgb(0, 0, 0)",
            "rgb(34, 139, 230) 0px 0px 0px 2px",
        );
        assert!(assert_ring_contrast(&strong).is_ok());
    }

    #[test]
    fn an_inset_ring_is_measured_by_its_deepest_band() {
        // Halo band 2px over a 4px stripe band, as a `-4px` inset ring computes.
        let shadow = "rgb(255, 255, 255) 2px 0px 0px 0px inset, rgb(255, 255, 255) -2px 0px 0px 0px inset, \
                      rgb(0, 0, 0) 4px 0px 0px 0px inset, rgb(0, 0, 0) -4px 0px 0px 0px inset, \
                      rgba(0, 0, 0, 0) 0px 0px 0px 0px";
        assert_eq!(inset_stripe_color(shadow), Some("rgb(0, 0, 0)"));
        let inset = ring("rgba(0, 0, 0, 0) solid 2px", "rgba(0, 0, 0, 0)", shadow);
        assert!(assert_ring_contrast(&inset).is_ok());
        let faint = ring(
            "rgba(0, 0, 0, 0) solid 2px",
            "rgba(0, 0, 0, 0)",
            "rgb(238, 238, 238) 2px 0px 0px 0px inset",
        );
        assert!(assert_ring_contrast(&faint).is_err());
    }

    #[test]
    fn a_two_tone_ring_is_measured_between_its_own_tones() {
        // A black stripe with a white halo passes wherever it lands, even
        // though the pass is told the surround is the same white.
        let mut two_tone = ring(
            "rgb(0, 0, 0) solid 2px",
            "rgb(0, 0, 0)",
            "rgb(255, 255, 255) 0px 0px 0px 6px, rgba(0, 0, 0, 0) 0px 0px 0px 0px",
        );
        assert!(assert_ring_contrast(&two_tone).is_ok());

        // Two tones too close together is the failure this rule can still see.
        two_tone.box_shadow = "rgb(60, 60, 60) 0px 0px 0px 6px".into();
        assert!(assert_ring_contrast(&two_tone).is_err());
    }

    #[test]
    fn a_resting_elevation_shadow_is_not_a_halo() {
        // Offset, blurred or inset: the component's own chrome. The ring is
        // then measured against the surround as before.
        assert_eq!(halo_color("rgb(0, 0, 0) 0px 2px 4px 0px"), None);
        assert_eq!(halo_color("rgb(0, 0, 0) 0px 0px 0px 1px inset"), None);
        assert_eq!(
            halo_color("rgb(255, 255, 255) 0px 0px 0px 6px"),
            Some("rgb(255, 255, 255)")
        );
    }

    #[test]
    fn an_unreadable_indicator_colour_fails() {
        let odd = ring("oklch(0.5 0.1 200) solid 2px", "oklch(0.5 0.1 200)", "none");
        assert!(assert_ring_contrast(&odd).is_err());
    }
}
