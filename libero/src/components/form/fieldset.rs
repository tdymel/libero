use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        Caption, HtmlTag, Input,
        common::base_props,
        form::{
            Binding, Disabled, FieldName, FieldStatus, FormScope, FormValue, Source, Validators,
            issues_of,
            use_field::{caption_content, join_ids, slot_node, status_node},
            worst,
        },
        layout::use_box,
    },
    sx::{StaticSx, sx},
    theme::{FIELD_CAPTION_FONT_SIZE, FIELD_LABEL_FONT_SIZE, FieldsetDefaults, Size},
    utils::warn,
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
        /// With no value - none of its own, and no bound `Form` above - the
        /// rules do not run.
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

    // With no value there is nothing to judge: a rule run over `V::default()`
    // would report on a value nobody entered.
    let inert = !props.validate.is_empty() && !binding.is_bound();
    use_hook(|| {
        if inert {
            warn(
                "Fieldset: `validate` without a `value` never runs. Give the fieldset a `value`, \
                 or put it in a `Form` with one and name its `path` with `FIELDS`.",
            );
        }
    });

    let prefix = binding.prefix();
    let issues = match props.validate.is_empty() {
        true => Vec::new(),
        false => binding
            .with::<V, _>(&[], |value| issues_of(&props.validate, value, prefix))
            .unwrap_or_default(),
    };
    scope.raise(key, issues);

    let revealed = scope.submitted() || scope.touched_under(prefix);
    let unnamed = match revealed {
        true => scope.unnamed_issue(key),
        false => FieldStatus::Valid,
    };
    let status = worst(props.status.as_ref().cloned().unwrap_or_default(), unnamed);

    let id = crate::hooks::use_root_id(&props.attributes)();
    // The same chrome a field draws around its control, from the same helpers.
    let describedby = join_ids(
        &id,
        [
            ("description", props.description.text().is_some()),
            ("helper", props.helper.text().is_some()),
            ("status", status.message().is_some()),
        ],
    );
    let legend = (!props.label.is_none()).then(|| {
        let content = caption_content(&props.label);
        rsx! { legend { {content} } }
    });

    let mut children = Vec::with_capacity(5);
    children.extend(legend);
    children.extend(slot_node("description", &id, &props.description));
    children.push(props.children);
    children.extend(slot_node("helper", &id, &props.helper));
    children.extend(status_node(&id, Some(&status)));

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
        .attr("aria-describedby", describedby)
        .render(HtmlTag::Fieldset, props.attributes, children)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use dioxus::prelude::*;

    use crate::{
        LiberoProvider,
        components::{Fieldset, Rule},
        utils::take_warnings,
    };

    thread_local! {
        /// What each rule run was handed.
        static JUDGED: Cell<Option<u32>> = const { Cell::new(None) };
    }

    fn judged_and_warnings(app: fn() -> Element) -> (Option<u32>, Vec<String>) {
        JUDGED.set(None);
        take_warnings();
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let warnings = take_warnings()
            .into_iter()
            .filter(|warning| warning.starts_with("Fieldset:"))
            .collect();
        (JUDGED.take(), warnings)
    }

    fn rule(value: &u32) -> bool {
        JUDGED.set(Some(*value));
        false
    }

    /// Todo 303: with no value, the rules used to judge `V::default()` - a
    /// value nobody entered.
    #[test]
    fn a_fieldset_without_a_value_runs_no_rule_and_says_so() {
        let (judged, warnings) = judged_and_warnings(|| {
            rsx! { LiberoProvider { Fieldset::<u32> { validate: rule.error("Never"), "" } } }
        });
        assert_eq!(judged, None);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
    }

    #[test]
    fn a_fieldset_with_a_value_runs_its_rules_on_it() {
        let (judged, warnings) = judged_and_warnings(|| {
            let value = use_store(|| 7_u32);
            rsx! { LiberoProvider { Fieldset { value, validate: rule.error("Never"), "" } } }
        });
        assert_eq!(judged, Some(7));
        assert!(warnings.is_empty(), "{warnings:?}");
    }
}
