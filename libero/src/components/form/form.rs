use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, Part, base_props, parts_enum},
        feedback::Alert,
        form::{
            Binding, Source, Validators,
            handle::{Control, FormHandle, Summary},
            issues_of,
            use_field::labelled_focus_selector,
        },
        layout::{Box, use_box},
    },
    hooks::use_element,
    platform::{self, ElementApi},
    sx::{StaticSx, sx},
    theme::FormDefaults,
};

/// What a `Form` or `Fieldset` can hold as its value.
pub trait FormValue: Clone + PartialEq + Default + 'static {}

impl<T: Clone + PartialEq + Default + 'static> FormValue for T {}

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
        // The line itself is the link: readable on the tint, no underline.
        .selector(
            "& a",
            sx().color("inherit")
                .text_decoration("none")
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
        /// Fires on a submit nothing blocks. Without an `action` the native submit is cancelled.
        #[props(default)]
        onsubmit: Option<EventHandler<FormEvent>>,
        /// A heading over the error summary.
        #[props(default, into)]
        summary_title: Option<String>,
        /// Controls the form from outside, made with `use_form()`. Taken once.
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
    let mut scope = handle.scope;
    use_context_provider(|| scope);
    use_context_provider(|| handle);
    let key = use_hook(|| scope.key());
    use_drop(move || scope.withdraw(key));
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
    let issues = match (props.validate.is_empty(), props.value) {
        (false, Some(value)) => issues_of(&props.validate, &value.read(), ""),
        _ => Vec::new(),
    };
    scope.raise(key, issues);

    // Both owned by the handle, which may belong to a parent - see `FormHandle`.
    let focus_requests = handle.focus_requests;
    let form_element = handle.element;
    let summary_element = use_element();

    let posts = props
        .attributes
        .iter()
        .any(|attribute| attribute.name == "action");
    let onsubmit = props.onsubmit;
    let submit = use_callback(move |event: FormEvent| {
        scope.settle();
        if !handle.validate() {
            event.prevent_default();
            return;
        }
        if !posts {
            event.prevent_default();
        }
        if let Some(onsubmit) = onsubmit {
            onsubmit.call(event);
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
    use_effect(move || {
        if focus_requests() > 0 {
            let _ = summary_element.focus();
        }
    });

    // Where the renderer fires no `submit`, its triggers run the same path.
    let (submit_click, implicit_submit) =
        platform::submit_listeners(move || form_element.mounted(), submit).unzip();

    let items = summary.visible(&scope);
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
                                    // A group has no control with the id: its tab stop is named by the label.
                                    let target = format!(
                                        "{}, {}",
                                        crate::hooks::id_selector(id),
                                        labelled_focus_selector(&format!("{id}-label"))
                                    );
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
            Fragment { key: "{summary_key}", {summary_node.unwrap_or_else(VNode::empty)} }
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
        .event("onsubmit", move |event: FormEvent| submit.call(event))
        // A native reset button resets no state of ours, but ends a reveal
        // and restarts a textarea's count.
        .event("onreset", move |_: FormEvent| scope.count_reset())
        .event("onclick", submit_click)
        .event("onkeydown", implicit_submit)
        .render(HtmlTag::Form, props.attributes, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

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
