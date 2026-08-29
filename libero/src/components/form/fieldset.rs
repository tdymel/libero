use dioxus::prelude::*;

use crate::{
    components::{
        Caption, HtmlTag, Input,
        common::base_props,
        form::{FieldStatus, FormScope, FormValue, Validators, issues_of, worst},
        layout::use_box,
    },
    sx::{StaticSx, sx},
    theme::{FIELD_CAPTION_FONT_SIZE, FIELD_LABEL_FONT_SIZE, FieldsetDefaults, Size},
};

static FIELDSET_SX: StaticSx = StaticSx::new(|| {
    FieldsetDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .margin("0")
        .padding("0")
        .border("0")
        .min_width("0")
        .selector(
            "& > legend",
            sx().padding("0")
                .margin_bottom("4px")
                .font_weight("600")
                .font_size(FIELD_LABEL_FONT_SIZE.value(Size::Md)),
        )
        .selector(
            "& > [data-slot]",
            sx().color("grey.7")
                .font_size(FIELD_CAPTION_FONT_SIZE.value(Size::Md)),
        )
        .when(
            "warning",
            sx().selector("& > [data-slot='status']", sx().color("warning.7")),
        )
        .when(
            "error",
            sx().selector("& > [data-slot='status']", sx().color("error.7")),
        )
});

base_props! {
    extends(fieldset);
    pub struct FieldsetProps<V: FormValue> {
        /// The group's value, which `validate` checks.
        #[props(default)]
        value: V,
        /// Composite rules over `value`. A rule naming fields with `.on(..)`
        /// shows on each of them; one naming none shows under the legend.
        #[props(default, into)]
        validate: Validators<V>,
        /// Where `value` sits in the form - `Signup::FIELDS.address()`. The
        /// paths of `validate` are relative to it.
        #[props(default, into)]
        path: String,
        /// The group's caption, a `<legend>`.
        #[props(default, into)]
        legend: Caption,
        #[props(default, into)]
        description: Caption,
        #[props(default, into)]
        helper: Caption,
        /// The group's own status, under the fields. A bare `&str` is an error.
        #[props(default, into)]
        status: Input<FieldStatus>,
        /// Disables every native control inside, the way `<fieldset disabled>`
        /// does.
        #[props(default)]
        disabled: Option<bool>,
        children: Element,
    }
}

/// Several fields that form one value - an address, a date range - under one
/// `<legend>`, with a status of their own. Composite rules run over the
/// group's `value` and land on the fields they name.
#[component]
pub fn Fieldset<V: FormValue>(props: FieldsetProps<V>) -> Element {
    // Inside a `Form` the group joins its scope; alone, it opens its own so its
    // rules still reach its fields.
    let mut scope = use_hook(|| try_consume_context::<FormScope>().unwrap_or_else(FormScope::new));
    use_context_provider(|| scope);
    let key = use_hook(|| scope.key());
    use_drop(move || scope.withdraw(key));

    let prefix = props.path.clone();
    scope.raise(key, issues_of(&props.validate, &props.value, &prefix));

    let revealed = scope.submitted() || scope.touched_under(&prefix);
    let unnamed = match revealed {
        true => scope.unnamed_issue(key),
        false => FieldStatus::Valid,
    };
    let status = worst(props.status.as_ref().cloned().unwrap_or_default(), unnamed);

    let id = crate::hooks::use_root_id(&props.attributes)();
    let slot = |slot: &'static str, caption: &Caption| {
        (!caption.is_none()).then(|| {
            let named = caption.text().is_some().then(|| format!("{id}-{slot}"));
            let content = match caption {
                Caption::Text(text) => rsx! { "{text}" },
                Caption::Node(node) => node.clone(),
                Caption::None => rsx! {},
            };
            rsx! { span { "data-slot": slot, id: named, {content} } }
        })
    };
    let describedby = [
        ("description", props.description.text().is_some()),
        ("helper", props.helper.text().is_some()),
        ("status", status.message().is_some()),
    ]
    .iter()
    .filter(|(_, present)| *present)
    .map(|(slot, _)| format!("{id}-{slot}"))
    .collect::<Vec<_>>()
    .join(" ");

    let legend = (!props.legend.is_none()).then(|| {
        let content = match &props.legend {
            Caption::Text(text) => rsx! { "{text}" },
            Caption::Node(node) => node.clone(),
            Caption::None => rsx! {},
        };
        rsx! { legend { {content} } }
    });
    let status_node = status.message().map(|message| {
        rsx! { span { "data-slot": "status", id: "{id}-status", "{message}" } }
    });

    let mut children = Vec::with_capacity(5);
    children.extend(legend);
    children.extend(slot("description", &props.description));
    children.push(props.children);
    children.extend(slot("helper", &props.helper));
    children.extend(status_node);

    let mut states = props.states.as_ref().cloned().unwrap_or_default();
    if let Some(state) = status.state() {
        states = states.active(state);
    }
    let states: Input<_> = states.into();

    use_box()
        .framework_sx(&FIELDSET_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .attr("id", id.clone())
        .attr("disabled", props.disabled.unwrap_or(false))
        .attr(
            "aria-describedby",
            (!describedby.is_empty()).then_some(describedby),
        )
        .render(HtmlTag::Fieldset, props.attributes, children)
}
