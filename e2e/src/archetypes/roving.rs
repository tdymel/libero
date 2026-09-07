//! The APG roving-tabindex pattern.
//!
//! A composite widget is **one** tab stop, and the arrow keys move within it.
//! `Tabs`, `Menubar` and `Toolbar` are this, and `Tree` shares part of it (todo
//! 310). A radio group is not: selection follows focus and Home and End are
//! not in its pattern, so `RadioGroup` and `SegmentedControl` use `RadioSet`.
//!
//! ## What this is checked against
//!
//! - **APG, Developing a Keyboard Interface - "Managing focus with roving
//!   tabindex"** - exactly one item carries `tabindex="0"` and the rest carry
//!   `-1`. <https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/>
//! - **APG, Tabs pattern** - Left/Right for a horizontal strip, Home and End
//!   for the ends, and whether the ends wrap is the component's choice.
//!   <https://www.w3.org/WAI/ARIA/apg/patterns/tabs/>
//! - **WCAG 2.1.1 Keyboard** and **2.4.3 Focus Order** - the order is
//!   meaningful and every item is reachable.
//!
//! The assertion that earns its keep is the tab-stop count. A widget whose
//! items are each independently tabbable still *works* with a keyboard - every
//! item is reachable, nothing throws - so it passes a naive "is it keyboard
//! accessible" check while being wrong in the way that matters: a thirty-item
//! toolbar becomes thirty tab stops between the user and the next control.
//! `TagsField` paid for this once already, dropping its chips' own tab stops -
//! see `codebase/components/tags-field` for what having them cost.

use anyhow::{Result, bail};
use chromiumoxide::Page;

use crate::passes::keyboard::{self, Key};
use crate::wait;

/// How many distinct tab stops lie inside the widget holding `items`.
///
/// **Counted by tabbing, not read off `tabindex`.** An attribute count over the
/// item selector answers a narrower question than the one the pattern asks: it
/// sees a tab stop that *is* an item and is blind to a tab stop *inside* one.
/// A tab strip whose tabs each carry a close button, a toolbar row with a
/// nested link, a tree row with an action - every one of those is several stops
/// where the pattern allows one, and every one of them counted as a single stop
/// (review 7, E3). The same blindness runs the other way: `RadioSet` measured
/// three stops on `SegmentedControl`'s native radios, which the browser groups
/// into one, and that false red is why it already counts this way.
///
/// The widget is the closest ancestor holding every item, so "inside" covers
/// the items, anything nested in them, and any chrome the widget draws between
/// them. Tab walks forward from a blurred document until focus has entered the
/// widget and left it again, and each distinct element focused inside counts
/// once.
pub async fn count_tab_stops(page: &Page, items: &str) -> Result<usize> {
    let sel = serde_json::to_string(items)?;
    // The widget, kept on `window` so the per-press read below is one cheap
    // expression rather than a repeated ancestor walk.
    let count: i64 = page
        .evaluate(format!(
            "(() => {{ const items = [...document.querySelectorAll({sel})]; \
             if (!items.length) return -1; \
             let root = items[0]; \
             for (const item of items) {{ while (root && !root.contains(item)) root = root.parentElement; }} \
             if (!root) return -1; \
             window.__e2eTabStops = {{ root, stops: [] }}; \
             return items.length; }})()"
        ))
        .await?
        .into_value()?;
    let Ok(count) = usize::try_from(count) else {
        bail!("counting tab stops: nothing matched {items}");
    };

    // From the top of the document, so what is counted is the walk a keyboard
    // user makes rather than whatever the last test step left focused.
    reset_tab_position(page).await?;

    // Enough presses to walk in from the top of the document, cross a stop per
    // item and any nested ones, and come back out.
    let budget = count * 3 + 8;
    let mut entered = false;
    for _ in 0..budget {
        keyboard::press(page, keyboard::TAB).await?;
        let inside: bool = page
            .evaluate(
                "(() => { const state = window.__e2eTabStops, el = document.activeElement; \
                 const inside = !!(el && state.root.contains(el)); \
                 if (inside && !state.stops.includes(el)) state.stops.push(el); \
                 return inside; })()",
            )
            .await?
            .into_value()?;
        if inside {
            entered = true;
        } else if entered {
            break;
        }
    }

    let stops: usize = page
        .evaluate("window.__e2eTabStops.stops.length")
        .await?
        .into_value()?;
    reset_tab_position(page).await?;
    Ok(stops)
}

/// Put the next Tab back at the top of the document.
///
/// `blur()` alone does not: it clears the focus and leaves the **sequential
/// focus navigation starting point** where the element was, so the next Tab
/// continues from there. Counting stops walks past the widget, so without this
/// the walk that follows tabbed into whatever comes after it and reported the
/// widget unreachable. Focusing `<body>` moves the starting point to the start
/// of the document, and the temporary `tabindex` is what makes `<body>`
/// focusable at all; it is removed again so nothing else - the AX snapshot
/// especially - sees it.
pub async fn reset_tab_position(page: &Page) -> Result<()> {
    page.evaluate(
        "(() => { if (document.activeElement) document.activeElement.blur(); \
         document.body.setAttribute('tabindex', '-1'); \
         document.body.focus(); \
         document.body.removeAttribute('tabindex'); })()",
    )
    .await?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    fn next(self) -> Key {
        match self {
            Orientation::Horizontal => keyboard::ARROW_RIGHT,
            Orientation::Vertical => keyboard::ARROW_DOWN,
        }
    }

    fn previous(self) -> Key {
        match self {
            Orientation::Horizontal => keyboard::ARROW_LEFT,
            Orientation::Vertical => keyboard::ARROW_UP,
        }
    }
}

pub struct RovingTabindex<'a> {
    /// Every item in the group, e.g. `[role=tab]`.
    pub items: &'a str,
    pub orientation: Orientation,
    /// Whether the arrows wrap from the last item back to the first. APG
    /// allows either, so it is the component's choice - but it must be
    /// consistent, and stating it here is what makes it a contract.
    pub wraps: bool,
}

impl RovingTabindex<'_> {
    pub async fn assert_contract(&self, page: &Page) -> Result<()> {
        let count = self.item_count(page).await?;
        if count < 2 {
            bail!(
                "roving tabindex needs at least two items, found {count} at {}",
                self.items
            );
        }

        self.assert_single_tab_stop(page, count).await?;

        // Focus the group, then walk it.
        reset_tab_position(page).await?;
        keyboard::press(page, keyboard::TAB).await?;
        let first = self.settle_on(page, 0).await?;
        if first.is_none() {
            bail!("tabbing into {} did not focus any item", self.items);
        }

        // Forward to the end.
        for step in 1..count {
            keyboard::press(page, self.orientation.next()).await?;
            let at = self.settle_on(page, step).await?;
            if at != Some(step) {
                bail!(
                    "after {step} forward arrow(s) in {}, focus is on item {at:?}, expected {step}",
                    self.items
                );
            }
        }

        // One more, which is where wrapping is decided.
        keyboard::press(page, self.orientation.next()).await?;
        let expected = if self.wraps { Some(0) } else { Some(count - 1) };
        let after_end = self.settle_on(page, expected.unwrap_or(0)).await?;
        if after_end != expected {
            bail!(
                "arrowing past the last item of {} landed on {after_end:?}; with wraps={} it \
                 should be {expected:?}",
                self.items,
                self.wraps
            );
        }

        // Home and End, which APG requires for every orientation.
        keyboard::press(page, keyboard::HOME).await?;
        if self.settle_on(page, 0).await? != Some(0) {
            bail!("Home did not move to the first item of {}", self.items);
        }
        keyboard::press(page, keyboard::END).await?;
        if self.settle_on(page, count - 1).await? != Some(count - 1) {
            bail!("End did not move to the last item of {}", self.items);
        }

        keyboard::press(page, self.orientation.previous()).await?;
        if self.settle_on(page, count - 2).await? != Some(count - 2) {
            bail!("the backward arrow did not move within {}", self.items);
        }

        Ok(())
    }

    /// Exactly one tab stop inside the widget.
    async fn assert_single_tab_stop(&self, page: &Page, count: usize) -> Result<()> {
        let tabbable = count_tab_stops(page, self.items).await?;
        if tabbable != 1 {
            bail!(
                "{} has {tabbable} tab stops across {count} items; a roving-tabindex widget is \
                 one tab stop, with the arrows moving inside it",
                self.items
            );
        }
        Ok(())
    }

    async fn item_count(&self, page: &Page) -> Result<usize> {
        Ok(page
            .evaluate(format!(
                "document.querySelectorAll({}).length",
                serde_json::to_string(self.items)?
            ))
            .await?
            .into_value()?)
    }

    /// Wait for focus to land on a given item, then report where it is.
    ///
    /// Focus is moved from a dioxus handler, so it has not necessarily happened
    /// when the keypress returns. Polling for the expected index and only then
    /// reading the actual one keeps the failure message honest while removing
    /// the race.
    async fn settle_on(&self, page: &Page, expected: usize) -> Result<Option<usize>> {
        let check = format!(
            "[...document.querySelectorAll({})].indexOf(document.activeElement) === {expected}",
            serde_json::to_string(self.items)?
        );
        let _ = wait::for_js_true(page, &check, &format!("focus on item {expected}")).await;
        self.focused_index(page).await
    }

    /// Which item the document actually has focused, by position.
    async fn focused_index(&self, page: &Page) -> Result<Option<usize>> {
        let index: i64 = page
            .evaluate(format!(
                "[...document.querySelectorAll({})].indexOf(document.activeElement)",
                serde_json::to_string(self.items)?
            ))
            .await?
            .into_value()?;
        Ok(usize::try_from(index).ok())
    }
}
