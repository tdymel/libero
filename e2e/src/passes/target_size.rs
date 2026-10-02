//! WCAG 2.5.8 target size (minimum), 24x24 CSS px, and its opt-in spacing exception.
//! The selector must name the press target (a slider's track, not its thumb), not what is drawn.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use serde::Deserialize;

pub const MINIMUM: f64 = 24.0;

/// The radius of the spacing exception's circle on an undersized target.
const RADIUS: f64 = MINIMUM / 2.0;

/// Slack for the browser's sub-pixel geometry, well under the half a pixel
/// that made `RadioGroup`'s rows fail (todo 302).
const EPSILON: f64 = 0.01;

/// Everything counting as "another target" for the exception; generous on purpose.
pub const TARGETS: &str = "a[href], button, input:not([type=hidden]), select, textarea, summary, \
                       [role=button], [role=link], [role=checkbox], [role=radio], [role=switch], \
                       [role=tab], [role=menuitem], [role=menuitemcheckbox], \
                       [role=menuitemradio], [role=option], [role=slider], [role=spinbutton], \
                       [role=treeitem], [tabindex]:not([tabindex='-1'])";

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
    /// Viewport-relative; only the spacing exception reads the position.
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
}

impl Size {
    fn centre(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    fn undersized(&self) -> bool {
        self.width < MINIMUM || self.height < MINIMUM
    }

    /// How far `(x, y)` lies from this box, 0 inside it.
    fn distance_from(&self, (x, y): (f64, f64)) -> f64 {
        let dx = (self.x - x).max(x - (self.x + self.width)).max(0.0);
        let dy = (self.y - y).max(y - (self.y + self.height)).max(0.0);
        dx.hypot(dy)
    }
}

/// One target and the other targets close enough to matter to it.
#[derive(Debug, Deserialize)]
pub struct Spaced {
    pub target: Size,
    /// Other targets within reach of the circle; ancestors and descendants are the same target.
    pub neighbours: Vec<Size>,
}

/// `box(el)`: `el`'s press target, grown by a positioned, unclipped element's `::before`
/// (`ActionIcon`'s invisible 24x24 hit area, todos 505, 566).
const PRESS_BOX: &str = r#"const box = el => {
    const r = el.getBoundingClientRect();
    let [x0, y0, x1, y1] = [r.x, r.y, r.right, r.bottom];
    const s = getComputedStyle(el), b = getComputedStyle(el, '::before');
    if (b.content !== 'none' && b.position === 'absolute' && b.pointerEvents !== 'none'
        && s.position !== 'static' && s.overflowX === 'visible' && s.overflowY === 'visible') {
        const px = v => parseFloat(v) || 0;
        x0 = Math.min(x0, r.x + px(s.borderLeftWidth) + px(b.left));
        y0 = Math.min(y0, r.y + px(s.borderTopWidth) + px(b.top));
        x1 = Math.max(x1, r.right - px(s.borderRightWidth) - px(b.right));
        y1 = Math.max(y1, r.bottom - px(s.borderBottomWidth) - px(b.bottom));
    }
    return { width: x1 - x0, height: y1 - y0, x: x0, y: y0 };
};"#;

/// Measure every element matching `selector`.
pub async fn measure_all(page: &Page, selector: &str) -> Result<Vec<Size>> {
    let sizes = page
        .evaluate(format!(
            r#"(() => {{ {PRESS_BOX}
                return [...document.querySelectorAll({})].map(box);
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    Ok(sizes)
}

/// Measure every match of `selector`, each with the targets it has to clear.
pub async fn measure_spacing(page: &Page, selector: &str) -> Result<Vec<Spaced>> {
    let measured = page
        .evaluate(format!(
            r#"(() => {{ {PRESS_BOX}
                const shown = el => {{
                    const r = el.getBoundingClientRect();
                    if (r.width <= 0 || r.height <= 0) return false;
                    const style = getComputedStyle(el);
                    return style.visibility !== 'hidden' && style.pointerEvents !== 'none';
                }};
                // The declared targets are neighbours too; `TARGETS` may not see them.
                const all = [...new Set([
                    ...document.querySelectorAll({targets}),
                    ...document.querySelectorAll({selector}),
                ])].filter(shown);
                return [...document.querySelectorAll({selector})].map(el => {{
                    const target = box(el);
                    // Only what a {minimum}px circle on the target could touch.
                    const reach = {minimum};
                    const neighbours = all
                        .filter(other => other !== el && !other.contains(el) && !el.contains(other))
                        .map(box)
                        .filter(other =>
                            other.x < target.x + target.width + reach &&
                            other.x + other.width > target.x - reach &&
                            other.y < target.y + target.height + reach &&
                            other.y + other.height > target.y - reach);
                    return {{ target, neighbours }};
                }});
            }})()"#,
            targets = serde_json::to_string(TARGETS)?,
            selector = serde_json::to_string(selector)?,
            minimum = MINIMUM,
        ))
        .await?
        .into_value()?;
    Ok(measured)
}

/// Fail if any match is smaller than 24x24.
pub async fn assert_minimum(page: &Page, selector: &str) -> Result<()> {
    let sizes = measure_all(page, selector).await?;
    if sizes.is_empty() {
        bail!("target size: nothing matched {selector}");
    }
    assert_sizes(selector, &sizes)
}

/// Fail if any match is smaller than 24x24 **and** the spacing exception does
/// not cover it.
pub async fn assert_minimum_or_spacing(page: &Page, selector: &str) -> Result<()> {
    let measured = measure_spacing(page, selector).await?;
    if measured.is_empty() {
        bail!("target size: nothing matched {selector}");
    }
    assert_sizes_spaced(selector, &measured)
}

/// The assertion on measurements already taken, where "nothing matched" is no failure.
pub fn assert_sizes(selector: &str, sizes: &[Size]) -> Result<()> {
    let small: Vec<_> = sizes.iter().filter(|s| s.undersized()).collect();
    if !small.is_empty() {
        bail!("target size below WCAG 2.5.8 minimum ({MINIMUM}px) for {selector}: {small:?}");
    }
    Ok(())
}

/// 2.5.8 with the spacing exception: an undersized target's 24px circle may reach no other target
/// (12px from its box) nor another undersized target's circle (centres 24px apart).
pub fn assert_sizes_spaced(selector: &str, measured: &[Spaced]) -> Result<()> {
    for Spaced { target, neighbours } in measured {
        if !target.undersized() {
            continue;
        }
        let centre = target.centre();
        for neighbour in neighbours {
            // An undersized neighbour's own box counts too: a long thin bar has a far centre.
            let clearance = neighbour.distance_from(centre);
            if clearance < RADIUS - EPSILON {
                bail!(
                    "target size: WCAG 2.5.8's spacing exception fails for {selector}: the \
                     {MINIMUM}px circle on a {}x{} target reaches a {}x{} target \
                     {clearance:.2}px away, where it needs {RADIUS}px of clearance",
                    target.width,
                    target.height,
                    neighbour.width,
                    neighbour.height
                );
            }
            if neighbour.undersized() {
                let (nx, ny) = neighbour.centre();
                let pitch = (centre.0 - nx).hypot(centre.1 - ny);
                if pitch < MINIMUM - EPSILON {
                    bail!(
                        "target size: WCAG 2.5.8's spacing exception fails for {selector}: a \
                         {}x{} target and a {}x{} target sit {pitch:.2}px apart, centre to \
                         centre, where an undersized target needs {MINIMUM}px",
                        target.width,
                        target.height,
                        neighbour.width,
                        neighbour.height
                    );
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn size(x: f64, y: f64, width: f64, height: f64) -> Size {
        Size {
            width,
            height,
            x,
            y,
        }
    }

    #[test]
    fn an_undersized_bar_beside_a_small_target_fails_by_its_box() {
        // Centres 65px apart, but the bar's box sits 5px from the button's centre.
        let measured = [Spaced {
            target: size(0.0, 0.0, 10.0, 10.0),
            neighbours: vec![size(10.0, 0.0, 120.0, 12.0)],
        }];
        let error = assert_sizes_spaced("#x", &measured).unwrap_err();
        assert!(
            format!("{error}").contains("needs 12px of clearance"),
            "{error}"
        );

        let apart = [Spaced {
            target: size(0.0, 0.0, 10.0, 10.0),
            neighbours: vec![size(30.0, 0.0, 120.0, 12.0)],
        }];
        assert_sizes_spaced("#x", &apart).unwrap();
    }
}
