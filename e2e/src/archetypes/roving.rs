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
        page.evaluate("document.activeElement && document.activeElement.blur()")
            .await?;
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

    /// Exactly one item may be in the tab order.
    async fn assert_single_tab_stop(&self, page: &Page, count: usize) -> Result<()> {
        let tabbable: usize = page
            .evaluate(format!(
                r#"[...document.querySelectorAll({})]
                    .filter(el => el.getAttribute('tabindex') !== '-1' && !el.hasAttribute('disabled'))
                    .length"#,
                serde_json::to_string(self.items)?
            ))
            .await?
            .into_value()?;
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
