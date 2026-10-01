use dioxus::prelude::*;

use super::core::{
    SegmentSpec, SegmentedControlPart, SegmentedControlView, render_segmented_control,
};
use crate::{
    components::{
        buttons::button_variables,
        common::{
            Input, OptionLabel, OptionSource, Options, Orientation, Variant, base_color,
            names_itself, use_name_warning, use_toolbar_item,
        },
        form::{field_props, use_bound, use_field, use_form_context},
    },
    hooks::{use_cache, use_element, use_form_owner, use_theme},
    sx::ThemeAwareValue,
    theme::Size,
    utils::warn,
};

field_props! {
    parts(SegmentedControlPart);
    pub struct SegmentedControlProps<T: Options> {
        /// Controlled: pair it with `onchange`. Optional only so a `Form`-bound
        /// control can leave it out.
        #[props(default)]
        value: Option<T>,
        /// Called with the segment that should become selected.
        #[props(default)]
        onchange: Option<EventHandler<T>>,
        /// What the control posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<T>,
        /// Rules over the selection, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<T>,
        /// The segments; `Options::options()` by default. Groups are flattened,
        /// and a pending source draws no segments.
        #[props(default, into)]
        options: OptionSource<T>,
        /// Overrides `Options::label`; runs during render, so it can read a locale.
        #[props(default)]
        option_label: Option<Callback<T, OptionLabel>>,
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// The *unselected* look, shared by every segment.
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Space between the segments; set, each keeps its own border and radius.
        #[props(default, into)]
        gap: Input<Size>,
        /// Segments share the width evenly instead of sizing to their label.
        #[props(default)]
        full_width: Option<bool>,
        /// `false` keeps the segments from taking focus, for a control inside
        /// a field's dropdown.
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// A connected strip of segments over an enum, exactly one of them selected.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Options, SegmentedControl};
/// #[derive(Clone, Copy, PartialEq, Options)]
/// enum View { List, Grid }
/// # fn app() -> Element {
/// let mut view = use_signal(|| View::List);
/// rsx! {
///     SegmentedControl {
///         label: "View",
///         value: view(),
///         onchange: move |next| view.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/segmented-control>
#[component]
pub fn SegmentedControl<T: Options>(props: SegmentedControlProps<T>) -> Element {
    let theme = use_theme();
    let element = use_element();

    let size = props.size.copied_or(theme.button.size);
    let radius = props.radius.copied_or(theme.button.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let toolbar_item = use_toolbar_item();
    // A toolbar keeps a disabled item focusable, in its arrow order: read-only instead.
    let soft_disabled = disabled && toolbar_item.is_some();
    let current = bound.value().or_else(|| props.value.clone());
    let in_form = use_form_context().is_some();
    // A raw `<form>` around the strip counts too (todo 660).
    let owner = use_form_owner(element, !in_form);
    let in_form = in_form || owner().is_some();

    if props.onchange.is_none() && !bound.is_bound() && !disabled {
        warn("SegmentedControl: without `onchange` the selection can never change.");
    }

    let list = props.options.or_static();
    let values = list.values();
    let option_disabled = list.disabled();
    if values.is_empty() {
        warn("SegmentedControl: no segments - a `T` without static `options()` needs `options`.");
    }
    let selected = current
        .as_ref()
        .and_then(|current| values.iter().position(|value| value == current));
    if selected.is_none() && !values.is_empty() {
        warn("SegmentedControl: `value` is not one of the options, so none is selected.");
    }

    let full_width = props.full_width.unwrap_or(false);
    let field_states: Input<crate::components::common::States> = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("full-width", full_width)
        .into();

    let field = use_field()
        .labelled_by()
        .names_group()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(
            current
                .as_ref()
                .and_then(|current| props.validate.check(current)),
        )
        .bound(&bound)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&field_states)
        .attributes(&props.attributes)
        .prepare();
    use_name_warning(
        field.label_id().is_some() || names_itself(&props.attributes),
        "SegmentedControl: no `label`, `aria-label` or `aria-labelledby`, so the group \
         has no name and only its options are read.",
    );

    let segments: Vec<SegmentSpec> = values
        .iter()
        .zip(&option_disabled)
        .map(|(value, &option_disabled)| {
            let label = match &props.option_label {
                Some(label) => label.call(value.clone()),
                None => OptionLabel::from(value.label()),
            };
            let name = label.name;
            SegmentSpec {
                // In a span, so a squeezed `full_width` segment can cut the
                // text with an ellipsis - a bare text node cannot take one.
                content: label.content.unwrap_or_else(|| rsx! { span { "{name}" } }),
                name,
                value: value.value(),
                disabled: (disabled && !soft_disabled) || option_disabled,
            }
        })
        .collect();

    let name = bound
        .name()
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}-segment", field.id()));
    let emit = bound.emit(props.onchange);
    let pick = use_callback(move |index: usize| {
        if let Some(emit) = &emit
            && let Some(value) = values.get(index)
        {
            emit(value.clone());
        }
    });

    let variant = props.variant.copied_or(theme.segmented_control.variant);
    let color = base_color(props.color.as_ref());
    // Colour resolution plus rendering is ~790 ns, and `(variant, color)` is
    // the same on almost every render.
    let style = use_cache((variant, color), |(variant, color)| {
        button_variables(*variant, color, true, false)
    });

    let control = render_segmented_control(
        SegmentedControlView {
            segments,
            selected,
            onselect: pick,
            orientation: props.orientation.copied_or(Orientation::Horizontal),
            variant,
            full_width,
            size,
            radius,
            gap: props.gap.as_ref().copied(),
            focusable: props.focusable.unwrap_or(true),
            readonly: props.readonly.unwrap_or(false) || soft_disabled,
            enter: !in_form,
            name,
            labelledby: field.label_id(),
            describedby: field.describedby(),
            invalid: field.invalid(),
            required,
            element,
            style,
            attributes: props.attributes,
            toolbar_item,
            soft_disabled,
        },
        field.id().to_string(),
    );

    field.render(control)
}
