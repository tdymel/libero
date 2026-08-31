use dioxus::prelude::*;

use super::core::{SegmentSpec, SegmentedControlView, render_segmented_control};
use crate::{
    components::{
        Input, OptionLabel, Options,
        common::{Orientation, base_color, field_props},
        form::{use_bound, use_field},
        inputs::{ButtonVariant, button_variables},
    },
    hooks::{use_cache, use_theme},
    sx::ThemeAwareValue,
    theme::Size,
    utils::warn,
};

field_props! {
    pub struct SegmentedControlProps<T: Options> {
        /// Strictly controlled - pair it with `onchange`. Exactly one segment
        /// is selected, which is what makes this a radio group and not a row
        /// of toggles. Optional only so a field bound to a `Form` can leave it
        /// out.
        #[props(default)]
        value: Option<T>,
        /// Called with the segment that should become selected.
        #[props(default)]
        onchange: Option<EventHandler<T>>,
        /// What the control posts as. A path - `Settings::FIELDS.align()` -
        /// also binds it to the surrounding `Form`'s value when it has no
        /// `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<T>,
        /// Rules over the selection, shown once the control loses focus or its
        /// form is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<T>,
        /// The segments to show. Defaults to every `Options::options()` - which
        /// `String` leaves empty, so a runtime set passes them here.
        #[props(default)]
        options: Option<Vec<T>>,
        /// Overrides `Options::label`. Runs during render, so it can read a
        /// locale from context - which is how a renamed control stays renamed.
        ///
        /// `"Konto".into()` names a segment; `OptionLabel::rich(name, rsx! { .. })`
        /// draws it and names it, because the rsx is what a screen reader cannot
        /// use.
        #[props(default)]
        option_label: Option<Callback<T, OptionLabel>>,
        /// Segments that render but cannot be picked. `disabled` disables all
        /// of them.
        #[props(default)]
        disabled_options: Vec<T>,
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// The *unselected* look, shared by every segment.
        #[props(default, into)]
        variant: Input<ButtonVariant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Space between the segments. Set it and they stop sharing borders -
        /// each keeps its own, and its own radius.
        #[props(default, into)]
        gap: Input<Size>,
        /// Segments share the width evenly instead of sizing to their label.
        #[props(default)]
        full_width: Option<bool>,
        /// `false` keeps the segments out of the tab order, and a click on one
        /// leaves focus where it is - for a control inside a field's dropdown.
        /// On by default.
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// A connected strip of segments over an enum, exactly one of them selected.
/// Controlled: it renders `value` and asks for a new one through `onchange`.
///
/// The segments are `T::options()` unless `options` narrows them - so a
/// misspelled segment is a compile error rather than a selection that never
/// matches. Each one is a `<label>` beside a radio, which is what gives the
/// control its "1 of 3" announcement and arrow-key navigation for free.
///
/// The control is a field: the label names the `radiogroup` through
/// `aria-labelledby`, and the captions describe it.
#[component]
pub fn SegmentedControl<T: Options>(props: SegmentedControlProps<T>) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.button.size);
    let radius = props.radius.copied_or(theme.button.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let current = bound.value().or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() && !disabled {
        warn("SegmentedControl: without `onchange` the selection can never change.");
    }

    let values = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
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
    let field_states: Input<crate::components::States> = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("full-width", full_width)
        .into();

    let field = use_field()
        .labelled_by()
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
        .states(&field_states)
        .attributes(&props.attributes)
        .prepare();

    let segments: Vec<SegmentSpec> = values
        .iter()
        .map(|value| {
            let label = match &props.option_label {
                Some(label) => label.call(value.clone()),
                None => OptionLabel::from(value.label()),
            };
            let name = label.name;
            SegmentSpec {
                content: label.content.unwrap_or_else(|| rsx! { "{name}" }),
                name,
                disabled: disabled || props.disabled_options.contains(value),
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

    let variant = props.variant.copied_or_default();
    let color = base_color(props.color.as_ref());
    // Colour resolution plus rendering is ~790 ns, and `(variant, color)` is
    // the same on almost every render.
    let style = use_cache((variant, color), |(variant, color)| {
        button_variables(*variant, color, true)
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
            name,
            labelledby: field.label_id(),
            describedby: field.describedby(),
            invalid: field.invalid(),
            required,
            style,
            attributes: props.attributes,
        },
        field.id().to_string(),
    );

    field.render(control)
}
