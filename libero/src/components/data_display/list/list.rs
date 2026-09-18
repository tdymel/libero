use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
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
        /// Item gap and nested-list indent - `theme.list.size` (`Md`) by default.
        #[props(default, into)]
        size: Input<Size>,
        /// An `ol` with visible numbers, for items whose order is the point.
        #[props(default)]
        ordered: bool,
        /// Shown at the start of every item, beside its first line. A
        /// `ListItem`'s own `icon` overrides it.
        #[props(default)]
        icon: Option<Element>,
        children: Element,
    }
}

/// The list's `icon`, for its items. Every `List` provides one, so a nested
/// list does not inherit its parent's.
#[derive(Clone, Copy)]
pub(crate) struct ListContext {
    pub(crate) icon: Signal<Option<Element>>,
}

/// A vertical list of [`ListItem`](super::ListItem)s: a bare `ul`, or with
/// `ordered` a numbered `ol`.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{List, ListItem};
/// # fn app() -> Element { rsx! {
/// List { ordered: true,
///     ListItem { "Install" }
///     ListItem { "Configure" }
/// }
/// # } }
/// ```
#[component]
pub fn List(props: ListProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.list.size);

    let context = use_context_provider(|| ListContext {
        icon: Signal::new(props.icon.clone()),
    });
    // Guarded render-time write, as `Grid` does, so an unchanged render wakes
    // no item.
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
        // Not redundant: Safari with VoiceOver drops list semantics from a
        // `list-style: none` list. A caller's own `role` still wins.
        .attr_default("role", "list")
        .render(tag, props.attributes, props.children)
}
