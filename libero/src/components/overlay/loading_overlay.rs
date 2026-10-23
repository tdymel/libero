use dioxus::prelude::*;

use super::overlay::Overlay;
use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{HtmlTag, Input, base_props},
        feedback::Loader,
        layout::use_box,
    },
    hooks::use_localization,
    sx::{StaticSx, sx},
};

/// The overlay's positioned ancestor; `z-index` keeps its layer inside the box.
static LOADING_OVERLAY_SX: StaticSx = StaticSx::new(|| sx().position("relative").z_index("0"));

base_props! {
    pub struct LoadingOverlayProps {
        /// Covers the children with a dim and a loader, and makes them `inert`.
        #[props(default)]
        loading: bool,
        /// The content that loads. `class`/`sx`/`attributes` style the box around it.
        children: Element,
    }
}

/// A loading state over one box: dims its children, blocks them and says
/// "Loading" once. The box is `aria-busy` while `loading`.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{LoadingOverlay, Paper, Text};
/// # fn app() -> Element {
/// # let saving = use_signal(|| true);
/// # rsx! {
/// LoadingOverlay { loading: saving(),
///     Paper { Text { "Ada Lovelace" } }
/// }
/// # } }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/overlay>
#[component]
pub fn LoadingOverlay(props: LoadingOverlayProps) -> Element {
    let localization = use_localization();
    let loading = props.loading;

    let busy = use_box()
        .framework_sx(&LOADING_OVERLAY_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .attr("aria-busy", loading.then_some("true"))
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                // `contents`: the wrapper only carries `inert`, never the layout.
                div { style: "display: contents", inert: loading.then_some("true"), {props.children} }
                if loading {
                    Overlay { contained: true, "data-slot": "overlay", Loader {} }
                }
            },
        );
    rsx! {
        {busy}
        // Outside the busy box, which a reader may mute, and always mounted: a live
        // region added with its text is not announced.
        VisuallyHidden { role: "status",
            if loading { {localization.common.loading} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(loading: bool) -> String {
        let mut dom = VirtualDom::new_with_props(
            |loading: bool| {
                rsx! {
                    crate::LiberoProvider {
                        LoadingOverlay { loading, button { "Edit" } }
                    }
                }
            },
            loading,
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    /// Todo 3985: busy, inert and covered while loading, and said once.
    #[test]
    fn a_loading_box_is_busy_and_inert() {
        let html = render(true);
        assert!(html.contains("aria-busy=\"true\""), "{html}");
        assert!(html.contains("inert"), "{html}");
        assert!(html.contains("data-slot=\"overlay\""), "{html}");
        assert!(html.contains("role=\"status\""), "{html}");
        assert!(html.contains("Loading"), "{html}");
    }

    /// The status region stays mounted, empty, so the next load is announced.
    #[test]
    fn a_loaded_box_is_plain() {
        let html = render(false);
        assert!(!html.contains("aria-busy"), "{html}");
        assert!(!html.contains("inert"), "{html}");
        assert!(!html.contains("data-slot=\"overlay\""), "{html}");
        assert!(html.contains("role=\"status\""), "{html}");
    }
}
