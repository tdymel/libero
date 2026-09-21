//! The APG radio group pattern: selection follows focus, both arrow axes wrap, Tab enters at
//! the checked radio. Unlike `RovingTabindex`, Home and End are not part of it (todo 310).

use anyhow::{Result, bail};
use chromiumoxide::Page;

use crate::passes::keyboard::{self, Key};
use crate::wait;

pub struct RadioSet<'a> {
    /// Every radio in the group, e.g. `[role=radiogroup] input[type=radio]`.
    pub radios: &'a str,
    /// Which radio starts checked. Not the first, or "Tab enters at the checked radio" cannot fail.
    pub checked: usize,
    /// How many Tab presses from the top of the document may pass before
    /// focus enters the group.
    pub tab_budget: usize,
}

impl RadioSet<'_> {
    pub async fn assert_contract(&self, page: &Page) -> Result<()> {
        let count = self.count(page).await?;
        if count < 3 {
            bail!(
                "a radio group contract needs at least three radios to tell wrapping from \
                 stopping, found {count} at {}",
                self.radios
            );
        }
        if self.checked == 0 || self.checked >= count {
            bail!(
                "start the fixture with a radio other than the first checked (got {}), or \
                 entering at the checked one proves nothing",
                self.checked
            );
        }
        if self.checked_index(page).await? != Some(self.checked) {
            bail!(
                "{} does not start with radio {} checked, as the declaration says",
                self.radios,
                self.checked
            );
        }

        // Tab in: lands on the checked radio.
        page.evaluate("document.activeElement && document.activeElement.blur()")
            .await?;
        let mut entered = None;
        for _ in 0..self.tab_budget {
            keyboard::press(page, keyboard::TAB).await?;
            entered = self.focused_index(page).await?;
            if entered.is_some() {
                break;
            }
        }
        if entered != Some(self.checked) {
            bail!(
                "tabbing into {} focused radio {entered:?}; APG enters a radio group at the \
                 checked radio, {}",
                self.radios,
                self.checked
            );
        }

        // Forward on both axes, all the way round: that crosses the wrap.
        let mut at = self.checked;
        for (press, key) in [keyboard::ARROW_DOWN, keyboard::ARROW_RIGHT]
            .into_iter()
            .cycle()
            .take(count)
            .enumerate()
        {
            at = (at + 1) % count;
            self.step(page, key, at, &format!("forward press {}", press + 1))
                .await?;
        }

        // Back on both axes, all the way round again.
        for (press, key) in [keyboard::ARROW_UP, keyboard::ARROW_LEFT]
            .into_iter()
            .cycle()
            .take(count)
            .enumerate()
        {
            at = (at + count - 1) % count;
            self.step(page, key, at, &format!("backward press {}", press + 1))
                .await?;
        }

        // A native radio leaves Alt/Ctrl/Meta+arrow to the browser.
        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_DOWN,
                keyboard::ARROW_RIGHT,
                keyboard::ARROW_UP,
                keyboard::ARROW_LEFT,
            ],
            &format!(
                "[{}, {}]",
                self.index_js("document.activeElement")?,
                self.checked_js()?
            ),
        )
        .await?;

        // Selection follows focus, so Space can only show it keeps the focused radio checked.
        keyboard::press(page, keyboard::SPACE).await?;
        let _ = wait::for_js_true(
            page,
            &format!("{} === {at}", self.checked_js()?),
            "Space to leave the focused radio checked",
        )
        .await;
        let (focused, checked) = (
            self.focused_index(page).await?,
            self.checked_index(page).await?,
        );
        if focused != Some(at) || checked != Some(at) {
            bail!(
                "Space on radio {at} of {} left focus on {focused:?} and radio {checked:?} \
                 checked; it should check the focused radio",
                self.radios
            );
        }

        // One tab stop, measured by tabbing: native radios group by `name`, not `tabindex`.
        // Needs a control on each side: at the document edge Chromium parks one Shift+Tab.
        keyboard::press(page, keyboard::TAB).await?;
        self.assert_left(page, "Tab").await?;
        // Back in, onto the radio the arrows left checked, and out again.
        keyboard::press_shift(page, keyboard::TAB).await?;
        let returned = self.settle_on(page, at).await?;
        if returned != Some(at) {
            bail!(
                "Shift+Tab back into {} focused radio {returned:?}; the checked radio {at} is \
                 the group's one tab stop",
                self.radios
            );
        }
        keyboard::press_shift(page, keyboard::TAB).await?;
        self.assert_left(page, "Shift+Tab").await?;
        Ok(())
    }

    /// One arrow press: focus lands on `expected`, and `expected` is checked.
    async fn step(&self, page: &Page, key: Key, expected: usize, which: &str) -> Result<()> {
        keyboard::press(page, key).await?;
        let focused = self.settle_on(page, expected).await?;
        if focused != Some(expected) {
            bail!(
                "{which} ({}) in {}: focus is on radio {focused:?}, expected {expected}",
                key.key,
                self.radios
            );
        }
        let _ = wait::for_js_true(
            page,
            &format!("{} === {expected}", self.checked_js()?),
            "the checked radio to follow focus",
        )
        .await;
        let checked = self.checked_index(page).await?;
        if checked != Some(expected) {
            bail!(
                "{which} ({}) in {} moved focus to radio {expected} but radio {checked:?} is \
                 checked; in a radio group selection follows focus",
                key.key,
                self.radios
            );
        }
        Ok(())
    }

    /// Focus is on no radio in the group: a Tab left it rather than walking
    /// its radios.
    async fn assert_left(&self, page: &Page, key: &str) -> Result<()> {
        let _ = wait::for_js_true(
            page,
            &format!("{} === -1", self.index_js("document.activeElement")?),
            "focus to leave the group",
        )
        .await;
        if let Some(still) = self.focused_index(page).await? {
            bail!(
                "{key} from inside {} landed on radio {still}; a radio group is one tab stop \
                 and {key} should leave it",
                self.radios
            );
        }
        Ok(())
    }

    fn index_js(&self, of: &str) -> Result<String> {
        Ok(format!(
            "[...document.querySelectorAll({})].indexOf({of})",
            serde_json::to_string(self.radios)?
        ))
    }

    /// Native `checked` or `aria-checked`, whichever the radio uses.
    fn checked_js(&self) -> Result<String> {
        Ok(format!(
            "[...document.querySelectorAll({})].findIndex(el => el.checked === true || \
             el.getAttribute('aria-checked') === 'true')",
            serde_json::to_string(self.radios)?
        ))
    }

    async fn count(&self, page: &Page) -> Result<usize> {
        wait::for_selector(page, self.radios).await?;
        Ok(page
            .evaluate(format!(
                "document.querySelectorAll({}).length",
                serde_json::to_string(self.radios)?
            ))
            .await?
            .into_value()?)
    }

    async fn checked_index(&self, page: &Page) -> Result<Option<usize>> {
        let index: i64 = page.evaluate(self.checked_js()?).await?.into_value()?;
        Ok(usize::try_from(index).ok())
    }

    async fn settle_on(&self, page: &Page, expected: usize) -> Result<Option<usize>> {
        let check = format!(
            "{} === {expected}",
            self.index_js("document.activeElement")?
        );
        let _ = wait::for_js_true(page, &check, &format!("focus on radio {expected}")).await;
        self.focused_index(page).await
    }

    async fn focused_index(&self, page: &Page) -> Result<Option<usize>> {
        let index: i64 = page
            .evaluate(self.index_js("document.activeElement")?)
            .await?
            .into_value()?;
        Ok(usize::try_from(index).ok())
    }
}
