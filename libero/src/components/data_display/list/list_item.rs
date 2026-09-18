use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    sx::{StaticSx, sx},
    theme::{Size, SizeCss},
};

use super::list::ListContext;

// Block, not flex, so a nested `List` stacks below this item's content rather
// than beside it. Row alignment is the content's own job.
static LIST_ITEM_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().padding("0").when(
        "with-icon",
        // Flex rows follow the writing direction, so the icon sits at the start.
        sx().display("flex")
            .align_items("flex-start")
            .gap(SizeCss::SPACING.value(Size::Sm))
            .selector(
                "& > [data-slot=\"icon\"]",
                // One line box tall, so the icon centres on the first line.
                sx().display("inline-flex")
                    .align_items("center")
                    .flex_shrink("0")
                    .height("1lh"),
            )
            .selector(
                "& > [data-slot=\"body\"]",
                sx().flex("1 1 auto").min_width("0"),
            ),
    )
});

base_props! {
    pub struct ListItemProps {
        /// Shown at the start of the item, beside its first line. Overrides the
        /// list's `icon`.
        #[props(default)]
        icon: Option<Element>,
        children: Element,
    }
}

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
            span { "data-slot": "icon", "aria-hidden": "true", {icon} }
            div { "data-slot": "body", {props.children} }
        },
        None => props.children,
    };

    use_box()
        .framework_sx(&LIST_ITEM_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(HtmlTag::Li, props.attributes, children)
}
