use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            ClassList, HtmlTag, Input, OptionSource, Options, Orientation, Part, Parts, States,
            names_itself, use_name_warning,
        },
        form::{
            Asks, Caption, FieldName, FieldStatus, Validators, field_parts_enum, field_props,
            use_bound, use_field,
        },
        layout::use_box,
    },
    hooks::{use_localization, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ChoiceVariant, FIELD_GAP, Size},
    utils::warn,
};

#[cfg(feature = "checkbox-group")]
use crate::components::form::Checkbox;
#[cfg(feature = "switch-group")]
use crate::components::form::Switch;

// As `RadioGroup`'s: rows 24px apart centre to centre (WCAG 2.5.8 spacing exception).
static CHECK_GROUP_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap(FIELD_GAP.value())
        .selector(
            "& > *",
            sx().min_height(format!("calc(24px - {})", FIELD_GAP.value())),
        )
        .when(
            "horizontal",
            sx().flex_direction("row")
                .flex_wrap("wrap")
                .column_gap("16px"),
        )
});

#[cfg(feature = "checkbox-group")]
field_parts_enum! {
    /// [`CheckboxGroup`]'s inner parts, for its `parts` prop: a field's, the
    /// group, and every option's box.
    pub enum CheckboxGroupPart {
        /// The `group` holding the options.
        Control = "control" => "& > [data-slot='control']",
        /// Each option's box.
        Box = "box" => "& > [data-slot='control'] > * > [data-slot='control'] > [data-slot='box']",
    }
}

#[cfg(feature = "switch-group")]
field_parts_enum! {
    /// [`SwitchGroup`]'s inner parts, for its `parts` prop: a field's, the
    /// group, and every option's track and thumb.
    pub enum SwitchGroupPart {
        /// The `group` holding the options.
        Control = "control" => "& > [data-slot='control']",
        /// Each option's track.
        Track = "track" => "& > [data-slot='control'] > * > [data-slot='control'] > [data-slot='track']",
        /// Each option's thumb.
        Thumb = "thumb" => "& > [data-slot='control'] > * > [data-slot='control'] > [data-slot='track'] > [data-slot='thumb']",
    }
}

/// The props `CheckboxGroup` and `SwitchGroup` share, past a field's.
macro_rules! check_group_props {
    ($part:ty; $vis:vis struct $name:ident) => {
        field_props! {
            parts($part);
            without(radius);
            $vis struct $name<T: Options> {
                /// Strictly controlled, in option order; empty checks nothing.
                #[props(default)]
                value: Vec<T>,
                /// Called with the whole selection to hold next, in option order.
                #[props(default)]
                onchange: Option<EventHandler<Vec<T>>>,
                /// One hidden input per checked option. A path also binds the group to the surrounding `Form`.
                #[props(default, into)]
                name: FieldName<Vec<T>>,
                /// Rules over the selection, shown on blur or submit. `required` asks for one at least.
                #[props(default, into)]
                validate: Validators<Vec<T>>,
                /// The options to show. Defaults to `Options::options()`. Groups are
                /// flattened; a pending source draws no options.
                #[props(default, into)]
                options: OptionSource<T>,
                /// Overrides `Options::label`.
                #[props(default)]
                option_label: Option<Callback<T, String>>,
                /// A line under each option's label; empty text renders none.
                #[props(default)]
                option_description: Option<Callback<T, String>>,
                /// At most this many checked: at the cap the unchecked options disable.
                #[props(default)]
                max_checked: Option<usize>,
                /// `Card` draws every option as a bordered surface that is its hit area.
                #[props(default, into)]
                variant: Input<ChoiceVariant>,
                /// Lays the options out in a row instead of a column.
                #[props(default, into)]
                orientation: Input<Orientation>,
                /// The checked options' colour.
                #[props(default, into)]
                color: Input<ThemeAwareValue>,
            }
        }
    };
}

#[cfg(feature = "checkbox-group")]
check_group_props!(CheckboxGroupPart; pub struct CheckboxGroupProps);
#[cfg(feature = "switch-group")]
check_group_props!(SwitchGroupPart; pub struct SwitchGroupProps);

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    #[cfg(feature = "checkbox-group")]
    Checkbox,
    #[cfg(feature = "switch-group")]
    Switch,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            #[cfg(feature = "checkbox-group")]
            Self::Checkbox => "CheckboxGroup",
            #[cfg(feature = "switch-group")]
            Self::Switch => "SwitchGroup",
        }
    }
}

/// Both groups' props, moved out of either struct.
struct Group<T: Options, P: Part> {
    value: Vec<T>,
    onchange: Option<EventHandler<Vec<T>>>,
    name: FieldName<Vec<T>>,
    validate: Validators<Vec<T>>,
    options: OptionSource<T>,
    option_label: Option<Callback<T, String>>,
    option_description: Option<Callback<T, String>>,
    max_checked: Option<usize>,
    variant: Input<ChoiceVariant>,
    orientation: Input<Orientation>,
    color: Input<ThemeAwareValue>,
    label: Caption,
    description: Caption,
    helper: Caption,
    status: Input<FieldStatus>,
    size: Input<Size>,
    disabled: Option<bool>,
    required: Option<bool>,
    readonly: Option<bool>,
    class: Input<ClassList>,
    sx: Input<crate::sx::Sx>,
    parts: Input<Parts<P>>,
    states: Input<States>,
    attributes: Vec<Attribute>,
}

macro_rules! group {
    ($props:ident) => {
        Group {
            value: $props.value,
            onchange: $props.onchange,
            name: $props.name,
            validate: $props.validate,
            options: $props.options,
            option_label: $props.option_label,
            option_description: $props.option_description,
            max_checked: $props.max_checked,
            variant: $props.variant,
            orientation: $props.orientation,
            color: $props.color,
            label: $props.label,
            description: $props.description,
            helper: $props.helper,
            status: $props.status,
            size: $props.size,
            disabled: $props.disabled,
            required: $props.required,
            readonly: $props.readonly,
            class: $props.class,
            sx: $props.sx,
            parts: $props.parts,
            states: $props.states,
            attributes: $props.attributes,
        }
    };
}

/// A group of checkboxes over the caller's options type, any number of them checked.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{CheckboxGroup, Options};
/// #[derive(Clone, Copy, PartialEq, Options)]
/// enum Topping { Cheese, Olives, Basil }
///
/// # fn app() -> Element {
/// let mut toppings = use_signal(Vec::<Topping>::new);
/// rsx! {
///     CheckboxGroup {
///         label: "Toppings",
///         value: toppings(),
///         max_checked: 2,
///         onchange: move |next| toppings.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/checkbox-group>
#[cfg(feature = "checkbox-group")]
#[component]
pub fn CheckboxGroup<T: Options>(props: CheckboxGroupProps<T>) -> Element {
    check_group(Kind::Checkbox, group!(props))
}

/// A group of switches over the caller's options type, any number of them on.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Options, SwitchGroup};
/// #[derive(Clone, Copy, PartialEq, Options)]
/// enum Alert { Email, Push, Sms }
///
/// # fn app() -> Element {
/// let mut alerts = use_signal(|| vec![Alert::Email]);
/// rsx! {
///     SwitchGroup {
///         label: "Alerts",
///         value: alerts(),
///         onchange: move |next| alerts.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/switch-group>
#[cfg(feature = "switch-group")]
#[component]
pub fn SwitchGroup<T: Options>(props: SwitchGroupProps<T>) -> Element {
    check_group(Kind::Switch, group!(props))
}

fn check_group<T: Options, P: Part>(kind: Kind, props: Group<T, P>) -> Element {
    let theme = use_theme();
    let (size, theme_variant) = match kind {
        #[cfg(feature = "checkbox-group")]
        Kind::Checkbox => (theme.checkbox.size, theme.checkbox.variant),
        #[cfg(feature = "switch-group")]
        Kind::Switch => (theme.switch.size, theme.switch.variant),
    };
    let size = props.size.copied_or(size);
    let required = crate::components::form::fieldset_required(props.required);
    let horizontal = props.orientation.copied_or_default() == Orientation::Horizontal;

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let readonly = props.readonly.unwrap_or(false);
    let held = bound.value().unwrap_or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() && !disabled {
        warn(&format!(
            "{}: without `onchange` the selection can never change.",
            kind.name()
        ));
    }

    let list = props.options.or_static();
    let values = list.values();
    let checked: Vec<bool> = values.iter().map(|option| held.contains(option)).collect();
    // Held values the options don't list still post, so a tick keeps them (todo 4288).
    let unlisted: Vec<T> = held
        .iter()
        .filter(|value| !values.contains(value))
        .cloned()
        .collect();
    if !unlisted.is_empty() && !props.options.is_pending() {
        warn(&format!(
            "{}: `value` holds options that are not in `options`, so none of them shows.",
            kind.name()
        ));
    }
    let count = checked.iter().filter(|on| **on).count();
    let full = props.max_checked.is_some_and(|max| count >= max);
    if props.max_checked.is_some_and(|max| count > max) {
        warn(&format!(
            "{}: `value` holds more than `max_checked` options.",
            kind.name()
        ));
    }

    let field = use_field()
        .labelled_by()
        .names_group()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&held))
        .bound(&bound)
        .required(required)
        .required_in_name(use_localization().common.required)
        .empty(count == 0)
        .readonly(readonly)
        .asks(Asks::Option)
        .disabled(disabled)
        .size(size)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();
    use_name_warning(
        field.label_id().is_some() || names_itself(&props.attributes),
        match kind {
            #[cfg(feature = "checkbox-group")]
            Kind::Checkbox => {
                "CheckboxGroup: no `label`, `aria-label` or `aria-labelledby`, so the group has \
                 no name and its question is never read."
            }
            #[cfg(feature = "switch-group")]
            Kind::Switch => {
                "SwitchGroup: no `label`, `aria-label` or `aria-labelledby`, so the group has \
                 no name and its question is never read."
            }
        },
    );

    let states: Input<States> = field
        .states()
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("horizontal", horizontal)
        .into();
    let group = use_box()
        .framework_sx(&CHECK_GROUP_SX)
        .states(&states)
        .prepare();

    // Through `emit`, so a pick also drops an error set from code (todo 4289).
    let emit = bound.emit(props.onchange);
    // Option order, whatever the click order: the value reads the same way the group does.
    let toggle = {
        let values = values.clone();
        let checked = checked.clone();
        use_callback(move |index: usize| {
            let Some(emit) = &emit else {
                return;
            };
            let next: Vec<T> = values
                .iter()
                .zip(&checked)
                .enumerate()
                .filter(|(at, (_, on))| **on != (*at == index))
                .map(|(_, (option, _))| option.clone())
                .chain(unlisted.iter().cloned())
                .collect();
            emit(next);
        })
    };

    let option_disabled = list.disabled();
    let variant = props.variant.copied_or(theme_variant);
    let options = values.iter().enumerate().map(|(index, option)| {
        let label = match &props.option_label {
            Some(format) => format.call(option.clone()),
            None => option.label(),
        };
        let description = props
            .option_description
            .as_ref()
            .map(|describe| describe.call(option.clone()))
            .filter(|description| !description.is_empty());
        rsx! {
            GroupOption {
                key: "{index}",
                kind,
                index,
                label,
                description,
                size,
                variant,
                color: props.color.clone(),
                checked: checked[index],
                disabled: disabled || option_disabled[index] || (full && !checked[index]),
                readonly,
                toggle,
            }
        }
    });

    // A disabled group posts nothing, as a disabled checkbox. Keyed, as its option siblings.
    let hidden = bound.name().filter(|_| !disabled).map(|name| {
        let posted: Vec<String> = held.iter().map(Options::value).collect();
        rsx! {
            Fragment { key: "hidden-{name}",
                for value in posted {
                    input { r#type: "hidden", name, value }
                }
            }
        }
    });

    let group = group
        .attr("data-slot", "control")
        .attr("role", "group")
        .attr("aria-labelledby", field.label_id())
        .attr("aria-describedby", field.describedby())
        // `aria-invalid` is allowed on a group, `aria-required` is not: the label says it (todo 4287).
        .attr("aria-invalid", field.invalid().then_some("true"))
        // An empty group while the options load is not a group with none.
        .attr("aria-busy", props.options.is_pending().then_some("true"))
        .render(
            HtmlTag::Div,
            Vec::new(),
            options.chain(hidden).collect::<Vec<_>>(),
        );

    field.render(group)
}

/// One option. Its own scope with plain props, so a toggle redraws the option it changes.
#[derive(Props, Clone, PartialEq)]
struct GroupOptionProps {
    kind: Kind,
    index: usize,
    label: String,
    description: Option<String>,
    size: Size,
    variant: ChoiceVariant,
    color: Input<ThemeAwareValue>,
    checked: bool,
    disabled: bool,
    readonly: bool,
    toggle: Callback<usize>,
}

#[component]
fn GroupOption(props: GroupOptionProps) -> Element {
    let GroupOptionProps {
        kind,
        index,
        label,
        description,
        size,
        variant,
        color,
        checked,
        disabled,
        readonly,
        toggle,
    } = props;
    // The group carries `required` and posts the value, so no option does either.
    match kind {
        #[cfg(feature = "checkbox-group")]
        Kind::Checkbox => rsx! {
            Checkbox {
                label,
                description,
                size,
                variant,
                color,
                checked,
                disabled,
                readonly,
                required: false,
                onchange: move |_| toggle.call(index),
            }
        },
        #[cfg(feature = "switch-group")]
        Kind::Switch => rsx! {
            Switch {
                label,
                description,
                size,
                variant,
                color,
                checked,
                disabled,
                readonly,
                required: false,
                onchange: move |_| toggle.call(index),
            }
        },
    }
}

#[cfg(all(test, feature = "checkbox-group", feature = "switch-group"))]
mod tests {
    use super::*;

    fn html(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    fn options() -> Vec<String> {
        ["A", "B", "C"].map(String::from).to_vec()
    }

    /// Todo 3984: at the cap the unchecked options disable; the checked ones stay free.
    #[test]
    fn at_max_checked_only_the_unchecked_options_disable() {
        let page = html(|| {
            rsx! {
                crate::LiberoProvider {
                    CheckboxGroup::<String> {
                        label: "Pick",
                        options: options(),
                        value: vec!["A".to_string(), "C".to_string()],
                        max_checked: 2,
                        onchange: |_| {},
                    }
                }
            }
        });
        assert!(page.contains("role=\"group\""), "{page}");
        assert_eq!(page.matches("type=\"checkbox\"").count(), 3, "{page}");
        assert_eq!(page.matches("disabled=true").count(), 1, "{page}");
    }

    /// The group posts its checked options as hidden inputs; the boxes post nothing.
    #[test]
    fn a_named_group_posts_each_checked_value() {
        let page = html(|| {
            rsx! {
                crate::LiberoProvider {
                    SwitchGroup::<String> {
                        label: "Alerts",
                        name: "alerts",
                        options: options(),
                        value: vec!["B".to_string()],
                        onchange: |_| {},
                    }
                }
            }
        });
        assert_eq!(page.matches("name=\"alerts\"").count(), 1, "{page}");
        assert!(page.contains("value=\"B\""), "{page}");
    }

    /// Todo 4046: a tick on a named group re-diffs the keyed options next to the hidden inputs.
    #[test]
    fn ticking_a_named_group_posts_the_new_value() {
        let mut dom = VirtualDom::new(|| {
            let mut value = use_context_provider(|| Signal::new(vec!["B".to_string()]));
            rsx! {
                crate::LiberoProvider {
                    CheckboxGroup::<String> {
                        label: "Pick",
                        name: "pick",
                        options: options(),
                        value: value(),
                        onchange: move |next| value.set(next),
                    }
                }
            }
        });
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            consume_context::<Signal<Vec<String>>>().set(vec!["A".to_string(), "B".to_string()])
        });
        dom.render_immediate(&mut dioxus::dioxus_core::NoOpMutations);
        let page = dioxus_ssr::render(&dom);
        assert_eq!(page.matches("name=\"pick\"").count(), 2, "{page}");
        assert!(page.contains("value=\"A\""), "{page}");
    }
}
