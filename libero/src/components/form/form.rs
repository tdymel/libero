use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input,
        common::base_props,
        form::{Binding, FormScope, Source, SummaryItem, Validators, issues_of},
        layout::use_box,
    },
    hooks::use_element,
    platform::ElementApi,
    sx::{StaticSx, sx},
    theme::FormDefaults,
};

/// What a `Form` or `Fieldset` can hold as its value.
pub trait FormValue: Clone + PartialEq + Default + 'static {}

impl<T: Clone + PartialEq + Default + 'static> FormValue for T {}

static FORM_SX: StaticSx = StaticSx::new(|| {
    FormDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .selector(
            "& > [data-slot='summary']",
            sx().border("1px solid")
                .border_color("error.6")
                .border_radius("sm")
                .background("error.1")
                .color("error.8")
                .padding("12px 16px")
                .selector("& ul", sx().margin("0").padding_left("20px"))
                .selector("& a", sx().color("error.8"))
                .selector("& > p", sx().margin("0 0 4px 0").font_weight("600")),
        )
});

base_props! {
    extends(form);
    pub struct FormProps<V: FormValue> {
        /// The whole form's value, which `validate` checks. A field inside
        /// whose `name` is a path into it - `Signup::FIELDS.email()` - reads
        /// and writes its place in it, unless it has a handler of its own.
        #[props(default)]
        value: Option<Signal<V>>,
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
        children: Element,
    }
}

/// A `<form>` that validates on submit. Every field inside it shows its
/// status, the composite rules run, and when anything is an error the submit
/// is cancelled and an error summary is shown and focused - so a screen reader
/// hears every problem at once.
#[component]
pub fn Form<V: FormValue>(props: FormProps<V>) -> Element {
    let mut scope = use_hook(FormScope::new);
    use_context_provider(|| scope);
    let key = use_hook(|| scope.key());
    use_drop(move || scope.withdraw(key));
    // Taken once: the fields resolve their binding when they mount.
    let binding =
        use_hook(|| Binding::root(props.value.map(|value| Rc::new(value) as Rc<dyn Source>)));
    use_context_provider(|| binding);
    // Read only with rules to run, so a form without any does not re-render on
    // every keystroke.
    let issues = match (props.validate.is_empty(), props.value) {
        (false, Some(value)) => issues_of(&props.validate, &value.read(), ""),
        _ => Vec::new(),
    };
    scope.raise(key, issues);

    let mut summary = use_signal(Vec::<SummaryItem>::new);
    let mut focus_requests = use_signal(|| 0_u32);
    let summary_element = use_element();
    use_effect(move || {
        if focus_requests() > 0 {
            let _ = summary_element.focus();
        }
    });

    let posts = props
        .attributes
        .iter()
        .any(|attribute| attribute.name == "action");
    let onsubmit = props.onsubmit;
    let handler = move |event: FormEvent| {
        scope.submit();
        scope.warn_unknown_paths();
        if scope.has_errors() {
            event.prevent_default();
            summary.set(scope.summary());
            focus_requests += 1;
            return;
        }
        summary.set(Vec::new());
        if !posts {
            event.prevent_default();
        }
        if let Some(onsubmit) = onsubmit {
            onsubmit.call(event);
        }
    };

    let items = summary.read();
    let summary_node = (!items.is_empty()).then(|| {
        rsx! {
            div {
                "data-slot": "summary",
                role: "alert",
                tabindex: "-1",
                onmounted: summary_element.mount(),
                if let Some(title) = &props.summary_title {
                    p { "{title}" }
                }
                ul {
                    for item in items.iter() {
                        li {
                            match &item.target {
                                Some(id) => rsx! { a { href: "#{id}", "{item.message}" } },
                                None => rsx! { "{item.message}" },
                            }
                        }
                    }
                }
            }
        }
    });

    let mut children = Vec::with_capacity(2);
    children.extend(summary_node);
    children.push(props.children);

    use_box()
        .framework_sx(&FORM_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .attr("novalidate", true)
        .event("onsubmit", handler)
        .render(HtmlTag::Form, props.attributes, children)
}
