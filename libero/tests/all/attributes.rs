//! `BoxStyle::attr` drops a `false`/`None` attribute instead of pushing it.
//!
//! That is only sound because a shrinking attribute list still *clears* what
//! went away. SSR cannot show this - it re-serialises the tree - so the check
//! has to read the mutations dioxus emits on the re-render.

use crate::dispatch::Page;
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Button, Header, Input},
    sx::ThemeAwareValue,
};

static DISABLED: GlobalSignal<bool> = Signal::global(|| true);

fn app() -> Element {
    rsx! { LiberoProvider { Button { disabled: DISABLED(), "x" } } }
}

#[test]
fn a_disabled_button_that_becomes_enabled_clears_the_attribute() {
    let mut page = Page::build(app);

    page.dom.in_runtime(|| *DISABLED.write() = false);
    page.render_fresh();
    let writes = page.rec;

    let disabled = writes
        .writes
        .iter()
        .find(|(name, _)| name == "disabled")
        .map(|(_, value)| value.as_str());

    assert_eq!(
        disabled,
        Some("None"),
        "the dropped attribute was never cleared: {:?}",
        writes.writes
    );
}

static COLOR: GlobalSignal<Input<ThemeAwareValue>> = Signal::global(|| Input::from("primary"));

fn header_app() -> Element {
    rsx! { LiberoProvider { Header { color: COLOR(), "x" } } }
}

/// The per-instance CSS variables ride the `style` attribute. Dropping the
/// attribute is not enough: the dioxus interpreter puts back every style
/// property the new value omits, so the header stayed blue in the browser.
/// Each var it ever set is written again as `revert-layer`, which computes to
/// what an absent var would.
#[test]
fn a_colored_header_that_becomes_unset_reverts_its_variables() {
    let mut page = Page::build(header_app);

    page.dom.in_runtime(|| *COLOR.write() = Input::None);
    page.render_fresh();
    let writes = page.rec;

    let style = writes
        .writes
        .iter()
        .find(|(name, _)| name == "style")
        .map(|(_, value)| value.as_str());

    assert_eq!(
        style,
        Some(
            "Text(\"--lsx-header-background:revert-layer;--lsx-header-color:revert-layer;--lsx-focus-ring-halo:revert-layer;--lsx-anchor-color:revert-layer;--lsx-surface-label:revert-layer;--lsx-focus-contrast:revert-layer;\")"
        ),
        "the header kept its old color variables: {:?}",
        writes.writes
    );
}
