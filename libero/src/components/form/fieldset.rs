use std::{cell::Cell, rc::Rc};

use dioxus::{core::current_scope_id, prelude::*};

use crate::{
    components::{
        common::{HtmlTag, Input, Part, base_props, names_itself, parts_enum, use_name_warning},
        form::{
            Binding, Caption, Disabled, FieldEntry, FieldName, FieldStatus, FormScope, FormValue,
            Required, Source, Validators, issues_of,
            use_field::{caption_content, focus_left, join_ids, slot_node, status_node},
            worst,
        },
        layout::use_box,
    },
    hooks::{use_element, use_form_owner, use_silent_focus_out_of},
    platform::lifts_legends,
    sx::{StaticSx, sx},
    theme::{FIELD_CAPTION_FONT_SIZE, FIELD_LABEL_FONT_SIZE, FIELDSET_GAP, FieldsetDefaults, Size},
    utils::warn,
};

static FIELDSET_SX: StaticSx = StaticSx::new(|| {
    let legend_gap = match lifts_legends() {
        true => "4px".to_string(),
        // Blitz's legend is a flex item: take the gap back out below it.
        false => format!("calc(4px - {})", FIELDSET_GAP.value()),
    };
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
                .margin_bottom(legend_gap)
                .font_weight("600")
                .font_size(FIELD_LABEL_FONT_SIZE.value(Size::Md)),
        )
        // `:where`: the specificity of a bare `[data-slot]`.
        .selector(
            "& > [data-slot]:where(:not([data-slot='legend']))",
            sx().color("muted.7")
                .font_size(FIELD_CAPTION_FONT_SIZE.value(Size::Md)),
        )
        // The severity icon sits in the text line, one em tall, as in a field's caption.
        .selector(
            "& > [data-slot='status'] > svg",
            sx().width("1em")
                .height("1em")
                .vertical_align("-0.15em")
                .with("margin-inline-end", "0.25em"),
        )
        // The fields inside dim themselves; the group's own text dims with them.
        .when(
            "disabled",
            sx().selector("& > legend", sx().color("muted.6")).selector(
                "& > [data-slot]:where(:not([data-slot='legend']))",
                sx().color("muted.6"),
            ),
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

parts_enum! {
    /// [`Fieldset`]'s inner parts, for its `parts` prop. Each is a direct
    /// child, so a nested `Fieldset` keeps its own.
    pub enum FieldsetPart {
        /// The `<legend>`, from `label`.
        Legend = "legend" => "& > [data-slot='legend']",
        /// The `description` caption, under the legend.
        Description = "description" => "& > [data-slot='description']",
        /// The `helper` caption, under the fields.
        Helper = "helper" => "& > [data-slot='helper']",
        /// The group's status, under the fields.
        Status = "status" => "& > [data-slot='status']",
    }
}

base_props! {
    extends(fieldset);
    parts(FieldsetPart);
    pub struct FieldsetProps<V: FormValue> {
        /// The group's own value, for a fieldset outside a `Form`. A different store,
        /// or a different `path`, remounts the fields.
        #[props(default)]
        value: Option<Store<V>>,
        /// Composite rules over `value`; `.on(..)` shows one on the named fields.
        #[props(default, into)]
        validate: Validators<V>,
        /// Where the group sits in the form's value; names inside are relative to it.
        /// Inside a `Form`, give one with `validate`: a pathless group reveals its rules on any touch.
        #[props(default, into)]
        path: FieldName<V>,
        /// The group's caption, rendered as its `<legend>`.
        #[props(default, into)]
        label: Caption,
        /// Under the label.
        #[props(default, into)]
        description: Caption,
        /// Under the fields.
        #[props(default, into)]
        helper: Caption,
        /// The group's own status, under the fields. A bare `&str` is an error.
        #[props(default, into)]
        status: Input<FieldStatus>,
        /// Disables every Libero control inside, nested fieldsets included. A raw `<input>`
        /// or `<button>` keeps its own `disabled`: the group renders no native one.
        #[props(default)]
        disabled: Option<bool>,
        /// Marks every field inside required, nested groups too; a field's own `required` wins,
        /// so `required: false` keeps one optional.
        #[props(default)]
        required: Option<bool>,
        children: Element,
    }
}

/// Several fields that form one value under one `<legend>`, with composite rules.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Fields, Fieldset, TextField};
/// #[derive(Clone, PartialEq, Default, Fields)]
/// struct Address {
///     street: String,
///     city: String,
/// }
///
/// # fn app() -> Element {
/// let address = use_store(Address::default);
/// rsx! {
///     Fieldset {
///         label: "Delivery address",
///         value: address,
///         TextField { label: "Street", name: Address::FIELDS.street() }
///         TextField { label: "City", name: Address::FIELDS.city() }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/fieldset>
#[component]
pub fn Fieldset<V: FormValue>(props: FieldsetProps<V>) -> Element {
    // Inside a `Form` the group joins its scope; alone, it opens its own.
    let (mut scope, shared) = use_hook(|| match try_consume_context::<FormScope>() {
        Some(scope) => (scope, true),
        None => (FormScope::new(), false),
    });
    use_context_provider(|| scope);
    let key = use_hook(|| scope.key());
    use_drop(move || {
        scope.withdraw(key);
        scope.unregister(key);
    });
    // Alone, its scope starts over on its raw `<form>`'s reset, so the fields inside hear it (4326).
    let element = use_element();
    let owner = use_form_owner(element, !shared);
    let mut heard = use_hook(|| CopyValue::new(None::<u32>));
    use_effect(move || {
        let now = owner();
        if heard
            .peek()
            .zip(now)
            .is_some_and(|(before, now)| before != now)
        {
            scope.reset();
        }
        heard.set(now);
    });

    // A field binds on mount, so another `value` or `path` remounts the fields, as in `Form`.
    let parent = use_hook(|| try_consume_context::<Binding>().unwrap_or_default());
    let bind = || match props.value {
        Some(value) => parent.rebased(props.path.as_str(), Rc::new(value) as Rc<dyn Source>),
        None => parent.narrow(props.path.as_str(), props.path.steps()),
    };
    let mut bound_to = use_hook(|| CopyValue::new((props.value, props.path.as_str().to_string())));
    let mut record = use_hook(|| CopyValue::new(0u32));
    let swapped = {
        let (value, path) = &*bound_to.peek();
        *value != props.value || path != props.path.as_str()
    };
    let mut binding = use_hook(|| CopyValue::new(bind()));
    // What a reset of the form or the raw `<form>` puts the group's own value back to (4347).
    let snapshot = |value: Option<Store<V>>| value.map(|value| value.peek().clone());
    let mut start = use_hook(|| CopyValue::new(snapshot(props.value)));
    if swapped {
        if bound_to.peek().0 != props.value {
            start.set(snapshot(props.value));
        }
        bound_to.set((props.value, props.path.as_str().to_string()));
        let next = *record.peek() + 1;
        record.set(next);
        binding.set(bind());
    }
    let mut resets_seen = use_hook(|| CopyValue::new(None::<u32>));
    use_effect(move || {
        let now = scope.resets();
        if resets_seen.peek().is_some_and(|before| before != now)
            && let Some(mut value) = bound_to.peek().0
        {
            value.set(start.peek().clone().unwrap_or_default());
        }
        resets_seen.set(Some(now));
    });
    let binding = binding.peek().clone();
    use_context_provider(|| binding.clone());
    if swapped {
        // The fields about to mount read it from the context.
        provide_context(binding.clone());
    }

    // No native `disabled`: it would drop a focused button's focus (todo 3873). Every
    // control inside reads this context and disables itself, nested groups included.
    let parent_disabled = use_hook(try_consume_context::<Disabled>);
    let mut own_disabled = use_hook(|| Disabled(Signal::new(false)));
    use_context_provider(|| own_disabled);
    let disabled =
        props.disabled.unwrap_or(false) || parent_disabled.is_some_and(|Disabled(parent)| parent());
    if *own_disabled.0.peek() != disabled {
        own_disabled.0.set(disabled);
    }
    let parent_required = use_hook(try_consume_context::<Required>);
    let mut own_required = use_hook(|| Required(Signal::new(false)));
    use_context_provider(|| own_required);
    let required = props
        .required
        .unwrap_or_else(|| parent_required.is_some_and(|Required(parent)| parent()));
    if *own_required.0.peek() != required {
        own_required.0.set(required);
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
    use_name_warning(
        !props.label.is_none() || names_itself(&props.attributes),
        "Fieldset: no `label`, `aria-label` or `aria-labelledby`, so the group has no name \
         and its question is never read.",
    );

    let prefix = binding.prefix();
    let warned_blank = use_hook(|| Rc::new(Cell::new(false)));
    // A disabled group's own rules are barred like its fields' (4208): the user cannot fix them.
    let issues = match props.validate.is_empty() || disabled {
        true => Vec::new(),
        false => binding
            .with::<V, _>(&[], |value| {
                issues_of(&props.validate, value, prefix, &warned_blank, &scope)
            })
            .unwrap_or_default(),
    };
    scope.raise(key, issues);

    // The group's own rule waits for focus to leave the group, or a submit (4257). Counted per
    // reset, like a field's touched flag; no rules, no unnamed issue, so no subscription.
    let left_at = use_signal(|| None::<u32>);
    let leave = move || {
        let mut left_at = left_at;
        let now = Some(scope.generation());
        if *left_at.peek() != now {
            left_at.set(now);
        }
    };
    use_silent_focus_out_of(element, leave);
    let revealed =
        !props.validate.is_empty() && (scope.submitted() || left_at() == Some(scope.generation()));
    let unnamed = match revealed {
        true => scope.unnamed_issue(key),
        false => FieldStatus::Valid,
    };
    let explicit = props.status.as_ref().cloned().unwrap_or_default();
    let status = worst(explicit.clone(), unnamed);

    let id = crate::hooks::use_root_id(&props.attributes)();
    // In error, the group is one input in error, as a field's explicit status: it blocks a
    // submit and has a summary line (Maintainer, todo 1273).
    scope.register(
        key,
        FieldEntry {
            id: id.clone(),
            label: props.label.text().map(str::to_string),
            name: None,
            explicit_error: explicit.is_error(),
            status: explicit,
            parse: crate::components::form::Parse::Valid,
            required: false,
            barred: disabled,
            owner: current_scope_id(),
        },
    );
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
        rsx! { legend { "data-slot": FieldsetPart::Legend.slot(), {content} } }
    });

    let description = slot_node(FieldsetPart::Description.slot(), &id, &props.description);
    let helper = slot_node(FieldsetPart::Helper.slot(), &id, &props.helper);
    let status_line = status_node(
        &id,
        Some(&status),
        crate::hooks::use_localization().common.warning,
    );
    let record = *record.peek();
    // Fixed slots: a caption that appears must not shift the fields and remount them (4926).
    let children = rsx! {
        {legend}
        {description}
        for record in [record] {
            Fragment { key: "{record}", {props.children.clone()} }
        }
        {helper}
        {status_line}
    };

    let mut states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("disabled", disabled);
    if let Some(state) = status.state() {
        states = states.active(state);
    }
    let states: Input<_> = states.into();

    use_box()
        .framework_sx(&FIELDSET_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .prepare()
        .element(&element)
        .event("onfocusout", move |event: FocusEvent| {
            if focus_left(&event, Some(element)) {
                leave();
            }
        })
        .attr("id", id.clone())
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("aria-describedby", describedby)
        .render(HtmlTag::Fieldset, props.attributes, children)
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, fmt::Debug};

    use dioxus::{dioxus_core::NoOpMutations, logger::tracing, prelude::*};

    use crate::{
        LiberoProvider,
        components::form::{Fieldset, FormScope, Rule, TextField},
        utils::warnings_of,
    };

    thread_local! {
        /// What each rule run was handed.
        static JUDGED: Cell<Option<u32>> = const { Cell::new(None) };
        static RENDERS: Cell<usize> = const { Cell::new(0) };
    }

    fn judged_and_warnings(app: fn() -> Element) -> (Option<u32>, Vec<String>) {
        JUDGED.set(None);
        let warnings = warnings_of(app)
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
            rsx! { LiberoProvider { Fieldset::<u32> { label: "G", validate: rule.error("Never"), "" } } }
        });
        assert_eq!(judged, None);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
    }

    #[test]
    fn a_fieldset_with_a_value_runs_its_rules_on_it() {
        let (judged, warnings) = judged_and_warnings(|| {
            let value = use_store(|| 7_u32);
            rsx! { LiberoProvider { Fieldset { label: "G", value, validate: rule.error("Never"), "" } } }
        });
        assert_eq!(judged, Some(7));
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    /// Todo 2567, 4257: a pathless group's rules no longer wait on every field of the form.
    #[test]
    fn a_pathless_fieldset_with_rules_in_a_form_warns_of_nothing() {
        let (_, warnings) = judged_and_warnings(|| {
            use_context_provider(FormScope::new);
            let value = use_store(|| 7_u32);
            rsx! { LiberoProvider { Fieldset { label: "G", value, validate: rule.error("Never"), "" } } }
        });
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    /// Counts the `render` spans dioxus opens for a `Fieldset` scope.
    struct FieldsetRenders;

    impl tracing::Subscriber for FieldsetRenders {
        fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
            metadata.name() == "render"
        }
        fn new_span(&self, span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            struct Scope(bool);
            impl tracing::field::Visit for Scope {
                fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn Debug) {
                    self.0 |=
                        field.name() == "scope" && format!("{value:?}").contains("::Fieldset");
                }
            }
            let mut scope = Scope(false);
            span.record(&mut scope);
            if scope.0 {
                RENDERS.set(RENDERS.get() + 1);
            }
            tracing::span::Id::from_u64(1)
        }
        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
        fn event(&self, _: &tracing::Event<'_>) {}
        fn enter(&self, _: &tracing::span::Id) {}
        fn exit(&self, _: &tracing::span::Id) {}
    }

    /// How often the fieldset re-renders when a field elsewhere in the form is touched.
    fn renders_after_a_touch(app: fn() -> Element) -> usize {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut NoOpMutations);
        RENDERS.set(0);
        tracing::subscriber::with_default(FieldsetRenders, || {
            dom.in_scope(ScopeId::APP, || {
                consume_context::<FormScope>().touch("elsewhere")
            });
            dom.process_events();
            dom.render_immediate(&mut NoOpMutations);
        });
        RENDERS.get()
    }

    #[derive(Clone, PartialEq, Default)]
    struct Pair {
        a: String,
    }

    #[derive(Clone, PartialEq, Default)]
    struct Two {
        x: Pair,
        y: Pair,
    }

    /// Renders `app`, flips its `Signal<bool>` context and renders again.
    fn before_and_after_a_flip(app: fn() -> Element) -> (String, String) {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut NoOpMutations);
        let before = dioxus_ssr::render(&dom);
        dom.in_scope(ScopeId::APP, || consume_context::<Signal<bool>>().set(true));
        dom.render_immediate(&mut NoOpMutations);
        (before, dioxus_ssr::render(&dom))
    }

    /// Todo 2565: the fields bound once at mount and kept editing the first record.
    #[test]
    fn a_different_value_or_path_rebinds_the_fields() {
        let (before, after) = before_and_after_a_flip(|| {
            let first = use_store(|| Pair { a: "first".into() });
            let second = use_store(|| Pair { a: "second".into() });
            let flip = use_context_provider(|| Signal::new(false));
            let value = if flip() { second } else { first };
            rsx! {
                LiberoProvider {
                    Fieldset { label: "G", value,
                        TextField { label: "A", name: crate::path!(Pair => a) }
                    }
                }
            }
        });
        assert!(before.contains("first"), "{before}");
        assert!(
            after.contains("second") && !after.contains("first"),
            "{after}"
        );

        let (before, after) = before_and_after_a_flip(|| {
            let two = use_store(|| Two {
                x: Pair { a: "first".into() },
                y: Pair { a: "second".into() },
            });
            let flip = use_context_provider(|| Signal::new(false));
            let path = match flip() {
                true => crate::path!(Two => y),
                false => crate::path!(Two => x),
            };
            rsx! {
                LiberoProvider {
                    Fieldset { label: "Both", value: two,
                        Fieldset { label: "One", path,
                            TextField { label: "A", name: crate::path!(Pair => a) }
                        }
                    }
                }
            }
        });
        assert!(before.contains("first"), "{before}");
        assert!(
            after.contains("second") && !after.contains("first"),
            "{after}"
        );
    }

    /// Todo 4347: a reset of the form puts the group's own value back to its value at mount.
    #[test]
    fn a_form_reset_restores_the_groups_own_value() {
        let mut dom = VirtualDom::new(|| {
            use_context_provider(FormScope::new);
            let value = use_store(|| Pair { a: "start".into() });
            use_context_provider(|| value);
            rsx! {
                LiberoProvider {
                    Fieldset { label: "G", value,
                        TextField { label: "A", name: crate::path!(Pair => a) }
                    }
                }
            }
        });
        dom.rebuild_in_place();
        dom.render_immediate(&mut NoOpMutations);
        dom.in_scope(ScopeId::APP, || {
            consume_context::<Store<Pair>>().set(Pair { a: "typed".into() });
            consume_context::<FormScope>().reset();
        });
        dom.render_immediate(&mut NoOpMutations);
        dom.render_immediate(&mut NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("start") && !html.contains("typed"), "{html}");
    }

    /// Todo 2570, 4257: a touch elsewhere re-renders no fieldset, ruled or not; only focus
    /// leaving the group reveals its rule.
    #[test]
    fn a_touch_elsewhere_re_renders_no_fieldset() {
        let plain = renders_after_a_touch(|| {
            use_context_provider(FormScope::new);
            rsx! { LiberoProvider { Fieldset::<()> { label: "G", "" } } }
        });
        assert_eq!(plain, 0);
        let ruled = renders_after_a_touch(|| {
            use_context_provider(FormScope::new);
            let value = use_store(|| 7_u32);
            rsx! { LiberoProvider { Fieldset { label: "G", value, validate: rule.error("Never"), "" } } }
        });
        assert_eq!(ruled, 0);
    }
}
