use std::rc::Rc;

use dioxus::prelude::*;

use super::overlay::Overlay;
use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{HtmlTag, Input, base_props},
        feedback::Loader,
        layout::use_box,
    },
    hooks::{use_element, use_localization},
    platform::{self, ElementApi, document},
    sx::{StaticSx, sx},
};

/// The overlay's positioned ancestor; `z-index` keeps its layer inside the box.
static LOADING_OVERLAY_SX: StaticSx = StaticSx::new(|| sx().position("relative").z_index("0"));

base_props! {
    pub struct LoadingOverlayProps {
        /// Covers the children with a dim and a loader, and makes them `inert`. Focus
        /// they held waits on the box and returns when loading ends, unless it moved on.
        #[props(default)]
        loading: bool,
        /// The content that loads. `class`/`sx`/`attributes` style the box around it.
        children: Element,
    }
}

/// A loading state over one box: dims its children, blocks them and says
/// "Loading" once. The box is `aria-busy` while `loading`; the status that says it
/// renders as the box's next sibling, outside it, since a reader may mute a busy box;
/// a parent's `nth-child` and sibling selectors count it.
/// Focus inside moves to the box while loading and back to its control after.
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
    let frame = use_element();
    let content = use_element();

    // `inert` would drop focus inside to the page: the box holds it while loading,
    // and the control gets it back after (todo 4545).
    let mut held = use_hook(|| CopyValue::new(None::<Rc<dyn ElementApi>>));
    let mut was_loading = use_hook(|| CopyValue::new(false));
    if loading != *was_loading.peek() {
        was_loading.set(loading);
        if loading {
            let focused = content
                .mounted()
                .and_then(|mounted| platform::element(&mounted).query_selector(":focus").ok());
            held.set(focused.map(Rc::from));
        }
    }
    use_effect(use_reactive!(|loading| {
        let Some(element) = held.peek().clone() else {
            return;
        };
        let frame = frame.mounted().map(|mounted| platform::element(&mounted));
        if loading {
            if let Some(frame) = frame {
                let _ = frame.focus();
            }
            return;
        }
        held.set(None);
        // Not when the user moved on meanwhile.
        let stayed = frame.is_some_and(|frame| frame.is_focused())
            || document()
                .and_then(|document| document.active_element())
                .is_none_or(|active| matches!(active.matches("body, html"), Ok(true)));
        if stayed && element.is_connected() {
            let _ = element.focus();
        }
    }));

    let busy = use_box()
        .framework_sx(&LOADING_OVERLAY_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .element(&frame)
        .attr("aria-busy", loading.then_some("true"))
        .attr("tabindex", loading.then_some("-1"))
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                // `contents`: the wrapper only carries `inert`, never the layout.
                div {
                    style: "display: contents",
                    inert: loading.then_some("true"),
                    onmounted: content.mount(),
                    {props.children}
                }
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
