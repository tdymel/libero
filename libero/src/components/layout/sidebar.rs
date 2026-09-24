use dioxus::dioxus_core::AttributeValue;
use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, Part, States, base_props, input_from_str, parts_enum},
        layout::{ScrollArea, use_box},
    },
    hooks::use_theme,
    sx::{StaticSx, sx},
    theme::{ColorCss, ColorShade, SIDEBAR_SIZE, Size},
};

pub use crate::theme::SidebarSide;

input_from_str!(SidebarSide);

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

parts_enum! {
    /// [`Sidebar`]'s inner parts, for its `parts` prop.
    pub enum SidebarPart {
        /// The `ScrollArea` holding the content, with the panel's padding.
        Scroll = "scroll" => "& > [data-slot='scroll']",
    }
}

base_props! {
    parts(SidebarPart);
    pub struct SidebarProps {
        /// The edge it borders and the axis `size` sizes; it does not place the panel.
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

/// An in-flow panel bordering one edge of its parent, scrolling its own content.
/// For the modal kind, see [`crate::hooks::use_drawer`].
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Flex, Sidebar};
/// # fn app() -> Element {
/// rsx! {
///     Flex { direction: "row",
///         Sidebar { aria_label: "Filters", "Filters" }
///         main { "Results" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/sidebar>
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
    let mut scroll_attributes: Vec<Attribute> = props
        .attributes
        .iter()
        .filter(|attribute| matches!(attribute.name, "aria-label" | "aria-labelledby"))
        .cloned()
        .collect();
    scroll_attributes.push(Attribute::new(
        "data-slot",
        AttributeValue::Text(SidebarPart::Scroll.slot().to_string()),
        None,
        false,
    ));

    use_box()
        .framework_sx(&SIDEBAR_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .prepare()
        .render(
            component,
            props.attributes,
            rsx! {
                ScrollArea { sx: &SIDEBAR_SCROLL_SX, attributes: scroll_attributes, {props.children} }
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<SidebarPart>(),
            [("scroll", "& > [data-slot='scroll']")]
        );
    }
}
