//! A reorderable list's second drag still previews (todo 1438): kanban and sortable.

use anyhow::{Result, ensure};
use chromiumoxide::Page;

use crate::passes::pointer;
use crate::wait;

/// One drag: the handle pressed, the item it passes (which must step up before the drop),
/// and `#order`'s text after the drop.
pub struct Round<'a> {
    pub handle: &'a str,
    pub passed: &'a str,
    pub order: &'a str,
}

/// Each round drags `handle` down 1.4 `pitch`es and holds it: `passed` steps up and `still`,
/// below the new slot, never moves; the release lands `order`. The second round proves the
/// first drop left the preview working.
pub async fn second_drag(page: &Page, rounds: &[Round<'_>], still: &str, pitch: f64) -> Result<()> {
    let shift = |selector: &str| {
        format!(
            "(() => {{ const t = getComputedStyle(document.querySelector({selector:?})).transform; \
             return t === 'none' ? 0 : new DOMMatrix(t).m42; }})()"
        )
    };
    for (
        round,
        Round {
            handle,
            passed,
            order,
        },
    ) in rounds.iter().enumerate()
    {
        let drag = round + 1;
        let from = pointer::centre_of(page, handle).await?;
        let to = pointer::Point {
            x: from.x,
            y: from.y + pitch * 1.4,
        };
        pointer::drag_held(page, from, to, 8).await?;
        wait::for_js_true(
            page,
            &format!("{} < -1", shift(passed)),
            &format!("drag {drag}: {passed} to step up"),
        )
        .await?;
        let moved: f64 = page.evaluate(shift(still)).await?.into_value()?;
        ensure!(moved.abs() < 1.0, "drag {drag}: {still} moved by {moved}");
        pointer::release(page, to).await?;
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('#order').textContent.trim() === {:?}",
                order.trim()
            ),
            &format!("drag {drag}: the drop"),
        )
        .await?;
    }
    Ok(())
}
