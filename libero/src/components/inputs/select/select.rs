use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, States,
        common::{base_props, focus_ring_sx},
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{SelectDefaults, Size},
};

static SELECT_WRAPPER_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").flex_direction("column").gap("4px"));

static SELECT_LABEL_SX: StaticSx = StaticSx::new(|| sx().font_size("0.75rem").color("grey.7"));

static SELECT_BASE_SX: StaticSx = StaticSx::new(|| {
    SelectDefaults::theme_vars()
        .display("block")
        .width("100%")
        .border_style("solid")
        .border_width("1px")
        .border_color("grey.4")
        .background("white")
        .color("black")
        .cursor("pointer")
        .hover(sx().border_color("grey.7"))
        .focus_visible(focus_ring_sx())
});

base_props! {
    pub struct SelectProps {
        #[props(default, into)]
        size: Input<Size>,
        /// Corner radius - `theme.select.radius` by default, independent
        /// of `size`.
        #[props(default, into)]
        radius: Input<Size>,
        #[props(into)]
        value: String,
        #[props(default)]
        onchange: EventHandler<String>,
        #[props(default)]
        label: Option<String>,
        children: Element,
    }
}

#[component]
pub fn Select(props: SelectProps) -> Element {
    let theme = use_theme();

    let size = props.size.as_ref().copied().unwrap_or(theme.select.size);
    let radius = props
        .radius
        .as_ref()
        .copied()
        .unwrap_or(theme.select.radius);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true);

    // `<option>`s come from `props.children` - a dynamic node mounted in the
    // same pass as `<select>` itself, so on the *creating* render there's no
    // matching option yet for the browser to select against `value` (it
    // silently falls back to the first option instead). Rendering no
    // `value` attribute at all on that first render, then flipping
    // `mounted` true in `use_effect` (which only runs once the whole
    // subtree - options included - is actually committed), defers the real
    // value application to a subsequent *update*, where the plain `value:`
    // attribute path already works correctly (dioxus-web's own
    // `set_attribute.js` sets `.value` as a DOM property, not just an
    // attribute - it just needs the options to already exist, which they do
    // by then).
    let mut mounted = use_signal(|| false);
    use_effect(move || mounted.set(true));
    let value = mounted().then(|| props.value.clone());

    rsx! {
        Box {
            component: "label",
            class: props.class,
            sx: props.sx,
            framework_sx: &SELECT_WRAPPER_SX,
            if let Some(label) = &props.label {
                Box { component: "span", framework_sx: &SELECT_LABEL_SX, {label.clone()} }
            }
            Box {
                component: "select",
                states,
                framework_sx: &SELECT_BASE_SX,
                value: value,
                onchange: move |event: FormEvent| props.onchange.call(event.value()),
                attributes: props.attributes,
                {props.children}
            }
        }
    }
}
