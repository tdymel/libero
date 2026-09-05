use dioxus::prelude::*;

use super::core::{StepSpec, StepState, StepperView, derived_state, render_stepper};
use crate::{
    components::{
        ClassList, Input, OptionLabel, Options, Orientation, States,
        common::{base_color, contrast_color, fill_color, input_from_str, text_color},
    },
    hooks::{use_root_id, use_theme},
    sx::{Sx, ThemeAwareValue},
    theme::{Size, StepLabelPosition},
    utils::warn,
};

input_from_str!(StepLabelPosition);

// Hand-written rather than `base_props!`, which is not generic - as
// `TabsProps` is.
#[derive(Props, Clone, PartialEq)]
pub struct StepperProps<T: Options> {
    /// The current step. `None` means every step is finished: all of them
    /// show as completed and none is current. Strictly controlled.
    #[props(!optional)]
    value: Option<T>,
    /// A step's body. Horizontal: called for the current step only, and
    /// shown below the strip. Vertical: called for every step, and shown
    /// under its own step while current - a closing step animates out around
    /// its content, so it has to have some. A closed step's rsx is never
    /// mounted either way, so it keeps no state.
    #[props(default)]
    panel: Option<Callback<T, Element>>,
    /// The steps to show, in order. Defaults to every `Options::options()`.
    #[props(default)]
    steps: Option<Vec<T>>,
    /// Overrides `Options::label`, like `Tabs::option_label`. `OptionLabel::rich`
    /// draws a label as rsx and still names it.
    #[props(default)]
    option_label: Option<Callback<T, OptionLabel>>,
    /// A second line under a step's label. An empty string means none.
    #[props(default)]
    option_description: Option<Callback<T, String>>,
    /// Overrides the state a step's position gives it. `None` keeps the
    /// derived one, so a caller names only the step that differs - and this is
    /// the only way to say `StepState::Error`.
    #[props(default)]
    state: Option<Callback<T, Option<StepState>>>,
    /// Called with the step a user picked. **Without it the steps are not
    /// interactive**: they render no buttons and no tab stops.
    #[props(default)]
    onstepclick: Option<EventHandler<T>>,
    /// With `onstepclick`, whether steps not reached yet can be picked.
    /// Defaults to `false`: only the completed steps and the current one.
    #[props(default)]
    allow_next_steps: Option<bool>,
    /// `horizontal` (default) or `vertical`. Vertical shows each step's
    /// content under the step itself, collapsing the rest.
    #[props(default, into)]
    orientation: Input<Orientation>,
    /// `side` or `below` the marker. Ignored when vertical.
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
    #[props(default, into)]
    states: Input<States>,
}

/// The stages of a process over an enum, with the current one's content.
/// Controlled: it renders `value` and reports a picked step through
/// `onstepclick`; moving on is the caller's.
///
/// The steps are `T::options()` unless `steps` narrows them, and `panel`
/// is a match over `T` - so a step without a body is a compile error.
/// Completed, current and pending come from the order; `state` adds errors.
///
/// ```ignore
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
/// ```
#[component]
pub fn Stepper<T: Options>(props: StepperProps<T>) -> Element {
    let root = use_root_id(&props.attributes);
    let theme = use_theme();

    // Hardcoded, not themed: the shared `Orientation` defaults to vertical,
    // and a themed default would have to move it below `components`.
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    let vertical = orientation == Orientation::Vertical;

    let values = props.steps.clone().unwrap_or_else(|| T::options().to_vec());
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
        let contrast = contrast_color(&base).and_then(|contrast| contrast.resolve(None));
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
            completed_label: theme.stepper.completed_label,
            error_label: theme.stepper.error_label,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
        },
        root(),
    )
}
