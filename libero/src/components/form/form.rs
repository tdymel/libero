use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        Alert, HtmlTag, Input,
        common::base_props,
        form::{
            Binding, Source, Validators,
            handle::{Control, FormHandle, Summary},
            issues_of,
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

base_props! {
    extends(form);
    pub struct FormProps<V: FormValue> {
        /// The whole form's value, which `validate` checks. A field inside
        /// whose `name` is a path into it - `Signup::FIELDS.email()` - reads
        /// and writes its place in it, unless it has a handler of its own.
        /// A `Store`, so typing into one field re-renders that field only.
        /// Handing over a different store - another record in an edit view -
        /// mounts the fields again on it, so nothing is left reading the old
        /// one.
        #[props(default)]
        value: Option<Store<V>>,
        /// Composite rules over `value`. Name the fields a rule concerns with
        /// `.on(..)` and its status shows on each of them.
        #[props(default, into)]
        validate: Validators<V>,
        /// Fires on a submit nothing blocks. Without an `action` the browser's
        /// own submit is cancelled.
        #[props(default)]
        onsubmit: Option<EventHandler<FormEvent>>,
        /// A heading over the error summary.
        #[props(default, into)]
        summary_title: Option<String>,
        /// Controls the form from outside, made with `use_form()`. Taken once.
        /// Without it the form makes its own, which `use_form_context()`
        /// reaches from inside.
        #[props(default)]
        form: Option<FormHandle>,
        children: Element,
    }
}

/// A `<form>` that validates on submit. Every field inside it shows its
/// status, the composite rules run, and when anything is an error the submit
/// is cancelled and an error summary is shown and focused - so a screen reader
/// hears every problem at once.
#[component]
pub fn Form<V: FormValue>(props: FormProps<V>) -> Element {
    let handle = use_hook(|| props.form.unwrap_or_else(FormHandle::new));
    let mut scope = handle.scope;
    use_context_provider(|| scope);
    use_context_provider(|| handle);
    let key = use_hook(|| scope.key());
    use_drop(move || scope.withdraw(key));
    // A field resolves its binding when it mounts, so the binding is keyed on
    // the store: a caller that hands over a *different* `Store` - another
    // record in an edit view, or `None` then `Some` - gets the fields mounted
    // again on it, because their binding, their form scope and their touched
    // state all belong to the store they resolved against. `Store`'s equality
    // is its identity and not its contents, so a write is not a swap.
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
        // The fields about to mount read it out of the context, so it has to
        // be the new one before they do.
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
            // Through `store`, so a reset clears the value the fields are on
            // and not the one the form mounted with.
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
                "data-slot": "summary",
                // Over `Alert`'s own default, which is the same role - stated so
                // the focus target below does not depend on that default.
                role: "alert",
                tabindex: "-1",
                onmounted: summary_element.mount(),
                Box { component: HtmlTag::Ul, framework_sx: &SUMMARY_LIST_SX,
                    for item in items.iter() {
                        li {
                            match &item.target {
                                Some(id) => {
                                    let target = crate::hooks::id_selector(id);
                                    rsx! {
                                        a {
                                            href: "#{id}",
                                            // Focus, not the browser's jump: a `#fragment` would
                                            // go through the router, and focus scrolls the
                                            // field into view anyway.
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

    // Always two children: a summary slot that appears would otherwise shift
    // the fields, and dioxus would remount them - new ids, touched state lost.
    // The fields are keyed on the record instead, which is the one time they
    // *must* be remounted.
    // Both keyed, or dioxus refuses to diff the pair: a key on one sibling
    // makes the whole list keyed. The summary's never changes.
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
