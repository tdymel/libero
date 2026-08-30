use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        Caption, HtmlTag, Input,
        common::base_props,
        form::{
            Binding, Disabled, FieldName, FieldStatus, FormScope, FormValue, Source, Validators,
            issues_of, worst,
        },
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
        /// The group's own value, for a fieldset outside a `Form`. Inside one
        /// the value is the form's, at `path`.
        #[props(default)]
        value: Option<Store<V>>,
        /// Composite rules over `value`. A rule naming fields with `.on(..)`
        /// shows on each of them; one naming none shows under the fields.
        #[props(default, into)]
        validate: Validators<V>,
        /// Where the group sits in the form's value - `Order::FIELDS.address()`.
        /// The names of the fields inside and the paths of `validate` are
        /// relative to it.
        #[props(default, into)]
        path: FieldName<V>,
        /// The group's caption, rendered as its `<legend>`. Named `label` like
        /// every field's caption.
        #[props(default, into)]
        label: Caption,
        #[props(default, into)]
        description: Caption,
        #[props(default, into)]
        helper: Caption,
        /// The group's own status, under the fields. A bare `&str` is an error.
        #[props(default, into)]
        status: Input<FieldStatus>,
        /// Disables every field inside, nested fieldsets included. The fields
        /// draw it themselves, so non-native controls follow too.
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

    let binding = use_hook(|| {
        let parent = try_consume_context::<Binding>().unwrap_or_default();
        match props.value {
            Some(value) => parent.rebased(props.path.as_str(), Rc::new(value) as Rc<dyn Source>),
            None => parent.narrow(props.path.as_str(), props.path.steps()),
        }
    });
    use_context_provider(|| binding.clone());

    // `<fieldset disabled>` only reaches native controls, so the group also
    // tells its fields - which draw their own disabled look - and nested groups.
    let parent_disabled = use_hook(try_consume_context::<Disabled>);
    let mut own_disabled = use_hook(|| Disabled(Signal::new(false)));
    use_context_provider(|| own_disabled);
    let disabled =
        props.disabled.unwrap_or(false) || parent_disabled.is_some_and(|Disabled(parent)| parent());
    if *own_disabled.0.peek() != disabled {
        own_disabled.0.set(disabled);
    }

    let prefix = binding.prefix();
    let issues = match props.validate.is_empty() {
        true => Vec::new(),
        false => binding
            .with::<V, _>(&[], |value| issues_of(&props.validate, value, prefix))
            .unwrap_or_else(|| issues_of(&props.validate, &V::default(), prefix)),
    };
    scope.raise(key, issues);

    let revealed = scope.submitted() || scope.touched_under(prefix);
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

    let legend = (!props.label.is_none()).then(|| {
        let content = match &props.label {
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
        .attr("disabled", disabled)
        .attr(
            "aria-describedby",
            (!describedby.is_empty()).then_some(describedby),
        )
        .render(HtmlTag::Fieldset, props.attributes, children)
}
