use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
    hooks::use_theme,
    sx::{StaticSx, sx},
    theme::{ComboboxDefaults, Size},
};

/// What the rows share with the `Combobox` around them. Signals, not values: a
/// provider runs once, so a plain field would freeze on the first render.
#[derive(Clone, Copy)]
pub(super) struct ComboboxContext {
    pub id: Signal<String>,
    pub size: Signal<Size>,
    pub radius: Signal<Size>,
    /// The highlighted row's `onpick`, so Enter can fire it.
    pub active_pick: Signal<Option<Callback<()>>>,
}

/// Where a row sits, provided per row so `ComboboxOption` needs no props to
/// know its own `id` or whether the arrows are on it.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ComboboxRowContext {
    pub index: usize,
    pub active: bool,
}

/// One option, handed to `Combobox`'s `option` callback.
#[derive(Clone, PartialEq)]
pub struct ComboboxOptionArgs<T> {
    pub value: T,
    pub index: usize,
    /// Whether the arrow keys are on this row. `ComboboxOption` reads it for
    /// itself - this is for a row drawn without one.
    pub active: bool,
}

static COMBOBOX_ROW_SX: StaticSx = StaticSx::new(|| {
    ComboboxDefaults::row_theme_vars()
        .display("flex")
        .align_items("center")
        .gap("8px")
        .width("100%")
        .cursor("pointer")
        .user_select("none")
        .white_space("nowrap")
        .overflow("hidden")
        .text_overflow("ellipsis")
        .hover(sx().background("grey.1"))
        .when("active", sx().background("grey.2"))
        .when("selected", sx().background("primary.1").color("primary.7"))
});

base_props! {
    pub struct ComboboxOptionProps {
        /// The current selection - `aria-selected` and a tint. Leave it unset
        /// for a suggestion list, where "selected" means nothing.
        #[props(default)]
        selected: Option<bool>,
        /// Overrides the keyboard highlight, which otherwise comes from the
        /// `Combobox` drawing this row.
        #[props(default)]
        active: Option<bool>,
        #[props(default)]
        onpick: Option<EventHandler<()>>,
        /// Row height and font size. Defaults to the `Combobox`'s own `size`.
        #[props(default, into)]
        size: Input<Size>,
        /// Corner radius. Defaults to the `Combobox`'s own, tightened by the
        /// dropdown's padding so the row nests inside it.
        #[props(default, into)]
        radius: Input<Size>,
        children: Element,
    }
}

/// A themed row for `Combobox`'s `option` callback. Registers itself as the
/// Enter target while it is the active row, and takes its `id` from the
/// `Combobox` so `aria-activedescendant` can point at it.
#[component]
pub fn ComboboxOption(props: ComboboxOptionProps) -> Element {
    let theme = use_theme();
    let combobox = try_consume_context::<ComboboxContext>();
    let row = try_consume_context::<Signal<ComboboxRowContext>>();

    let size = props.size.copied_or(match combobox {
        Some(context) => (context.size)(),
        None => theme.combobox.size,
    });
    let radius = props.radius.copied_or(match combobox {
        Some(context) => (context.radius)(),
        None => theme.combobox.radius,
    });

    let active = props
        .active
        .unwrap_or_else(|| row.is_some_and(|row| row().active));
    let id = combobox
        .zip(row)
        .map(|(combobox, row)| super::aria::option_id(&(combobox.id)(), row().index));

    let onpick = props.onpick;
    let pick = use_callback(move |()| {
        if let Some(onpick) = &onpick {
            onpick.call(());
        }
    });
    use_effect(use_reactive!(|active| {
        if let (true, Some(mut context)) = (active, combobox) {
            context.active_pick.set(Some(pick));
        }
    }));

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("active", active)
        .with("selected", props.selected.unwrap_or(false))
        .into();

    use_box()
        .framework_sx(&COMBOBOX_ROW_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .attr_default("id", id)
        .attr_default("role", "option")
        .attr("aria-selected", props.selected.map(|on| on.to_string()))
        // Or the click blurs whatever the caller focused first.
        .event("onmousedown", move |event: MouseEvent| {
            event.prevent_default();
        })
        .event("onclick", move |_: MouseEvent| pick.call(()))
        .render(HtmlTag::Div, props.attributes, props.children)
}
