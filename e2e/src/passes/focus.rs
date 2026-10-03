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

/// Whether focus is on the element matching `selector` right now; one read, no wait.
pub async fn is_focused(page: &Page, selector: &str) -> Result<bool> {
    Ok(page
        .evaluate(format!(
            "document.activeElement === document.querySelector({})",
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?)
}

/// Assert focus is on the element matching `selector`. One read: after an action, use
/// [`wait_for_focus`].
pub async fn assert_focused(page: &Page, selector: &str, during: &str) -> Result<()> {
    if !is_focused(page, selector).await? {
        let actual = active_element(page).await?;
        bail!("after {during}, expected focus on {selector} but the document holds {actual:?}");
    }
    Ok(())
}

/// Wait for focus to land on `selector`, for a repair made in an effect after
/// the render. On timeout it fails as `assert_focused` does, naming the holder.
pub async fn wait_for_focus(page: &Page, selector: &str, during: &str) -> Result<()> {
    let settled = crate::wait::until(&format!("focus on {selector} after {during}"), || {
        is_focused(page, selector)
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
    pub outline_style: String,
    pub outline_width: f64,
    pub box_shadow: String,
    /// A field frame shows focus by its border colour (`use_field_frame`'s `focus_within`).
    pub border_color: String,
    /// A `[data-ring]` overlay: the sibling drawing a field's or checkbox's keyboard ring
    /// (`ring_overlay` in `components/common/focus_ring.rs`).
    pub overlay: bool,
    /// The backgrounds walking up to the first opaque one, composited: what the ring is drawn against.
    pub against: String,
    /// The border box grown by the outline and spread shadows, viewport `[left, top, right, bottom]`.
    pub extent: [f64; 4],
    /// The box its clipping ancestors (`overflow`, `contain: paint`) leave visible, if any clip.
    pub clip: Option<[f64; 4]>,
}

impl Ring {
    /// An outline that draws something: a style other than none and a width.
    pub fn has_visible_outline(&self) -> bool {
        self.outline_width > 0.0 && !matches!(self.outline_style.as_str(), "none" | "hidden")
    }

    /// The edges where a clipping ancestor cuts the ring (WCAG 2.4.7), if any.
    pub fn clipped(&self) -> Option<String> {
        const SLACK: f64 = 1.0;
        let clip = self.clip?;
        let [x0, y0, x1, y1] = self.extent;
        let cut: Vec<&str> = [
            ("left", x0 < clip[0] - SLACK),
            ("top", y0 < clip[1] - SLACK),
            ("right", x1 > clip[2] + SLACK),
            ("bottom", y1 > clip[3] + SLACK),
        ]
        .into_iter()
        .filter_map(|(edge, cut)| cut.then_some(edge))
        .collect();
        (!cut.is_empty()).then(|| {
            format!(
                "the focus ring on {} ({:?}) is cut at its {} edge by a clipping ancestor \
                 ({clip:?}) - WCAG 2.4.7 wants the whole ring seen",
                self.selector,
                self.extent,
                cut.join(", ")
            )
        })
    }
}

/// `clipOf(el)`: the viewport box `el`'s clipping ancestors leave visible, or null. An absolute box
/// skips ancestors below its containing block, a fixed one all but a transformed one.
const CLIP_OF: &str = r#"const clipOf = el => {
    const box = [-1e9, -1e9, 1e9, 1e9];
    let clipped = false, held = el;
    for (let p = el.parentElement; p && p !== document.body && p !== document.documentElement; p = p.parentElement) {
        const pos = getComputedStyle(held).position, ps = getComputedStyle(p);
        const paint = /paint|strict|content/.test(ps.contain);
        const transformed = ps.transform !== 'none' || ps.filter !== 'none' || paint;
        if (pos === 'fixed' ? !transformed : pos === 'absolute' && ps.position === 'static' && !transformed) continue;
        held = p;
        const cx = paint || ps.overflowX !== 'visible', cy = paint || ps.overflowY !== 'visible';
        if (!cx && !cy) continue;
        clipped = true;
        const r = p.getBoundingClientRect();
        const x0 = r.left + p.clientLeft, y0 = r.top + p.clientTop;
        if (cx) { box[0] = Math.max(box[0], x0); box[2] = Math.min(box[2], x0 + p.clientWidth); }
        if (cy) { box[1] = Math.max(box[1], y0); box[3] = Math.min(box[3], y0 + p.clientHeight); }
    }
    return clipped ? box : null;
};"#;

/// Assert that tabbing (scripted focus may miss `:focus-visible`) draws a visible ring
/// on the element, an ancestor or a ring overlay, uncut and uncovered: a field's ring sits on its frame.
pub async fn assert_focus_ring(page: &Page, selector: &str, tab_budget: usize) -> Result<Ring> {
    let ring = focus_ring(page, selector, tab_budget).await?;
    if let Some(cut) = ring.clipped() {
        bail!("tabbing to {selector}: {cut}");
    }
    assert_focus_not_obscured(page, selector).await?;
    Ok(ring)
}

/// [`assert_focus_ring`] without its clip and cover checks, for a caller that waives them.
pub async fn focus_ring(page: &Page, selector: &str, tab_budget: usize) -> Result<Ring> {
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

/// WCAG 2.4.11: the focused element's visible centre, clamped to its scrollport, must hit the
/// element itself. A cover with `pointer-events: none` escapes this.
pub async fn assert_focus_not_obscured(page: &Page, during: &str) -> Result<()> {
    let problem: String = page
        .evaluate(format!(
            r#"(() => {{ {CLIP_OF}
                const name = el => el.tagName.toLowerCase() + (el.id ? '#' + el.id : '')
                    + (el.getAttribute('role') ? `[role=${{el.getAttribute('role')}}]` : '');
                const el = document.activeElement;
                if (!el || el === document.body) return 'nothing holds focus';
                // A visually hidden input shows its focus on the box around it.
                let subject = el;
                const tiny = e => {{ const r = e.getBoundingClientRect(); return r.width < 2 || r.height < 2; }};
                while (tiny(subject) && subject.parentElement) subject = subject.parentElement;
                const r = subject.getBoundingClientRect(), c = clipOf(subject) || [-1e9, -1e9, 1e9, 1e9];
                const x0 = Math.max(r.left, c[0], 0), y0 = Math.max(r.top, c[1], 0);
                const x1 = Math.min(r.right, c[2], innerWidth), y1 = Math.min(r.bottom, c[3], innerHeight);
                if (x1 - x0 < 1 || y1 - y0 < 1)
                    return `the focused ${{name(el)}} lies outside its scrollport or the viewport`;
                const hit = document.elementFromPoint((x0 + x1) / 2, (y0 + y1) / 2);
                if (!hit) return `the focused ${{name(el)}} has nothing at its centre`;
                if (subject.contains(hit) || [...(el.labels || [])].some(l => l.contains(hit))) return '';
                // Only a painted layer hides it: a transparent drag surface over it does not.
                const alpha = c => {{ const m = c.match(/[\d.]+/g); return m && m.length > 3 ? +m[3] : 1; }};
                const paints = e => {{
                    const s = getComputedStyle(e);
                    return /^(img|video|canvas|iframe|svg)$/i.test(e.tagName) || s.backgroundImage !== 'none'
                        || alpha(s.backgroundColor) > 0;
                }};
                for (let p = hit; p && !p.contains(subject); p = p.parentElement)
                    if (paints(p)) return `the focused ${{name(el)}} is covered at its centre by ${{name(p)}}`;
                return '';
            }})()"#
        ))
        .await?
        .into_value()?;
    // A null result reads as no value at all: '' is the all-clear.
    if !problem.is_empty() {
        bail!("tabbing to {during}: {problem} - WCAG 2.4.11 wants focus not obscured");
    }
    Ok(())
}

/// The level of a [`ring_chain`] whose styles changed on focus into something drawn. An outline or
/// shadow wins; a border change counts only without a `[data-ring]` overlay (review 7, E4).
pub fn pick_ring<'a>(before: &[Ring], after: &'a [Ring]) -> Option<&'a Ring> {
    let pairs: Vec<(&Ring, &Ring)> = before.iter().zip(after.iter()).collect();
    let drawn = pairs.iter().find(|(b, a)| {
        (b.outline != a.outline && a.has_visible_outline())
            || (b.box_shadow != a.box_shadow && a.box_shadow != "none")
    });
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
    let Some(bg) = parse_rgba(&ring.against).map(|c| over(c, WHITE)) else {
        bail!(
            "the surface behind the focus ring on {} cannot be read ({:?})",
            ring.selector,
            ring.against
        );
    };

    // A two-tone ring (`focus_ring_sx`) brings its own surround: stripe against halo. One
    // tone must still hold 3:1 against the page, or the ring as a whole fades into it.
    if ring.has_visible_outline()
        && let Some(halo) = halo_color(&ring.box_shadow)
    {
        let (Some(stripe), Some(halo_rgba)) = (parse_rgba(&ring.outline_color), parse_rgba(halo))
        else {
            bail!(
                "the two-tone focus ring on {} has a tone this pass cannot read \
                 (stripe {:?}, halo {halo:?}), so its contrast is unmeasured",
                ring.selector,
                ring.outline_color
            );
        };
        // The outline paints over the halo, the halo over the surface.
        let halo_rgb = over(halo_rgba, bg);
        let stripe_rgb = over(stripe, halo_rgb);
        let ratio = contrast(stripe_rgb, halo_rgb);
        if ratio < 3.0 {
            bail!(
                "the two-tone focus ring on {} is {ratio:.2}:1 between its stripe ({}) and its \
                 halo ({halo}) - the pair is what carries the indicator, so it wants 3:1",
                ring.selector,
                ring.outline_color
            );
        }
        let surround = contrast(stripe_rgb, bg).max(contrast(halo_rgb, bg));
        if surround < 3.0 {
            bail!(
                "the two-tone focus ring on {} has neither tone at 3:1 against {} (best \
                 {surround:.2}:1) - WCAG 1.4.11 wants one of them to stand out",
                ring.selector,
                ring.against
            );
        }
        return Ok(());
    }

    // Whichever property actually carries the indicator. An inset ring's
    // outline is transparent (forced colours only): its stripe is the deepest inset band.
    let transparent = parse_rgba(&ring.outline_color).is_some_and(|(.., a)| a == 0.0);
    let indicator = if ring.has_visible_outline() && !transparent {
        ring.outline_color.as_str()
    } else if let Some(stripe) = inset_stripe_color(&ring.box_shadow) {
        stripe
    } else if ring.box_shadow != "none" {
        shadow_color(&ring.box_shadow)
    } else {
        ring.border_color.as_str()
    };
    // An unreadable or fully transparent colour fails: passing it once left box-shadow rings unmeasured.
    let Some(fg) = parse_rgba(indicator).filter(|&(.., a)| a > 0.0) else {
        bail!(
            "the focus indicator on {} has a colour this pass cannot read ({indicator:?}), \
             so its contrast is unmeasured",
            ring.selector
        );
    };
    // A translucent ring shows the surface through it.
    let ratio = contrast(over(fg, bg), bg);
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
            r#"(() => {{ {CLIP_OF}
                const start = document.querySelector({});
                if (!start) return [];
                const px = v => parseFloat(v) || 0;
                const describe = (el, overlay) => {{
                    const s = getComputedStyle(el);
                    // An offset ring is painted over the parent; an overlay's over its owner's parent;
                    // an inset ring (`inset_focus_ring_sx`) over the element itself.
                    const inset = s.outlineStyle !== 'none' && px(s.outlineOffset) + px(s.outlineWidth) <= 0;
                    const from = overlay
                        ? (el.offsetParent || el.parentElement).parentElement
                        : inset ? el : el.parentElement;
                    // Translucent layers down to the first opaque one, composited over it (or the white canvas).
                    let against = 'rgba(0, 0, 0, 0)';
                    const layers = [];
                    for (let p = from; p; p = p.parentElement) {{
                        const bg = getComputedStyle(p).backgroundColor;
                        if (!bg.startsWith('rgb')) {{ against = bg; layers.length = 0; break; }}
                        const [r, g, b, a = 1] = bg.match(/[\d.]+/g).map(Number);
                        if (a === 0) continue;
                        layers.push([r, g, b, a]);
                        if (a >= 1) break;
                    }}
                    if (layers.length) {{
                        let rgb = [255, 255, 255];
                        for (const [r, g, b, a] of layers.reverse())
                            rgb = [r, g, b].map((c, i) => c * a + rgb[i] * (1 - a));
                        against = `rgb(${{rgb.map(c => Math.round(c)).join(', ')}})`;
                    }}
                    // How far the indicator reaches past the border box: the outline, else spread
                    // shadows. A two-tone ring's halo may be cut where its stripe is whole.
                    let reach = 0;
                    const outlined = !['none', 'hidden'].includes(s.outlineStyle) && px(s.outlineWidth) > 0;
                    if (outlined) reach = px(s.outlineOffset) + px(s.outlineWidth);
                    for (const layer of outlined || s.boxShadow === 'none' ? [] : s.boxShadow.split(/,(?![^(]*\))/)) {{
                        if (layer.includes('inset')) continue;
                        const [x = 0, y = 0, , spread = 0] = (layer.replace(/rgba?\([^)]*\)/, '').match(/-?[\d.]+px/g) || []).map(parseFloat);
                        if (spread > 0) reach = Math.max(reach, spread + Math.max(Math.abs(x), Math.abs(y)));
                    }}
                    reach = Math.max(0, reach);
                    const r = el.getBoundingClientRect();
                    const name = el.tagName.toLowerCase()
                        + (el.getAttribute('role') ? `[role=${{el.getAttribute('role')}}]` : '')
                        + (el.getAttribute('data-slot') ? `[data-slot=${{el.getAttribute('data-slot')}}]` : '');
                    return {{
                        selector: name,
                        outline: s.outline,
                        outline_color: s.outlineColor,
                        outline_style: s.outlineStyle,
                        outline_width: px(s.outlineWidth),
                        box_shadow: s.boxShadow,
                        border_color: s.borderColor,
                        overlay,
                        against,
                        extent: [r.left - reach, r.top - reach, r.right + reach, r.bottom + reach],
                        clip: clipOf(el),
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

const WHITE: (f64, f64, f64) = (255.0, 255.0, 255.0);

/// `rgb()` or `rgba()`, comma or space separated, with its alpha (1 when absent).
fn parse_rgba(value: &str) -> Option<(f64, f64, f64, f64)> {
    let inner = value.trim().strip_prefix("rgb")?;
    let inner = inner
        .trim_start_matches('a')
        .strip_prefix('(')?
        .strip_suffix(')')?;
    let parts = inner
        .split([',', ' ', '/'])
        .filter(|p| !p.is_empty())
        .map(|p| p.parse::<f64>().ok())
        .collect::<Option<Vec<f64>>>()?;
    match parts[..] {
        [r, g, b] => Some((r, g, b, 1.0)),
        [r, g, b, a] => Some((r, g, b, a)),
        _ => None,
    }
}

/// `top` composited over the opaque `bottom`.
fn over((r, g, b, a): (f64, f64, f64, f64), bottom: (f64, f64, f64)) -> (f64, f64, f64) {
    let mix = |top: f64, under: f64| top * a + under * (1.0 - a);
    (mix(r, bottom.0), mix(g, bottom.1), mix(b, bottom.2))
}

/// The sRGB curve `contrast::COLOUR_JS` uses: Rust reads the ring's parsed `rgb()` here.
fn relative_luminance((r, g, b): (f64, f64, f64)) -> f64 {
    let channel = |c: f64| {
        let c = c / 255.0;
        if c <= 0.04045 {
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

    /// Style and width come from the shorthand's last two words, as Chromium writes it.
    fn ring(outline: &str, outline_color: &str, box_shadow: &str) -> Ring {
        let words: Vec<&str> = outline.split_whitespace().collect();
        Ring {
            selector: "x".into(),
            outline: outline.into(),
            outline_color: outline_color.into(),
            outline_style: words[words.len() - 2].into(),
            outline_width: words[words.len() - 1]
                .trim_end_matches("px")
                .parse()
                .unwrap(),
            box_shadow: box_shadow.into(),
            border_color: "rgb(0, 0, 0)".into(),
            overlay: false,
            against: "rgb(255, 255, 255)".into(),
            extent: [10.0, 10.0, 50.0, 30.0],
            clip: None,
        }
    }

    #[test]
    fn a_zero_width_outline_is_no_ring() {
        let zero = ring("rgb(0, 0, 0) solid 0px", "rgb(0, 0, 0)", "none");
        assert!(!zero.has_visible_outline());
        let mut rest = zero.clone();
        rest.outline = "rgb(0, 0, 0) none 3px".into();
        rest.outline_style = "none".into();
        assert_eq!(pick_ring(&[rest], &[zero]), None);
        assert!(ring("rgb(0, 0, 0) solid 2px", "rgb(0, 0, 0)", "none").has_visible_outline());
    }

    #[test]
    fn colours_keep_their_alpha_and_composite() {
        assert_eq!(parse_rgba("rgb(1, 2, 3)"), Some((1.0, 2.0, 3.0, 1.0)));
        assert_eq!(parse_rgba("rgba(0, 0, 0, 0.5)"), Some((0.0, 0.0, 0.0, 0.5)));
        assert_eq!(parse_rgba("rgb(0 0 0 / 0.25)"), Some((0.0, 0.0, 0.0, 0.25)));
        assert_eq!(parse_rgba("oklch(0.5 0.1 200)"), None);
        assert_eq!(over((0.0, 0.0, 0.0, 0.5), WHITE), (127.5, 127.5, 127.5));
    }

    #[test]
    fn a_translucent_ring_is_measured_as_it_shows() {
        // Solid black would be 21:1; at 10% it is a faint grey on the white surface.
        let faint = ring("rgba(0, 0, 0, 0.1) solid 2px", "rgba(0, 0, 0, 0.1)", "none");
        assert!(assert_ring_contrast(&faint).is_err());
        let mut tinted = ring("rgb(0, 0, 0) solid 2px", "rgb(0, 0, 0)", "none");
        tinted.against = "rgba(0, 0, 0, 0.5)".into();
        assert!(assert_ring_contrast(&tinted).is_ok());
    }

    #[test]
    fn a_two_tone_ring_needs_one_tone_against_the_surround() {
        // 6.6:1 between the tones, but each under 3:1 against a mid grey.
        let mut grey = ring(
            "rgb(60, 60, 60) solid 2px",
            "rgb(60, 60, 60)",
            "rgb(200, 200, 200) 0px 0px 0px 6px",
        );
        grey.against = "rgb(128, 128, 128)".into();
        let error = assert_ring_contrast(&grey).unwrap_err();
        assert!(format!("{error}").contains("neither tone"), "{error}");
        // A translucent halo shows the surface through it.
        let mut faint_halo = ring(
            "rgb(0, 0, 0) solid 2px",
            "rgb(0, 0, 0)",
            "rgba(255, 255, 255, 0.1) 0px 0px 0px 6px",
        );
        faint_halo.against = "rgb(0, 0, 0)".into();
        assert!(assert_ring_contrast(&faint_halo).is_err());
    }

    #[test]
    fn a_ring_past_a_clipping_ancestor_is_clipped() {
        let mut cut = ring("rgb(0, 0, 0) solid 2px", "rgb(0, 0, 0)", "none");
        cut.clip = Some([12.0, 0.0, 100.0, 100.0]);
        assert!(cut.clipped().unwrap().contains("left edge"));
        cut.clip = Some([10.0, 9.5, 50.0, 30.0]);
        assert_eq!(cut.clipped(), None);
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
