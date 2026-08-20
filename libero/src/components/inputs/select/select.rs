use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        HtmlTag, Input, States,
        common::{base_props, focus_ring_sx},
        layout::use_box,
    },
    hooks::{use_css, use_theme},
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
        .border_color("grey.5")
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
        /// Corner radius, independent of `size`.
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

    let size = props.size.copied_or(theme.select.size);
    let radius = props.radius.copied_or(theme.select.radius);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .into();

    // The `<option>`s mount in the same pass as the `<select>`, so on the
    // creating render there is nothing for `value` to select and the browser
    // silently takes the first option. Emitting no `value` until `use_effect`
    // fires defers it to an update, by which point the options exist.
    let mut mounted = use_signal(|| false);
    use_effect(move || mounted.set(true));
    let value = mounted().then(|| props.value.clone());

    // Both `prepare()`s and the `use_css` run unconditionally: they are hooks,
    // and the label below is optional.
    let wrapper = use_box()
        .framework_sx(&SELECT_WRAPPER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .prepare();
    let select = use_box()
        .framework_sx(&SELECT_BASE_SX)
        .states(&states)
        .prepare();
    let label_class = use_css(Some(&SELECT_LABEL_SX), CssLayer::Framework);

    let label = match &props.label {
        Some(label) => rsx! { span { class: label_class, {label.clone()} } },
        None => rsx! {},
    };

    let select = select
        .attr("value", value)
        .event("onchange", move |event: FormEvent| {
            props.onchange.call(event.value())
        })
        .render(HtmlTag::Select, props.attributes, props.children);

    wrapper.render(
        HtmlTag::Label,
        Vec::new(),
        rsx! {
            {label}
            {select}
        },
    )
}
