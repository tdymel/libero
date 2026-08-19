use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, ScrollArea, States,
        common::{base_props, input_from_str},
    },
    sx::{StaticSx, Sx, sx},
    theme::{ColorCss, ColorShade, SIDEBAR_SIZE, Size},
};

pub use crate::theme::SidebarSide;

input_from_str!(SidebarSide);

// A sidebar is usually a flex item that must not be squeezed. This only sizes
// the panel; the scrolling is `ScrollArea`'s job.
static SIDEBAR_BASE_SX: StaticSx = StaticSx::new(|| {
    let border = format!("1px solid {}", ColorCss::GREY.value(ColorShade::S4));

    let base = sx()
        .flex_shrink("0")
        .min_height("0")
        .when("side-left", sx().border_right(border.clone()))
        .when("side-right", sx().border_left(border.clone()))
        .when("side-top", sx().border_bottom(border.clone()))
        .when("side-bottom", sx().border_top(border));

    Size::ALL.into_iter().fold(base, |acc, size| {
        let state = size.state_name();
        let value = SIDEBAR_SIZE.value(size);

        acc.when(
            format!("side-left && {state} || side-right && {state}"),
            sx().width(value.clone()),
        )
        .when(
            format!("side-top && {state} || side-bottom && {state}"),
            sx().height(value),
        )
    })
});

base_props! {
    pub struct SidebarProps {
        /// Which edge this panel borders and which axis `size` applies to.
        /// It does not place the panel - an in-flow item is positioned by its
        /// parent's layout, so put it at the matching end of the DOM yourself.
        #[props(default, into)]
        side: Input<SidebarSide>,
        #[props(default, into)]
        size: Input<Size>,
        children: Element,
    }
}

/// An in-flow panel bordering one edge of its parent, scrolling its own
/// content - a sidebar, nav rail or inspector. For the portaled, dimmed,
/// focus-trapped kind, see `Drawer`.
#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let side = props.side.copied_or_default();
    let size = props.size.copied_or(Size::Md);

    let states = props
        .states
        .unwrap_or_default()
        .active(side.state_name())
        .active(size.state_name());

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            states,
            framework_sx: &SIDEBAR_BASE_SX,
            attributes: props.attributes,
            ScrollArea {
                sx: sx().padding("lg"),
                {props.children}
            }
        }
    }
}
