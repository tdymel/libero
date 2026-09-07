//! The battery every component gets for free.
//!
//! This is the piece the next implementer actually touches. Adding a component
//! should be a fixture route and a dozen declarative lines, not a file of
//! copied browser plumbing - if it is not, the suite stops growing after about
//! five components and the rest of the library goes untested.
//!
//! ```ignore
//! #[test]
//! fn autocomplete_meets_the_baseline() {
//!     Suite::new("autocomplete", "/autocomplete")
//!         .focusable("[role=combobox]")
//!         .waive(contrast::TODO_297)
//!         .state("open", &[Step::TabTo("[role=combobox]"), Step::Press(ARROW_DOWN)], "[role=listbox]")
//!         .run();
//! }
//! ```
//!
//! What it deliberately does **not** do is the behaviour contract. That is
//! per-component by nature, and hiding it behind a builder would mean either a
//! builder that grows a method per component, or a suite that quietly tests
//! nothing specific. Contracts live in `archetypes` (shared by a pattern) or in
//! the component's own test (unique to it).

use crate::passes::keyboard::{self, Key};
use crate::passes::{contrast, focus, pointer, target_size};
use crate::{Fixture, Viewport, ax, browser};

/// How long a state's entry animation may take before the wait fails. Well
/// past anything the theme animates (a notification's entry is 200 ms), and
/// short enough that a stuck animation is a failure within a test's patience.
const ANIMATION_BUDGET: std::time::Duration = std::time::Duration::from_secs(5);

/// One move towards a state worth snapshotting.
///
/// Deliberately an enum and not a closure. An async closure in a builder means
/// lifetimes and boxing in every call site, and the three moves below cover
/// every state this library's components have: get to a control, press
/// something, click something. When a fourth is genuinely needed, add it here
/// rather than opening the door to arbitrary code in a declaration.
#[derive(Clone, Copy)]
pub enum Step {
    /// Tab until this selector holds focus.
    TabTo(&'static str),
    /// Press a key.
    Press(Key),
    /// Click an element at its centre.
    Click(&'static str),
}

/// A control the target-size pass must measure, and which of WCAG 2.5.8's two
/// ways of passing it may use.
struct Target {
    selector: &'static str,
    /// Whether an undersized target may conform through the spacing exception.
    /// Opt-in per control, so a page that leans on the exception says so.
    spacing_exception: bool,
}

struct State {
    name: &'static str,
    steps: &'static [Step],
    /// What must become visible before the state counts as reached. Waiting on
    /// *placed* rather than on a sleep is what keeps this from being flaky.
    /// It must not be visible before the steps run, or the wait proves nothing.
    settled: &'static str,
}

/// A component's generic pass battery.
pub struct Suite {
    name: &'static str,
    route: &'static str,
    root: &'static str,
    focusable: Vec<&'static str>,
    targets: Vec<Target>,
    waivers: &'static [contrast::Waiver],
    covers: Vec<&'static str>,
    snapshot: bool,
    states: Vec<State>,
    viewports: Vec<Viewport>,
    tab_budget: usize,
}

impl Suite {
    pub fn new(name: &'static str, route: &'static str) -> Self {
        Suite {
            name,
            route,
            // Everything the fixture renders, portal outlet included - the
            // marker wraps the provider for exactly that reason. A component's
            // own root would miss a portaled popover, which is the whole
            // scoping problem.
            root: "[data-fixture-ready]",
            focusable: Vec::new(),
            targets: Vec::new(),
            waivers: &[],
            covers: Vec::new(),
            snapshot: true,
            states: Vec::new(),
            viewports: Viewport::ALL.to_vec(),
            tab_budget: 10,
        }
    }

    /// Narrow the subtree axe and the snapshot look at.
    pub fn root(mut self, root: &'static str) -> Self {
        self.root = root;
        self
    }

    /// A control that must show a focus ring when tabbed to. Repeatable.
    pub fn focusable(mut self, selector: &'static str) -> Self {
        self.focusable.push(selector);
        self
    }

    /// A control that must meet WCAG 2.5.8's 24x24. Repeatable.
    pub fn targets(mut self, selector: &'static str) -> Self {
        self.targets.push(Target {
            selector,
            spacing_exception: false,
        });
        self
    }

    /// A control that may be under 24x24 and conform through WCAG 2.5.8's
    /// **spacing exception** instead: a 24px circle on its centre reaching no
    /// other target. Repeatable.
    ///
    /// The exception is a real part of the criterion, and several units need
    /// it: a `RadioGroup` row, a `Notifications` close button. It is still
    /// opt-in rather than automatic, for two reasons. It is a weaker claim, and a unit
    /// that makes it should say so where a reviewer reads it. And it is only
    /// sound when the selector names the **press target** rather than what is
    /// drawn: the exception measures clearance around a region, so pointing it
    /// at a 20px glyph inside a 44px button computes clearance nobody needs.
    pub fn targets_spaced(mut self, selector: &'static str) -> Self {
        self.targets.push(Target {
            selector,
            spacing_exception: true,
        });
        self
    }

    /// Known, tracked axe violations. Every waiver names a todo, and waived
    /// violations are still printed.
    pub fn waive(mut self, waivers: &'static [contrast::Waiver]) -> Self {
        self.waivers = waivers;
        self
    }

    /// Text that axe must be shown to have **looked at**, not merely found
    /// nothing wrong with. Repeatable.
    ///
    /// `assert_clean` reports absence, so a rule that never ran and a rule that
    /// found nothing are the same green. Todo 327 was four components' worth of
    /// the first kind, surviving a `#ddd` plant in silence. Point this at the
    /// text a unit believes it is checking - a result row, a dialog's body -
    /// and the belief becomes an assertion.
    ///
    /// Like `targets`, it is measured wherever the text exists: content that
    /// only appears in an open state matches nothing at rest. A selector that
    /// matched in no state at all is a failure, not a pass.
    pub fn contrast_covers(mut self, selector: &'static str) -> Self {
        self.covers.push(selector);
        self
    }

    /// Snapshot the component in another state, and run axe there too.
    ///
    /// A component's resting state is rarely the interesting one. An
    /// accessibility tree that is correct closed and wrong open is the normal
    /// shape of these bugs, so a suite that only ever looks at the initial
    /// render tests the easy half.
    pub fn state(
        mut self,
        name: &'static str,
        steps: &'static [Step],
        settled: &'static str,
    ) -> Self {
        self.states.push(State {
            name,
            steps,
            settled,
        });
        self
    }

    /// Turn off the accessibility-tree baseline, for a fixture whose tree is
    /// genuinely not stable. Prefer fixing the fixture.
    pub fn no_snapshot(mut self) -> Self {
        self.snapshot = false;
        self
    }

    pub fn tab_budget(mut self, budget: usize) -> Self {
        self.tab_budget = budget;
        self
    }

    /// Run the battery at every viewport.
    ///
    /// On failure it writes a screenshot and names it in the panic, because a
    /// layout or focus failure described only in prose costs the next person a
    /// re-run to see.
    pub fn run(self) {
        browser::block_on(async move {
            for viewport in self.viewports.iter().copied() {
                if let Err(error) = self.run_at(viewport).await {
                    panic!("{} at {}: {error:?}", self.name, viewport.name());
                }
            }
        });
    }

    async fn run_at(&self, viewport: Viewport) -> anyhow::Result<()> {
        let fixture = Fixture::open(self.route, viewport).await?;

        let outcome = self.battery(&fixture).await;

        if let Err(error) = outcome {
            let shot = fixture.screenshot(self.name).await;
            let where_to_look = match shot {
                Ok(path) => format!("\n  screenshot: {}", path.display()),
                Err(e) => format!("\n  (no screenshot: {e})"),
            };
            // Closed on the failure path too. Without this a run with several
            // failures leaves a page open per failure, and with a shared
            // browser those outlive the test that made them.
            let _ = fixture.close().await;
            return Err(error.context(format!("at {}{where_to_look}", viewport.name())));
        }

        fixture.close().await?;
        Ok(())
    }

    async fn battery(&self, fixture: &Fixture) -> anyhow::Result<()> {
        let page = &fixture.page;

        // Contrast and the ARIA-validity rules, at rest.
        contrast::assert_clean_except(page, self.root, self.waivers).await?;
        let mut covered: Vec<usize> = vec![0; self.covers.len()];
        self.assert_coverage(page, &mut covered).await?;

        // A ring on every control that can be tabbed to, and enough contrast on
        // it to be seen.
        for selector in &self.focusable {
            let ring = focus::assert_focus_ring(page, selector, self.tab_budget).await?;
            focus::assert_ring_contrast(&ring)?;
        }

        // Target sizes are measured wherever the control exists. A control that
        // only appears in an open state - an option in a dropdown, a button in
        // a dialog - matches nothing at rest, and treating that as a failure
        // would make the check unusable for exactly the components that need
        // it most.
        let mut seen: Vec<bool> = vec![false; self.targets.len()];
        self.measure_targets(page, &mut seen).await?;

        if self.snapshot {
            self.take_snapshot(fixture, "rest").await?;
        }

        // Then every declared state, snapshotted and axe-checked in place.
        for state in &self.states {
            self.reach(page, state).await?;
            contrast::assert_clean_except(page, self.root, self.waivers).await?;
            self.assert_coverage(page, &mut covered).await?;
            self.measure_targets(page, &mut seen).await?;
            if self.snapshot {
                self.take_snapshot(fixture, state.name).await?;
            }
        }

        // Same rule as the targets below: a coverage selector that found text
        // in no state checked nothing, which is the failure this method exists
        // to make impossible.
        for (selector, found) in self.covers.iter().zip(&covered) {
            if *found == 0 {
                anyhow::bail!(
                    "contrast coverage: {selector} held no on-screen text at rest or in any \
                     declared state, so requiring axe to have checked it asserts nothing"
                );
            }
        }

        // A selector that matched in no state at all is a stale test, not a
        // pass. Saying so is what stops a renamed role silently disabling the
        // check.
        for (target, matched) in self.targets.iter().zip(&seen) {
            if !matched {
                anyhow::bail!(
                    "target size: {} matched nothing at rest or in any declared state",
                    target.selector
                );
            }
        }

        // Last, so it covers everything the battery just did - including the
        // first mount, since the recorder attached before navigation.
        fixture
            .console
            .assert_clean(&format!("the {} battery", self.name))?;
        Ok(())
    }

    /// `covered[i]` accumulates the most text `covers[i]` ever held, so a
    /// selector that matches only in an open state still counts.
    async fn assert_coverage(
        &self,
        page: &chromiumoxide::Page,
        covered: &mut [usize],
    ) -> anyhow::Result<()> {
        for (index, selector) in self.covers.iter().enumerate() {
            let wanted = contrast::assert_covers(page, self.root, selector).await?;
            covered[index] = covered[index].max(wanted);
        }
        Ok(())
    }

    async fn measure_targets(
        &self,
        page: &chromiumoxide::Page,
        seen: &mut [bool],
    ) -> anyhow::Result<()> {
        for (index, target) in self.targets.iter().enumerate() {
            let selector = target.selector;
            if target.spacing_exception {
                let measured = target_size::measure_spacing(page, selector).await?;
                if measured.is_empty() {
                    continue;
                }
                seen[index] = true;
                target_size::assert_sizes_spaced(selector, &measured)?;
            } else {
                let sizes = target_size::measure_all(page, selector).await?;
                if sizes.is_empty() {
                    continue;
                }
                seen[index] = true;
                target_size::assert_sizes(selector, &sizes)?;
            }
        }
        Ok(())
    }

    async fn reach(&self, page: &chromiumoxide::Page, state: &State) -> anyhow::Result<()> {
        // A `settled` selector visible before the steps run waits on nothing:
        // the wait returns at once and the snapshot races the re-render. Tabs'
        // "second" state settled on `[role=tab]` that way (review 7, E8).
        if crate::wait::is_visible(page, state.settled).await? {
            anyhow::bail!(
                "the {:?} state settles on {}, which is already visible before its steps, \
                 so reaching it waits on nothing. Name what the steps change.",
                state.name,
                state.settled
            );
        }
        for step in state.steps {
            match *step {
                Step::TabTo(selector) => {
                    page.evaluate("document.activeElement && document.activeElement.blur()")
                        .await?;
                    keyboard::tab_to(page, selector, self.tab_budget).await?;
                }
                Step::Press(key) => keyboard::press(page, key).await?,
                Step::Click(selector) => pointer::click(page, selector).await?,
            }
        }
        crate::wait::for_visible(page, state.settled)
            .await
            .map_err(|e| e.context(format!("reaching the {:?} state", state.name)))?;
        self.wait_for_animations(page, state).await?;
        self.assert_settled_in_root(page, state).await
    }

    /// Visible is not finished.
    ///
    /// A notification is visible from the first frame of its fade-in, and axe
    /// measured its text blended into the background there: 3.72:1, against
    /// black on the settled card (todo 315). Every animated popup after it
    /// inherits that, so the wait is here and not in one unit.
    ///
    /// Infinite animations never finish, so a spinner, a marquee or a skeleton
    /// does not hold a state up. A finite one that never ends is a failure
    /// naming it, never a silent pass: a state measured mid-animation is
    /// measured against a colour, a position and a size that nobody ever sees.
    async fn wait_for_animations(
        &self,
        page: &chromiumoxide::Page,
        state: &State,
    ) -> anyhow::Result<()> {
        const RUNNING: &str = "[...document.getAnimations()] \
             .filter(a => a.playState === 'running' && a.effect \
               && a.effect.getComputedTiming().iterations !== Infinity) \
             .map(a => a.animationName || (a.effect.target && a.effect.target.tagName) || 'animation')";
        let deadline = std::time::Instant::now() + ANIMATION_BUDGET;
        loop {
            let running: Vec<String> = page.evaluate(RUNNING).await?.into_value()?;
            if running.is_empty() {
                return Ok(());
            }
            if std::time::Instant::now() >= deadline {
                anyhow::bail!(
                    "reaching the {:?} state: {:?} still running after {ANIMATION_BUDGET:?}. \
                     A state measured mid-animation is measured against a colour and a \
                     geometry nobody sees, so this is a failure rather than a measurement",
                    state.name,
                    running
                );
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
    }

    /// The state's `settled` element must lie inside the root, or axe and the
    /// snapshot never see the thing the state was declared for.
    ///
    /// That was the case for every overlay until `dc3766db`: the root wrapped
    /// only the fixture's outlet while the portal rendered beside it, and each
    /// "open" state checked the trigger alone and reported clean (review 7,
    /// E1). This makes the same hole fail loudly if a narrowed `root` or a
    /// moved marker reopens it.
    async fn assert_settled_in_root(
        &self,
        page: &chromiumoxide::Page,
        state: &State,
    ) -> anyhow::Result<()> {
        let (has_root, outside): (bool, usize) = page
            .evaluate(format!(
                "(() => {{ const root = document.querySelector({}); \
                 const hits = Array.from(document.querySelectorAll({})); \
                 return [!!root, hits.filter(el => !root || !root.contains(el)).length]; }})()",
                serde_json::to_string(self.root)?,
                serde_json::to_string(state.settled)?
            ))
            .await?
            .into_value()?;
        if !has_root {
            anyhow::bail!(
                "the suite root {} matched nothing in the {:?} state",
                self.root,
                state.name
            );
        }
        if outside > 0 {
            anyhow::bail!(
                "the {:?} state settled on {}, but {outside} match(es) lie outside the suite root {}, \
                 so axe and the snapshot never see them",
                state.name,
                state.settled,
                self.root
            );
        }
        Ok(())
    }

    /// Snapshots live beside the tests, not beside this file.
    ///
    /// `insta` writes next to the *calling* source file by default, which for a
    /// shared battery means every component's baseline lands in `e2e/src/` under
    /// a name like `e2e__suite__slider_desktop.snap`. Nobody reviewing the
    /// slider would look there.
    async fn take_snapshot(&self, fixture: &Fixture, state: &str) -> anyhow::Result<()> {
        let tree = ax::snapshot(&fixture.page, self.root).await?;
        let name = format!("{}_{}_{}", self.name, state, fixture.viewport.name());
        insta::with_settings!({
            snapshot_path => "../tests/all/snapshots",
            prepend_module_to_snapshot => false,
            description => format!("{} at {} ({state})", self.name, fixture.viewport.name()),
        }, {
            insta::assert_snapshot!(name, tree);
        });
        Ok(())
    }
}
