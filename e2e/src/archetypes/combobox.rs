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
    /// How many options the fixture offers. Checked against the drawn list
    /// before the walk, so a stale count fails instead of shortening it.
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

        // Once the list is open something must be highlighted, and it must be
        // one of the drawn options. An absent attribute is a failure here: it
        // used to count as valid, so a combobox that never set it passed.
        self.active_descendant(page, &listbox, "opening the list")
            .await?;

        // The ends first, so the walk below starts from a known row whatever
        // the component highlights on opening (`Select` opens on its value).
        let options = self.option_ids(page, &listbox).await?;
        let (first, last) = (&options[0], &options[options.len() - 1]);
        keyboard::press(page, keyboard::END).await?;
        self.expect_active(page, &listbox, last, "pressing End")
            .await?;
        keyboard::press(page, keyboard::HOME).await?;
        self.expect_active(page, &listbox, first, "pressing Home")
            .await?;

        // Every arrow must move the highlight exactly one row. A highlight
        // that stays put, skips a row or names a row that is not drawn fails
        // here. What happens past either end (clamp or wrap) is the
        // component's choice, so it is not pressed.
        for (index, id) in options.iter().enumerate().skip(1) {
            keyboard::press(page, keyboard::ARROW_DOWN).await?;
            self.expect_active(page, &listbox, id, &format!("arrowing down to row {index}"))
                .await?;
        }
        for (index, id) in options.iter().enumerate().rev().skip(1) {
            keyboard::press(page, keyboard::ARROW_UP).await?;
            self.expect_active(page, &listbox, id, &format!("arrowing up to row {index}"))
                .await?;
        }

        self.assert_chords(page, &listbox, first).await?;

        keyboard::press(page, keyboard::ESCAPE).await?;
        wait::for_hidden(page, &listbox).await?;
        self.assert_expanded(page, false).await?;

        // Focus return, which for this pattern means focus never moved.
        focus::assert_focused(page, self.trigger, "Escape closing the list").await?;
        Ok(())
    }

    /// On an open list highlighting `first`: Ctrl and Meta chords are the
    /// caret's or the browser's, Alt+ArrowDown keeps the highlight, Alt+ArrowUp
    /// closes and Alt+ArrowDown reopens (APG). Todo 509.
    async fn assert_chords(&self, page: &Page, listbox: &str, first: &str) -> Result<()> {
        let probe = format!(
            "(t => [t.getAttribute('aria-expanded'), t.getAttribute('aria-activedescendant')])(document.querySelector({}))",
            serde_json::to_string(self.trigger)?
        );
        let navigation = [
            keyboard::ARROW_DOWN,
            keyboard::ARROW_UP,
            keyboard::HOME,
            keyboard::END,
            keyboard::ARROW_LEFT,
            keyboard::ARROW_RIGHT,
        ];
        keyboard::assert_chords_ignored_with(
            page,
            &[("Ctrl", keyboard::CTRL), ("Meta", keyboard::META)],
            &navigation,
            &probe,
        )
        .await?;
        keyboard::assert_chords_ignored_with(
            page,
            &[("Alt", keyboard::ALT)],
            &navigation[2..],
            &probe,
        )
        .await?;

        keyboard::press_with(page, keyboard::ARROW_DOWN, keyboard::ALT).await?;
        page.evaluate("new Promise(r => setTimeout(() => r(1), 60))")
            .await?;
        self.expect_active(
            page,
            listbox,
            first,
            "pressing Alt+ArrowDown on the open list",
        )
        .await?;
        keyboard::press_with(page, keyboard::ARROW_UP, keyboard::ALT).await?;
        wait::for_hidden(page, listbox).await?;
        self.assert_expanded(page, false).await?;
        focus::assert_focused(page, self.trigger, "Alt+ArrowUp closing the list").await?;
        keyboard::press_with(page, keyboard::ARROW_DOWN, keyboard::ALT).await?;
        wait::for_visible(page, listbox).await?;
        self.assert_expanded(page, true).await
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

    /// The ids of the drawn options, in order. The fixture's declared count
    /// must match, so a list that lost or gained rows is not walked silently.
    async fn option_ids(&self, page: &Page, listbox: &str) -> Result<Vec<String>> {
        let ids: Vec<String> = page
            .evaluate(format!(
                "Array.from(document.querySelectorAll({})).map(el => el.id)",
                serde_json::to_string(&format!("{listbox} [role=option]"))?
            ))
            .await?
            .into_value()?;
        if ids.len() != self.option_count {
            bail!(
                "{listbox} drew {} options, the fixture declares {}",
                ids.len(),
                self.option_count
            );
        }
        if ids.iter().any(String::is_empty) {
            bail!("an option in {listbox} has no id, so aria-activedescendant cannot name it");
        }
        Ok(ids)
    }

    /// The highlight, once it names a drawn option. Polled, because it is
    /// written on re-render and not necessarily there when the key returns.
    ///
    /// Todo 101 shipped a combobox whose highlight named `option-0` while the
    /// list was loading, when no such row was drawn: the attribute was present
    /// and well-formed, and only its target was missing. Review 7 (E2) found
    /// the opposite hole: an absent attribute was accepted as valid.
    async fn active_descendant(&self, page: &Page, listbox: &str, during: &str) -> Result<String> {
        let read = format!(
            r#"(() => {{
                const el = document.querySelector({});
                const id = el && el.getAttribute('aria-activedescendant');
                if (!id) return [null, false, false];
                const target = document.getElementById(id);
                const list = document.querySelector({});
                return [id, !!target, !!(target && list && list.contains(target))];
            }})()"#,
            serde_json::to_string(self.trigger)?,
            serde_json::to_string(listbox)?
        );
        // The last reading that named something is kept for the diagnosis. A
        // single read after the deadline can find the list already gone and
        // report "absent" for a highlight that was dangling all along.
        let seen = std::cell::RefCell::new((None::<String>, false, false));
        let _ = wait::until("aria-activedescendant to name an option", || async {
            let reading: (Option<String>, bool, bool) =
                page.evaluate(read.as_str()).await?.into_value()?;
            let done = reading.2;
            if reading.0.is_some() {
                *seen.borrow_mut() = reading;
            }
            Ok(done)
        })
        .await;

        let (named, exists, inside) = seen.into_inner();
        match named {
            None => bail!("while {during}, the list is open but aria-activedescendant is absent"),
            Some(id) if !exists => bail!(
                "while {during}, aria-activedescendant named {id:?} but no such element is in the DOM"
            ),
            Some(id) if !inside => {
                bail!(
                    "while {during}, aria-activedescendant named {id:?}, which is not in {listbox}"
                )
            }
            Some(id) => Ok(id),
        }
    }

    /// The highlight must reach `expected`.
    async fn expect_active(
        &self,
        page: &Page,
        listbox: &str,
        expected: &str,
        during: &str,
    ) -> Result<()> {
        let check = format!(
            "(() => {{ const el = document.querySelector({}); return !!el && el.getAttribute('aria-activedescendant') === {}; }})()",
            serde_json::to_string(self.trigger)?,
            serde_json::to_string(expected)?
        );
        if wait::for_js_true(page, &check, &format!("aria-activedescendant={expected}"))
            .await
            .is_ok()
        {
            return Ok(());
        }
        let actual = self.active_descendant(page, listbox, during).await?;
        bail!("while {during}, aria-activedescendant stayed on {actual:?}, expected {expected:?}")
    }

    /// The listbox, found through `aria-controls` rather than the DOM tree: it
    /// is portaled to an outlet at the document root and is no descendant of
    /// the trigger (`codebase/use-popover`). Polled: the attribute is set only
    /// while the listbox is mounted, by the re-render the opening key starts.
    async fn listbox_selector(&self, page: &Page) -> Result<String> {
        let read = format!(
            "(() => {{ const el = document.querySelector({}); return el ? el.getAttribute('aria-controls') : null; }})()",
            serde_json::to_string(self.trigger)?
        );
        // A timeout falls through to the read, whose `None` is the bail below.
        let _ = wait::for_js_true(page, &format!("!!{read}"), "aria-controls").await;
        let id: Option<String> = page.evaluate(read).await?.into_value()?;
        match id {
            Some(id) if !id.is_empty() => Ok(format!("#{id}")),
            _ => bail!(
                "{} has no aria-controls, so its listbox cannot be found",
                self.trigger
            ),
        }
    }
}
