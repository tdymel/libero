use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, Options, States, common::focus_ring_sx, layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{SelectDefaults, Size},
    utils::warn,
};

static SELECT_WRAPPER_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").flex_direction("column").gap("4px"));

static SELECT_LABEL_SX: StaticSx = StaticSx::new(|| sx().font_size("0.75rem"));

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
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .background("grey.1"),
        )
        .focus_visible(focus_ring_sx())
});

// Hand-written rather than `base_props!`, which is not generic - as
// `TabsProps` is.
#[derive(Props, Clone, PartialEq)]
pub struct SelectProps<T: Options> {
    /// Strictly controlled - pair it with `onchange`. `None` shows
    /// `placeholder` and selects nothing.
    #[props(default)]
    value: Option<T>,
    /// Called with the option the caller should select next. Never fires for
    /// the placeholder, which cannot be picked.
    #[props(default)]
    onchange: Option<EventHandler<T>>,
    /// The options to show. Defaults to every `Options::options()` - which
    /// `String` and any other runtime type leave empty, so those pass them
    /// here.
    #[props(default)]
    options: Option<Vec<T>>,
    /// Overrides `Options::label`. Runs during render, so it can read a
    /// locale from context.
    ///
    /// Returns a `String`, not an `OptionLabel`: `<option>` holds text and
    /// nothing else, so there is no rich form to offer.
    #[props(default)]
    option_label: Option<Callback<T, String>>,
    /// Shown while `value` is `None`, as an unpickable first entry.
    #[props(default)]
    placeholder: Option<String>,
    #[props(default, into)]
    size: Input<Size>,
    /// Corner radius, independent of `size`.
    #[props(default, into)]
    radius: Input<Size>,
    #[props(default)]
    disabled: Option<bool>,
    /// The field's own caption, above the control.
    #[props(default)]
    label: Option<String>,
    /// Styles the caption alone - the rest of `sx` lands on the wrapper.
    #[props(default, into)]
    label_sx: Input<Sx>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
}

/// A styled native `<select>` over an enum, with its own caption. Controlled:
/// it renders `value` and asks for a new one through `onchange`.
///
/// The options are `T::options()` unless `options` narrows them - so a
/// misspelled option is a compile error, and `onchange` hands back the value
/// itself rather than a string the caller has to look up again.
#[component]
pub fn Select<T: Options>(props: SelectProps<T>) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.select.size);
    let radius = props.radius.copied_or(theme.select.radius);

    if props.onchange.is_none() {
        warn("Select: without `onchange` the selection can never change.");
    }

    let values = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
    if values.is_empty() {
        warn("Select: no options - a `T` without static `options()` needs `options`.");
    }
    let selected = props
        .value
        .as_ref()
        .and_then(|value| values.iter().position(|option| option == value));
    if props.value.is_some() && selected.is_none() && !values.is_empty() {
        warn("Select: `value` is not one of the options, so none is selected.");
    }

    let labels: Vec<String> = values
        .iter()
        .map(|value| match &props.option_label {
            Some(label) => label.call(value.clone()),
            None => value.label(),
        })
        .collect();

    let disabled = props.disabled.unwrap_or(false);
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("disabled", disabled)
        .into();

    // Both `prepare()`s run unconditionally: they are hooks, and the caption
    // below is optional.
    let wrapper = use_box()
        .framework_sx(&SELECT_WRAPPER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .prepare();
    let select = use_box()
        .framework_sx(&SELECT_BASE_SX)
        .states(&states)
        .prepare();
    let label_box = use_box()
        .framework_sx(&SELECT_LABEL_SX)
        .sx(&props.label_sx)
        .prepare();

    let caption = match &props.label {
        Some(label) => label_box.render(HtmlTag::Span, Vec::new(), rsx! { {label.clone()} }),
        None => rsx! {},
    };

    let onchange = props.onchange;
    let pick = use_callback(move |index: usize| {
        if let Some(onchange) = &onchange
            && let Some(value) = values.get(index)
        {
            onchange.call(value.clone());
        }
    });

    let placeholder = props.placeholder.clone().unwrap_or_default();

    // `selected` on each `<option>`, not `value` on the `<select>`: the
    // property is written to the option itself, so it does not depend on the
    // parent's children already existing - which is what made the old
    // `value` path miss on the creating render, and what left SSR with no
    // selection at all.
    let select = select
        .attr("disabled", disabled)
        .event("onchange", move |event: FormEvent| {
            if let Ok(index) = event.value().parse::<usize>() {
                pick.call(index);
            }
        })
        .render(
            HtmlTag::Select,
            props.attributes,
            rsx! {
                if selected.is_none() {
                    option {
                        value: "",
                        selected: true,
                        disabled: true,
                        hidden: true,
                        "{placeholder}"
                    }
                }
                for (index, label) in labels.iter().enumerate() {
                    option {
                        key: "{index}",
                        value: "{index}",
                        selected: selected == Some(index),
                        "{label}"
                    }
                }
            },
        );

    wrapper.render(HtmlTag::Label, Vec::new(), vec![caption, select])
}
