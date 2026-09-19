//! Pointer and drag.
//!
//! The only pass that reaches a class of behaviour nothing else can: SSR
//! dispatches no pointer events at all, so `Slider`, `Splitter`, `Carousel`
//! and `Lightbox` drags are untested by the Rust suite
//! (`codebase/testing`). Real `Input.dispatchMouseEvent` calls go through
//! Chromium's own pointer plumbing, including pointer capture.

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
///
/// Returns a defined object even when nothing matched: a bare JS `null` cannot
/// be deserialised into an `Option<T>` and surfaces as "No value found", which
/// reads like a harness fault rather than a missing element.
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

/// Press, move in steps, release.
///
/// The intermediate moves matter: a component that captures the pointer and
/// tracks movement sees nothing from a press followed straight by a release,
/// and `Scroller` only captures after 5px of travel
/// (`codebase/components/scroller`). One jump would test the wrong thing.
pub async fn drag(page: &Page, from: Point, to: Point, steps: usize) -> Result<()> {
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
    mouse(page, DispatchMouseEventType::MouseReleased, to, 0).await?;
    Ok(())
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
