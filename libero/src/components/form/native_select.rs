use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, OptionSource, Options, recast_parts},
        form::{
            Asks, LiveControl, Select, SelectOptionArgs, field_control_sx, field_props, row_label,
            use_bound, use_field, use_field_frame,
        },
        layout::use_box,
    },
    hooks::use_theme,
    platform::opens_select_picker,
    sx::{StaticSx, sx},
    tokens::NamedColorCss,
    utils::warn,
};

/// Keeps the UA's chevron: our own would need `appearance: none`.
static NATIVE_SELECT_CONTROL_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .cursor("pointer")
        .selector("&:disabled", sx().cursor("not-allowed"))
        // The placeholder dims like a text field's (todo 582). The options
        // inherit the colour, so they take ink back.
        .selector("&[data-placeholder]", sx().color("text-dimmed"))
        .selector(
            "&[data-placeholder] option, &[data-placeholder] optgroup",
            sx().color(NamedColorCss::INK.value()),
        )
});

// No `readonly`: HTML has no read-only `<select>`; `Select` is the read-only picker (todo 304).
field_props! {
    without(readonly);
    pub struct NativeSelectProps<T: Options> {
        /// Strictly controlled; `None` shows `placeholder`.
        #[props(default)]
        value: Option<T>,
        /// Called with the option the caller should select next.
        #[props(default)]
        onchange: Option<EventHandler<T>>,
        /// What the select posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<Option<T>>,
        /// Rules over the selection, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<Option<T>>,
        /// The options to show. Defaults to `Options::options()`.
        #[props(default, into)]
        options: OptionSource<T>,
        /// Overrides `Options::label`; plain text, as `<option>` holds nothing else.
        #[props(default)]
        option_label: Option<Callback<T, String>>,
        /// Shown while `value` is `None`, as an unpickable first entry.
        #[props(default)]
        placeholder: Option<String>,
    }
}

/// Whether two options post the same string. `onchange` finds the first,
/// so the later one could never be picked.
fn repeats(posted: &[String]) -> bool {
    posted
        .iter()
        .enumerate()
        .any(|(index, value)| posted[..index].contains(value))
}

/// `[start, end)` of each run of equal group labels: one `<optgroup>` per
/// named run, as `OptionList::group` documents.
fn runs(groups: &[Option<String>]) -> Vec<(usize, usize)> {
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for (index, group) in groups.iter().enumerate() {
        match runs.last_mut() {
            Some((start, end)) if groups[*start] == *group => *end = index + 1,
            _ => runs.push((index, index + 1)),
        }
    }
    runs
}

/// A styled native `<select>` over the caller's options type.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{NativeSelect, Options};
/// #[derive(Clone, Copy, PartialEq, Options)]
/// enum Plan { Free, Pro }
///
/// # fn app() -> Element {
/// let mut plan = use_signal(|| Some(Plan::Free));
/// rsx! {
///     NativeSelect {
///         label: "Plan",
///         value: plan(),
///         onchange: move |next| plan.set(Some(next)),
///     }
/// }
/// # }
/// ```
///
/// No `readonly`, unlike every other field:
///
/// ```compile_fail,E0599
/// # use dioxus::prelude::*;
/// # use libero::components::{NativeSelect, Options};
/// # fn app() -> Element {
/// # #[derive(Clone, Copy, PartialEq, Options)] enum Plan { Free, Pro }
/// rsx! { NativeSelect::<Plan> { value: Some(Plan::Free), readonly: true } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/native-select>
#[component]
pub fn NativeSelect<T: Options>(props: NativeSelectProps<T>) -> Element {
    // Fixed per build, so the hooks below always run in the same order.
    if !opens_select_picker() {
        return listbox(props);
    }
    let mut props = props;
    let value = props.value.take();
    let mut live = use_signal(|| value.clone());
    if *live.peek() != value {
        live.set(value);
    }
    rsx! { NativeSelectShell::<T> { live, field: props } }
}

/// Everything but the selection, so a pick skips it and redraws the options.
#[component]
fn NativeSelectShell<T: Options>(live: Signal<Option<T>>, field: NativeSelectProps<T>) -> Element {
    let props = field;
    let theme = use_theme();

    let size = props.size.copied_or(theme.native_select.size);
    let radius = props.radius.copied_or(theme.native_select.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let bound_value = bound.value();

    if props.onchange.is_none() && !bound.is_bound() {
        warn("NativeSelect: without `onchange` the selection can never change.");
    }

    let list = props.options.or_static();
    let values = list.values();
    // A memo, so the shell redraws when no option starts or stops matching, not per pick.
    let live_unpicked = use_memo(use_reactive!(|values| {
        !live
            .read()
            .as_ref()
            .is_some_and(|value| values.contains(value))
    }));
    let unpicked = match &bound_value {
        Some(value) => !value.as_ref().is_some_and(|value| values.contains(value)),
        None => live_unpicked(),
    };
    let refused = list.disabled();
    let groups = list.group_labels();
    if values.is_empty() && !props.options.is_pending() {
        warn("NativeSelect: no options - a `T` without static `options()` needs `options`.");
    }
    let labels: Vec<String> = values
        .iter()
        .map(|value| match &props.option_label {
            Some(label) => label.call(value.clone()),
            None => value.label(),
        })
        .collect();
    // Rules read the selection here, so only a validated select redraws per pick.
    let rules = (!props.validate.is_empty())
        .then(|| {
            let current = bound_value.clone().unwrap_or_else(|| live.cloned());
            props.validate.check(&current)
        })
        .flatten();

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(rules)
        .bound(&bound)
        .required(required)
        .empty(unpicked)
        .asks(Asks::Select)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    let frame = use_field_frame().states(field.states()).prepare();

    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&NATIVE_SELECT_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    // What each `<option>` posts, and so what `onchange` reads back. Only a
    // runtime set can repeat one - the derive posts variant names.
    let posted: Vec<String> = values.iter().map(Options::value).collect();
    if repeats(&posted) {
        warn(
            "NativeSelect: two options share one `Options::value`, so picking the later one \
             selects the first.",
        );
    }

    let choices = values.clone();
    let onchange = props.onchange;
    let setter = bound.setter();
    let pick = {
        let posted = posted.clone();
        use_callback(move |value: String| {
            let Some(value) = posted
                .iter()
                .position(|option| *option == value)
                .and_then(|index| values.get(index))
            else {
                return;
            };
            match (&onchange, &setter) {
                (Some(onchange), _) => onchange.call(value.clone()),
                (None, Some(setter)) => setter.set(Some(value.clone())),
                (None, None) => {}
            }
        })
    };

    let placeholder = props.placeholder.clone().unwrap_or_default();

    // `selected` on each `<option>`, not `value` on the `<select>`: that missed
    // on the creating render and under SSR.
    let draw = Rc::new(move || {
        let current = bound_value.clone().unwrap_or_else(|| live.cloned());
        let selected = current
            .as_ref()
            .and_then(|value| choices.iter().position(|option| option == value));
        if current.is_some() && selected.is_none() && !choices.is_empty() {
            warn("NativeSelect: `value` is not one of the options, so none is selected.");
        }
        let option = |index: usize| {
            rsx! {
                option {
                    key: "{index}",
                    value: "{posted[index]}",
                    selected: (selected == Some(index)).then_some(true),
                    disabled: refused[index].then_some(true),
                    "{labels[index]}"
                }
            }
        };
        // One keyed node per `<option>` or `<optgroup>`: a nested unkeyed
        // fragment panicked Dioxus' diff once the placeholder went.
        let nodes = runs(&groups)
            .into_iter()
            .flat_map(|(start, end)| match &groups[start] {
                Some(group) => vec![rsx! {
                    optgroup { key: "g{start}", label: "{group}", {(start..end).map(option)} }
                }],
                None => (start..end).map(option).collect(),
            })
            .collect::<Vec<_>>();
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
            {nodes.into_iter()}
        }
    });
    let select = field
        .aria(control)
        .attr("name", bound.name().map(str::to_string))
        .attr("disabled", disabled)
        // `aria-required` only: native `required` plus the empty placeholder
        // matches `:invalid`, announced before anyone touched it (todo 581).
        .attr("data-controlled", true)
        .attr("data-placeholder", unpicked)
        .event("onchange", move |event: FormEvent| pick.call(event.value()))
        .render(
            HtmlTag::Select,
            props.attributes,
            rsx! { LiveControl { draw } },
        );

    field.render(frame.render(select))
}

/// `Select`'s listbox over the same props, where a `<select>` opens no picker.
fn listbox<T: Options>(props: NativeSelectProps<T>) -> Element {
    let theme = use_theme();
    let option_label = props.option_label;
    let label = move |value: &T| match &option_label {
        Some(label) => label.call(value.clone()),
        None => value.label(),
    };
    let option = use_callback(move |args: SelectOptionArgs<T>| row_label(label(&args.value)));
    let selection = use_callback(move |value: T| rsx! { "{label(&value)}" });
    let onchange = props.onchange;
    // `Select` fires `None` only from `clearable`, which is never set here.
    let pick = use_callback(move |value: Option<T>| {
        if let (Some(onchange), Some(value)) = (&onchange, value) {
            onchange.call(value);
        }
    });

    rsx! {
        Select::<T> {
            value: props.value,
            onchange: props.onchange.is_some().then_some(pick),
            name: props.name,
            validate: props.validate,
            options: props.options,
            option: option_label.is_some().then_some(option),
            selection: option_label.is_some().then_some(selection),
            placeholder: props.placeholder,
            label: props.label,
            description: props.description,
            helper: props.helper,
            status: props.status,
            size: props.size.copied_or(theme.native_select.size),
            radius: props.radius.copied_or(theme.native_select.radius),
            disabled: props.disabled,
            required: props.required,
            class: props.class,
            sx: props.sx,
            parts: recast_parts(props.parts),
            states: props.states,
            attributes: props.attributes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{repeats, runs};

    /// Todo 583: adjacent equal labels are one `<optgroup>`, a repeat later is another.
    #[test]
    fn groups_split_into_runs() {
        let group = |label: &str| Some(label.to_string());
        let groups = [None, group("A"), group("A"), group("B"), group("A")];
        assert_eq!(runs(&groups), [(0, 1), (1, 3), (3, 4), (4, 5)]);
        assert!(runs(&[]).is_empty());
    }

    /// Todo 20: a runtime set can post one value twice; the derive cannot.
    #[test]
    fn a_repeated_value_is_caught() {
        let posted = |values: &[&str]| values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
        assert!(!repeats(&posted(&["Free", "Pro"])));
        assert!(repeats(&posted(&["Free", "Pro", "Free"])));
        assert!(!repeats(&[]));
    }
}
