//! `Collapse`'s rendered contract: what is in the DOM when, which state
//! tokens carry the animation, and that the reduced-motion guard is emitted
//! at a specificity that can actually win.

use crate::common::{attributes_of, body, render};

use std::cell::Cell;
use std::thread;
use std::time::{Duration, Instant};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Button, Collapse},
};

fn open_app() -> Element {
    rsx! {
        LiberoProvider {
            Collapse { open: true, "panel body" }
        }
    }
}

fn closed_app() -> Element {
    rsx! {
        LiberoProvider {
            Collapse { open: false, "panel body" }
        }
    }
}

fn closed_unmounting_app() -> Element {
    rsx! {
        LiberoProvider {
            Collapse { open: false, keep_mounted: false, "panel body" }
        }
    }
}

fn open_unmounting_app() -> Element {
    rsx! {
        LiberoProvider {
            Collapse { open: true, keep_mounted: false, "panel body" }
        }
    }
}

fn custom_duration_app() -> Element {
    rsx! {
        LiberoProvider {
            Collapse { open: true, duration: 350, "panel body" }
        }
    }
}

/// The class both elements' rules are hung off - the grid root's, which is
/// the first `<div>`.
fn root_class(html: &str) -> String {
    attributes_of(html, "div")["class"]
        .split_whitespace()
        .next()
        .expect("the root carries a framework class")
        .to_string()
}

#[test]
fn an_open_collapse_renders_its_children_behind_an_open_state() {
    let html = render(open_app);

    assert!(body(&html).contains("panel body"), "{}", body(&html));
    assert_eq!(attributes_of(&html, "div")["data-state"], "open");
}

/// The default: closed content stays in the DOM, so a half-typed form
/// survives being collapsed and a trigger's `aria-controls` still resolves.
#[test]
fn a_closed_collapse_keeps_its_children_mounted_by_default() {
    let html = render(closed_app);

    assert!(body(&html).contains("panel body"), "{}", body(&html));
    assert_eq!(attributes_of(&html, "div")["data-state"], "closed");
}

/// `keep_mounted: false` starts closed *and empty* - there is no exit
/// transition to wait out on the very first render.
#[test]
fn keep_mounted_false_renders_no_children_while_closed() {
    let html = render(closed_unmounting_app);

    assert!(!body(&html).contains("panel body"), "{}", body(&html));
}

/// The SSR case `use_presence` was fixed for: an initially-open panel must be
/// sent open, not sent closed for a screen reader to find empty until wasm
/// boots.
#[test]
fn keep_mounted_false_still_renders_an_initially_open_panel() {
    let html = render(open_unmounting_app);

    assert!(body(&html).contains("panel body"), "{}", body(&html));
    assert_eq!(attributes_of(&html, "div")["data-state"], "open");
}

/// The root is always present, whatever `keep_mounted` says: C2 and C3 hang
/// `aria-controls` off it.
#[test]
fn the_root_is_in_the_dom_even_when_the_content_is_not() {
    let html = body(&render(closed_unmounting_app));

    assert_eq!(html.matches("<div").count(), 3, "{html}");
}

#[test]
fn the_duration_prop_sets_the_override_variable() {
    let html = render(custom_duration_app);

    assert_eq!(
        attributes_of(&html, "div")["style"],
        "--lsx-collapse-duration-override:350ms;"
    );
}

#[test]
fn no_duration_prop_leaves_the_theme_variable_alone() {
    let html = render(open_app);

    assert!(!attributes_of(&html, "div").contains_key("style"), "{html}");
}

/// The mechanism itself: `0fr` -> `1fr` on the root, and the content row able
/// to reach zero.
#[test]
fn the_height_animation_is_a_grid_row() {
    let html = render(open_app);
    let class = root_class(&html);

    assert!(
        html.contains(&format!(
            ".{class}[data-state~=\"closed\"]{{grid-template-rows:0fr;"
        )),
        "{html}"
    );
    assert!(
        html.contains(&format!(
            ".{class}[data-state~=\"open\"]{{grid-template-rows:1fr;"
        )),
        "{html}"
    );
    assert!(html.contains("min-height:0;overflow:hidden;"));
}

/// Closed content is not a tab stop, and the delay is what keeps it visible
/// and announced for the whole close rather than vanishing on the first
/// frame.
#[test]
fn closed_content_hides_from_the_accessibility_tree_as_the_animation_ends() {
    let html = render(open_app);

    assert!(html.contains("visibility:hidden;"), "{html}");
    assert!(
        html.contains("visibility 0s linear var(--lsx-collapse-duration-override,"),
        "{html}"
    );
    assert!(html.contains("visibility:visible;"), "{html}");
    assert!(html.contains("visibility 0s linear 0s;"), "{html}");
}

/// The specificity trap this library has already been bitten by: a `@media`
/// block adds nothing, so the guard has to carry the same
/// `[data-state~=".."]` the rule it overrides does. A guard written beside the
/// conditions would be `0-1-0` against their `0-2-0` and the motion would
/// still play.
#[test]
fn the_reduced_motion_guard_is_nested_inside_each_condition() {
    let html = render(open_app);
    let class = root_class(&html);

    assert!(
        html.contains(&format!(
            "@media (prefers-reduced-motion: reduce){{.{class}[data-state~=\"open\"]{{transition:none;}}}}"
        )),
        "{html}"
    );
    assert!(
        html.contains(&format!(
            "@media (prefers-reduced-motion: reduce){{.{class}[data-state~=\"closed\"]{{transition:none;}}}}"
        )),
        "{html}"
    );
    // Both elements, not just the root - the content's opacity and its
    // delayed `visibility` need the same guard.
    assert_eq!(
        html.matches("@media (prefers-reduced-motion: reduce)")
            .count(),
        4,
        "{html}"
    );
}

fn identified_app() -> Element {
    rsx! {
        LiberoProvider {
            Collapse { open: false, keep_mounted: false, id: "shipping-panel", "panel body" }
        }
    }
}

/// A consumer's `aria-controls` points at this `id`, so it has to survive the
/// mode where the children are gone - `Box` drops every `id` after the first,
/// and a wrapper between `Collapse` and `Box` would silently eat it.
#[test]
fn a_callers_id_reaches_the_root_even_with_the_content_unmounted() {
    let html = render(identified_app);

    assert_eq!(attributes_of(&html, "div")["id"], "shipping-panel");
    assert_eq!(body(&html).matches("shipping-panel").count(), 1, "{html}");
}

/// Not a themed value: a settled open panel at anything but `1` would be a
/// permanent contrast regression on every open panel in the app.
#[test]
fn an_open_panel_settles_at_full_opacity() {
    let html = render(open_app);

    assert!(html.contains("[data-state~=\"open\"]{opacity:1;"), "{html}");
    assert!(
        html.contains("[data-state~=\"closed\"]{opacity:var(--lsx-collapse-opacity-closed);"),
        "{html}"
    );
}

fn disclosure_app() -> Element {
    rsx! {
        LiberoProvider {
            Button {
                aria_expanded: false,
                aria_controls: "shipping-panel",
                "Shipping details"
            }
            Collapse { open: false, id: "shipping-panel", "panel body" }
        }
    }
}

/// `Collapse` renders no ARIA on purpose, so the docs example's trigger wiring
/// is the only place a consumer learns it - and all of it rests on dioxus
/// emitting `aria-expanded=false` rather than dropping it the way it drops a
/// falsey boolean attribute. Nothing else in the repo sets `aria_expanded`, so
/// nothing else would catch a dioxus bump taking that away.
#[test]
fn a_false_aria_expanded_is_still_rendered_on_the_trigger() {
    let html = body(&render(disclosure_app));

    assert!(html.contains("aria-expanded"), "{html}");
    assert!(html.contains("aria-expanded=false"), "{html}");
    // The pair only works if both halves survive: the control reference has to
    // resolve to the panel that is still in the DOM while closed.
    assert!(html.contains(r#"aria-controls="shipping-panel""#), "{html}");
    assert!(html.contains(r#"id="shipping-panel""#), "{html}");
}

thread_local! {
    /// How many times [`Counted`] was created: a remount discards the
    /// content's state, which is what a stale fallback would do.
    static MOUNTS: Cell<usize> = const { Cell::new(0) };
}

#[component]
fn Counted() -> Element {
    use_hook(|| MOUNTS.with(|mounts| mounts.set(mounts.get() + 1)));
    rsx! { "panel body" }
}

/// A `Collapse` whose `open` the test flips from outside, through the signal
/// the app puts in context. `keep_mounted: false`, so the content's presence
/// in the markup is the hook's `mounted()`.
fn toggled_app(duration: u32) -> Element {
    let open = use_context_provider(|| Signal::new(true));
    rsx! {
        LiberoProvider {
            Collapse { open: open(), keep_mounted: false, duration, Counted {} }
        }
    }
}

fn toggled_30ms() -> Element {
    toggled_app(30)
}

fn toggled_0ms() -> Element {
    toggled_app(0)
}

fn set_open(dom: &mut VirtualDom, value: bool) {
    let mut open = dom.in_scope(ScopeId::APP, consume_context::<Signal<bool>>);
    dom.in_runtime(|| open.set(value));
}

/// Polls `dom` until `done` holds for its markup or `limit` runs out.
/// `process_events` drains the task the timer delivers through.
fn drive_until(dom: &mut VirtualDom, limit: Duration, done: impl Fn(&str) -> bool) -> String {
    let start = Instant::now();
    loop {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = body(&dioxus_ssr::render(dom));
        if done(&html) || start.elapsed() > limit {
            return html;
        }
        thread::sleep(Duration::from_millis(2));
    }
}

/// No renderer here fires `transitionend`, which is exactly the position of a
/// suppressed exit: before the fallback timer the content stayed mounted for
/// good (todo 36b).
#[test]
fn a_close_with_no_transitionend_unmounts_after_the_duration() {
    let mut dom = VirtualDom::new(toggled_30ms);
    dom.rebuild_in_place();
    let html = drive_until(&mut dom, Duration::from_millis(50), |_| false);
    assert!(html.contains("panel body"), "{html}");

    set_open(&mut dom, false);
    let start = Instant::now();
    // Still there straight after the close: the exit is running.
    let html = drive_until(&mut dom, Duration::ZERO, |_| true);
    assert!(
        html.contains("panel body"),
        "unmounted before the exit: {html}"
    );

    let html = drive_until(&mut dom, Duration::from_secs(2), |html| {
        !html.contains("panel body")
    });
    assert!(!html.contains("panel body"), "never unmounted: {html}");
    // Not before the duration plus the slack.
    assert!(
        start.elapsed() >= Duration::from_millis(30 + 150),
        "{:?}",
        start.elapsed()
    );
}

#[test]
fn a_zero_duration_close_unmounts_without_waiting() {
    let mut dom = VirtualDom::new(toggled_0ms);
    dom.rebuild_in_place();
    set_open(&mut dom, false);

    let html = drive_until(&mut dom, Duration::from_millis(100), |html| {
        !html.contains("panel body")
    });
    assert!(!html.contains("panel body"), "{html}");
}

/// Reopening mid-exit drops the fallback. A stale one unmounts the open
/// panel, and the open arm then mounts it again at once - so the markup heals
/// and only the mount count shows the content's state was thrown away.
#[test]
fn reopening_before_the_fallback_keeps_the_content() {
    let mut dom = VirtualDom::new(toggled_30ms);
    dom.rebuild_in_place();
    set_open(&mut dom, false);
    // Long enough for the close effect to arm the 180ms fallback.
    drive_until(&mut dom, Duration::from_millis(60), |_| false);
    set_open(&mut dom, true);

    // Well past the 180ms the dropped timer was due at.
    let html = drive_until(&mut dom, Duration::from_millis(400), |_| false);
    assert!(html.contains("panel body"), "{html}");
    assert_eq!(MOUNTS.with(Cell::get), 1, "the content was remounted");
    assert_eq!(attributes_of(&html, "div")["data-state"], "open", "{html}");
}
