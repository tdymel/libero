use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, Options, Select, SelectOptionArgs,
        common::field_props,
        form::{field_control_sx, row_label, use_bound, use_field, use_field_frame},
        layout::use_box,
    },
    hooks::use_theme,
    platform::select_picker,
    sx::StaticSx,
    utils::warn,
};

/// The control keeps the UA's own chevron - drawing our own would mean
/// `appearance: none`, and with it the native picker's arrow on every
/// platform.
static NATIVE_SELECT_CONTROL_SX: StaticSx = StaticSx::new(|| field_control_sx().cursor("pointer"));

// No `readonly`: HTML has no read-only `<select>`, and every way to stop a
// native picker takes it out of the tab order or out of the post. `Select` is
// the read-only picker (todo 304).
field_props! {
    without(readonly);
    pub struct NativeSelectProps<T: Options> {
        /// Strictly controlled - pair it with `onchange`. `None` shows
        /// `placeholder` and selects nothing.
        #[props(default)]
        value: Option<T>,
        /// Called with the option the caller should select next. Never fires
        /// for the placeholder, which cannot be picked.
        #[props(default)]
        onchange: Option<EventHandler<T>>,
        /// What the select posts as. A path - `Order::FIELDS.size()` - also
        /// binds it to the surrounding `Form`'s value when it has no
        /// `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<Option<T>>,
        /// Rules over the selection, shown once the select loses focus or its
        /// form is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Option<T>>,
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

/// A styled native `<select>` over an enum, with a label, a description,
/// helper text and a validation message stacked around it. Controlled: it
/// renders `value` and asks for a new one through `onchange`.
///
/// The options are `T::options()` unless `options` narrows them - so a
/// misspelled option is a compile error, and `onchange` hands back the value
/// itself rather than a string the caller has to look up again.
///
/// The real `<select>`, not a listbox: it keeps the OS picker on phones, works
/// without wasm, and renders its selection correctly under SSR. Reach for it
/// when those matter; reach for `Select` when the rows have to be styled.
/// Under the `native` renderer a `<select>` opens no picker, so there it draws
/// `Select`'s listbox from the same props.
///
/// There is no `readonly`, unlike every other field: a native `<select>` has
/// no read-only state. `Select` is the read-only picker.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{NativeSelect, Options};
/// # fn app() -> Element {
/// # #[derive(Clone, Copy, PartialEq, Options)] enum Plan { Free, Pro }
/// rsx! { NativeSelect::<Plan> { value: Some(Plan::Free) } }
/// # }
/// ```
///
/// ```compile_fail,E0599
/// # use dioxus::prelude::*;
/// # use libero::components::{NativeSelect, Options};
/// # fn app() -> Element {
/// # #[derive(Clone, Copy, PartialEq, Options)] enum Plan { Free, Pro }
/// rsx! { NativeSelect::<Plan> { value: Some(Plan::Free), readonly: true } }
/// # }
/// ```
#[component]
pub fn NativeSelect<T: Options>(props: NativeSelectProps<T>) -> Element {
    // Fixed per build, so the hooks below always run in the same order.
    if !select_picker() {
        return listbox(props);
    }
    let theme = use_theme();

    let size = props.size.copied_or(theme.native_select.size);
    let radius = props.radius.copied_or(theme.native_select.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let current = bound.value().unwrap_or_else(|| props.value.clone());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("NativeSelect: without `onchange` the selection can never change.");
    }

    let values = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());
    if values.is_empty() {
        warn("NativeSelect: no options - a `T` without static `options()` needs `options`.");
    }
    let selected = current
        .as_ref()
        .and_then(|value| values.iter().position(|option| option == value));
    if current.is_some() && selected.is_none() && !values.is_empty() {
        warn("NativeSelect: `value` is not one of the options, so none is selected.");
    }

    let labels: Vec<String> = values
        .iter()
        .map(|value| match &props.option_label {
            Some(label) => label.call(value.clone()),
            None => value.label(),
        })
        .collect();

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&current))
        .bound(&bound)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
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

    // `selected` on each `<option>`, not `value` on the `<select>`: the
    // property is written to the option itself, so it does not depend on the
    // parent's children already existing - which is what made the old
    // `value` path miss on the creating render, and what left SSR with no
    // selection at all.
    let select = field
        .aria(control)
        .attr("name", bound.name().map(str::to_string))
        .attr("disabled", disabled)
        .attr("required", required)
        .attr("data-controlled", true)
        .event("onchange", move |event: FormEvent| pick.call(event.value()))
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
                for (index, (label, value)) in labels.iter().zip(&posted).enumerate() {
                    option {
                        key: "{index}",
                        value: "{value}",
                        selected: selected == Some(index),
                        "{label}"
                    }
                }
            },
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
    let options = props
        .options
        .clone()
        .unwrap_or_else(|| T::options().to_vec());

    rsx! {
        Select::<T> {
            value: props.value,
            onchange: props.onchange.is_some().then_some(pick),
            name: props.name,
            validate: props.validate,
            options,
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
            states: props.states,
            attributes: props.attributes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::repeats;

    /// Todo 20: a runtime set can post one value twice; the derive cannot.
    #[test]
    fn a_repeated_value_is_caught() {
        let posted = |values: &[&str]| values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
        assert!(!repeats(&posted(&["Free", "Pro"])));
        assert!(repeats(&posted(&["Free", "Pro", "Free"])));
        assert!(!repeats(&[]));
    }
}
