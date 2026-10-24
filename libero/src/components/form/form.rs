use std::{cell::Cell, rc::Rc};

use dioxus::{dioxus_core::AttributeValue, prelude::*};

use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::{FOCUSABLE_SELECTOR, HtmlTag, Input, Part, allowed_href, base_props, parts_enum},
        feedback::Alert,
        form::{
            Binding, FormValue, Source, Submit, Validators,
            handle::{Control, FormHandle, Summary},
            issues_of,
            submit::Outcome,
            use_field::labelled_focus_selector,
        },
        layout::{Box, use_box},
    },
    hooks::{use_element, use_localization},
    platform::{self, ElementApi},
    sx::{StaticSx, sx},
    theme::FormDefaults,
    utils::warn,
};

/// The root binding a `Store` gives the fields, with its type erased.
fn root_binding<V: FormValue>(value: Option<Store<V>>) -> Binding {
    Binding::root(value.map(|value| Rc::new(value) as Rc<dyn Source>))
}

static FORM_SX: StaticSx = StaticSx::new(|| {
    FormDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
});

// The summary's list is `Form`'s content, not `Alert`'s chrome.
static SUMMARY_LIST_SX: StaticSx = StaticSx::new(|| {
    sx().margin("0")
        .padding_left("20px")
        .rtl(sx().padding_left("0").padding_right("20px"))
        // The line itself is the link: readable on the tint, and underlined so it reads as one
        // without colour, in forced colours too (GOV.UK's summary, todo 1272).
        .selector(
            "& a",
            sx().color("inherit")
                .text_decoration("underline")
                .cursor("pointer"),
        )
});

parts_enum! {
    /// [`Form`]'s inner parts, for its `parts` prop. The fields are the caller's
    /// and take their own `parts`.
    pub enum FormPart {
        /// The error summary, an `Alert`, shown after a blocked submit.
        Summary = "summary" => "& > [data-slot='summary']",
        /// The summary's heading, with `summary_title`.
        SummaryTitle = "title" => "& > [data-slot='summary'] > [data-slot='body'] > [data-slot='title']",
        /// The summary's list of errors.
        SummaryList = "list" => "& > [data-slot='summary'] > [data-slot='body'] > [data-slot='message'] > [data-slot='list']",
    }
}

/// How a [`Form`] points at its errors after a blocked submit or
/// [`set_errors`](FormHandle::set_errors).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Form, SummaryMode};
/// # fn app() -> Element {
/// rsx! {
///     // The fields show their errors; focus goes to the first one.
///     Form::<()> { summary: SummaryMode::Hidden }
/// }
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SummaryMode {
    /// A summary above the fields lists every error, linked to its field, and takes focus.
    #[default]
    Visible,
    /// No list: focus goes to the first invalid field and a screen reader hears how many errors
    /// there are. Errors on no field, such as [`FormErrors::form`](crate::components::FormErrors::form),
    /// still show above the fields.
    Hidden,
}

base_props! {
    extends(form);
    parts(FormPart);
    pub struct FormProps<V: FormValue> {
        /// The whole form's value. Fields named by a path into it read and write
        /// their place. A different store remounts the fields.
        #[props(default)]
        value: Option<Store<V>>,
        /// Composite rules over `value`; `.on(..)` shows one on the named fields.
        #[props(default, into)]
        validate: Validators<V>,
        /// Fires on a submit nothing blocks: a closure over the event, or [`Submit::parse`],
        /// which builds a command from `value` first. Without an `action` the native submit is
        /// cancelled.
        #[props(default, into)]
        onsubmit: Submit<V>,
        /// A heading over the error summary.
        #[props(default, into)]
        summary_title: Option<String>,
        /// Whether a blocked submit shows and focuses the error summary, or focuses the first
        /// invalid field.
        #[props(default)]
        summary: SummaryMode,
        /// Draws a `Loader` in a framed field while an async rule checks its value, once the
        /// field shows that (after a blur or a submit). Off by default; `aria-busy` is set either way.
        #[props(default)]
        pending_loader: bool,
        /// Controls the form from outside, made with `use_form()`. Taken once: a swapped
        /// handle is ignored with a warning, so `key` the `Form` to use another.
        #[props(default)]
        form: Option<FormHandle>,
        children: Element,
    }
}

/// A `<form>` that validates on submit and focuses an error summary.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Fields, Form, Rule, TextField, not_empty};
/// #[derive(Clone, PartialEq, Default, Fields)]
/// struct Signup {
///     email: String,
/// }
///
/// # fn app() -> Element {
/// let signup = use_store(Signup::default);
/// rsx! {
///     Form {
///         value: signup,
///         onsubmit: move |_| {},
///         TextField {
///             label: "Email",
///             name: Signup::FIELDS.email(),
///             validate: [not_empty::<String>.error("Enter your email.")],
///         }
///         button { r#type: "submit", "Sign up" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/form>
#[component]
pub fn Form<V: FormValue>(props: FormProps<V>) -> Element {
    let handle = use_hook(|| props.form.unwrap_or_else(FormHandle::new));
    if props.form.is_some_and(|form| form != handle) {
        warn("Form: a `form` handle swapped after mount is ignored; key the Form to use another.");
    }
    let mut scope = handle.scope;
    use_context_provider(|| scope);
    use_context_provider(|| handle);
    let key = use_hook(|| scope.key());
    use_drop(move || scope.withdraw(key));
    scope.set_pending_loader(props.pending_loader);
    // A field binds on mount, so a different `Store` remounts the fields. `Store`
    // equality is identity, so a write is not a swap.
    let mut store = use_hook(|| CopyValue::new(props.value));
    let mut record = use_hook(|| CopyValue::new(0u32));
    let swapped = *store.peek() != props.value;
    if swapped {
        store.set(props.value);
        let next = *record.peek() + 1;
        record.set(next);
    }
    let mut binding = use_hook(|| CopyValue::new(root_binding(props.value)));
    if swapped {
        binding.set(root_binding(props.value));
    }
    let current = binding.peek().clone();
    use_context_provider(|| current.clone());
    if swapped {
        // The fields about to mount read it from the context.
        provide_context(current);
    }
    // Read only with rules to run, so a form without any does not re-render on
    // every keystroke.
    let warned_blank = use_hook(|| Rc::new(Cell::new(false)));
    let issues = match (props.validate.is_empty(), props.value) {
        (false, Some(value)) => {
            issues_of(&props.validate, &value.read(), "", &warned_blank, &scope)
        }
        _ => Vec::new(),
    };
    scope.raise(key, issues);

    // Both owned by the handle, which may belong to a parent - see `FormHandle`.
    let focus_requests = handle.focus_requests;
    let form_element = handle.element;
    let summary_element = use_element();

    let attributes = safe_action(props.attributes);
    let posts = attributes
        .iter()
        .any(|attribute| attribute.name == "action");
    let onsubmit = props.onsubmit;
    // Set by a valid submit to the reset count then: a server status after it rebuilds the summary.
    let mut awaiting = use_signal(|| None::<u32>);
    let mut submitting = handle.submitting;
    let submit = use_callback(move |event: FormEvent| {
        // One async handler at a time: a second submit meanwhile does nothing.
        if *submitting.peek() {
            event.prevent_default();
            return;
        }
        // An async rule still asking: shown as pending, its answer awaited, then submitted afresh.
        if scope.checking() {
            event.prevent_default();
            scope.submit();
            submitting.set(true);
            let answered = scope.answered();
            spawn(async move {
                answered.await;
                submitting.set(false);
                let _ = handle.submit();
            });
            return;
        }
        scope.settle();
        // Errors set from code answered the last submit; this one is judged afresh.
        scope.clear_errors();
        if !handle.validate() {
            awaiting.set(None);
            event.prevent_default();
            return;
        }
        // Before `onsubmit`, so a status it sets at once counts too.
        awaiting.set(Some(scope.resets()));
        if !posts {
            event.prevent_default();
        }
        if !onsubmit.is_some() {
            return;
        }
        let value = (*store.peek())
            .map(|value| value.peek().clone())
            .unwrap_or_default();
        match onsubmit.run(event.clone(), value) {
            Ok(()) => {}
            Err(Outcome::Refused(errors)) => {
                // `set_errors` shows the summary itself.
                awaiting.set(None);
                event.prevent_default();
                handle.set_errors(errors);
            }
            Err(Outcome::Running(task)) => {
                submitting.set(true);
                // In the form's scope, so the task ends with the form.
                spawn(async move {
                    let result = task.await;
                    submitting.set(false);
                    if let Err(errors) = result {
                        awaiting.set(None);
                        handle.set_errors(errors);
                    }
                });
            }
        }
    });

    let summary = use_hook(|| {
        let summary = Summary::new(focus_requests);
        handle.attach(Control {
            // Through `store`: a reset clears the current value, not the first.
            reset_value: Rc::new(move || {
                if let Some(mut value) = *store.peek() {
                    value.set(V::default());
                }
            }),
            summary: summary.clone(),
            submit,
        });
        summary
    });
    use_drop(move || handle.detach());
    let mut mode = use_hook(|| CopyValue::new(props.summary));
    if *mode.peek() != props.summary {
        mode.set(props.summary);
    }
    use_effect({
        let summary = summary.clone();
        move || {
            if focus_requests() == 0 {
                return;
            }
            // Hidden, the first field showing its error takes focus; the alert only without one.
            let field = (*mode.peek() == SummaryMode::Hidden)
                .then(|| summary.first_inline())
                .flatten();
            let _ = match field {
                Some(id) => form_element
                    .query_selector(&field_target(&id))
                    .and_then(|field| field.focus()),
                None => summary_element.focus(),
            };
        }
    });
    // A field's explicit error after a valid submit shows the summary once, as a blocked
    // submit would; a rule error from editing does not (2573). A reset gives up waiting.
    use_effect({
        let summary = summary.clone();
        move || {
            let Some(at) = awaiting() else {
                return;
            };
            if scope.resets() != at {
                awaiting.set(None);
            } else if scope.explicit_error_tracked() {
                awaiting.set(None);
                summary.set(scope.summary());
            }
        }
    });

    // Where the renderer fires no `submit`, its triggers run the same path, after the
    // event's writes rendered: the web submits in a later task (a date Enter commits, 2329).
    let mut requested = use_signal(|| false);
    use_effect(move || {
        if requested() {
            requested.set(false);
            platform::when_free(move || {
                submit.call(platform::submit_event(form_element.mounted()));
            });
        }
    });
    let request = use_callback(move |()| requested.set(true));
    let (submit_click, implicit_submit) =
        platform::submit_listeners(move || form_element.mounted(), request).unzip();

    let mut items = summary.visible(&scope);
    let hidden = props.summary == SummaryMode::Hidden;
    // Counted before the lines the fields show leave the list.
    let count = items.len();
    if hidden {
        items.retain(|item| !item.inline);
    }
    let words = use_localization();
    let announcer = hidden.then(|| {
        rsx! {
            VisuallyHidden { role: "status",
                if count > 0 { {(words.form.error_count)(count)} }
            }
        }
    });
    let summary_node = (!items.is_empty()).then(|| {
        rsx! {
            Alert {
                color: "error",
                title: props.summary_title.clone(),
                "data-slot": FormPart::Summary.slot(),
                // Stated so the focus target does not depend on `Alert`'s default.
                role: "alert",
                tabindex: "-1",
                onmounted: summary_element.mount(),
                Box {
                    component: HtmlTag::Ul,
                    framework_sx: &SUMMARY_LIST_SX,
                    "data-slot": FormPart::SummaryList.slot(),
                    for item in items.iter() {
                        li {
                            match &item.target {
                                Some(id) => {
                                    let target = field_target(id);
                                    rsx! {
                                        a {
                                            href: "#{id}",
                                            // Focus, not a `#fragment` jump through the router.
                                            onclick: move |event: MouseEvent| {
                                                event.prevent_default();
                                                let _ = form_element
                                                    .query_selector(&target)
                                                    .and_then(|field| field.focus());
                                            },
                                            "{item.message}"
                                        }
                                    }
                                }
                                None => rsx! { "{item.message}" },
                            }
                        }
                    }
                }
            }
        }
    });

    // Always two keyed children, so a summary appearing never remounts the
    // fields; they remount only on a record swap.
    let summary_key = "summary";
    let children = vec![
        rsx! {
            Fragment { key: "{summary_key}", {announcer}, {summary_node} }
        },
        rsx! {
            Fragment { key: "{record.peek()}", {props.children} }
        },
    ];

    use_box()
        .framework_sx(&FORM_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .prepare()
        .element(&form_element)
        .attr("novalidate", true)
        .attr("aria-busy", if submitting() { "true" } else { "false" })
        .event("onsubmit", move |event: FormEvent| submit.call(event))
        // A native reset button also takes the form back to pristine: nothing
        // touched, no summary, as `FormHandle::reset`.
        .event("onreset", {
            let summary = summary.clone();
            move |_: FormEvent| {
                summary.set(Vec::new());
                scope.reset();
            }
        })
        .event("onclick", submit_click)
        .event("onkeydown", implicit_submit)
        .render(HtmlTag::Form, attributes, children)
}

/// The tab stop of the field with `id`, for a summary line or the focus a hidden summary moves.
/// A group has no control with the id: its tab stop is named by the label. A `Fieldset`'s goes
/// to its first tab stop.
fn field_target(id: &str) -> String {
    let at = crate::hooks::id_selector(id);
    format!(
        "{at}:not(fieldset), {at} {FOCUSABLE_SELECTOR}:not([type=\"hidden\"]), {}",
        labelled_focus_selector(&format!("{id}-label"))
    )
}

/// `attributes` without a text `action` whose scheme would run script on submit.
fn safe_action(mut attributes: Vec<Attribute>) -> Vec<Attribute> {
    attributes.retain(|attribute| match &attribute.value {
        AttributeValue::Text(url) if attribute.name == "action" => allowed_href(url, "Form action"),
        _ => true,
    });
    attributes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    fn action(url: &str) -> Vec<Attribute> {
        let text = AttributeValue::Text(url.to_string());
        vec![Attribute::new("action", text, None, false)]
    }

    #[test]
    fn a_script_action_is_dropped_and_warned() {
        for url in [
            "javascript:alert(1)",
            " Java\tScript:x",
            "data:text/html,x",
            "vbscript:x",
        ] {
            assert!(safe_action(action(url)).is_empty(), "{url:?}");
        }
        assert!(
            crate::utils::take_warnings()
                .iter()
                .any(|w| w.starts_with("Form action:"))
        );
        for url in ["/save", "https://example.com/post", ""] {
            assert_eq!(safe_action(action(url)).len(), 1, "{url:?}");
        }
    }

    #[test]
    fn a_form_with_a_script_action_renders_none() {
        let html = dioxus_ssr::render_element(rsx! {
            crate::LiberoProvider {
                Form::<()> { action: "javascript:alert(1)", "x" }
            }
        });
        assert!(!html.contains("javascript"), "{html}");
    }

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<FormPart>(),
            [
                ("summary", "& > [data-slot='summary']"),
                (
                    "title",
                    "& > [data-slot='summary'] > [data-slot='body'] > [data-slot='title']"
                ),
                (
                    "list",
                    "& > [data-slot='summary'] > [data-slot='body'] > [data-slot='message'] > [data-slot='list']"
                ),
            ]
        );
    }
}
