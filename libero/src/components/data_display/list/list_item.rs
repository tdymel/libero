use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, Part, States, base_props, parts_enum},
        layout::use_box,
    },
    sx::{StaticSx, sx},
    theme::{Size, SizeCss},
};

use super::list::ListContext;

// Block, so a nested `List` stacks below this item's content. The icon floats instead of
// flexing: only a `list-item` counts in an ordered list's numbering (todo 2923).
static LIST_ITEM_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().padding("0").when(
        "with-icon",
        sx().list_style_type("none")
            .selector(
                ListItemPart::Icon.selector(),
                // One line box tall, so the icon centres on the first line.
                // Physical, flipped under rtl: native's engine may not parse `inline-start`.
                sx().with("float", "left")
                    .rtl(sx().with("float", "right"))
                    .display("inline-flex")
                    .align_items("center")
                    .height("1lh")
                    .margin_inline_end(SizeCss::SPACING.value(Size::Sm)),
            )
            .selector(
                ListItemPart::Body.selector(),
                sx().display("flow-root").overflow_wrap("anywhere"),
            ),
    )
});

parts_enum! {
    /// [`ListItem`]'s inner parts, for its `parts` prop. Present only with an
    /// icon; each is a direct child, so a nested `List` keeps its own styles.
    pub enum ListItemPart {
        Icon = "icon" => "& > [data-slot='icon']",
        /// The content beside the icon.
        Body = "body" => "& > [data-slot='body']",
    }
}

base_props! {
    parts(ListItemPart);
    pub struct ListItemProps {
        /// Shown beside the first line. Overrides the list's `icon`.
        #[props(default)]
        icon: Option<Element>,
        children: Element,
    }
}

/// One `<li>` of a [`List`](super::List).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{List, ListItem};
/// # fn app() -> Element {
/// rsx! {
///     List {
///         ListItem { icon: rsx! { "✓" }, "Tested" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/list>
#[component]
pub fn ListItem(props: ListItemProps) -> Element {
    let inherited =
        try_use_context::<ListContext>().and_then(|context| context.icon.read().clone());
    let icon = props.icon.clone().or(inherited);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("with-icon", icon.is_some())
        .into();

    let children = match icon {
        Some(icon) => rsx! {
            span { "data-slot": ListItemPart::Icon.slot(), "aria-hidden": "true", {icon} }
            div { "data-slot": ListItemPart::Body.slot(), {props.children} }
        },
        None => props.children,
    };

    use_box()
        .framework_sx(&LIST_ITEM_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .prepare()
        .render(HtmlTag::Li, props.attributes, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<ListItemPart>(),
            [
                ("icon", "& > [data-slot='icon']"),
                ("body", "& > [data-slot='body']"),
            ]
        );
    }
}
