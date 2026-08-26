use dioxus::prelude::*;

use super::core::{SegmentSpec, SegmentedControlView, render_segmented_control};
use crate::{
    components::{
        ClassList, Input, OptionLabel, Options, States,
        common::{Orientation, base_color},
        inputs::{ButtonVariant, button_variables},
    },
    hooks::{use_cache, use_root_id, use_theme},
    sx::{Sx, ThemeAwareValue},
    theme::Size,
    utils::warn,
};

// Hand-written rather than `base_props!`, which is not generic - as
// `TabsProps` is.
#[derive(Props, Clone, PartialEq)]
pub struct SegmentedControlProps<T: Options> {
    /// Strictly controlled - pair it with `onchange`. Exactly one segment is
    /// selected, which is what makes this a radio group and not a row of
    /// toggles.
    value: T,
    /// Called with the segment that should become selected.
    #[props(default)]
    onchange: Option<EventHandler<T>>,
    /// The segments to show. Defaults to every `Options::options()` - which
    /// `String` leaves empty, so a runtime set passes them here.
    #[props(default)]
    segments: Option<Vec<T>>,
    /// Overrides `Options::label`. Runs during render, so it can read a
    /// locale from context - which is how a renamed control stays renamed.
    ///
    /// `"Konto".into()` names a segment; `OptionLabel::rich(name, rsx! { .. })`
    /// draws it and names it, because the rsx is what a screen reader cannot
    /// use.
    #[props(default)]
    label: Option<Callback<T, OptionLabel>>,
    /// Segments that render but cannot be picked.
    #[props(default)]
    disabled: Vec<T>,
    #[props(default, into)]
    orientation: Input<Orientation>,
    /// The *unselected* look, shared by every segment.
    #[props(default, into)]
    variant: Input<ButtonVariant>,
    #[props(default, into)]
    color: Input<ThemeAwareValue>,
    #[props(default, into)]
    size: Input<Size>,
    /// Corner radius of the control's outer corners; inner ones are square.
    #[props(default, into)]
    radius: Input<Size>,
    /// Space between the segments. Set it and they stop sharing borders -
    /// each keeps its own, and its own radius.
    #[props(default, into)]
    gap: Input<Size>,
    /// Segments share the width evenly instead of sizing to their label.
    #[props(default)]
    full_width: Option<bool>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
}

/// A connected strip of segments over an enum, exactly one of them selected.
/// Controlled: it renders `value` and asks for a new one through `onchange`.
///
/// The segments are `T::options()` unless `segments` narrows them - so a
/// misspelled segment is a compile error rather than a selection that never
/// matches. Each one is a `<label>` around a radio, which is what gives the
/// control its "1 of 3" announcement and arrow-key navigation for free.
#[component]
pub fn SegmentedControl<T: Options>(props: SegmentedControlProps<T>) -> Element {
    let root = use_root_id(&props.attributes);
    let theme = use_theme();

    if props.onchange.is_none() {
        warn("SegmentedControl: without `onchange` the selection can never change.");
    }

    let values = props
        .segments
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
    if values.is_empty() {
        warn("SegmentedControl: no segments - a `T` without static `options()` needs `segments`.");
    }
    let selected = values.iter().position(|value| *value == props.value);
    if selected.is_none() && !values.is_empty() {
        warn("SegmentedControl: `value` is not one of the segments, so none is selected.");
    }

    let segments: Vec<SegmentSpec> = values
        .iter()
        .map(|value| {
            let label = match &props.label {
                Some(label) => label.call(value.clone()),
                None => OptionLabel::from(value.label()),
            };
            let name = label.name;
            SegmentSpec {
                content: label.content.unwrap_or_else(|| rsx! { "{name}" }),
                name,
                disabled: props.disabled.contains(value),
            }
        })
        .collect();

    let onchange = props.onchange;
    let pick = use_callback(move |index: usize| {
        if let Some(onchange) = &onchange
            && let Some(value) = values.get(index)
        {
            onchange.call(value.clone());
        }
    });

    let variant = props.variant.copied_or_default();
    let color = base_color(props.color.as_ref());
    // Colour resolution plus rendering is ~790 ns, and `(variant, color)` is
    // the same on almost every render.
    let style = use_cache((variant, color), |(variant, color)| {
        button_variables(*variant, color, true)
    });

    render_segmented_control(
        SegmentedControlView {
            segments,
            selected,
            onselect: pick,
            orientation: props.orientation.copied_or(Orientation::Horizontal),
            variant,
            full_width: props.full_width.unwrap_or(false),
            size: props.size.copied_or(theme.button.size),
            radius: props.radius.copied_or(theme.button.radius),
            gap: props.gap.as_ref().copied(),
            style,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        },
        root(),
    )
}
