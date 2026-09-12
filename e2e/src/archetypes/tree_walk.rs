//! The APG tree view pattern.
//!
//! A tree looks like a vertical roving-tabindex strip and is not one, which is
//! why it has its own archetype rather than a flag on [`RovingTabindex`]
//! (todo 310, the same call [`RadioSet`] made for radio groups):
//!
//! - **The navigable set changes while you walk it.** Expanding a row inserts
//!   its children into the walk; collapsing removes them. "Item N of M" is a
//!   stable statement in a strip and is not one here, so every step re-reads
//!   the visible rows rather than counting once at the start.
//! - **Right and Left are structure, not movement.** Right expands a collapsed
//!   row, or moves into an expanded one; Left collapses an expanded row, or
//!   moves out to its parent. A strip's Left/Right are the same movement as
//!   Up/Down on the other axis, which is why `RovingTabindex` has an
//!   [`Orientation`] and a tree cannot use it.
//! - **The hierarchy is part of the contract.** A strip is flat, so it owes a
//!   reader nothing about shape; a tree owes it the level, and ARIA takes that
//!   either from nested `role=group` or from `aria-level`/`aria-setsize`/
//!   `aria-posinset`. Either is correct, so this asserts the disjunction.
//!
//! What it shares with the strip is the single tab stop and the vertical
//! arrows, and that half is measured here with the strip's own
//! [`count_tab_stops`] rather than a second copy of it.
//!
//! It asserts only what APG requires, so a component that answers more is not
//! reported:
//!
//! - **`*` is not checked.** APG marks expand-all optional.
//! - **Typeahead is not checked.** APG recommends it; it does not require it.
//! - **Enter and Space are not checked.** APG says they perform "the default
//!   action", and what that is belongs to the component - `Tree` leaves it to
//!   the caller's `render_node` entirely. Asserting one meaning here would be
//!   bending the archetype around one widget.
//! - **The end of the walk is not checked.** APG says Down moves to the next
//!   focusable node and says nothing about the last one, so whether a tree
//!   wraps is the component's choice and not the pattern's.
//!
//! [`RovingTabindex`]: super::RovingTabindex
//! [`RadioSet`]: super::RadioSet
//! [`Orientation`]: super::Orientation
//!
//! ## What this is checked against
//!
//! - **APG, Tree View pattern** - the keyboard table, and which of its keys are
//!   required rather than optional or recommended.
//!   <https://www.w3.org/WAI/ARIA/apg/patterns/treeview/>
//! - **WAI-ARIA, `treeitem`** - `aria-level`, `aria-setsize` and `aria-posinset`
//!   are required only where the DOM does not already carry the hierarchy, so a
//!   tree built from nested `role=group` owes none of them.
//!   <https://www.w3.org/TR/wai-aria-1.2/#treeitem>
//! - **WCAG 2.1.1 Keyboard** and **2.4.3 Focus Order**.

use anyhow::{Result, bail};
use chromiumoxide::Page;

use crate::passes::keyboard;
use crate::wait;

use super::roving::{count_tab_stops, reset_tab_position};

pub struct TreeWalk<'a> {
    /// Every **visible** row, e.g. `[role=treeitem]`. Rows inside a collapsed
    /// branch are not rendered, so this set grows and shrinks under the walk.
    pub rows: &'a str,
    /// A row that starts **collapsed and has children**, by index in the
    /// visible order. Right and Left are the whole difference between this
    /// pattern and a strip, and neither can be exercised without one.
    pub collapsed_parent: usize,
    /// A row that has **no children**, by index in the visible order. Right on
    /// one must do nothing, which is the case a strip would get wrong by
    /// moving.
    pub leaf: usize,
    /// How many Tab presses from the top of the document may pass before focus
    /// enters the tree.
    pub tab_budget: usize,
}

impl TreeWalk<'_> {
    pub async fn assert_contract(&self, page: &Page) -> Result<()> {
        let count = self.visible_count(page).await?;
        if count < 3 {
            bail!(
                "a tree contract needs at least three visible rows, found {count} at {}",
                self.rows
            );
        }
        self.assert_fixture_shape(page, count).await?;

        // The half a strip and a tree share, measured the strip's way.
        let tabbable = count_tab_stops(page, self.rows).await?;
        if tabbable != 1 {
            bail!(
                "{} has {tabbable} tab stops; a tree is one, with the arrows moving inside it",
                self.rows
            );
        }

        self.enter(page).await?;
        // Ctrl/Alt/Meta chords are the browser's: no move, no expansion.
        let rows = serde_json::to_string(self.rows)?;
        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_DOWN,
                keyboard::ARROW_UP,
                keyboard::ARROW_RIGHT,
                keyboard::ARROW_LEFT,
                keyboard::HOME,
                keyboard::END,
            ],
            &format!(
                "[...document.querySelectorAll({rows})].map(row => \
                 (row === document.activeElement) + ':' + row.getAttribute('aria-expanded'))"
            ),
        )
        .await?;
        self.assert_vertical_walk(page, count).await?;
        self.assert_expansion(page).await?;
        self.assert_a_leaf_does_not_open(page).await
    }

    /// What the declaration promises is actually in the page. A fixture whose
    /// `collapsed_parent` is a leaf would make every expansion assertion below
    /// pass by doing nothing.
    async fn assert_fixture_shape(&self, page: &Page, count: usize) -> Result<()> {
        for (index, what) in [
            (self.collapsed_parent, "collapsed_parent"),
            (self.leaf, "leaf"),
        ] {
            if index >= count {
                bail!("{what} is row {index} of {count} in {}", self.rows);
            }
        }
        if self.expanded_at(page, self.collapsed_parent).await? != Some(false) {
            bail!(
                "row {} of {} does not start collapsed, as `collapsed_parent` says; a row that \
                 is already open cannot show that Right opens one",
                self.collapsed_parent,
                self.rows
            );
        }
        if self.expanded_at(page, self.leaf).await?.is_some() {
            bail!(
                "row {} of {} carries aria-expanded, so it is a branch and not the `leaf` the \
                 declaration promises",
                self.leaf,
                self.rows
            );
        }
        Ok(())
    }

    /// The level reaches a reader one of the two ways ARIA allows.
    ///
    /// Nested `role=group` is the structural form and needs no attributes; a
    /// flattened tree has to say the same thing with `aria-level`,
    /// `aria-setsize` and `aria-posinset`. Requiring the attributes outright
    /// would report a defect against every correctly nested tree.
    ///
    /// Called with a branch open, because an all-collapsed tree has no nesting
    /// on show and the structural form would have nothing to be read from.
    async fn assert_hierarchy_is_exposed(&self, page: &Page) -> Result<()> {
        let sel = serde_json::to_string(self.rows)?;
        let verdict: String = page
            .evaluate(format!(
                "(() => {{ const rows = [...document.querySelectorAll({sel})]; \
                 const flat = rows.every(r => r.hasAttribute('aria-level') \
                   && r.hasAttribute('aria-setsize') && r.hasAttribute('aria-posinset')); \
                 if (flat) return 'ok'; \
                 const branches = rows.filter(r => r.getAttribute('aria-expanded') === 'true'); \
                 if (!branches.length) return 'no open branch to check the nesting on'; \
                 const bad = branches.filter(r => \
                   ![...r.querySelectorAll('[role=group]')].some(g => g.querySelector({sel}))); \
                 return bad.length ? bad.length + ' open branch(es) hold their rows outside a \
                   role=group, and carry no aria-level either' : 'ok'; }})()"
            ))
            .await?
            .into_value()?;
        if verdict != "ok" {
            bail!(
                "{} does not expose its hierarchy: {verdict}. ARIA takes a treeitem's level from \
                 nested role=group or from aria-level/aria-setsize/aria-posinset - one or the \
                 other, not neither",
                self.rows
            );
        }
        Ok(())
    }

    /// Tab in, from the top of the document.
    async fn enter(&self, page: &Page) -> Result<()> {
        reset_tab_position(page).await?;
        for _ in 0..self.tab_budget {
            keyboard::press(page, keyboard::TAB).await?;
            if self.focused_index(page).await?.is_some() {
                return Ok(());
            }
        }
        bail!(
            "{} Tab presses did not reach any row of {}; a tree is one tab stop and it has to be \
             reachable",
            self.tab_budget,
            self.rows
        )
    }

    /// Down and Up walk the **visible** rows, and Home and End reach the ends.
    ///
    /// The count is re-read on every press rather than trusted from the start:
    /// that is the difference this archetype exists for, and a walk that
    /// assumed a fixed set would pass here and break the moment a row opened.
    async fn assert_vertical_walk(&self, page: &Page, count: usize) -> Result<()> {
        keyboard::press(page, keyboard::HOME).await?;
        if self.settle_on(page, 0).await? != Some(0) {
            bail!("Home did not move to the first row of {}", self.rows);
        }

        for step in 1..count {
            keyboard::press(page, keyboard::ARROW_DOWN).await?;
            let now = self.visible_count(page).await?;
            if now != count {
                bail!(
                    "arrowing down {} changed the visible row count from {count} to {now}; the \
                     arrows walk a tree, they do not open it",
                    self.rows
                );
            }
            let at = self.settle_on(page, step).await?;
            if at != Some(step) {
                bail!(
                    "after {step} Down press(es) in {}, focus is on row {at:?}, expected {step}",
                    self.rows
                );
            }
        }

        keyboard::press(page, keyboard::ARROW_UP).await?;
        if self.settle_on(page, count - 2).await? != Some(count - 2) {
            bail!("Up did not move back a row in {}", self.rows);
        }
        keyboard::press(page, keyboard::END).await?;
        if self.settle_on(page, count - 1).await? != Some(count - 1) {
            bail!("End did not move to the last row of {}", self.rows);
        }
        Ok(())
    }

    /// Right opens, then descends; Left ascends, then closes.
    ///
    /// Each half is two different jobs for one key, chosen by whether the row
    /// is open - which is exactly what a flat archetype has no way to say.
    async fn assert_expansion(&self, page: &Page) -> Result<()> {
        let before = self.visible_count(page).await?;
        self.walk_to(page, self.collapsed_parent).await?;
        self.mark_focus(page).await?;

        // Right on a closed branch opens it and moves nothing.
        keyboard::press(page, keyboard::ARROW_RIGHT).await?;
        let _ = wait::for_js_true(
            page,
            &format!("{} > {before}", self.count_js()?),
            "the branch to open",
        )
        .await;
        let after = self.visible_count(page).await?;
        if after <= before {
            bail!(
                "Right on the closed branch (row {}) of {} left {after} visible rows; it should \
                 open the branch and bring its children into the walk",
                self.collapsed_parent,
                self.rows
            );
        }
        if !self.focus_is_marked(page).await? {
            bail!(
                "Right on the closed branch (row {}) of {} moved focus; opening a branch leaves \
                 focus on it, and only a second Right descends",
                self.collapsed_parent,
                self.rows
            );
        }

        // With a branch open, the nesting is on show and can be read.
        self.assert_hierarchy_is_exposed(page).await?;

        // Right again descends into the first child.
        keyboard::press(page, keyboard::ARROW_RIGHT).await?;
        let _ = wait::for_js_true(
            page,
            &self.focus_is_first_child_js()?,
            "focus on the first child",
        )
        .await;
        if !self.focus_is_first_child(page).await? {
            bail!(
                "Right on the open branch (row {}) of {} did not move focus to its first child",
                self.collapsed_parent,
                self.rows
            );
        }

        // Left ascends back out of it.
        keyboard::press(page, keyboard::ARROW_LEFT).await?;
        let _ = wait::for_js_true(
            page,
            &self.focus_is_marked_js()?,
            "focus back on the branch",
        )
        .await;
        if !self.focus_is_marked(page).await? {
            bail!(
                "Left on a child row of {} did not move focus to its parent",
                self.rows
            );
        }

        // Left again closes it, and moves nothing.
        keyboard::press(page, keyboard::ARROW_LEFT).await?;
        let _ = wait::for_js_true(
            page,
            &format!("{} === {before}", self.count_js()?),
            "the branch to close",
        )
        .await;
        let closed = self.visible_count(page).await?;
        if closed != before {
            bail!(
                "Left on the open branch (row {}) of {} left {closed} visible rows, not the \
                 {before} it started with; it should close the branch",
                self.collapsed_parent,
                self.rows
            );
        }
        if !self.focus_is_marked(page).await? {
            bail!(
                "Left on the open branch (row {}) of {} moved focus; closing a branch leaves \
                 focus on it",
                self.collapsed_parent,
                self.rows
            );
        }
        Ok(())
    }

    /// A row with no children answers Right with nothing: no movement, and no
    /// row appearing from somewhere else.
    async fn assert_a_leaf_does_not_open(&self, page: &Page) -> Result<()> {
        self.walk_to(page, self.leaf).await?;
        self.mark_focus(page).await?;
        let before = self.visible_count(page).await?;

        keyboard::press(page, keyboard::ARROW_RIGHT).await?;
        let after = self.visible_count(page).await?;
        if after != before || !self.focus_is_marked(page).await? {
            bail!(
                "Right on the leaf (row {}) of {} moved focus or changed the visible rows \
                 ({before} -> {after}); a row with no children has nothing to open or descend \
                 into",
                self.leaf,
                self.rows
            );
        }
        Ok(())
    }

    /// Put focus on a row by walking to it with the keyboard.
    ///
    /// Home then Down, rather than a click: a click on a tree row means
    /// whatever the component decided it means, and may select or toggle.
    async fn walk_to(&self, page: &Page, index: usize) -> Result<()> {
        keyboard::press(page, keyboard::HOME).await?;
        let _ = self.settle_on(page, 0).await?;
        for _ in 0..index {
            keyboard::press(page, keyboard::ARROW_DOWN).await?;
        }
        let at = self.settle_on(page, index).await?;
        if at != Some(index) {
            bail!("walking to row {index} of {} landed on {at:?}", self.rows);
        }
        Ok(())
    }

    /// Remember the focused element itself, so a later check is identity and
    /// not an index - the indices move as rows open and close.
    async fn mark_focus(&self, page: &Page) -> Result<()> {
        page.evaluate("window.__e2eTreeRow = document.activeElement")
            .await?;
        Ok(())
    }

    fn focus_is_marked_js(&self) -> Result<String> {
        Ok("document.activeElement === window.__e2eTreeRow".to_string())
    }

    async fn focus_is_marked(&self, page: &Page) -> Result<bool> {
        Ok(page
            .evaluate(self.focus_is_marked_js()?)
            .await?
            .into_value()?)
    }

    /// The focused element is the first row nested inside the marked one.
    fn focus_is_first_child_js(&self) -> Result<String> {
        Ok(format!(
            "(() => {{ const first = window.__e2eTreeRow.querySelector({}); \
             return !!first && document.activeElement === first; }})()",
            serde_json::to_string(self.rows)?
        ))
    }

    async fn focus_is_first_child(&self, page: &Page) -> Result<bool> {
        Ok(page
            .evaluate(self.focus_is_first_child_js()?)
            .await?
            .into_value()?)
    }

    fn count_js(&self) -> Result<String> {
        Ok(format!(
            "document.querySelectorAll({}).length",
            serde_json::to_string(self.rows)?
        ))
    }

    async fn visible_count(&self, page: &Page) -> Result<usize> {
        wait::for_selector(page, self.rows).await?;
        Ok(page.evaluate(self.count_js()?).await?.into_value()?)
    }

    /// `aria-expanded` on a row, as a tri-state: `Some(true)` open,
    /// `Some(false)` closed, `None` a leaf.
    ///
    /// The absent attribute comes back as the string `"none"` rather than as
    /// `null`: CDP sends a JS `null` as no value at all, and deserializing that
    /// into an `Option` fails with "No value found" instead of giving `None`.
    async fn expanded_at(&self, page: &Page, index: usize) -> Result<Option<bool>> {
        let value: String = page
            .evaluate(format!(
                "[...document.querySelectorAll({})][{index}]?.getAttribute('aria-expanded') \
                 ?? 'none'",
                serde_json::to_string(self.rows)?
            ))
            .await?
            .into_value()?;
        Ok((value != "none").then_some(value == "true"))
    }

    fn index_js(&self) -> Result<String> {
        Ok(format!(
            "[...document.querySelectorAll({})].indexOf(document.activeElement)",
            serde_json::to_string(self.rows)?
        ))
    }

    async fn settle_on(&self, page: &Page, expected: usize) -> Result<Option<usize>> {
        let check = format!("{} === {expected}", self.index_js()?);
        let _ = wait::for_js_true(page, &check, &format!("focus on row {expected}")).await;
        self.focused_index(page).await
    }

    async fn focused_index(&self, page: &Page) -> Result<Option<usize>> {
        let index: i64 = page.evaluate(self.index_js()?).await?.into_value()?;
        Ok(usize::try_from(index).ok())
    }
}
