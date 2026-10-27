use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        common::{Glyph, HtmlTag, Input, LogicalTextAlign, States, base_props},
        data_display::{Icon, SvgData},
        form::{LightState, use_fieldset_disabled},
        layout::{Box, use_box},
    },
    context::IconSlot,
    sx::{REDUCED_MOTION, StaticSx, Sx, sx},
    theme::{ICON_SIZE, Size, TREE_CHEVRON_DURATION},
};

use super::{
    tree::{Check, TreeMarks},
    tree_row::{TreeRowContext, TreeRowExpand, TreeRowToggle},
};

/// The chevron's turn and the label's stretch, for both standard rows.
fn item_parts_sx(base: Sx) -> Sx {
    base.selector(
        "& [data-tree-chevron]",
        sx().flex_shrink("0")
            .transition(format!("transform {} ease", TREE_CHEVRON_DURATION.value()))
            .transform("rotate(0deg)")
            .media(REDUCED_MOTION, sx().transition("none")),
    )
    // A closed row's chevron points to the start of the line. Before the
    // expanded rule, which wins at equal specificity.
    .rtl(sx().selector("& [data-tree-chevron]", sx().transform("rotate(180deg)")))
    .selector(
        "& [data-tree-chevron][data-state~=\"expanded\"]",
        sx().transform("rotate(90deg)"),
    )
    // Takes the free space, so the trailing parts sit at the inline end.
    .selector(
        "& > [data-tree-label]",
        sx().flex("1 1 auto").min_width("0"),
    )
    .selector("& > [data-tree-icon]", sx().flex_shrink("0"))
}

// The page's type and the row's vertical padding are this label's (or button's) job.
static TREE_ITEM_SX: StaticSx = StaticSx::new(|| {
    item_parts_sx(
        sx().display("flex")
            .align_items("center")
            // Blitz's UA sheet centres a button's content.
            .justify_content("flex-start")
            .gap("6px")
            .width("100%")
            .padding("6px 0")
            .border("none")
            .background("none")
            .color("inherit")
            .font("inherit")
            // Not in the `font` shorthand, and a `<button>`'s UA sheet resets it (todo 2376).
            .letter_spacing("inherit")
            .text_align_start()
            .cursor("pointer")
            // A row's label takes the row's cursor, `not-allowed` while disabled.
            .selector("&[data-tree-activate]", sx().cursor("inherit"))
            // The button form, soft-disabled by a `Fieldset` (todos 514, 3873).
            .selector(
                "&[aria-disabled=\"true\"]",
                sx().cursor("not-allowed").opacity("0.5"),
            ),
    )
});

// Row and chevron in one static, so a visible row builds no `Sx` of its own.
static TREE_ITEM_CONTENT_SX: StaticSx = StaticSx::new(|| {
    item_parts_sx(
        sx().display("flex")
            // Fills the row, so the trailing parts reach its end.
            .flex("1 1 auto")
            .min_width("0")
            .align_items("center")
            .gap("6px")
            .padding("6px 0"),
    )
});

// A 24px target around the 16px glyph; the margins keep the column chevron-wide.
static CHEVRON_TARGET_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        .width("24px")
        .height("24px")
        .margin_inline_start("-4px")
        .margin_inline_end("-4px")
        .cursor("pointer")
});

// Chevron-width, so leaves line up with their branch siblings.
static LEADING_SPACER_SX: StaticSx =
    StaticSx::new(|| sx().flex_shrink("0").width(ICON_SIZE.value(Size::Xs)));

/// The row's expansion: `None` for a leaf and outside a `Tree`.
fn row_expanded(row: Option<TreeRowContext>) -> Option<Option<bool>> {
    row.map(|row| row.0.read().expanded)
}

/// The row's checkbox, with `Tree`'s `checked` only. `aria-hidden`: the treeitem carries `aria-checked`.
fn use_check_mark(row: Option<TreeRowContext>) -> Option<Element> {
    let marks = use_hook(try_consume_context::<TreeMarks>);
    let toggle = use_hook(try_consume_context::<TreeRowToggle>);
    let state = *row?.0.read();
    let checked = state.checked?;
    let look = marks?.look;
    let look = look.read();
    let TreeRowToggle(toggle) = toggle?;
    let state = LightState {
        checked: checked == Check::On,
        indeterminate: checked == Check::Mixed,
        disabled: state.disabled,
    };
    // The box alone: the row's own click would open a branch too.
    Some(look.as_ref()?.render_mark(state, move |event: MouseEvent| {
        event.stop_propagation();
        toggle.call(());
    }))
}

/// Chevron or spacer, check box, icon, label, trailing icon and trailing element, as siblings.
fn item_parts(
    expanded: Option<Option<bool>>,
    expand: Option<TreeRowExpand>,
    mark: Option<Element>,
    icon: Option<SvgData>,
    trailing_icon: Option<SvgData>,
    trailing: Option<Element>,
    children: Element,
) -> Element {
    let leading = match expanded {
        Some(Some(expanded)) => rsx! {
            Box {
                framework_sx: &CHEVRON_TARGET_SX,
                // The glyph alone opens the branch; the row's pick and click stay out of it.
                onclick: move |event: MouseEvent| {
                    event.stop_propagation();
                    if let Some(TreeRowExpand(expand)) = expand {
                        expand.call(());
                    }
                },
                Icon {
                    variant: "standard",
                    size: "xs",
                    color: "muted.6",
                    "data-tree-chevron": true,
                    states: States::new().with("expanded", expanded),
                    Glyph { slot: IconSlot::ChevronRight, icon: lucide::chevron_right::outlined }
                }
            }
        },
        Some(None) => rsx! {
            Box { framework_sx: &LEADING_SPACER_SX }
        },
        // Outside a `Tree`, no column to line up with.
        None => rsx! {},
    };

    rsx! {
        {leading}
        {mark}
        if let Some(icon) = icon {
            Icon { variant: "standard", size: "sm", color: "muted.6", "data-tree-icon": true, svg: icon }
        }
        span { "data-tree-label": true, {children} }
        if let Some(icon) = trailing_icon {
            Icon { variant: "standard", size: "sm", color: "muted.6", "data-tree-icon": true, svg: icon }
        }
        {trailing}
    }
}

base_props! {
    pub struct TreeItemContentProps {
        /// Drawn after the chevron, before the label.
        #[props(default, into)]
        icon: Option<SvgData>,
        /// Drawn at the row's inline end.
        #[props(default, into)]
        trailing_icon: Option<SvgData>,
        /// After `trailing_icon`, such as a count. Inside a `TreeItem` it is part of
        /// the activated label: never interactive.
        #[props(default)]
        trailing: Option<Element>,
        /// The label.
        children: Element,
    }
}

/// The standard row content: the row's chevron, an icon, the label and a trailing
/// icon. `default_tree_render` draws one; any `render_node` can too.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Tree, TreeItemContent, TreeNode, TreeNodeRenderArgs};
/// # use pictogram_icons_lucide as lucide;
/// # fn app() -> Element {
/// rsx! {
///     Tree {
///         aria_label: "Files",
///         data: vec![TreeNode::new("src", "src").children(vec![TreeNode::new("lib", "lib.rs")])],
///         render_node: |args: TreeNodeRenderArgs<&'static str>| rsx! {
///             TreeItemContent {
///                 icon: match args.expanded {
///                     Some(true) => lucide::folder_open::outlined,
///                     Some(false) => lucide::folder::outlined,
///                     None => lucide::file::outlined,
///                 },
///                 "{args.data}"
///             }
///         },
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/tree>
#[component]
pub fn TreeItemContent(props: TreeItemContentProps) -> Element {
    let row = use_hook(try_consume_context::<TreeRowContext>);
    let mark = use_check_mark(row);
    let expand = use_hook(try_consume_context::<TreeRowExpand>);
    let parts = item_parts(
        row_expanded(row),
        expand,
        mark,
        props.icon,
        props.trailing_icon,
        props.trailing,
        props.children,
    );

    use_box()
        .framework_sx(&TREE_ITEM_CONTENT_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Div, props.attributes, parts)
}

base_props! {
    pub struct TreeItemProps {
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Drawn after the chevron, before the label.
        #[props(default, into)]
        icon: Option<SvgData>,
        /// Drawn at the row's inline end.
        #[props(default, into)]
        trailing_icon: Option<SvgData>,
        /// After `trailing_icon`, such as a count. Never interactive: it is part of the activated label.
        #[props(default)]
        trailing: Option<Element>,
        /// The label.
        children: Element,
    }
}

/// An activatable row for a custom `render_node`, laid out like [`TreeItemContent`]:
/// inside a `Tree` a plain label the treeitem names and Enter clicks, outside one
/// an ordinary button. A link row is a `NavLink` with `TreeNodeRenderArgs::tabindex`.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Tree, TreeItem, TreeNode, TreeNodeRenderArgs};
/// # fn app() -> Element {
/// rsx! {
///     Tree {
///         aria_label: "Actions",
///         data: vec![TreeNode::new("open", "Open")],
///         render_node: |args: TreeNodeRenderArgs<&'static str>| rsx! {
///             TreeItem {
///                 onclick: |_| {},
///                 icon: pictogram_icons_lucide::folder_open::outlined,
///                 "{args.data}"
///             }
///         },
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/tree>
#[component]
pub fn TreeItem(props: TreeItemProps) -> Element {
    // Outside a `Tree`, an ordinary button.
    let row = use_hook(try_consume_context::<TreeRowContext>);
    let group_disabled = use_fieldset_disabled();
    let (tabindex, disabled) = match row {
        Some(row) => {
            let state = row.0.read();
            (state.tabindex, state.disabled)
        }
        None => (if group_disabled { "-1" } else { "0" }, group_disabled),
    };
    let mark = use_check_mark(row);
    let expand = use_hook(try_consume_context::<TreeRowExpand>);
    let parts = item_parts(
        row_expanded(row),
        expand,
        mark,
        props.icon,
        props.trailing_icon,
        props.trailing,
        props.children,
    );

    let in_tree = row.is_some();
    let item = use_box()
        .framework_sx(&TREE_ITEM_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .event("onclick", move |event: MouseEvent| {
            if disabled {
                return;
            }
            if let Some(onclick) = &props.onclick {
                onclick.call(event);
            }
        });
    // The treeitem is the widget: a nested button would read the label twice.
    if in_tree {
        item.attr("data-tree-activate", true)
            .render(HtmlTag::Span, props.attributes, parts)
    } else {
        // Soft-disabled as `Button`: a native `disabled` would drop its focus (todo 3873).
        item.attr("type", "button")
            .attr("tabindex", tabindex)
            .attr("aria-disabled", disabled.then_some("true"))
            .render(HtmlTag::Button, props.attributes, parts)
    }
}
