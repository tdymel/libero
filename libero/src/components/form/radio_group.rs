use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, OptionSource, Options, Orientation, Part, States,
            has_shortcut_modifier, names_itself, neighbour, use_name_warning,
        },
        form::{Radio, field_parts_enum, field_props, use_bound, use_field},
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_theme},
    platform::{ElementApi, logical_key, next_task},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ChoiceVariant, FIELD_GAP, Size},
    utils::warn,
};

static RADIO_GROUP_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap(FIELD_GAP.value())
        // WCAG 2.5.8 spacing exception: rows 24px apart centre to centre, topped
        // up per row so a captioned row keeps the theme's gap.
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

/// Scoped to this group's root, so two groups with the same options don't collide.
fn focus_option(root: &ElementHandle, index: usize) {
    let selector = format!("input[data-radio-index=\"{index}\"]");
    let _ = root.query_selector(&selector).and_then(|el| el.focus());
}

/// The index of the option under focus; the radios render in index order.
fn focused_option(root: &ElementHandle) -> Option<usize> {
    let radios = root.query_selector_all("input[data-radio-index]").ok()?;
    radios.iter().position(|radio| radio.is_focused())
}

field_parts_enum! {
    /// [`RadioGroup`]'s inner parts, for its `parts` prop: a field's, the
    /// group, and every option's circle and dot.
    pub enum RadioGroupPart {
        /// The `radiogroup` holding the options.
        Control = "control" => "& > [data-slot='control']",
        /// Each option's ring.
        Circle = "circle" => "& > [data-slot='control'] > * > [data-slot='control'] > [data-slot='circle']",
        /// Each option's checked mark.
        Dot = "dot" => "& > [data-slot='control'] > * > [data-slot='control'] > [data-slot='circle'] > [data-slot='dot']",
    }
}

field_props! {
    parts(RadioGroupPart);
    without(radius);
    pub struct RadioGroupProps<T: Options> {
        /// Strictly controlled; `None` selects nothing.
        #[props(default)]
        value: Option<T>,
        /// Called with the option the caller should select next.
        #[props(default)]
        onchange: Option<EventHandler<T>>,
        /// What the group posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<Option<T>>,
        /// Rules over the selection, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<Option<T>>,
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
        /// `Card` draws every option as a bordered surface that is its hit area.
        #[props(default, into)]
        variant: Input<ChoiceVariant>,
        /// Lays the options out in a row instead of a column.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// The ring and dot colour of the selected option.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
    }
}

/// A group of radios over the caller's options type, one of them selected.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Options, RadioGroup};
/// #[derive(Clone, Copy, PartialEq, Options)]
/// enum Plan { Free, Pro }
///
/// # fn app() -> Element {
/// let mut plan = use_signal(|| None::<Plan>);
/// rsx! {
///     RadioGroup {
///         label: "Plan",
///         value: plan(),
///         onchange: move |next| plan.set(Some(next)),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/radio-group>
#[component]
pub fn RadioGroup<T: Options>(props: RadioGroupProps<T>) -> Element {
    let theme = use_theme();
    let root = use_element();

    let size = props.size.copied_or(theme.radio.size);
    let required = props.required.unwrap_or(false);
    let horizontal = props.orientation.copied_or_default() == Orientation::Horizontal;

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    // The arrow keys move and select in one press (APG), so read-only refuses
    // them outright; moving alone would strand the roving `tabindex`.
    let readonly = props.readonly.unwrap_or(false);
    let current = bound.value().unwrap_or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() && !disabled {
        warn("RadioGroup: without `onchange` the selection can never change.");
    }

    let list = props.options.or_static();
    let values = list.values();
    let selected = current
        .as_ref()
        .and_then(|value| values.iter().position(|option| option == value));

    let field = use_field()
        .labelled_by()
        .names_group()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&current))
        .bound(&bound)
        .required(required)
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
        "RadioGroup: no `label`, `aria-label` or `aria-labelledby`, so the group has no \
         name and its question is never read.",
    );

    let states: Input<States> = field
        .states()
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("horizontal", horizontal)
        .into();

    let group = use_box()
        .framework_sx(&RADIO_GROUP_SX)
        .states(&states)
        .prepare();

    let onchange = props.onchange;
    let setter = bound.setter();
    // One identity across renders, so an unchanged option skips the re-render.
    let pick = {
        let values = values.clone();
        use_callback(move |index: usize| {
            let Some(option) = values.get(index) else {
                return;
            };
            match (&onchange, &setter) {
                (Some(onchange), _) => onchange.call(option.clone()),
                (None, Some(setter)) => setter.set(Some(option.clone())),
                (None, None) => {}
            }
        })
    };

    let option_disabled = list.disabled();
    // One tab stop: the selected option, or the first enabled one. A disabled
    // selected option cannot hold it, or the group drops out of the Tab order.
    let tab_stop = selected
        .filter(|&index| !option_disabled[index])
        .or_else(|| option_disabled.iter().position(|off| !off))
        .unwrap_or(0);

    let arrows = {
        let option_disabled = option_disabled.clone();
        move |event: Event<KeyboardData>| {
            // A native radio leaves Alt/Ctrl/Meta+arrow to the browser.
            if disabled || has_shortcut_modifier(&event) {
                return;
            }
            let step = match logical_key(&event) {
                Key::ArrowDown | Key::ArrowRight => 1,
                Key::ArrowUp | Key::ArrowLeft => -1,
                _ => return,
            };
            // A lone enabled option has nowhere to go: the keys scroll the page (todo 2465).
            if neighbour(&option_disabled, tab_stop, step) == Some(tab_stop) {
                return;
            }
            // Cancel the native move: one code path, the same on Blitz, and
            // read-only focus stays on the tab stop (todo 320).
            event.prevent_default();
            if readonly {
                return;
            }
            // From the focused option: a parent that applies the pick late
            // leaves the tab stop behind (todo 2443).
            let from = focused_option(&root).unwrap_or(tab_stop);
            let Some(next) = neighbour(&option_disabled, from, step) else {
                return;
            };
            pick.call(next);
            focus_option(&root, next);
        }
    };

    let label_id = field.label_id();
    let describedby = field.describedby();
    let invalid = field.invalid();
    let option_label = props.option_label;
    let option_description = props.option_description;
    let variant = props.variant.copied_or(theme.radio.variant);
    let name = bound
        .name()
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}-radio", field.id()));
    let color = props.color.clone();

    let options = values.iter().enumerate().map(|(index, option)| {
        let label = match &option_label {
            Some(format) => format.call(option.clone()),
            None => option.label(),
        };
        let description = option_description
            .as_ref()
            .map(|describe| describe.call(option.clone()))
            .filter(|description| !description.is_empty());
        rsx! {
            GroupRadio {
                key: "{index}",
                index,
                value: option.value(),
                label,
                description,
                name: name.clone(),
                size,
                variant,
                color: color.clone(),
                checked: selected == Some(index),
                disabled: disabled || option_disabled[index],
                readonly,
                tab_stop: index == tab_stop,
                pick,
            }
        }
    });

    let group = group
        .attr("data-slot", RadioGroupPart::Control.slot())
        .attr("role", "radiogroup")
        .attr("aria-readonly", readonly.then_some("true"))
        .attr("aria-labelledby", label_id)
        .attr("aria-describedby", describedby)
        .attr("aria-invalid", invalid.then_some("true"))
        .attr("aria-required", required.then_some("true"))
        .element(&root)
        .event("onkeydown", arrows)
        // Read-only keeps focus on the checked option (todo 746). A task later:
        // the move fires `focusin` again, inside this handler.
        .event("onfocusin", move |_: FocusEvent| {
            if readonly && !disabled {
                spawn(async move {
                    next_task().await;
                    focus_option(&root, tab_stop);
                });
            }
        })
        // Blitz fires no `focusin` for a click's move; its click comes first.
        .event("onclick", move |_: MouseEvent| {
            if readonly && !disabled {
                focus_option(&root, tab_stop);
            }
        })
        .render(HtmlTag::Div, props.attributes, options.collect::<Vec<_>>());

    field.render(group)
}

/// One option of a group. Its own scope with plain props, so a new selection
/// redraws the two options it changes, not every one.
#[derive(Props, Clone, PartialEq)]
struct GroupRadioProps {
    index: usize,
    /// What the radio posts when it is the checked one: `Options::value`,
    /// never the index or the browser's default `on`.
    value: String,
    label: String,
    description: Option<String>,
    name: String,
    size: Size,
    variant: ChoiceVariant,
    color: Input<ThemeAwareValue>,
    checked: bool,
    disabled: bool,
    readonly: bool,
    tab_stop: bool,
    pick: Callback<usize>,
}

#[component]
fn GroupRadio(props: GroupRadioProps) -> Element {
    let GroupRadioProps {
        index,
        value,
        label,
        description,
        name,
        size,
        variant,
        color,
        checked,
        disabled,
        readonly,
        tab_stop,
        pick,
    } = props;
    rsx! {
        Radio {
            label,
            description,
            name,
            size,
            variant,
            color,
            checked,
            disabled,
            readonly,
            tabindex: if tab_stop { "0" } else { "-1" },
            onselect: move |_| pick.call(index),
            "data-radio-index": "{index}",
            value,
        }
    }
}
