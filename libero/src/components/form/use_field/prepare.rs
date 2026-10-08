use dioxus::{core::Runtime, prelude::*};

use super::{
    builder::FieldBuilder,
    hook::FieldHook,
    nodes::{attribute_text, join_ids, label_node, slot_node, status_node},
    prepared::PreparedField,
    styles::FIELD_SX,
};
use crate::{
    components::{
        common::{Input, Part, States},
        form::{Caption, FieldEntry, FieldPart, FieldStatus, worst},
        layout::use_box,
    },
    hooks::{ElementHandle, use_localization, use_root_id, use_silent_focus_out},
    platform,
    utils::warn,
};

impl FieldBuilder<'_> {
    /// Resolves the chrome. **This is the hook** - see [`use_field`].
    pub fn prepare(self) -> PreparedField {
        let id = use_root_id(self.attributes);
        let id_value = id();
        let words = use_localization();
        let warning_word = words.common.warning;
        let required_text = self.asks.text(&words.form);

        // One hook for the touched flag, the form registration, the binding and
        // their cleanup: every field pays for it, validated or not.
        let hook = match self.hook {
            Some(hook) => hook,
            None => use_hook(FieldHook::new),
        };
        let scope = hook.form.map(|(scope, _)| scope);

        let name = scope.and(
            self.name
                .map(str::to_string)
                .or_else(|| attribute_text(self.attributes, "name")),
        );
        let label = self.label.unwrap_or(&Caption::None);
        let explicit = self
            .status
            .and_then(Input::as_ref)
            .cloned()
            .unwrap_or_default();
        // `required` is a rule at submit inside a form; the field's own rules speak first.
        let missing =
            self.required && self.empty && !self.disabled && !self.readonly && scope.is_some();
        let validated = self.rules.is_some();
        let own_rules = self.rules.unwrap_or_default();
        let rules = match missing {
            true => worst(own_rules.clone(), FieldStatus::Error(required_text.into())),
            false => own_rules.clone(),
        };
        let blank = [&explicit, &rules]
            .into_iter()
            .any(|status| status.message().is_some_and(|text| text.trim().is_empty()));
        if blank && !hook.warned_blank.replace(true) {
            warn(
                "Field: a status or rule message is blank, so the field is invalid with nothing \
                 to read. Give it a text, or use `FieldStatus::Valid`.",
            );
        }

        if let Some((mut scope, key)) = hook.form {
            let generation = scope.generation();
            if hook.generation.replace(generation) != generation {
                hook.touched.set(false);
            }
            scope.register(
                key,
                FieldEntry {
                    id: id_value.clone(),
                    // A field named by `aria_label` only is still told apart in the summary (1526).
                    label: label
                        .text()
                        .or(self.aria_label)
                        .map(str::to_string)
                        .or_else(|| attribute_text(self.attributes, "aria-label")),
                    name: name.clone(),
                    status: worst(explicit.clone(), rules.clone()),
                    owner: hook.owner,
                },
            );
        }

        // Own rules wait for the first blur or submit, the implicit required error for the
        // submit; an explicit status - a server's answer - never waits.
        let submitted = scope.is_some_and(|scope| scope.submitted());
        let revealed = match (submitted, hook.touched.get()) {
            (true, _) => rules,
            (false, true) => own_rules,
            (false, false) => FieldStatus::Valid,
        };
        let composite = match (scope, &name) {
            (Some(scope), Some(name)) => scope.visible_issue(name),
            _ => FieldStatus::Valid,
        };
        let shown = worst(worst(explicit, revealed), composite);

        let status = Some(&shown);
        let status_state = status.and_then(FieldStatus::state);

        let mut states = self
            .states
            .and_then(Input::as_ref)
            .cloned()
            .unwrap_or_default()
            .with(self.size.state_name(), true)
            .with(self.radius.radius_state_name(), true)
            .with("disabled", self.disabled)
            .with("required", self.required)
            .with("inline", self.inline)
            .with("card", self.card);
        if let Some(state) = status_state {
            states = states.active(state);
        }
        let states: Input<States> = states.into();

        let description = self.description.unwrap_or(&Caption::None);
        let helper = self.helper.unwrap_or(&Caption::None);

        // Only a `Text` caption is named: markup the caller built is theirs to
        // describe, and the status message is always text. In reading order.
        let [leading, trailing] = self.text_slots;
        let described = [
            ("description", description.text().is_some()),
            ("leading", leading),
            ("trailing", trailing),
            ("helper", helper.text().is_some()),
            ("status", status.and_then(FieldStatus::message).is_some()),
        ];
        let describedby = join_ids(&id_value, described);

        let mut wrapper = use_box().framework_sx(&FIELD_SX).states(&states);
        if let Some(class) = self.class {
            wrapper = wrapper.class(class);
        }
        if let Some(sx) = self.sx {
            wrapper = wrapper.sx(sx);
        }
        let mut wrapper = wrapper.parts_source(self.parts).prepare();
        let activation = self.activation.map(|mut activation| {
            activation.card = self.card;
            activation
        });
        // The card's own click. The label's and the input's stop at
        // themselves under `card`, so each click activates once.
        if let Some(activation) = activation.as_ref().filter(|_| self.card) {
            wrapper = wrapper.event("onclick", activation.card_click());
        }
        let touches = validated || scope.is_some();
        let touch = {
            let hook = hook.clone();
            move || {
                if !hook.touched.replace(true) {
                    Runtime::current().needs_update(hook.owner);
                }
                if let (Some(mut scope), Some(name)) = (scope, &name) {
                    scope.touch(name);
                }
            }
        };
        // Blitz's Tab fires no `focusout`: the same touch, from the silent move.
        let silent = use_silent_focus_out({
            let touch = touch.clone();
            move || {
                if touches {
                    touch();
                }
            }
        });
        // The label of a `labelled_by` control finds that control in here.
        let focuses = self.labelled_by && !self.group;
        // A touching field also asks whether focus left it. Made for every field: rules added
        // after mount would otherwise find no handle (todo 2368).
        let own = use_hook(ElementHandle::new);
        let root = silent.or(Some(own));
        if let Some(element) = &root {
            wrapper = wrapper.element(element);
        }
        // `focusout` bubbles, so one listener on the wrapper sees the control lose focus whatever
        // it is - and leaves a caller's own `onblur` alone. A move inside (a `PinField` cell to the
        // next) is no blur (1564).
        if touches {
            wrapper = wrapper.event("onfocusout", move |event: FocusEvent| {
                if focus_left(&event, root) {
                    touch();
                }
            });
        }
        let root = root.filter(|_| silent.is_some() || focuses);

        PreparedField {
            label: label_node(
                &id_value,
                label,
                self.required.then_some(self.required_word),
                self.labelled_by,
                self.label_with_id,
                activation.clone(),
                root.filter(|_| focuses),
            ),
            labelled_by: (self.labelled_by || self.label_with_id) && !label.is_none(),
            description: slot_node(FieldPart::Description.slot(), &id_value, description),
            helper: slot_node(FieldPart::Helper.slot(), &id_value, helper),
            status: status_node(&id_value, status, warning_word),
            id: id_value,
            text_slots: self.text_slots,
            describedby,
            invalid: matches!(status, Some(FieldStatus::Error(_))),
            required: self.required,
            inline: self.inline,
            activation,
            states,
            wrapper,
            root,
        }
    }
}

/// Whether a `focusout` inside `root` took focus out of it. The web reads `relatedTarget`; Blitz
/// fires it after the move, so asks where focus is. A WebView cannot answer: any move counts.
fn focus_left(event: &FocusEvent, root: Option<ElementHandle>) -> bool {
    let Some(mounted) = root.and_then(|root| root.mounted()) else {
        return true;
    };
    match platform::focus_left(event, &mounted) {
        Some(left) => left,
        None => !platform::reads_dom_synchronously() || !platform::focus_is_in(&mounted),
    }
}
