use std::cell::Cell;

use dioxus::dioxus_core::{NoOpMutations, VirtualDom};
use dioxus::prelude::*;

use super::registration::{CssSource, SxSource, use_box_css};
use crate::{
    LiberoContext, LiberoProvider,
    css::Stylesheet,
    sx::{Input, StaticSx, sx},
};

static PADDING: StaticSx = StaticSx::new(|| sx().padding("lg"));
static SAME_PADDING: StaticSx = StaticSx::new(|| sx().padding("lg"));
static COLOR: StaticSx = StaticSx::new(|| sx().color("red"));

#[test]
fn a_memoized_build_matches_an_unmemoized_one() {
    let memoized: &'static StaticSx = &PADDING;

    assert_eq!(memoized.build(), Stylesheet::from(&PADDING));
}

#[test]
fn a_second_build_of_the_same_static_is_the_same_sheet() {
    let first: &'static StaticSx = &COLOR;
    let second: &'static StaticSx = &COLOR;

    assert_eq!(first.build(), second.build());
}

/// Two statics, but the CSS is keyed by content: one class downstream.
#[test]
fn two_statics_with_equal_content_build_one_class() {
    let padding: &'static StaticSx = &PADDING;
    let same: &'static StaticSx = &SAME_PADDING;

    assert_eq!(padding.build().class_name(), same.build().class_name());
}

#[test]
fn different_statics_build_different_sheets() {
    let padding: &'static StaticSx = &PADDING;
    let color: &'static StaticSx = &COLOR;

    assert_ne!(padding.build().as_str(), color.build().as_str());
}

/// The variant decides only how the identity is derived, never what is registered.
#[test]
fn a_static_and_an_owned_sx_source_build_the_same_sheet() {
    let owned = sx().padding("lg");

    assert_eq!(
        SxSource::Static(&PADDING).build(),
        SxSource::Owned(&owned).build()
    );
}

thread_local! {
    /// The registry version and sheet count the latest render of `Probe` saw.
    static SEEN: Cell<(u64, usize)> = const { Cell::new((0, 0)) };
    /// Whether `Probe` passes an owned `sx` instead of `PADDING`.
    static OWNED: Cell<bool> = const { Cell::new(false) };
    static TICK: Cell<Option<Signal<u32>>> = const { Cell::new(None) };
}

#[component]
fn Probe(tick: u32) -> Element {
    let _ = tick;
    let owned = sx().margin("sm");
    let source = match OWNED.get() {
        true => SxSource::Owned(&owned),
        false => SxSource::Static(&PADDING),
    };
    let class = use_box_css(&Input::None, Some(&COLOR), None, Some(source));
    let context = use_context::<LiberoContext>();
    let version = *context.stylesheet_registry_version.peek();
    SEEN.set((version, context.stylesheet_registry.stylesheets().len()));
    rsx! { div { class } }
}

fn app() -> Element {
    let tick = use_signal(|| 0u32);
    TICK.set(Some(tick));
    rsx! {
        LiberoProvider { Probe { tick: tick() } }
    }
}

fn first_render() -> (VirtualDom, (u64, usize)) {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let seen = SEEN.get();
    (dom, seen)
}

/// Renders `Probe` again with nothing of its own changed.
fn rerender(dom: &mut VirtualDom) -> (u64, usize) {
    let mut tick = TICK.get().expect("app rendered");
    dom.in_runtime(|| tick += 1);
    dom.process_events();
    dom.render_immediate(&mut NoOpMutations);
    SEEN.get()
}

/// What a caller's `sx: &STATIC` buys: a render that changed nothing
/// registers nothing, so the outlet does not re-render.
#[test]
fn an_unchanged_render_registers_nothing() {
    OWNED.set(false);
    let (mut dom, first) = first_render();
    assert_eq!(first.1, 2, "COLOR and PADDING");
    assert_eq!(rerender(&mut dom), first);
}

/// A changed `sx` swaps its sheet: a new version, and the old sheet released.
#[test]
fn a_changed_sx_swaps_its_sheet() {
    OWNED.set(false);
    let (mut dom, (version, sheets)) = first_render();
    OWNED.set(true);
    let (changed, now) = rerender(&mut dom);
    assert!(changed > version);
    assert_eq!(now, sheets);
}
