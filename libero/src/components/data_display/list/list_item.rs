use dioxus::core::current_scope_id;
use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, Part, States, base_props, parts_enum},
        layout::use_box,
    },
    sx::{StaticSx, Sx, sx},
    theme::{Size, SizeCss},
};

use super::list::{ListContext, NATIVE};

// Block, so a nested `List` stacks below this item's content. The icon floats instead of
// flexing: only a `list-item` counts in an ordered list's numbering (todo 2923).
static LIST_ITEM_BASE_SX: StaticSx = StaticSx::new(|| {
    // Blitz floats only among inline content, so beside the block body the icon flexes
    // (todo 2980). Physical float: native may not parse `inline-start`.
    let (item, icon, body) = match NATIVE {
        true => (
            sx().display("flex"),
            sx().flex_shrink("0"),
            sx().flex_grow("1").min_width("0"),
        ),
        false => (
            sx(),
            sx().with("float", "left").rtl(sx().with("float", "right")),
            sx(),
        ),
    };
    sx().padding("0")
        .when(
            "with-icon",
            item.list_style_type("none")
                .selector(
                    ListItemPart::Icon.selector(),
                    // One line box tall, so the icon centres on the first line.
                    icon.display("inline-flex")
                        .align_items("center")
                        .height("1lh")
                        .margin_inline_end(SizeCss::SPACING.value(Size::Sm)),
                )
                .selector(
                    ListItemPart::Body.selector(),
                    body.display("flow-root").overflow_wrap("anywhere"),
                ),
        )
        .when("with-action", action_sx())
});

/// The action takes the item's end, so the body is no float's neighbour.
fn action_sx() -> Sx {
    let item = sx().list_style_type("none");
    if NATIVE {
        return item
            .display("flex")
            .selector(
                ListItemPart::Body.selector(),
                sx().flex("1 1 auto")
                    .min_width("0")
                    .display("flow-root")
                    .overflow_wrap("anywhere"),
            )
            .selector(
                ListItemPart::Action.selector(),
                sx().flex_shrink("0")
                    .align_self("center")
                    .margin_inline_start(SizeCss::SPACING.value(Size::Sm)),
            );
    }
    // Two table cells, not a flex row: only a `list-item` counts in an ordered list's
    // numbering (todo 4172), and the full-width body pushes the action to the end.
    item.selector(
        ListItemPart::Body.selector(),
        sx().display("table-cell")
            .width("100%")
            .vertical_align("top")
            .overflow_wrap("anywhere"),
    )
    .selector(
        ListItemPart::Action.selector(),
        sx().display("table-cell")
            .vertical_align("middle")
            .white_space("nowrap")
            .padding_inline_start(SizeCss::SPACING.value(Size::Sm)),
    )
}

/// Native only: the item's number in an ordered list, counting icon and action items as
/// a browser does. Blitz numbers only a `list-item` with a marker (todo 4213).
fn use_number(context: Option<ListContext>) -> Option<i64> {
    if !NATIVE {
        return None;
    }
    let id = current_scope_id();
    use_hook(|| {
        if let Some(mut items) = context.map(|context| context.items) {
            items.write().push(id);
        }
    });
    use_effect(move || {
        if let Some(mut version) = context.map(|context| context.version) {
            *version.write() += 1;
        }
    });
    use_drop(move || {
        let Some(ListContext {
            mut items,
            mut version,
            ..
        }) = context
        else {
            return;
        };
        if let Ok(mut items) = items.try_write() {
            items.retain(|item| *item != id);
        }
        if let Ok(mut version) = version.try_write() {
            *version += 1;
        }
    });

    let context = context?;
    let numbering = (*context.numbering.read())?;
    context.version.read();
    let items = context.items.read();
    let index = items.iter().position(|item| *item == id)?;
    Some(numbering.of(index, items.len()))
}

parts_enum! {
    /// [`ListItem`]'s inner parts, for its `parts` prop. Present only with an
    /// icon or an action; each is a direct child, so a nested `List` keeps its own styles.
    pub enum ListItemPart {
        Icon = "icon" => "& > [data-slot='icon']",
        /// The content beside the icon and the action.
        Body = "body" => "& > [data-slot='body']",
        /// The trailing `action`, at the item's end.
        Action = "action" => "& > [data-slot='action']",
    }
}

base_props! {
    parts(ListItemPart);
    pub struct ListItemProps {
        /// Shown beside the first line. Overrides the list's `icon`.
        #[props(default)]
        icon: Option<Element>,
        /// A control at the item's end, such as a button. Keep it out of `children`.
        #[props(default)]
        action: Option<Element>,
        children: Element,
    }
}

/// One `<li>` of a [`List`](super::List). Keep the list's font size: the icon
/// gutter is in `em` of the list, so a `font_size` on one item misaligns its icon.
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
    let context = try_use_context::<ListContext>();
    let inherited = context.and_then(|context| context.icon.read().clone());
    let icon = props.icon.clone().or(inherited);
    // An icon or action item draws no number; a plain one writes out its own.
    let number = use_number(context)
        .filter(|_| icon.is_none() && props.action.is_none())
        .map(|number| format!("list-style-type: \"{number}. \""));

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("with-icon", icon.is_some())
        .with("with-action", props.action.is_some())
        .into();

    let children = match (icon, props.action) {
        (None, None) => props.children,
        (icon, action) => rsx! {
            if let Some(icon) = icon {
                span { "data-slot": ListItemPart::Icon.slot(), "aria-hidden": "true", {icon} }
            }
            div { "data-slot": ListItemPart::Body.slot(), {props.children} }
            if let Some(action) = action {
                span { "data-slot": ListItemPart::Action.slot(), {action} }
            }
        },
    };

    use_box()
        .framework_sx(&LIST_ITEM_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .style(number)
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
                ("action", "& > [data-slot='action']"),
            ]
        );
    }
}
