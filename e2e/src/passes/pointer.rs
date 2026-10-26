//! Pointer and drag through real `Input.dispatchMouseEvent`, pointer capture included.
//! SSR dispatches no pointer events, so drags are only tested here (`codebase/testing`).

use crate::frames::in_front;
use crate::wait::PageBounded;
use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Deserialize)]
struct Located {
    found: bool,
    x: f64,
    y: f64,
}

/// The centre of an element, in viewport coordinates.
pub async fn centre_of(page: &Page, selector: &str) -> Result<Point> {
    let located: Located = page
        .evaluate_bounded(format!(
            r#"(() => {{
                const el = document.querySelector({});
                if (!el) return {{ found: false, x: 0, y: 0 }};
                const r = el.getBoundingClientRect();
                return {{ found: true, x: r.x + r.width / 2, y: r.y + r.height / 2 }};
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    if !located.found {
        anyhow::bail!("no element at {selector} to point at");
    }
    Ok(Point {
        x: located.x,
        y: located.y,
    })
}

async fn mouse(page: &Page, kind: DispatchMouseEventType, at: Point, buttons: i64) -> Result<()> {
    mouse_n(page, kind, at, buttons, 1).await
}

/// `clicks` is the press's click count: 2 on the second press makes a `dblclick`.
async fn mouse_n(
    page: &Page,
    kind: DispatchMouseEventType,
    at: Point,
    buttons: i64,
    clicks: i64,
) -> Result<()> {
    mouse_held(page, kind, at, buttons, clicks, 0).await
}

/// `modifiers` as CDP counts them: Alt 1, Ctrl 2, Meta 4, Shift 8.
async fn mouse_held(
    page: &Page,
    kind: DispatchMouseEventType,
    at: Point,
    buttons: i64,
    clicks: i64,
    modifiers: i64,
) -> Result<()> {
    let what = format!("the mouse {kind:?} at ({:.0}, {:.0})", at.x, at.y);
    let event = DispatchMouseEventParams::builder()
        .r#type(kind)
        .x(at.x)
        .y(at.y)
        .button(MouseButton::Left)
        .buttons(buttons)
        .click_count(clicks)
        .modifiers(modifiers)
        .build()
        .map_err(anyhow::Error::msg)?;
    crate::wait::bounded(&what, page.execute(event)).await?;
    Ok(())
}

/// Press, move in steps, release. The steps matter: `Scroller` captures only
/// after 5px of travel (`codebase/components/scroller`).
pub async fn drag(page: &Page, from: Point, to: Point, steps: usize) -> Result<()> {
    drag_held(page, from, to, steps).await?;
    release(page, to).await
}

/// [`drag`] without the release, to read the page mid-drag; [`release`] ends it.
pub async fn drag_held(page: &Page, from: Point, to: Point, steps: usize) -> Result<()> {
    mouse(page, DispatchMouseEventType::MouseMoved, from, 0).await?;
    mouse(page, DispatchMouseEventType::MousePressed, from, 1).await?;
    for step in 1..=steps {
        let t = step as f64 / steps as f64;
        let at = Point {
            x: from.x + (to.x - from.x) * t,
            y: from.y + (to.y - from.y) * t,
        };
        mouse(page, DispatchMouseEventType::MouseMoved, at, 1).await?;
    }
    Ok(())
}

pub async fn release(page: &Page, at: Point) -> Result<()> {
    mouse(page, DispatchMouseEventType::MouseReleased, at, 0).await
}

/// Move the pointer onto an element's centre, pressing nothing.
pub async fn hover(page: &Page, selector: &str) -> Result<()> {
    let at = centre_of(page, selector).await?;
    move_to(page, at).await
}

/// Move the pointer to a viewport point, pressing nothing.
pub async fn move_to(page: &Page, at: Point) -> Result<()> {
    mouse(page, DispatchMouseEventType::MouseMoved, at, 0).await
}

/// A touch held at an element's centre for `ms`, with touch emulation turned on
/// for the page.
pub async fn long_press(page: &Page, selector: &str, ms: u64) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::input::DispatchTouchEventType;
    let at = touch_down(page, selector).await?;
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    touch_at(page, DispatchTouchEventType::TouchEnd, at).await
}

/// A finger tap on an element's centre, the page in front for it. A scenario sends touches
/// through this, [`swipe`] or its own [`in_front`] block: a page behind waits out a 500 ms
/// frame on every touch event, and the home tab takes the front back after each gesture.
pub async fn tap(page: &Page, selector: &str) -> Result<()> {
    in_front(page, async {
        let at = touch_down(page, selector).await?;
        touch_up(page, at).await
    })
    .await
}

/// [`touch_drag`] with the page in front; see [`tap`].
pub async fn swipe(page: &Page, from: Point, to: Point, steps: usize) -> Result<()> {
    in_front(page, touch_drag(page, from, to, steps)).await
}

/// A touch pressed at an element's centre and held, with touch emulation
/// turned on for the page; returns where, for [`touch_up`]. Run it inside [`in_front`].
pub async fn touch_down(page: &Page, selector: &str) -> Result<Point> {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    use chromiumoxide::cdp::browser_protocol::input::DispatchTouchEventType;
    page.execute_bounded(SetTouchEmulationEnabledParams::new(true))
        .await?;
    let at = centre_of(page, selector).await?;
    touch_at(page, DispatchTouchEventType::TouchStart, at).await?;
    Ok(at)
}

/// Lifts a [`touch_down`] at `at`: CDP rejects a `touchEnd` without a point.
pub async fn touch_up(page: &Page, at: Point) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::input::DispatchTouchEventType;
    touch_at(page, DispatchTouchEventType::TouchEnd, at).await
}

/// A finger tap at a viewport point; turn touch emulation on first.
pub async fn tap_at(page: &Page, at: Point) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::input::DispatchTouchEventType;
    touch_at(page, DispatchTouchEventType::TouchStart, at).await?;
    touch_at(page, DispatchTouchEventType::TouchEnd, at).await
}

/// Moves a held [`touch_down`] to `to`, still held.
pub async fn touch_move(page: &Page, to: Point) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::input::DispatchTouchEventType;
    touch_at(page, DispatchTouchEventType::TouchMove, to).await
}

async fn touch_at(
    page: &Page,
    kind: chromiumoxide::cdp::browser_protocol::input::DispatchTouchEventType,
    at: Point,
) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::input::{DispatchTouchEventParams, TouchPoint};
    let point = TouchPoint::builder()
        .x(at.x)
        .y(at.y)
        .build()
        .map_err(anyhow::Error::msg)?;
    let what = format!("the touch {kind:?} at ({:.0}, {:.0})", at.x, at.y);
    let event = DispatchTouchEventParams::builder()
        .r#type(kind)
        .touch_point(point)
        .build()
        .map_err(anyhow::Error::msg)?;
    crate::wait::bounded(&what, page.execute(event)).await?;
    Ok(())
}

/// A touch pressed at `from`, moved to `to` in `steps`, lifted there; `steps: 0`
/// is a tap. Turns on touch emulation for the page. The events carry their own
/// times and the lift rests, so the swipe starts no fling that swallows the next tap.
pub async fn touch_drag(page: &Page, from: Point, to: Point, steps: usize) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    use chromiumoxide::cdp::browser_protocol::input::{
        DispatchTouchEventParams, DispatchTouchEventType, TimeSinceEpoch, TouchPoint,
    };
    const STEP_S: f64 = 0.016;
    const REST_S: f64 = 0.15;
    page.execute_bounded(SetTouchEmulationEnabledParams::new(true))
        .await?;
    let moves = (1..=steps).map(|step| {
        let t = step as f64 / steps as f64;
        let at = Point {
            x: from.x + (to.x - from.x) * t,
            y: from.y + (to.y - from.y) * t,
        };
        (DispatchTouchEventType::TouchMove, at)
    });
    let events = std::iter::once((DispatchTouchEventType::TouchStart, from))
        .chain(moves)
        .chain(std::iter::once((DispatchTouchEventType::TouchEnd, to)));
    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs_f64();
    for (index, (kind, at)) in events.enumerate() {
        let rest = if steps > 0 && kind == DispatchTouchEventType::TouchEnd {
            REST_S
        } else {
            0.0
        };
        let point = TouchPoint::builder()
            .x(at.x)
            .y(at.y)
            .build()
            .map_err(anyhow::Error::msg)?;
        let event = DispatchTouchEventParams::builder()
            .r#type(kind)
            .touch_point(point)
            .timestamp(TimeSinceEpoch::new(start + index as f64 * STEP_S + rest))
            .build()
            .map_err(anyhow::Error::msg)?;
        page.execute_bounded(event).await?;
    }
    Ok(())
}

/// Two touches either side of `at`, `from` apart sideways, spread to `to` apart in
/// `steps`, then lifted one by one. Turns on touch emulation for the page.
pub async fn pinch(page: &Page, at: Point, from: f64, to: f64, steps: usize) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    use chromiumoxide::cdp::browser_protocol::input::{
        DispatchTouchEventParams, DispatchTouchEventType, TouchPoint,
    };
    page.execute_bounded(SetTouchEmulationEnabledParams::new(true))
        .await?;
    let fingers = |gap: f64, count: usize| -> Result<Vec<TouchPoint>> {
        [-1.0, 1.0]
            .into_iter()
            .take(count)
            .enumerate()
            .map(|(id, side)| {
                TouchPoint::builder()
                    .x(at.x + side * gap / 2.0)
                    .y(at.y)
                    .id(id as f64)
                    .build()
                    .map_err(anyhow::Error::msg)
            })
            .collect()
    };
    let mut events = vec![
        (DispatchTouchEventType::TouchStart, fingers(from, 1)?),
        (DispatchTouchEventType::TouchStart, fingers(from, 2)?),
    ];
    for step in 1..=steps {
        let gap = from + (to - from) * step as f64 / steps as f64;
        events.push((DispatchTouchEventType::TouchMove, fingers(gap, 2)?));
    }
    // One end lifts every touch; CDP rejects one without a point.
    events.push((DispatchTouchEventType::TouchEnd, fingers(to, 2)?));
    for (kind, points) in events {
        let event = DispatchTouchEventParams::builder()
            .r#type(kind)
            .touch_points(points)
            .build()
            .map_err(anyhow::Error::msg)?;
        page.execute_bounded(event).await?;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Aimed {
    found: bool,
    x: f64,
    y: f64,
    /// What the centre hits when it is not the element: a covered target.
    miss: Option<String>,
}

/// Click an element at its centre, scrolled in first when the centre is off screen. Fails
/// when the centre hits another element, so a covered target is no silent miss (todo 1756).
pub async fn click(page: &Page, selector: &str) -> Result<()> {
    click_with(page, selector, 0).await
}

/// [`click`] with CDP `modifiers` held: Alt 1, Ctrl 2, Meta 4, Shift 8.
pub async fn click_with(page: &Page, selector: &str, modifiers: i64) -> Result<()> {
    let aimed: Aimed = page
        .evaluate_bounded(format!(
            r#"(() => {{
                const el = document.querySelector({});
                if (!el) return {{ found: false, x: 0, y: 0, miss: null }};
                const centre = () => {{
                    const r = el.getBoundingClientRect();
                    return [r.x + r.width / 2, r.y + r.height / 2];
                }};
                let [x, y] = centre();
                if (x < 0 || y < 0 || x >= innerWidth || y >= innerHeight) {{
                    el.scrollIntoView({{ block: 'nearest', inline: 'nearest', behavior: 'instant' }});
                    [x, y] = centre();
                }}
                const hit = document.elementFromPoint(x, y);
                const lands = !!hit && (el.contains(hit)
                    || (hit.contains(el) && getComputedStyle(el).pointerEvents === 'none'));
                const name = n => n.tagName.toLowerCase() + (n.id ? '#' + n.id : '')
                    + [...n.classList].map(c => '.' + c).join('');
                return {{ found: true, x, y, miss: lands ? null : (hit ? name(hit) : 'nothing') }};
            }})()"#,
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    if !aimed.found {
        anyhow::bail!("no element at {selector} to point at");
    }
    if let Some(miss) = aimed.miss {
        anyhow::bail!(
            "a click on {selector} at ({:.0}, {:.0}) would land on {miss}",
            aimed.x,
            aimed.y
        );
    }
    let at = Point {
        x: aimed.x,
        y: aimed.y,
    };
    mouse_held(
        page,
        DispatchMouseEventType::MousePressed,
        at,
        1,
        1,
        modifiers,
    )
    .await?;
    mouse_held(
        page,
        DispatchMouseEventType::MouseReleased,
        at,
        0,
        1,
        modifiers,
    )
    .await
}

/// A primary click at a viewport point; fails on a point that hits nothing, off the
/// viewport (todo 1756). See [`click_at_unchecked`].
pub async fn click_at(page: &Page, at: Point) -> Result<()> {
    let hits: bool = page
        .evaluate_bounded(format!(
            "document.elementFromPoint({}, {}) !== null",
            at.x, at.y
        ))
        .await?
        .into_value()?;
    if !hits {
        anyhow::bail!("a click at ({:.0}, {:.0}) hits nothing", at.x, at.y);
    }
    click_at_unchecked(page, at).await
}

/// A primary click at a viewport point, with no move first: a background page holds a
/// move for its next frame, up to 1 s (todo 1632). The press still sends the pointer's
/// boundary events (`field_frame::a_press_without_a_move_enters_its_target`). Unchecked:
/// each caller says why a miss is what it wants.
pub async fn click_at_unchecked(page: &Page, at: Point) -> Result<()> {
    mouse(page, DispatchMouseEventType::MousePressed, at, 1).await?;
    mouse(page, DispatchMouseEventType::MouseReleased, at, 0).await
}

/// Double-click an element at its centre: two clicks, then the `dblclick`. No move first,
/// as [`click_at`].
pub async fn double_click(page: &Page, selector: &str) -> Result<()> {
    let at = centre_of(page, selector).await?;
    for clicks in 1..=2 {
        mouse_n(page, DispatchMouseEventType::MousePressed, at, 1, clicks).await?;
        mouse_n(page, DispatchMouseEventType::MouseReleased, at, 0, clicks).await?;
    }
    Ok(())
}
