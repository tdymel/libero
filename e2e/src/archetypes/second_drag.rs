//! A reorderable list's second drag still previews (todo 1438): kanban and sortable.

use anyhow::{Result, bail, ensure};
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
/// below the new slot, never moves; the release lands `order` and every item rests in its slot.
/// The second round proves the first drop left the preview working.
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
        let stepped = wait::for_js_true(
            page,
            &format!("{} < -1", shift(passed)),
            &format!("drag {drag}: {passed} to step up"),
        )
        .await;
        if let Err(error) = stepped {
            let state: String = page.evaluate(state_of(passed)).await?.into_value()?;
            return Err(error.context(format!("from ({from:?}) to ({to:?}); {state}")));
        }
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
        // The dropped item slides into its slot (1129); a press during the slide measures it mid-way.
        wait::for_js_true(
            page,
            "document.getAnimations().length === 0",
            &format!("drag {drag}: the dropped item to settle"),
        )
        .await?;
        // The drop's offset once stayed on the item after its slide (3039).
        let resting: bool = page
            .evaluate(format!(
                "[...document.querySelector({passed:?}).parentElement.children].every(e => {{ \
                 const t = getComputedStyle(e).transform; return t === 'none' || Math.abs(new DOMMatrix(t).m42) < 1; }})"
            ))
            .await?
            .into_value()?;
        if !resting {
            let state: String = page.evaluate(state_of(passed)).await?.into_value()?;
            bail!("drag {drag}: an item rests off its slot after the drop: {state}");
        }
    }
    Ok(())
}

/// Each sibling of `passed`: id, transform, inline style and attributes, plus the focus.
fn state_of(passed: &str) -> String {
    format!(
        "(() => {{ const items = [...document.querySelector({passed:?}).parentElement.children]; \
         return items.map(e => `${{e.id}} ${{getComputedStyle(e).transform}} \
         [${{e.getAttribute('style') ?? ''}}] ${{[...e.attributes].filter(a => a.name.startsWith('data-')).map(a => a.name + '=' + a.value).join(' ')}}`).join('; ') \
         + `; list ${{[...document.querySelector({passed:?}).parentElement.attributes].map(a => a.name + '=' + a.value).join(' ')}}` \
         + `; said ${{[...document.querySelectorAll('[aria-live]')].map(e => e.textContent).join(' | ')}}` \
         + `; focus ${{document.activeElement?.outerHTML.slice(0, 80)}}`; }})()"
    )
}
