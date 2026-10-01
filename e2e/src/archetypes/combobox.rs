//! The APG combobox pattern: <https://www.w3.org/WAI/ARIA/apg/patterns/combobox/>.
//! DOM focus stays on the combobox while `aria-activedescendant` names a drawn option.

use anyhow::{Result, bail};
use chromiumoxide::Page;

use crate::passes::{focus, keyboard};
use crate::{clock, wait};

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
    /// The full pattern: open, navigate, close. Focus must never leave the trigger.
    pub async fn assert_contract(&self, page: &Page) -> Result<()> {
        self.assert_closed(page).await?;

        // Tabs to it itself, like `Overlay`: callers need no focus precondition.
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

        // An open list must highlight a drawn option; an absent attribute fails.
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

        // Each arrow moves exactly one row. Past the ends (clamp or wrap) is the component's choice.
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

    /// Ctrl/Meta chords are ignored; Alt+ArrowUp closes, Alt+ArrowDown keeps or reopens (APG, 509).
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

        // Keeps the state it found, so read it once the handler ran, not after a sleep.
        keyboard::press_with(page, keyboard::ARROW_DOWN, keyboard::ALT).await?;
        clock::settle(page).await?;
        self.assert_expanded(page, true).await?;
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
        // Polled: the re-render may not have run when the keypress returns.
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

    /// The highlight, polled until it names a drawn option in the listbox.
    /// Absent and dangling ids both fail (todo 101, review 7 E2).
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
        // Keeps the last named reading: a read after the deadline may find the list gone.
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

    /// The listbox via `aria-controls`: it is portaled, not a descendant of the trigger.
    /// Polled, since the attribute is set only once the listbox mounts.
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
