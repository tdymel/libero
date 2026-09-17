use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, OptionSource, Options, States,
        common::{
            Orientation, field_props, has_shortcut_modifier, names_itself, neighbour,
            use_name_warning,
        },
        form::{Radio, use_bound, use_field},
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
        // A row is under 24px tall at every step below `xxl`, so WCAG 2.5.8
        // holds only by its spacing exception: rows 24px apart, centre to
        // centre. Topped up per row rather than by a bigger gap, so a row
        // that a caption already makes taller keeps the theme's gap.
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

/// Scoped to this group's own root, so two groups can hold the same options
/// without colliding.
fn focus_option(root: &ElementHandle, index: usize) {
    let selector = format!("input[data-radio-index=\"{index}\"]");
    let _ = root.query_selector(&selector).and_then(|el| el.focus());
}

field_props! {
    pub struct RadioGroupProps<T: Options> {
        /// Strictly controlled - pair it with `onchange`. `None` selects
        /// nothing, which is what an unanswered question looks like.
        #[props(default)]
        value: Option<T>,
        /// Called with the option the caller should select next.
        #[props(default)]
        onchange: Option<EventHandler<T>>,
        /// What the group posts as. A path - `Survey::FIELDS.plan()` - also
        /// binds it to the surrounding `Form`'s value when it has no
        /// `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<Option<T>>,
        /// Rules over the selection, shown once the group loses focus or its
        /// form is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Option<T>>,
        /// The options to show. Defaults to every `Options::options()` - which
        /// `String` and any other runtime type leave empty, so those pass them
        /// here.
        ///
        /// `OptionItem::new(value).disabled(true)` renders an option that
        /// cannot be picked and that the arrow keys step over; `disabled`
        /// disables all of them. A grouped
        /// [`OptionList`](crate::components::OptionList) is accepted and its
        /// options are drawn flattened, in the order given: the group is the
        /// field, so it draws no group headings inside itself.
        ///
        /// A group has no dropdown to put a loader in, so a **pending**
        /// [`OptionSource`](crate::components::OptionSource) - one built from
        /// a [`Resource`] that has not answered yet - simply draws no
        /// options. Await the fetch above the field if that matters.
        #[props(default, into)]
        options: OptionSource<T>,
        /// Overrides `Options::label`. Runs during render, so it can read a
        /// locale from context.
        #[props(default)]
        option_label: Option<Callback<T, String>>,
        /// A line under each option's label. Empty text renders none. What
        /// makes a `Card` option worth its surface.
        #[props(default)]
        option_description: Option<Callback<T, String>>,
        /// `Card` draws every option as a bordered surface that is its own
        /// hit area. A row of cards stretches them to one height.
        #[props(default, into)]
        variant: Input<ChoiceVariant>,
        /// Lays the options out in a row instead of a column. A form stacks;
        /// a row is for two or three short options.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// The ring and dot colour of the selected option.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
    }
}

/// A group of radios over an enum, exactly one of them selected.
///
/// The group is the field: it owns the question's label, description, helper
/// text and status, the `name` that makes the set exclusive, and the single
/// tab stop the ARIA pattern asks for. Arrow keys move through the options and
/// select as they go, wrapping at the ends.
#[component]
pub fn RadioGroup<T: Options>(props: RadioGroupProps<T>) -> Element {
    let theme = use_theme();
    let root = use_element();

    let size = props.size.copied_or(theme.radio.size);
    let required = props.required.unwrap_or(false);
    let horizontal = props.orientation.copied_or_default() == Orientation::Horizontal;

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    // In a radio group the arrow keys *are* the change - APG has them move and
    // select in one press - so read-only refuses them outright rather than
    // moving focus without selecting, which would leave the roving `tabindex`
    // on an option that is not focused. Each option refuses its own click too.
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
    // One identity across renders, so an option whose own props did not
    // change skips the re-render.
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
    // One tab stop for the whole group: the selected option, or the first one
    // that can be picked. A disabled input takes no focus, so a selected
    // option the caller has disabled cannot hold the stop - the group would
    // drop out of the Tab order. Arrow keys move between them from there.
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
            // The web moves and selects on its own for a native radio group;
            // cancelling it keeps one code path, and gives Blitz - which does
            // neither - the same behaviour. Read-only too: the native arrow
            // would otherwise walk focus onto an option that is not the tab
            // stop, while the selection stays put (todo 320, measured).
            event.prevent_default();
            if readonly {
                return;
            }
            let Some(next) = neighbour(&option_disabled, tab_stop, step) else {
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
        .attr("role", "radiogroup")
        .attr("aria-readonly", readonly.then_some("true"))
        .attr("aria-labelledby", label_id)
        .attr("aria-describedby", describedby)
        .attr("aria-invalid", invalid.then_some("true"))
        .attr("aria-required", required.then_some("true"))
        .element(&root)
        .event("onkeydown", arrows)
        // Read-only keeps focus on the tab stop, the checked option (APG): a
        // click elsewhere would leave a second tab stop behind (todo 746). A
        // task later: the move fires `focusin` again, inside this handler.
        .event("onfocusin", move |_: FocusEvent| {
            if readonly && !disabled {
                spawn(async move {
                    next_task().await;
                    focus_option(&root, tab_stop);
                });
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
