use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::core::AttributeValue;
use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_theme},
    platform::ElementApi,
    sx::{StaticSx, sx},
    theme::{ListDefaults, Size},
};

static LIST_BASE_SX: StaticSx = StaticSx::new(|| {
    ListDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .list_style("none")
        .margin("0")
        .padding("0")
        // Outside markers, in em so two-digit numbers still fit the gutter.
        // An icon replaces the number, so its item pulls the gutter back.
        .when(
            "ordered",
            sx().list_style_type("decimal")
                .padding_inline_start("2em")
                .selector(
                    "& > [data-state~='with-icon']",
                    sx().margin_inline_start("-2em"),
                )
                .selector(
                    "& > [data-state~='subheader']",
                    sx().margin_inline_start("-2em"),
                ),
        )
});

base_props! {
    pub struct ListProps {
        /// Item gap and nested-list indent.
        #[props(default, into)]
        size: Input<Size>,
        /// A numbered `ol`.
        #[props(default)]
        ordered: bool,
        /// Shown beside every item's first line, unless the item sets its own.
        #[props(default)]
        icon: Option<Element>,
        children: Element,
    }
}

/// The list's `icon`, for its items. Every `List` provides one, so nesting does not inherit.
#[derive(Clone, Copy)]
pub(crate) struct ListContext {
    pub(crate) icon: Signal<Option<Element>>,
    /// Native only, for an ordered list: Blitz skips a flex item in its count (todo 4213).
    pub(crate) numbering: Signal<Option<Numbering>>,
    /// Native only: the items' [`order_id`]s, put in DOM order after each render.
    pub(crate) items: CopyValue<Vec<u64>>,
    /// The list's own [`order_id`], which its items name so a nested list's stay apart.
    pub(crate) list: u64,
    /// Bumped when an item mounts or drops, so the items renumber.
    pub(crate) version: Signal<u32>,
}

/// An ordered list's first number and direction, from its `start` and `reversed` attributes.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Numbering {
    pub(crate) start: Option<i64>,
    pub(crate) reversed: bool,
}

impl Numbering {
    fn read(attributes: &[Attribute]) -> Self {
        let value = |name| {
            attributes
                .iter()
                .rev()
                .find(|attribute| attribute.name == name)
                .map(|attribute| &attribute.value)
        };
        let start = match value("start") {
            Some(AttributeValue::Int(start)) => Some(*start),
            Some(AttributeValue::Float(start)) => Some(*start as i64),
            Some(AttributeValue::Text(start)) => start.trim().parse().ok(),
            _ => None,
        };
        let reversed = match value("reversed") {
            Some(AttributeValue::Bool(reversed)) => *reversed,
            Some(AttributeValue::Text(_)) => true,
            _ => false,
        };
        Self { start, reversed }
    }

    /// The number of the item at `index` among `count`, as a browser counts `list-item`s.
    pub(crate) fn of(self, index: usize, count: usize) -> i64 {
        match self.reversed {
            true => self.start.unwrap_or(count as i64) - index as i64,
            false => self.start.unwrap_or(1) + index as i64,
        }
    }
}

pub(super) const NATIVE: bool = cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// A process-wide unique id for a list or an item, written into the DOM on native.
pub(super) fn order_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Native only: after each render, and each item mount or drop, sorts the items as the
/// DOM has them. Mount order misses a keyed insert or move.
fn use_dom_order(context: ListContext, element: ElementHandle) {
    let mut renders = use_hook(|| CopyValue::new(0_u64));
    if NATIVE {
        *renders.write() += 1;
    }
    let renders = *renders.peek();
    use_effect(use_reactive!(|(renders,)| {
        let _ = renders;
        let ListContext {
            numbering,
            mut items,
            list,
            mut version,
            ..
        } = context;
        // Read for the subscription: an item mounting alone renders no list.
        version.read();
        if !NATIVE || numbering.peek().is_none() {
            return;
        }
        let Ok(nodes) = element.query_selector_all(&format!("[data-list-of=\"{list}\"]")) else {
            return;
        };
        let order: Vec<u64> = nodes
            .iter()
            .filter_map(|node| node.attribute("data-list-item").ok().flatten())
            .filter_map(|id| id.parse().ok())
            .filter(|id| items.peek().contains(id))
            .collect();
        let moved = {
            let items = items.peek();
            order.len() == items.len() && *items != order
        };
        if moved {
            items.set(order);
            *version.write() += 1;
        }
    }));
}

/// A vertical list of [`ListItem`](super::ListItem)s: a bare `ul`, or a numbered `ol`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{List, ListItem};
/// # fn app() -> Element {
/// rsx! {
///     List { ordered: true,
///         ListItem { "Install" }
///         ListItem { "Configure" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/list>
#[component]
pub fn List(props: ListProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.list.size);

    let context = use_context_provider(|| ListContext {
        icon: Signal::new(props.icon.clone()),
        numbering: Signal::new(None),
        items: CopyValue::new(Vec::new()),
        list: order_id(),
        version: Signal::new(0),
    });
    let element = use_element();
    use_dom_order(context, element);
    // Only a hoisted or absent icon skips the write: `VNode` compares by `Rc`
    // pointer, so an inline `rsx!` icon is unequal and wakes the items every render.
    if *context.icon.peek() != props.icon {
        let mut icon = context.icon;
        icon.set(props.icon.clone());
    }
    let numbering = (NATIVE && props.ordered).then(|| Numbering::read(&props.attributes));
    if *context.numbering.peek() != numbering {
        let mut signal = context.numbering;
        signal.set(numbering);
    }

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("ordered", props.ordered)
        .into();
    let tag = if props.ordered {
        HtmlTag::Ol
    } else {
        HtmlTag::Ul
    };

    use_box()
        .framework_sx(&LIST_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .element(&element)
        // Safari/VoiceOver drops list semantics under `list-style: none`.
        .attr_default("role", "list")
        .render(tag, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 2468: the indent reaches a `List` directly under an item, not any `ul` below.
    #[test]
    fn the_indent_is_scoped_to_the_lists_own_nested_lists() {
        let sheet = crate::css::Stylesheet::from(&LIST_BASE_SX);
        let css = sheet.as_str();

        assert!(css.contains("] > li > ul[role='list']"), "{css}");
        assert!(!css.contains("] ul"), "{css}");
    }

    /// A `Tree`'s branch is a `List` with `role=group`, which the indent reaches unless its row has guides.
    #[test]
    fn the_indent_reaches_a_trees_group_but_not_a_guided_one() {
        let sheet = crate::css::Stylesheet::from(&LIST_BASE_SX);
        let css = sheet.as_str();

        assert!(
            css.contains("] > li:not([data-state~='guides']) > ul[role='group']"),
            "{css}"
        );
    }

    /// Todo 2466: an ordered list's icon item pulls back the gutter its number would have used.
    #[test]
    fn an_icon_item_of_an_ordered_list_pulls_back_the_number_gutter() {
        let sheet = crate::css::Stylesheet::from(&LIST_BASE_SX);
        let css = sheet.as_str();

        assert!(
            css.contains("> [data-state~='with-icon']") && css.contains("margin-inline-start:-2em"),
            "{css}"
        );
    }
}
