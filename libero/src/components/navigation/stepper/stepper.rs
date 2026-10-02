use dioxus::prelude::*;

use super::core::{StepSpec, StepState, StepperPart, StepperView, derived_state, render_stepper};
use crate::{
    components::common::{
        ClassList, Input, OptionLabel, Options, Orientation, Parts, States, base_color,
        contrast_color, fill_color, input_from_str, literal_contrast, parts_under_sx, text_color,
    },
    hooks::{use_localization, use_root_id, use_theme},
    sx::{Sx, ThemeAwareValue},
    theme::{Size, StepLabelPosition},
    utils::warn,
};

input_from_str!(StepLabelPosition);

// Hand-written: `base_props!` is not generic.
#[derive(Props, Clone, PartialEq)]
pub struct StepperProps<T: Options> {
    /// The current step; `None` marks every step completed. Strictly controlled.
    #[props(!optional)]
    value: Option<T>,
    /// A step's body: the current one's below the strip, or each under its step when vertical.
    #[props(default)]
    panel: Option<Callback<T, Element>>,
    /// The steps, in order. Defaults to every `Options::options()`.
    #[props(default)]
    options: Option<Vec<T>>,
    /// Overrides `Options::label`; `OptionLabel::rich` draws rsx.
    #[props(default)]
    option_label: Option<Callback<T, OptionLabel>>,
    /// A second line under a step's label; empty means none.
    #[props(default)]
    option_description: Option<Callback<T, String>>,
    /// Overrides a step's derived state; the only way to set `StepState::Error`.
    #[props(default)]
    state: Option<Callback<T, Option<StepState>>>,
    /// The picked step. Without it the steps are not interactive.
    #[props(default)]
    onstepclick: Option<EventHandler<T>>,
    /// Whether steps not reached yet can be picked. Default `false`.
    #[props(default)]
    allow_next_steps: Option<bool>,
    /// `horizontal` (default) or `vertical`.
    #[props(default, into)]
    orientation: Input<Orientation>,
    /// `side` or `below` the marker; `side` falls back to `below` under 120px a step.
    #[props(default, into)]
    label_position: Input<StepLabelPosition>,
    #[props(default, into)]
    size: Input<Size>,
    /// The current and completed markers, and the connectors behind them.
    #[props(default, into)]
    color: Input<ThemeAwareValue>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    /// Styles for the inner parts, keyed by [`StepperPart`]; `sx` wins a tie.
    #[props(default, into)]
    parts: Input<Parts<StepperPart>>,
    #[props(default, into)]
    states: Input<States>,
}

/// The stages of a process over an enum, with the current step's content.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Options, Stepper};
/// # fn app() -> Element {
/// # #[derive(Clone, Copy, PartialEq, Options)] enum Stage { Account, Shipping, Review }
/// let mut stage = use_signal(|| Some(Stage::Account));
/// rsx! {
///     Stepper {
///         value: stage(),
///         onstepclick: move |s| stage.set(Some(s)),
///         panel: |s: Stage| match s {
///             Stage::Account => rsx! { AccountForm {} },
///             Stage::Shipping => rsx! { AddressForm {} },
///             Stage::Review => rsx! { OrderSummary {} },
///         },
///     }
/// }
/// # }
/// # #[component] fn AccountForm() -> Element { rsx! {} }
/// # #[component] fn AddressForm() -> Element { rsx! {} }
/// # #[component] fn OrderSummary() -> Element { rsx! {} }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/stepper>
#[component]
pub fn Stepper<T: Options>(props: StepperProps<T>) -> Element {
    let root = use_root_id(&props.attributes);
    let theme = use_theme();
    let labels = use_localization().stepper;

    // Not themed: the shared `Orientation` defaults to vertical.
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    let vertical = orientation == Orientation::Vertical;

    let values = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
    let (reached, current) = match &props.value {
        None => (values.len(), None),
        Some(active) => match values.iter().position(|value| value == active) {
            Some(index) => (index, Some(index)),
            None => {
                warn("Stepper: `value` is not one of the steps, so none is current.");
                (0, None)
            }
        },
    };

    let steps: Vec<StepSpec> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let label = match &props.option_label {
                Some(label) => label.call(value.clone()),
                None => OptionLabel::from(value.label()),
            };
            let derived = derived_state(index, reached, current);
            StepSpec {
                name: label.name,
                rich: label.content,
                description: props
                    .option_description
                    .as_ref()
                    .map(|description| description.call(value.clone()))
                    .filter(|description| !description.is_empty()),
                state: props
                    .state
                    .as_ref()
                    .and_then(|state| state.call(value.clone()))
                    .unwrap_or(derived),
                content: match &props.panel {
                    Some(content) if vertical => content.call(value.clone()),
                    _ => rsx! {},
                },
            }
        })
        .collect();

    let content = match (&props.panel, &props.value, current, vertical) {
        (Some(content), Some(active), Some(_), false) => Some(content.call(active.clone())),
        _ => None,
    };

    let interactive = props.onstepclick.is_some();
    let onstepclick = props.onstepclick;
    let pick = use_callback(move |index: usize| {
        if let Some(onstepclick) = &onstepclick
            && let Some(value) = values.get(index)
        {
            onstepclick.call(value.clone());
        }
    });

    let color = props.color.as_ref().and_then(|color| {
        let base = base_color(Some(color));
        let contrast = contrast_color(&base)
            .and_then(|contrast| contrast.resolve(None))
            .or_else(|| literal_contrast(&base));
        let fill = fill_color(&base)?;
        text_color(&base).map(|text| (text, fill, contrast))
    });

    render_stepper(
        StepperView {
            steps,
            reached,
            current,
            content,
            onstepclick: interactive.then_some(pick),
            allow_next_steps: props.allow_next_steps.unwrap_or(false),
            orientation,
            label_position: props.label_position.copied_or(theme.stepper.label_position),
            size: props.size.copied_or(theme.stepper.size),
            color,
            completed_label: labels.completed,
            error_label: labels.error,
            sx: parts_under_sx(&props.parts, props.sx),
            class: props.class,
            states: props.states,
            attributes: props.attributes,
        },
        root(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::{Part, part_table};

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        let header = "& > [data-slot='list'] > [data-slot='step'] > [data-slot='header']";
        let slots: Vec<_> = part_table::<StepperPart>()
            .into_iter()
            .map(|(slot, _)| slot)
            .collect();

        assert_eq!(
            slots,
            [
                "list",
                "step",
                "header",
                "marker",
                "body",
                "label",
                "description",
                "panel"
            ]
        );
        assert_eq!(StepperPart::Header.selector(), header);
        assert_eq!(
            StepperPart::Label.selector(),
            format!("{header} > [data-slot='body'] > [data-slot='label']")
        );
        assert_eq!(
            StepperPart::Panel.selector(),
            "& > [data-slot='panel'], & > [data-slot='list'] > [data-slot='step'] > * > * > [data-slot='panel']"
        );
    }
}
