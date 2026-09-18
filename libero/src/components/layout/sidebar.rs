use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, ScrollArea, States,
        common::{base_props, input_from_str},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, sx},
    theme::{ColorCss, ColorShade, SIDEBAR_SIZE, Size},
};

pub use crate::theme::SidebarSide;

input_from_str!(SidebarSide);

// A sidebar is usually a flex item that must not be squeezed. This only sizes
// the panel; the scrolling is `ScrollArea`'s job.
static SIDEBAR_SCROLL_SX: StaticSx = StaticSx::new(|| sx().padding("lg"));

static SIDEBAR_BASE_SX: StaticSx = StaticSx::new(|| {
    let border = format!("1px solid {}", ColorCss::MUTED.value(ColorShade::S4));

    let base = sx()
        .flex_shrink("0")
        .min_height("0")
        .when("side-start", sx().border_inline_end(border.clone()))
        .when("side-end", sx().border_inline_start(border.clone()))
        .when("side-top", sx().border_bottom(border.clone()))
        .when("side-bottom", sx().border_top(border));

    Size::ALL.into_iter().fold(base, |acc, size| {
        let state = size.state_name();
        let value = SIDEBAR_SIZE.value(size);

        acc.when(
            format!("side-start && {state} || side-end && {state}"),
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
        /// Element to render as; `aside` (the `complementary` landmark) by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        children: Element,
    }
}

/// An in-flow panel bordering one edge of its parent, scrolling its own
/// content - a sidebar, nav rail or inspector. For the portaled, dimmed,
/// focus-trapped kind, see [`crate::hooks::use_drawer`].
#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let side = props.side.copied_or_default();
    let theme = use_theme();
    let size = props.size.copied_or(theme.sidebar.size);
    let component = props.component.copied_or(HtmlTag::Aside);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .active(side.state_name())
        .active(size.state_name())
        .into();

    // An overflowing sidebar's scroll area is a tab stop: it carries the panel's name.
    let name: Vec<Attribute> = props
        .attributes
        .iter()
        .filter(|attribute| matches!(attribute.name, "aria-label" | "aria-labelledby"))
        .cloned()
        .collect();

    use_box()
        .framework_sx(&SIDEBAR_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(
            component,
            props.attributes,
            rsx! {
                ScrollArea { sx: &SIDEBAR_SCROLL_SX, attributes: name, {props.children} }
            },
        )
}
