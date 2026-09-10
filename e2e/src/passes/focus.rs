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
    /// A field frame indicates focus by changing its border colour
    /// (`use_field_frame`'s `focus_within`), not by drawing an outline. A pass
    /// that looks only at outline and box-shadow reports "no focus ring" for a
    /// component whose indicator is perfectly visible.
    pub border_color: String,
    /// A `[data-ring]` overlay: the sibling that draws a field's or a
    /// checkbox's keyboard ring, since focus lands on a child of the box the
    /// ring belongs to (`ring_overlay` in `components/common/util.rs`).
    pub overlay: bool,
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

    // A drawn ring, outline or shadow, wins over a border that changed colour.
    // A field frame's border turns on `:focus-within`, which a click shows as
    // well, and the keyboard ring is drawn by its `[data-ring]` overlay. With
    // the border counted first, deleting the overlay's rule left every field
    // green (review 7, E4). So a border counts only where there is no overlay
    // to carry the ring.
    let pairs: Vec<(&Ring, &Ring)> = before.iter().zip(after.iter()).collect();
    let drawn = pairs
        .iter()
        .find(|(b, a)| b.outline != a.outline || b.box_shadow != a.box_shadow);
    let has_overlay = after.iter().any(|r| r.overlay);
    let bordered = pairs
        .iter()
        .find(|(b, a)| b.border_color != a.border_color)
        .filter(|_| !has_overlay);

    let Some((_, ring)) = drawn.or(bordered) else {
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

    Ok((*ring).clone())
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
    // A two-tone ring brings its own surround.
    //
    // `focus_ring_sx` paints a light halo on both sides of a dark stripe, so
    // whatever the caller's surface turns out to be, the stripe has the halo
    // next to it. WCAG's adjacent-colour rule is then discharged by the two
    // tones against each other, and measuring the stripe against the page
    // behind it asks a question the pattern deliberately does not answer -
    // `Splitter`'s divider draws its ring entirely over the caller's panes.
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

    // Whichever property actually carries the indicator.
    let indicator = if has_visible_outline(&ring.outline) {
        ring.outline_color.as_str()
    } else if ring.box_shadow != "none" {
        shadow_color(&ring.box_shadow)
    } else {
        ring.border_color.as_str()
    };
    // An indicator colour that cannot be read is a failure, not a pass. This
    // returned Ok once, so a box-shadow ring was never measured at all.
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

async fn ring_chain(page: &Page, selector: &str) -> Result<Vec<Ring>> {
    let chain = page
        .evaluate(format!(
            r#"(() => {{
                const start = document.querySelector({});
                if (!start) return [];
                const describe = (el, overlay) => {{
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
                    //
                    // An overlay covers its owner, its containing block, so its
                    // ring sits outside the owner: start at the owner's parent.
                    //
                    // An inset ring (`inset_focus_ring_sx`) lies wholly on the
                    // element's own fill: start at the element.
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
                // Four levels is enough for a field frame; more would start
                // reporting the page's own chrome as the component's ring.
                //
                // Each level's following `[data-ring]` siblings too: that is
                // where a field draws its keyboard ring.
                for (let i = 0; el && i < 5; i++, el = el.parentElement) {{
                    out.push(describe(el, false));
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

/// An `outline` shorthand that actually draws something. Chromium reports
/// `"rgb(0, 0, 0) none 3px"` for an element with no outline at all, so a
/// non-empty string proves nothing.
fn has_visible_outline(outline: &str) -> bool {
    !outline.is_empty() && !outline.contains("none") && !outline.starts_with("0px")
}

/// The first shadow in the list, split into its colour and its lengths.
/// Chromium computes one as `rgb(34, 139, 230) 0px 0px 0px 2px`, and a colour
/// carries commas of its own, so the list cannot be split on `,` alone.
fn first_shadow(shadow: &str) -> Option<(&str, &str)> {
    let start = shadow.find("rgb")?;
    let close = start + shadow[start..].find(')')?;
    let rest = &shadow[close + 1..];
    let end = rest.find(',').unwrap_or(rest.len());
    Some((&shadow[start..=close], rest[..end].trim()))
}

/// The colour of a two-tone ring's halo, if the first shadow is one.
///
/// The halo is spread-only and outset - no offset, no blur - which is the
/// shape [`focus_ring_sx`](../../../libero/src/components/common/util.rs)
/// emits and which a resting elevation shadow never has. An inset shadow or
/// one with an offset is the component's own chrome, not a ring.
fn halo_color(shadow: &str) -> Option<&str> {
    let (color, lengths) = first_shadow(shadow)?;
    // Chromium puts the keyword after the lengths: `rgb(0, 0, 0) 0px 0px 0px
    // 1px inset`.
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

/// Assert focus returned to where it started, which is the half of focus
/// management that is easiest to get wrong and hardest to notice
/// (`principles/focus-after-removal`).
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
