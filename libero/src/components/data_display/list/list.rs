use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    hooks::use_theme,
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
        .when(
            "ordered",
            sx().list_style_type("decimal").padding_inline_start("2em"),
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
    });
    // Only a hoisted or absent icon skips the write: `VNode` compares by `Rc`
    // pointer, so an inline `rsx!` icon is unequal and wakes the items every render.
    if *context.icon.peek() != props.icon {
        let mut icon = context.icon;
        icon.set(props.icon.clone());
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
        // Safari/VoiceOver drops list semantics under `list-style: none`.
        .attr_default("role", "list")
        .render(tag, props.attributes, props.children)
}
