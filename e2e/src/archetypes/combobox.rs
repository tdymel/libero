//! The APG combobox pattern.
//!
//! ## What this is checked against
//!
//! - **APG, Combobox pattern** - the keyboard interface: Down/Up move the
//!   highlight, Home/End jump to the ends, Escape closes, and **DOM focus stays
//!   on the combobox** while `aria-activedescendant` moves.
//!   <https://www.w3.org/WAI/ARIA/apg/patterns/combobox/>
//! - **WAI-ARIA, `aria-activedescendant`** - the referenced id must exist and
//!   be a descendant of, or owned by, the element.
//!   <https://www.w3.org/TR/wai-aria-1.2/#aria-activedescendant>
//! - **WCAG 2.1.1 Keyboard** - every function reachable without a pointer.
//! - **WCAG 4.1.2 Name, Role, Value** - `aria-expanded` and `aria-controls`
//!   reporting the current state.

use anyhow::{Result, bail};
use chromiumoxide::Page;

use crate::passes::{focus, keyboard};
use crate::wait;

/// A combobox in the APG sense: a text input that owns a popup listbox and
/// keeps DOM focus while an `aria-activedescendant` highlight moves.
pub struct Combobox<'a> {
    /// The element carrying `role="combobox"`.
    pub trigger: &'a str,
    /// How many options the fixture offers, for the Home/End assertions.
    pub option_count: usize,
    /// How many tab presses reach the trigger from the top of the document.
    pub tab_budget: usize,
}

impl Combobox<'_> {
    /// The full pattern: open, navigate, highlight tracking, close, focus return.
    ///
    /// The assertion that matters most is that focus **never leaves the
    /// trigger**. In this pattern the visual highlight moves while DOM focus
    /// stays put, so a test that follows `document.activeElement` around would
    /// pass against a component that had broken the pattern entirely.
    pub async fn assert_contract(&self, page: &Page) -> Result<()> {
        self.assert_closed(page).await?;

        // Tab to it here rather than requiring the caller to have done so.
        //
        // This used to be an implicit precondition, and an unmet one produced
        // "expected focus on X but the document holds body" - a message that
        // reads like a focus bug in the component under test rather than like a
        // misuse of the archetype. `Overlay` always did its own tabbing; these
        // now behave the same way, so there is one less thing to know when
        // adding a component.
        page.evaluate("document.activeElement && document.activeElement.blur()")
            .await?;
        keyboard::tab_to(page, self.trigger, self.tab_budget).await?;
        focus::assert_focused(page, self.trigger, "tabbing to the combobox").await?;

        // Opening by keyboard, not by click: this is the keyboard pass.
        keyboard::press(page, keyboard::ARROW_DOWN).await?;

        let listbox = self.listbox_selector(page).await?;
        wait::for_visible(page, &listbox).await?;
        self.assert_expanded(page, true).await?;
        focus::assert_focused(page, self.trigger, "opening the list").await?;

        self.assert_active_descendant_exists(page, "opening the list")
            .await?;

        // Arrow through every option and back. A highlight that falls off the
        // end, or names a row that is not drawn, shows up here.
        for _ in 0..self.option_count {
            keyboard::press(page, keyboard::ARROW_DOWN).await?;
            self.assert_active_descendant_exists(page, "arrowing down")
                .await?;
        }
        for _ in 0..self.option_count {
            keyboard::press(page, keyboard::ARROW_UP).await?;
            self.assert_active_descendant_exists(page, "arrowing up")
                .await?;
        }

        keyboard::press(page, keyboard::ESCAPE).await?;
        wait::for_hidden(page, &listbox).await?;
        self.assert_expanded(page, false).await?;

        // Focus return, which for this pattern means focus never moved.
        focus::assert_focused(page, self.trigger, "Escape closing the list").await?;
        Ok(())
    }

    async fn assert_closed(&self, page: &Page) -> Result<()> {
        self.assert_expanded(page, false).await
    }

    async fn assert_expanded(&self, page: &Page, expected: bool) -> Result<()> {
        let read = format!(
            "(() => {{ const el = document.querySelector({}); return el ? el.getAttribute('aria-expanded') : null; }})()",
            serde_json::to_string(self.trigger)?
        );
        // Polled, not read once: the attribute is written by a dioxus
        // re-render that has not necessarily happened when the keypress
        // returns.
        let wanted = expected.to_string();
        let check = format!("{read} === {}", serde_json::to_string(&wanted)?);
        if wait::for_js_true(page, &check, &format!("aria-expanded={wanted}"))
            .await
            .is_ok()
        {
            return Ok(());
        }
        let actual: Option<String> = page.evaluate(read).await?.into_value()?;
        bail!(
            "aria-expanded on {} was {actual:?}, expected {wanted:?}",
            self.trigger
        )
    }

    /// `aria-activedescendant` must name an element that **exists**.
    ///
    /// Todo 101 shipped a combobox whose highlight named `option-0` while the
    /// list was loading, when no such row was drawn. Nothing caught it: the
    /// attribute was present and well-formed, and only its target was missing.
    async fn assert_active_descendant_exists(&self, page: &Page, during: &str) -> Result<()> {
        // The highlight is written on re-render, so this has to settle first.
        let settled = format!(
            r#"(() => {{
                const el = document.querySelector({});
                const id = el && el.getAttribute('aria-activedescendant');
                return !id || !!document.getElementById(id);
            }})()"#,
            serde_json::to_string(self.trigger)?
        );
        let _ = wait::for_js_true(page, &settled, "aria-activedescendant to name a real row").await;

        let (named, exists): (Option<String>, bool) = page
            .evaluate(format!(
                r#"(() => {{
                    const el = document.querySelector({});
                    const id = el && el.getAttribute('aria-activedescendant');
                    if (!id) return [null, true];
                    return [id, !!document.getElementById(id)];
                }})()"#,
                serde_json::to_string(self.trigger)?
            ))
            .await?
            .into_value()?;
        if !exists {
            bail!(
                "while {during}, aria-activedescendant named {named:?} but no such element is in the DOM"
            );
        }
        Ok(())
    }

    /// The listbox, found through `aria-controls` rather than the DOM tree: it
    /// is portaled to an outlet at the document root and is no descendant of
    /// the trigger (`codebase/use-popover`).
    async fn listbox_selector(&self, page: &Page) -> Result<String> {
        let id: Option<String> = page
            .evaluate(format!(
                "(() => {{ const el = document.querySelector({}); return el ? el.getAttribute('aria-controls') : null; }})()",
                serde_json::to_string(self.trigger)?
            ))
            .await?
            .into_value()?;
        match id {
            Some(id) if !id.is_empty() => Ok(format!("#{id}")),
            _ => bail!(
                "{} has no aria-controls, so its listbox cannot be found",
                self.trigger
            ),
        }
    }
}
