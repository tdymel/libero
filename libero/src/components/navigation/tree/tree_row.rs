use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
};

use dioxus::prelude::*;

use crate::{
    components::{
        common::states,
        common::{
            HtmlTag, Part, ROW_HOVER_TINT, focus_ring_sx, forced_on_sx, on_start_bar_sx,
            on_tint_color,
        },
        data_display::List,
        layout::use_box,
    },
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        Color, ColorShade, ColorValue, CssVar, ICON_SIZE, LIST_INDENT, Size,
        TREE_GUIDE_ACTIVE_COLOR, TREE_GUIDE_ACTIVE_WIDTH, TREE_GUIDE_COLOR, TREE_GUIDE_WIDTH,
    },
};

use super::{
    tree::{Expansion, TreePart},
    tree_node::{ErasedRenderNode, TreeNodeErased, TreeNodeRenderArgsErased},
};

/// The row's tabindex, `disabled` and expansion, for its content. A signal, as
/// `use_context_provider` runs once.
#[derive(Clone, Copy)]
pub(super) struct TreeRowContext(pub Signal<TreeRowState>);

#[derive(Clone, Copy, PartialEq)]
pub(super) struct TreeRowState {
    pub tabindex: &'static str,
    pub disabled: bool,
    /// `None` for a leaf.
    pub expanded: Option<bool>,
    /// The row draws the current look itself, so a `NavLink` in it skips its own (todo 2037).
    pub current_look: bool,
}

/// Whether the enclosing tree row already draws the current look; outside a tree, `false`.
pub(crate) fn row_draws_current() -> bool {
    try_consume_context::<TreeRowContext>().is_some_and(|row| row.0.read().current_look)
}

// No hover here: the `<li>` holds its descendants, so it would paint the chain.
// Only the subtree drops the pointer, so the `<li>` shows `not-allowed` (todo 596).
static TREE_ROW_SX: StaticSx = StaticSx::new(|| {
    // The ring goes on the row's line: on the `<li>` it would wrap the open subtree.
    // `box-shadow` too: the stylesheet's plain focus rule rings every focusable.
    sx().selector("&:focus-visible", sx().outline("none").box_shadow("none"))
        .selector(
            format!(
                "&:focus-visible > [data-slot='{}']",
                TreePart::Content.slot()
            ),
            focus_ring_sx(),
        )
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .selector("& *", sx().pointer_events("none")),
        )
        // The size state is only set with guides, as only the guide's padding reads it.
        .per_size(|size| sx().var(TREE_GUIDE_PADDING, guide_padding(size)))
        // The guide sits under the parent's chevron; `[role]` outranks `List`'s `& ul` indent.
        .when(
            "guides",
            sx().selector(
                "& > [role=\"group\"]",
                sx().margin_inline_start(guide_offset())
                    .padding_inline_start(TREE_GUIDE_PADDING.value())
                    .border_inline_start(format!(
                        "{} solid {}",
                        TREE_GUIDE_WIDTH.value(),
                        TREE_GUIDE_COLOR.value()
                    )),
            ),
        )
});

/// The group's padding after the guide: what is left of `List`'s indent, at least none.
const TREE_GUIDE_PADDING: CssVar = CssVar::new("--lsx-tree-guide-padding");

/// From the row's inline start to the guide: content padding plus half a chevron.
fn guide_offset() -> String {
    format!(
        "calc(8px + {} / 2 - {} / 2)",
        ICON_SIZE.value(Size::Xs),
        TREE_GUIDE_WIDTH.value()
    )
}

fn guide_padding(size: Size) -> String {
    format!(
        "max(0px, calc({} - {} - {}))",
        LIST_INDENT.value(size),
        guide_offset(),
        TREE_GUIDE_WIDTH.value()
    )
}

// A sibling of the children `List`, keeping hover row-local. No vertical padding,
// so a border in `render_node`'s content spans the full row.
static TREE_ROW_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap("6px")
        .padding_left("8px")
        .padding_right("8px")
        .border_radius("4px")
        .cursor("pointer")
        // A long unbreakable label wraps instead of widening the page (1.4.10).
        .with("overflow-wrap", "anywhere")
        .hover(sx().background(ROW_HOVER_TINT))
        // Pulled back over the parent's guide from its outer edge, the content left
        // in place. Not centred: half pixels round apart natively.
        .when(
            "guide-current",
            sx().margin_inline_start(format!(
                "calc(-1 * ({} + {}))",
                TREE_GUIDE_PADDING.value(),
                TREE_GUIDE_WIDTH.value()
            ))
            .padding_inline_start(format!(
                "calc({} + {} - {} + 8px)",
                TREE_GUIDE_PADDING.value(),
                TREE_GUIDE_WIDTH.value(),
                TREE_GUIDE_ACTIVE_WIDTH.value()
            ))
            .border_inline_start(format!(
                "{} solid {}",
                TREE_GUIDE_ACTIVE_WIDTH.value(),
                TREE_GUIDE_ACTIVE_COLOR.value()
            ))
            .with("border-start-start-radius", "0")
            .with("border-end-start-radius", "0"),
        )
        // NavLink's active look where no guide marks the current row (todo 1571); a
        // NavLink in it skips its own through `row_draws_current` (todo 2037).
        .when("current", current_look_sx().hover(current_look_sx()))
});

fn current_look_sx() -> crate::sx::Sx {
    let bar = on_tint_color(&ThemeAwareValue::ColorValue(ColorValue::Shade(
        Color::Primary,
        ColorShade::S6,
    )))
    .unwrap_or_default();
    sx().background("primary.1")
        .and(on_start_bar_sx("0px", &bar))
        .and(forced_on_sx())
}

#[derive(Props, Clone, PartialEq)]
pub(super) struct TreeRowProps {
    pub node: TreeNodeErased,
    pub size: Size,
    pub depth: usize,
    pub expansion: Expansion,
    /// [`open_key`] of this row, from the parent, so a toggle re-renders only the
    /// rows whose shown subtree changed (todo 2093).
    pub open_key: u64,
    /// The tab stop's path from this row: `Some(&[])` is this row, `None` off the
    /// path, so a move re-renders only the rows it leaves and enters.
    pub active: Option<Vec<usize>>,
    /// Where `Tree`'s `current` row is, relative to this row, as `active`.
    #[props(default)]
    pub current: Option<Vec<usize>>,
    pub active_id: Signal<Option<String>>,
    pub render_node: ErasedRenderNode,
    /// Under a disabled branch, so the row acts disabled too.
    #[props(default)]
    pub ancestor_disabled: bool,
    pub guides: bool,
}

#[component]
pub(super) fn TreeRow(props: TreeRowProps) -> Element {
    let node = &props.node;
    let has_children = node.has_children();
    // `peek`: `open_key` brings the change, a read would wake every branch.
    let open = props.expansion.open.peek();
    let is_expanded = has_children.then(|| open.contains(&node.id));
    let child_keys: Vec<u64> = match is_expanded {
        Some(true) => node
            .children
            .iter()
            .map(|child| open_key(child, &open))
            .collect(),
        _ => Vec::new(),
    };
    drop(open);
    let disabled = node.disabled || props.ancestor_disabled;
    let is_roving_active = props.active.as_deref().is_some_and(<[usize]>::is_empty);
    // The `<li>` is the roving tab stop; `render_node`'s content is always "-1".
    let li_tabindex = if is_roving_active { "0" } else { "-1" };

    let is_current = props.current.as_deref().is_some_and(<[usize]>::is_empty);
    // A top-level row has no guide to mark.
    let guide_current = props.guides && is_current && props.depth > 0;
    let current_look = is_current && !guide_current;
    let state = TreeRowState {
        tabindex: "-1",
        disabled,
        expanded: is_expanded,
        current_look,
    };
    let mut row_context = use_context_provider(|| Signal::new(state));
    if *row_context.peek() != state {
        row_context.set(state);
    }
    use_context_provider(|| TreeRowContext(row_context));

    // No chevron column here: that's `default_tree_render`'s.
    let content = props.render_node.call(TreeNodeRenderArgsErased {
        id: node.id.clone(),
        data: node.data.clone(),
        expanded: is_expanded,
        disabled,
        tabindex: "-1",
        depth: props.depth,
    });

    let mut active_id = props.active_id;
    let expansion = props.expansion;
    let id = node.id.clone();

    // Any focus on the row takes the tab stop: a click in a child group's indent
    // focuses this `<li>`, and the root's keys only work on the stop.
    let focus_id = node.id.clone();
    let onfocus = move |event: Event<FocusData>| {
        // Should a renderer bubble focus, the ancestors' rows must not take the stop.
        event.stop_propagation();
        if active_id.peek().as_deref() != Some(focus_id.as_str()) {
            active_id.set(Some(focus_id.clone()));
        }
    };

    let onclick = move |_: Event<MouseData>| {
        if disabled {
            return;
        }
        active_id.set(Some(id.clone()));
        if has_children {
            expansion.toggle(&id);
        }
    };

    // The node's own flag: the ancestor's opacity already covers this row.
    let row_states = states()
        .with("disabled", node.disabled)
        .with("guides", props.guides)
        .with(props.size.state_name(), props.guides)
        .into();
    let row = use_box()
        .framework_sx(&TREE_ROW_SX)
        .states(&row_states)
        .prepare();
    let content_states = states()
        .with("guide-current", guide_current)
        .with("current", current_look)
        .into();
    // On the inner div, a sibling of the children `List`, so a child's click
    // never toggles the parent.
    let row_content = use_box()
        .framework_sx(&TREE_ROW_CONTENT_SX)
        .states(&content_states)
        .prepare()
        .attr("data-slot", TreePart::Content.slot())
        // Marks the row's ring for the e2e focus pass, as the `<li>` holds the focus.
        .attr("data-ring", "true")
        .event("onclick", onclick)
        .render(HtmlTag::Div, Vec::new(), content);

    // A leaf skips the conditional slot. A branch keeps one template either
    // way, so a toggle does not remount its content.
    let children = match is_expanded {
        Some(open) => rsx! {
            {row_content}
            if open {
                List {
                    "role": "group",
                    "data-slot": TreePart::Group.slot(),
                    size: props.size,
                    for (index , child) in node.children.iter().enumerate() {
                        TreeRow {
                            key: "{child.id}",
                            node: child.clone(),
                            size: props.size,
                            depth: props.depth + 1,
                            expansion: props.expansion,
                            open_key: child_keys[index],
                            active: child_active(props.active.as_deref(), index),
                            current: child_active(props.current.as_deref(), index),
                            active_id: props.active_id,
                            render_node: props.render_node.clone(),
                            ancestor_disabled: disabled,
                            guides: props.guides,
                        }
                    }
                }
            }
        },
        None => row_content,
    };

    row.attr("role", "treeitem")
        .attr("data-slot", TreePart::Row.slot())
        .attr("data-tree-id", node.id.to_string())
        // No selection model: without an explicit "false" Chrome announces the
        // tab-stop row as selected. `current` speaks through `aria-current`.
        .attr("aria-selected", "false")
        .attr("aria-current", is_current.then_some("true"))
        .attr("aria-expanded", is_expanded.map(|value| value.to_string()))
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("tabindex", li_tabindex)
        .event("onfocus", onfocus)
        // A press would focus a row the roving stop is not on, where no key works.
        .event("onmousedown", move |event: Event<MouseData>| {
            if disabled {
                event.prevent_default();
            }
        })
        .render(HtmlTag::Li, Vec::new(), children)
}

/// A hash of the open branches `node` shows, itself included: a closed branch
/// hides its open descendants, so they do not count.
pub(super) fn open_key(node: &TreeNodeErased, open: &HashSet<String>) -> u64 {
    fn walk(node: &TreeNodeErased, open: &HashSet<String>, hasher: &mut DefaultHasher) {
        if node.has_children() && open.contains(&node.id) {
            node.id.hash(hasher);
            for child in node.children.iter() {
                walk(child, open, hasher);
            }
            // Closes the subtree, so `a/b` open differs from `a` and its sibling `b` open.
            hasher.write_u8(0xff);
        }
    }
    let mut hasher = DefaultHasher::new();
    walk(node, open, &mut hasher);
    hasher.finish()
}

/// The part of a row's `active` path that its child `index` sees.
pub(super) fn child_active(active: Option<&[usize]>, index: usize) -> Option<Vec<usize>> {
    match active? {
        [head, rest @ ..] if *head == index => Some(rest.to_vec()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{TREE_ROW_CONTENT_SX, TREE_ROW_SX};
    use crate::css::Stylesheet;

    /// Todo 1571: where no guide marks it.
    #[test]
    fn a_current_row_takes_the_tint_and_start_bar() {
        let css = Stylesheet::from(&*TREE_ROW_CONTENT_SX);
        let css = css.as_str();

        let rule = css
            .find(r#"[data-state~="current"]{"#)
            .expect("the current look");
        let body = &css[rule..css[rule..].find('}').map_or(css.len(), |end| rule + end)];
        assert!(body.contains("background-image:linear-gradient("), "{css}");
        assert!(
            body.contains("background:var(--lsx-primary-fill-1)"),
            "{css}"
        );
    }

    /// An open branch's `<li>` holds its subtree, so the ring goes on the row's line (todo 1570).
    #[test]
    fn the_focus_ring_draws_on_the_row_line_not_the_subtree() {
        let css = Stylesheet::from(&*TREE_ROW_SX);
        let css = css.as_str();

        assert!(
            css.contains(":focus-visible{outline:none;box-shadow:none;}"),
            "{css}"
        );
        let line = css
            .find(":focus-visible > [data-slot='content']{")
            .expect("the ring on the row's line");
        assert!(
            css[line..].contains("outline:var(--lsx-focus-ring-width) solid"),
            "{css}"
        );
    }
}
