//! The APG dialog / dismissible-overlay pattern.
//!
//! `Modal`, `Drawer`, `Menu`, `Lightbox`, `Spotlight` and `FloatingWindow` all
//! promise the same four things: it opens, focus moves inside, Escape closes
//! it, and focus returns to whatever opened it. The fourth is the one that
//! breaks silently - nothing looks wrong on screen when focus falls back to
//! `<body>`, and the keyboard user is simply dumped at the top of the document.
//!
//! `principles/focus-after-removal` is the house rule this encodes.
//!
//! ## What this is checked against
//!
//! - **APG, Dialog (Modal) pattern** - focus moves into the dialog on open,
//!   Escape closes it, and focus returns to the element that invoked it.
//!   <https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/>
//! - **WCAG 2.1.2 No Keyboard Trap** - the trap assertion here is the
//!   *intended* modal trap; 2.1.2 requires that Escape is always a way out,
//!   which is why the contract closes with it rather than only tabbing.
//! - **WCAG 2.4.3 Focus Order** - focus return is the half of focus order that
//!   fails silently.
//! - **WCAG 3.2.1 On Focus** - closing must not move focus somewhere
//!   unexpected, which for this pattern means the trigger and nowhere else.

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

        // It must not still be readable once it says it is closed. Read at the
        // close signal, not after the panel has gone: waiting for it to be
        // hidden first waited for exactly what this asserts, so a phantom
        // panel failed as a timeout, or never (review 7, E6).
        dismissal::assert_gone_from_at(page, self.panel).await?;

        // The assertion that silently fails everywhere else. Given time to
        // land, since a component may return focus once its exit has ended.
        if !focus_returned {
            let _ = self.wait_for_focus_on_trigger(page).await;
        }
        focus::assert_focused(page, self.trigger, "Escape closing the overlay").await?;
        Ok(())
    }

    /// Wait for the overlay to report itself closed. Returns whether focus is
    /// already back on the trigger.
    ///
    /// A trigger with `aria-expanded` says so there: a menu's or a popover's.
    /// A dialog's trigger carries no state, and returning focus to it is the
    /// dialog's own close signal (APG). When that never comes, the wait gives
    /// up quietly and the focus assertion below reports it.
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

    /// Focus must land inside the panel, not stay on the trigger.
    ///
    /// Read from the document, never from a `focus()` result: focusing an
    /// element inside a box that is still `visibility: hidden` does nothing and
    /// reports success (`codebase/use-popover`), which is exactly how a
    /// searchable `Select` shipped with a search box that never took focus.
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

    /// Tab all the way round, forwards then backwards, and confirm focus never
    /// leaves the panel.
    ///
    /// The budget is the panel's own tabbable count plus two, so the walk
    /// wraps at least once. Tabbing a fixed large number instead would pass
    /// trivially on a panel with many controls.
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
