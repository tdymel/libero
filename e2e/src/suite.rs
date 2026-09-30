//! The generic pass battery every component gets from a fixture route and a few declarative lines.
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
//! Behaviour contracts live in `archetypes` or the component's own test, not here.

use crate::passes::keyboard::{self, Key};
use crate::passes::{contrast, focus, motion, pointer, target_size};
use crate::{Fixture, Scheme, Viewport, ax, browser};

/// How long a state's entry animation may take; well past the theme's longest (200 ms).
const ANIMATION_BUDGET: std::time::Duration = std::time::Duration::from_secs(5);

/// One move towards a state worth snapshotting.
/// An enum, not an async closure: add a variant rather than arbitrary code.
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
    /// Whether an undersized target may conform through the spacing exception (opt-in).
    spacing_exception: bool,
}

struct State {
    name: &'static str,
    steps: &'static [Step],
    /// Visible once the state is reached, and not before the steps run,
    /// or the wait proves nothing.
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
    /// `Some(why)` when this unit opted out of the default coverage run over `root`.
    coverage_waived: Option<&'static str>,
    snapshot: bool,
    /// `Some(why)` when the dark run keeps baselines of its own.
    dark_snapshot: Option<&'static str>,
    states: Vec<State>,
    viewports: Vec<Viewport>,
    tab_budget: usize,
    reduced_motion: bool,
    /// Visible once the page is at rest; the battery waits for it first.
    ready: Option<&'static str>,
}

impl Suite {
    /// A battery named `name` (the snapshot prefix) over the fixture at `route`.
    pub fn new(name: &'static str, route: &'static str) -> Self {
        Suite {
            name,
            route,
            // Wraps the provider, so portaled popovers are inside it too.
            root: "[data-fixture-ready]",
            focusable: Vec::new(),
            targets: Vec::new(),
            waivers: &[],
            covers: Vec::new(),
            coverage_waived: None,
            snapshot: true,
            dark_snapshot: None,
            states: Vec::new(),
            viewports: Viewport::ALL.to_vec(),
            tab_budget: 10,
            reduced_motion: false,
            ready: None,
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

    /// A control that may conform through WCAG 2.5.8's spacing exception instead of 24x24. Repeatable.
    /// Name the press target, not a glyph drawn inside it.
    pub fn targets_spaced(mut self, selector: &'static str) -> Self {
        self.targets.push(Target {
            selector,
            spacing_exception: true,
        });
        self
    }

    /// Known axe violations, each naming a todo; still printed. A `target-size`
    /// waiver holds a target whose selector contains its `contains`.
    pub fn waive(mut self, waivers: &'static [contrast::Waiver]) -> Self {
        self.waivers = waivers;
        self
    }

    /// Text axe must have looked at, not merely found clean (todo 327). Repeatable.
    /// Measured in every state; a selector matching in none fails.
    pub fn contrast_covers(mut self, selector: &'static str) -> Self {
        self.covers.push(selector);
        self
    }

    /// Opt out of the default coverage run over `root` (on by default, todo 378), saying why.
    /// `assert_clean` and explicit `contrast_covers` still run.
    pub fn no_contrast_coverage(mut self, why: &'static str) -> Self {
        assert!(
            !why.trim().is_empty(),
            "{}: an opt-out from the default contrast coverage must say why",
            self.name
        );
        self.coverage_waived = Some(why);
        self
    }

    /// Every selector the coverage pass runs over: `root` unless this unit has
    /// waived it, plus whatever it declared explicitly.
    fn coverage_selectors(&self) -> Vec<&'static str> {
        let default = self.coverage_waived.is_none().then_some(self.root);
        default
            .into_iter()
            .chain(self.covers.iter().copied())
            .collect()
    }

    /// Snapshot the component in another state, and run axe there too.
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

    /// Emulate `prefers-reduced-motion: reduce`, so scrolls are instant: a smooth scroll does not
    /// land in a background page (`codebase/e2e-harness`). For state derived from a scroll offset.
    pub fn reduced_motion(mut self) -> Self {
        self.reduced_motion = true;
        self
    }

    /// Separate dark baselines (`<name>_<state>_<viewport>_dark`) for a tree that names the scheme.
    /// By default the dark tree must equal the light one.
    pub fn dark_snapshot(mut self, why: &'static str) -> Self {
        assert!(
            !why.trim().is_empty(),
            "{}: a separate dark baseline must say why the tree differs",
            self.name
        );
        self.dark_snapshot = Some(why);
        self
    }

    /// Wait for `selector` to be visible before the battery, for a rest state
    /// that lands after the first render (a broken picture's fallback).
    pub fn ready(mut self, selector: &'static str) -> Self {
        self.ready = Some(selector);
        self
    }

    /// How many Tab presses may reach a control (default 10).
    pub fn tab_budget(mut self, budget: usize) -> Self {
        self.tab_budget = budget;
        self
    }

    /// Run the battery at every viewport, light then dark (todo 314); a failure names a screenshot.
    /// The dark run skips target sizes and compares against the light AX baseline.
    pub fn run(self) {
        browser::block_on(async move {
            for scheme in [Scheme::Light, Scheme::Dark] {
                for viewport in self.viewports.iter().copied() {
                    if let Err(error) = self.run_at(viewport, scheme).await {
                        panic!(
                            "{} at {} ({}): {error:?}",
                            self.name,
                            viewport.name(),
                            scheme.name()
                        );
                    }
                }
            }
        });
    }

    async fn run_at(&self, viewport: Viewport, scheme: Scheme) -> anyhow::Result<()> {
        let fixture = Fixture::open_in(self.route, viewport, scheme).await?;
        if self.reduced_motion {
            browser::emulate_media(&fixture.page, scheme, Some(true)).await?;
            motion::assert_reduced_motion_matches(&fixture.page).await?;
        }

        let outcome = self.battery(&fixture).await;

        if let Err(error) = outcome {
            let shot = fixture
                .screenshot(&format!("{}-{}", self.name, scheme.name()))
                .await;
            let where_to_look = match shot {
                Ok(path) => format!("\n  screenshot: {}", path.display()),
                Err(e) => format!("\n  (no screenshot: {e})"),
            };
            // Closed on failure too: pages on the shared browser outlive the test.
            let _ = fixture.close().await;
            return Err(error.context(format!("at {}{where_to_look}", viewport.name())));
        }

        fixture.close().await?;
        Ok(())
    }

    async fn battery(&self, fixture: &Fixture) -> anyhow::Result<()> {
        let page = &fixture.page;
        if let Some(ready) = self.ready {
            crate::wait::for_visible(page, ready)
                .await
                .map_err(|e| e.context("waiting for the page to come to rest"))?;
        }

        // Contrast and the ARIA-validity rules, at rest.
        let covers = self.coverage_selectors();
        let mut covered: Vec<usize> = vec![0; covers.len()];
        self.assert_clean_and_covered(page, &covers, &mut covered)
            .await?;

        // A ring on every control that can be tabbed to, and enough contrast on
        // it to be seen.
        for selector in &self.focusable {
            let ring = focus::assert_focus_ring(page, selector, self.tab_budget).await?;
            focus::assert_ring_contrast(&ring)?;
        }

        // Targets are measured wherever they exist: an open-state control matches nothing at rest.
        let light = fixture.scheme == Scheme::Light;
        let mut seen: Vec<bool> = vec![false; self.targets.len()];
        if light {
            self.measure_targets(page, &mut seen).await?;
        }

        if self.snapshot {
            self.take_snapshot(fixture, "rest").await?;
        }

        // Then every declared state, snapshotted and axe-checked in place.
        for state in &self.states {
            self.reach(page, state).await?;
            self.assert_clean_and_covered(page, &covers, &mut covered)
                .await?;
            if light {
                self.measure_targets(page, &mut seen).await?;
            }
            if self.snapshot {
                self.take_snapshot(fixture, state.name).await?;
            }
        }

        // A coverage selector that found text in no state checked nothing.
        for (selector, found) in covers.iter().zip(&covered) {
            if *found == 0 {
                anyhow::bail!(
                    "contrast coverage: {selector} held no on-screen text at rest or in any \
                     declared state, so requiring axe to have checked it asserts nothing"
                );
            }
        }

        // A target matching in no state is a stale test, e.g. after a renamed role.
        for (target, matched) in self.targets.iter().zip(&seen) {
            if light && !matched {
                anyhow::bail!(
                    "target size: {} matched nothing at rest or in any declared state",
                    target.selector
                );
            }
        }

        // Last, so it covers everything the battery just did - including the
        // first mount, since the recorder attached before navigation.
        fixture.console.assert_clean(&format!(
            "the {} battery ({})",
            self.name,
            fixture.scheme.name()
        ))?;
        Ok(())
    }

    /// Axe at rest or in a state, one run for the violations and the coverage. `covered[i]`
    /// accumulates the most text `covers[i]` ever held, so a selector matching only in an open state counts.
    async fn assert_clean_and_covered(
        &self,
        page: &chromiumoxide::Page,
        covers: &[&'static str],
        covered: &mut [usize],
    ) -> anyhow::Result<()> {
        let wanted =
            contrast::assert_clean_and_covered(page, self.root, self.waivers, covers).await?;
        for (most, wanted) in covered.iter_mut().zip(wanted) {
            *most = (*most).max(wanted);
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
            let verdict = if target.spacing_exception {
                let measured = target_size::measure_spacing(page, selector).await?;
                if measured.is_empty() {
                    continue;
                }
                seen[index] = true;
                target_size::assert_sizes_spaced(selector, &measured)
            } else {
                let sizes = target_size::measure_all(page, selector).await?;
                if sizes.is_empty() {
                    continue;
                }
                seen[index] = true;
                target_size::assert_sizes(selector, &sizes)
            };
            let Err(error) = verdict else { continue };
            let waiver = self
                .waivers
                .iter()
                .find(|w| w.rule == "target-size" && selector.contains(w.contains));
            match waiver {
                Some(w) => eprintln!("waived: target-size - {}\n      {error:#}", w.why),
                None => return Err(error),
            }
        }
        Ok(())
    }

    async fn reach(&self, page: &chromiumoxide::Page, state: &State) -> anyhow::Result<()> {
        // A `settled` already visible waits on nothing and races the re-render (review 7, E8).
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

    /// Waits for finite animations to end: axe measured a fading-in notification at 3.72:1 (todo 315).
    /// Infinite ones are ignored, and scroll-driven ones, which run as long as their
    /// scroller exists (1010's drawn bars); a finite one past the budget fails.
    async fn wait_for_animations(
        &self,
        page: &chromiumoxide::Page,
        state: &State,
    ) -> anyhow::Result<()> {
        const RUNNING: &str = "[...document.getAnimations()] \
             .filter(a => a.playState === 'running' && a.effect \
               && a.effect.getComputedTiming().iterations !== Infinity \
               && !(typeof ScrollTimeline !== 'undefined' && a.timeline instanceof ScrollTimeline)) \
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
    /// snapshot never see it (review 7, E1: portals outside the root).
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
            // Seen blank in full runs (todo 597): says whether the page reloaded
            // under the test rather than the component removing the app.
            let page_state: String = page
                .evaluate(
                    "(() => { const n = performance.getEntriesByType('navigation')[0]; \
                     return `${location.href}, ${document.readyState}, navigation ${n && n.type}, \
                     ${Math.round(performance.now())}ms since it began`; })()",
                )
                .await
                .map_err(anyhow::Error::from)
                .and_then(|value| Ok(value.into_value::<String>()?))
                .unwrap_or_else(|error| format!("unreadable: {error}"));
            anyhow::bail!(
                "the suite root {} matched nothing in the {:?} state ({page_state})",
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

    /// Snapshots live beside the tests: `insta` would write next to this file by default.
    async fn take_snapshot(&self, fixture: &Fixture, state: &str) -> anyhow::Result<()> {
        let tree = ax::snapshot(&fixture.page, self.root).await?;
        let mut name = format!("{}_{}_{}", self.name, state, fixture.viewport.name());
        let mut described = state.to_string();
        // Otherwise the dark tree is held to the light baseline.
        if fixture.scheme == Scheme::Dark && self.dark_snapshot.is_some() {
            name.push_str("_dark");
            described.push_str(", dark");
        }
        insta::with_settings!({
            snapshot_path => "../tests/all/snapshots",
            prepend_module_to_snapshot => false,
            description => format!("{} at {} ({described})", self.name, fixture.viewport.name()),
        }, {
            insta::assert_snapshot!(name, tree);
        });
        Ok(())
    }
}
