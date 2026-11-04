use dioxus::dioxus_core::AttributeValue;
use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, Part, SAFE_AREA_INLINE_END_VAR, SAFE_AREA_INLINE_START_VAR, ScaleOrCss,
            States, Variables, base_props, input_from_str, names_itself, parts_enum, responsive_sx,
            safe_inline_padding, variables, with_own,
        },
        layout::{ScrollArea, use_box},
    },
    hooks::{use_range_css, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ColorCss, ColorShade, Responsive, SIDEBAR_SIZE, Size, SizeCss},
};

pub use crate::theme::SidebarSide;

input_from_str!(SidebarSide);

static SIDEBAR_SCROLL_SX: StaticSx = StaticSx::new(|| sx().padding("lg"));
static SIDEBAR_SCROLL_START_SX: StaticSx = StaticSx::new(|| safe_side(true));
static SIDEBAR_SCROLL_END_SX: StaticSx = StaticSx::new(|| safe_side(false));

/// A start or end panel clears a landscape notch on its side (5469), unless a Drawer or a
/// same-side Sidebar around it does (5519, 5532). Zeroed on the children: the panel reads it itself.
fn safe_side(start: bool) -> Sx {
    let padding = SizeCss::SPACING.value(Size::Lg);
    let var = if start {
        SAFE_AREA_INLINE_START_VAR
    } else {
        SAFE_AREA_INLINE_END_VAR
    };
    sx().padding("lg")
        .and(safe_inline_padding(&padding, start, true))
        .selector("& > *", sx().var(var, "0"))
}

static SIDEBAR_BASE_SX: StaticSx = StaticSx::new(|| {
    let border = format!("1px solid {}", ColorCss::MUTED.value(ColorShade::S4));

    // Half the row or column at most, so a small screen leaves the content room (todos 2477, 2645).
    let base = sx()
        .flex_shrink("0")
        .min_height("0")
        .when(
            "side-start",
            sx().border_inline_end(border.clone()).max_width("50%"),
        )
        .when(
            "side-end",
            sx().border_inline_start(border.clone()).max_width("50%"),
        )
        .when(
            "side-top",
            sx().border_bottom(border.clone()).max_height("50%"),
        )
        .when("side-bottom", sx().border_top(border).max_height("50%"))
        // A nested sidebar must not inherit this one's custom size.
        .var(SIDEBAR_SIZE.override_var(), "initial");

    Size::ALL.into_iter().fold(base, |acc, size| {
        let state = size.state_name();
        let value = SIDEBAR_SIZE.overridable(size);

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

/// A size's block for a range copy, under its run word.
fn size_copy_sx(size: Size) -> Sx {
    let value = SIDEBAR_SIZE.overridable(size);
    sx().when("side-start || side-end", sx().width(value.clone()))
        .when("side-top || side-bottom", sx().height(value))
}

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
        /// The side it sits on, its border facing the content, and the axis `size` sizes; it does not place the panel.
        /// `start` and `end` clear that side's safe area, unless a drawer around it does; `top` and `bottom` leave the insets to the `Header` or `NavigationBar`.
        #[props(default, into)]
        side: Input<SidebarSide>,
        /// The width (or height, on `top` or `bottom`): a size word or any CSS, as `size: "18rem"`,
        /// or one per breakpoint, `size: responsive(Size::Sm).lg(Size::Md)`.
        #[props(default, into)]
        size: Input<Responsive<ThemeAwareValue>>,
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
    let fallback = theme.sidebar.size;
    let size = ScaleOrCss::new(props.size.as_ref().map(Responsive::base_ref), fallback);
    let ranged = props.size.as_ref().filter(|sizes| !sizes.is_plain());
    let steps = match ranged {
        Some(sizes) => sizes.map(|value| ScaleOrCss::new(Some(&value), fallback).size),
        None => size.size.into(),
    };
    // Custom CSS per breakpoint rides the override var in the user layer, as on `Button`.
    let sx = with_own(
        ranged.and_then(|sizes| {
            responsive_sx(sizes, |sx, value| {
                let css = ScaleOrCss::new(Some(&value), fallback)
                    .custom_css(SIDEBAR_SIZE)
                    .unwrap_or_else(|| "initial".into());
                sx.var(SIDEBAR_SIZE.override_var(), css)
            })
        }),
        &props.sx,
    );
    let variables: Input<Variables> = variables()
        .with(
            SIDEBAR_SIZE.override_var(),
            ranged.map_or_else(|| size.custom_css(SIDEBAR_SIZE), |_| None),
        )
        .into();
    use_range_css(
        &SIDEBAR_BASE_SX,
        steps.range_copies(|size| size.state_name(), size_copy_sx),
    );
    let component = props.component.copied_or(HtmlTag::Aside);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .active(side.state_name())
        .responsive(&steps, |size| size.state_name())
        .into();

    // A blank name names nothing: it is dropped, not copied to the tab stop.
    let mut attributes = props.attributes;
    attributes.retain(|attribute| {
        !matches!(attribute.name, "aria-label" | "aria-labelledby")
            || names_itself(std::slice::from_ref(attribute))
    });
    // An overflowing sidebar's scroll area is a tab stop: it carries the panel's name.
    let mut scroll_attributes: Vec<Attribute> = attributes
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
        .visibility(&props.hidden_from, &props.visible_from)
        .sx(&sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(
            component,
            attributes,
            rsx! {
                ScrollArea {
                    scrollbars: "both",
                    sx: match side {
                        SidebarSide::Start => &SIDEBAR_SCROLL_START_SX,
                        SidebarSide::End => &SIDEBAR_SCROLL_END_SX,
                        SidebarSide::Top | SidebarSide::Bottom => &SIDEBAR_SCROLL_SX,
                    },
                    attributes: scroll_attributes,
                    {props.children}
                }
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
