//! WCAG 2.5.8 target size (minimum): 24 by 24 CSS pixels.
//!
//! <https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html>
//!
//! Straight off the box model, so it is objective and cheap. The criterion has
//! exemptions (inline targets in a sentence, targets whose spacing gives them
//! a 24px circle), so this reports the measurement and the caller decides
//! which controls it applies to, rather than the pass failing a whole page.
//!
//! ## What is measured is the press target, not the handle
//!
//! 2.5.8 measures "the region that accepts the pointer action", which is
//! frequently larger than what is drawn. A slider's press target is the track
//! its `use_drag` captures, not the thumb painted on it - two passes measured
//! the thumb before that was noticed (`ColorPicker`, 2026-09-20). So the
//! selector a unit hands this pass has to name the element the listener is on.
//! Nothing here can work that out; naming the wrong element is a measurement of
//! the wrong thing, and it will look like a clean one.
//!
//! ## The spacing exception
//!
//! An undersized target still conforms when "the target offset is at least 24
//! CSS pixels to every adjacent target": put a 24px-diameter circle on the
//! centre of each undersized target's bounding box, and no circle may touch
//! another target or another undersized target's circle. That is what
//! [`assert_sizes_spaced`] computes, and it is the only thing that makes a
//! `RadioGroup` row or a `Notifications` close button conform. It is not on by
//! default: a unit opts in with `Suite::targets_spaced`, so the page says which
//! controls lean on the exception rather than the pass quietly relaxing for
//! everything.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use serde::Deserialize;

pub const MINIMUM: f64 = 24.0;

/// Half the minimum: the radius of the circle the spacing exception centres on
/// an undersized target.
const RADIUS: f64 = MINIMUM / 2.0;

/// Slack for the browser's sub-pixel geometry, well under the half a pixel
/// that made `RadioGroup`'s rows fail (todo 302).
const EPSILON: f64 = 0.01;

/// Everything on the page that counts as "another target" for the exception.
///
/// The roles a pointer press acts on, plus anything a caller put in the tab
/// order. Deliberately generous: a target the exception forgets is clearance
/// this pass would grant and a user would not have.
const TARGETS: &str = "a[href], button, input:not([type=hidden]), select, textarea, summary, \
                       [role=button], [role=link], [role=checkbox], [role=radio], [role=switch], \
                       [role=tab], [role=menuitem], [role=menuitemcheckbox], \
                       [role=menuitemradio], [role=option], [role=slider], [role=spinbutton], \
                       [role=treeitem], [tabindex]:not([tabindex='-1'])";

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
    /// Viewport-relative, as `getBoundingClientRect` reports it. Only the
    /// spacing exception reads these; a plain size check does not care where a
    /// control is.
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
    /// Every other target within reach of the 24px circle. Neither an ancestor
    /// nor a descendant of `target`: a radio inside the row that activates it,
    /// or a button inside a clickable card, is the same target pressed twice
    /// over, not an adjacent one to clear.
    pub neighbours: Vec<Size>,
}

/// Measure every element matching `selector`.
pub async fn measure_all(page: &Page, selector: &str) -> Result<Vec<Size>> {
    let sizes = page
        .evaluate(format!(
            r#"[...document.querySelectorAll({})].map(el => {{
                const r = el.getBoundingClientRect();
                return {{ width: r.width, height: r.height, x: r.x, y: r.y }};
            }})"#,
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
            r#"(() => {{
                const box = el => {{
                    const r = el.getBoundingClientRect();
                    return {{ width: r.width, height: r.height, x: r.x, y: r.y }};
                }};
                const shown = el => {{
                    const r = el.getBoundingClientRect();
                    if (r.width <= 0 || r.height <= 0) return false;
                    const style = getComputedStyle(el);
                    return style.visibility !== 'hidden' && style.pointerEvents !== 'none';
                }};
                const all = [...document.querySelectorAll({targets})].filter(shown);
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

/// The assertion on its own, for a caller that already has the measurements.
///
/// Split out because a control that only exists in an open state has to be
/// measured where it exists: `[role=option]` matches nothing while the list is
/// closed, and "nothing matched" is not a failure there.
pub fn assert_sizes(selector: &str, sizes: &[Size]) -> Result<()> {
    let small: Vec<_> = sizes.iter().filter(|s| s.undersized()).collect();
    if !small.is_empty() {
        bail!("target size below WCAG 2.5.8 minimum ({MINIMUM}px) for {selector}: {small:?}");
    }
    Ok(())
}

/// 2.5.8 with the spacing exception applied, for measurements already taken.
///
/// A target of at least 24x24 passes outright. An undersized one passes only
/// if the 24px circle on its centre reaches no other target and no other
/// undersized target's circle - which for two undersized targets is their
/// centres 24px apart, and for an undersized target beside a full-size one is
/// 12px of clearance from the centre to that target's box.
pub fn assert_sizes_spaced(selector: &str, measured: &[Spaced]) -> Result<()> {
    for Spaced { target, neighbours } in measured {
        if !target.undersized() {
            continue;
        }
        let centre = target.centre();
        for neighbour in neighbours {
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
            } else {
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
            }
        }
    }
    Ok(())
}
