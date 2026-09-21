//! The APG dialog / dismissible-overlay pattern: opens, focus moves inside, Escape closes,
//! focus returns to the trigger (`principles/focus-after-removal`).

use anyhow::{Result, bail};
use chromiumoxide::Page;

use crate::passes::{dismissal, focus, keyboard};
use crate::wait;

pub struct Overlay<'a> {
    /// The control that opens it.
    pub trigger: &'a str,
    /// The panel, dialog or menu that appears.
    pub panel: &'a str,
    /// Whether focus should be trapped inside while open. True for a modal
    /// dialog, false for a non-modal popover the user can Tab out of.
    pub traps_focus: bool,
    /// How many tab presses it takes to reach the trigger from the top.
    pub tab_budget: usize,
}

impl Overlay<'_> {
    pub async fn assert_contract(&self, page: &Page) -> Result<()> {
        // Open it from the keyboard, since that is the path that has to work.
        keyboard::tab_to(page, self.trigger, self.tab_budget).await?;
        keyboard::press(page, keyboard::ENTER).await?;
        wait::for_visible(page, self.panel).await?;

        self.assert_focus_moved_inside(page).await?;

        if self.traps_focus {
            self.assert_focus_is_trapped(page).await?;
        }

        keyboard::press(page, keyboard::ESCAPE).await?;
        let focus_returned = self.wait_for_close_signal(page).await?;

        // Read at the close signal: waiting for hidden first masked a phantom panel (review 7, E6).
        dismissal::assert_gone_from_at(page, self.panel).await?;

        // Given time: a component may return focus once its exit has ended.
        if !focus_returned {
            let _ = self.wait_for_focus_on_trigger(page).await;
        }
        focus::assert_focused(page, self.trigger, "Escape closing the overlay").await?;
        Ok(())
    }

    /// Waits for `aria-expanded="false"`, or for a dialog, for focus on the trigger.
    /// Returns whether focus is already back.
    async fn wait_for_close_signal(&self, page: &Page) -> Result<bool> {
        let trigger = serde_json::to_string(self.trigger)?;
        let has_state: bool = page
            .evaluate(format!(
                "(() => {{ const t = document.querySelector({trigger}); \
                 return !!t && t.hasAttribute('aria-expanded'); }})()"
            ))
            .await?
            .into_value()?;
        if has_state {
            wait::for_js_true(
                page,
                &format!(
                    "document.querySelector({trigger})?.getAttribute('aria-expanded') === 'false'"
                ),
                &format!("Escape to set aria-expanded=\"false\" on {}", self.trigger),
            )
            .await?;
            return Ok(false);
        }
        Ok(self.wait_for_focus_on_trigger(page).await.is_ok())
    }

    async fn wait_for_focus_on_trigger(&self, page: &Page) -> Result<()> {
        wait::for_js_true(
            page,
            &format!(
                "document.activeElement === document.querySelector({})",
                serde_json::to_string(self.trigger)?
            ),
            &format!("focus to return to {}", self.trigger),
        )
        .await
    }

    /// Focus must land inside the panel. Read from the document: `focus()` inside a
    /// `visibility: hidden` box silently does nothing (`codebase/use-popover`).
    async fn assert_focus_moved_inside(&self, page: &Page) -> Result<()> {
        let inside: bool = page
            .evaluate(format!(
                r#"(() => {{
                    const panel = document.querySelector({});
                    return !!panel && panel.contains(document.activeElement);
                }})()"#,
                serde_json::to_string(self.panel)?
            ))
            .await?
            .into_value()?;
        if !inside {
            let actual = focus::active_element(page).await?;
            bail!(
                "opening {} did not move focus into {}; the document holds {actual:?}",
                self.trigger,
                self.panel
            );
        }
        Ok(())
    }

    /// Tabs round both ways; focus never leaves the panel. Tabbable count + 2 wraps at least once.
    async fn assert_focus_is_trapped(&self, page: &Page) -> Result<()> {
        let tabbables: usize = page
            .evaluate(format!(
                r#"(() => {{
                    const panel = document.querySelector({});
                    if (!panel) return 0;
                    const focusable = 'a[href],button,input,select,textarea,[tabindex]:not([tabindex="-1"])';
                    return panel.querySelectorAll(focusable).length;
                }})()"#,
                serde_json::to_string(self.panel)?
            ))
            .await?
            .into_value()?;

        // Both directions: a trap can leak backwards from its first item alone.
        for shift in [false, true] {
            for step in 0..tabbables + 2 {
                if shift {
                    keyboard::press_shift(page, keyboard::TAB).await?;
                } else {
                    keyboard::press(page, keyboard::TAB).await?;
                }
                let inside: bool = page
                    .evaluate(format!(
                        "(() => {{ const p = document.querySelector({}); return !!p && p.contains(document.activeElement); }})()",
                        serde_json::to_string(self.panel)?
                    ))
                    .await?
                    .into_value()?;
                if !inside {
                    let actual = focus::active_element(page).await?;
                    bail!(
                        "focus escaped {} after {} {}tab press(es); the document holds {actual:?}",
                        self.panel,
                        step + 1,
                        if shift { "shift+" } else { "" }
                    );
                }
            }
        }
        Ok(())
    }
}
