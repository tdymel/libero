//! Pointer and drag through real `Input.dispatchMouseEvent`, pointer capture included.
//! SSR dispatches no pointer events, so drags are only tested here (`codebase/testing`).

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
        .evaluate(format!(
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
    page.execute(
        DispatchMouseEventParams::builder()
            .r#type(kind)
            .x(at.x)
            .y(at.y)
            .button(MouseButton::Left)
            .buttons(buttons)
            .click_count(clicks)
            .build()
            .map_err(anyhow::Error::msg)?,
    )
    .await?;
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
/// for the page. CDP rejects a `touchEnd` without a point.
pub async fn long_press(page: &Page, selector: &str, ms: u64) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    use chromiumoxide::cdp::browser_protocol::input::{
        DispatchTouchEventParams, DispatchTouchEventType, TouchPoint,
    };
    page.execute(SetTouchEmulationEnabledParams::new(true))
        .await?;
    let at = centre_of(page, selector).await?;
    for kind in [
        DispatchTouchEventType::TouchStart,
        DispatchTouchEventType::TouchEnd,
    ] {
        if kind == DispatchTouchEventType::TouchEnd {
            tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        }
        let point = TouchPoint::builder()
            .x(at.x)
            .y(at.y)
            .build()
            .map_err(anyhow::Error::msg)?;
        let event = DispatchTouchEventParams::builder()
            .r#type(kind)
            .touch_point(point)
            .build()
            .map_err(anyhow::Error::msg)?;
        page.execute(event).await?;
    }
    Ok(())
}

/// A touch pressed at `from`, moved to `to` in `steps`, lifted there; `steps: 0`
/// is a tap. Turns on touch emulation for the page.
pub async fn touch_drag(page: &Page, from: Point, to: Point, steps: usize) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    use chromiumoxide::cdp::browser_protocol::input::{
        DispatchTouchEventParams, DispatchTouchEventType, TouchPoint,
    };
    page.execute(SetTouchEmulationEnabledParams::new(true))
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
    for (kind, at) in events {
        let point = TouchPoint::builder()
            .x(at.x)
            .y(at.y)
            .build()
            .map_err(anyhow::Error::msg)?;
        let event = DispatchTouchEventParams::builder()
            .r#type(kind)
            .touch_point(point)
            .build()
            .map_err(anyhow::Error::msg)?;
        page.execute(event).await?;
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
    page.execute(SetTouchEmulationEnabledParams::new(true))
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
        page.execute(event).await?;
    }
    Ok(())
}

/// Click an element at its centre.
pub async fn click(page: &Page, selector: &str) -> Result<()> {
    click_at(page, centre_of(page, selector).await?).await
}

/// A primary click at a viewport point.
pub async fn click_at(page: &Page, at: Point) -> Result<()> {
    mouse(page, DispatchMouseEventType::MouseMoved, at, 0).await?;
    mouse(page, DispatchMouseEventType::MousePressed, at, 1).await?;
    mouse(page, DispatchMouseEventType::MouseReleased, at, 0).await?;
    Ok(())
}

/// Double-click an element at its centre: two clicks, then the `dblclick`.
pub async fn double_click(page: &Page, selector: &str) -> Result<()> {
    let at = centre_of(page, selector).await?;
    mouse(page, DispatchMouseEventType::MouseMoved, at, 0).await?;
    for clicks in 1..=2 {
        mouse_n(page, DispatchMouseEventType::MousePressed, at, 1, clicks).await?;
        mouse_n(page, DispatchMouseEventType::MouseReleased, at, 0, clicks).await?;
    }
    Ok(())
}
