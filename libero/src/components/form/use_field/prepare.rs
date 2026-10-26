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
        common::{Input, Part, States, Variables, variables},
        form::{Caption, FieldEntry, FieldPart, FieldStatus, Parse, worst},
        layout::use_box,
    },
    hooks::{ElementHandle, use_localization, use_root_id, use_silent_focus_out},
    platform,
    theme::SizeCss,
    utils::warn,
};

impl FieldBuilder<'_> {
    /// Resolves the chrome. **This is the hook** - see `use_field`.
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
        // An error set from code on this name counts as an explicit status until the next edit.
        let set = scope
            .zip(name.as_deref())
            .and_then(|(scope, name)| scope.error_at(name));
        // The entry leaves it out: the form joins it in, so a cleared one stops blocking at once.
        let own_explicit = explicit.clone();
        let explicit = match set {
            Some(set) => worst(explicit, FieldStatus::Error(set)),
            None => explicit,
        };
        let validated = self.rules.is_some() || self.mapped.is_some();
        let is_mapped = self.mapped.is_some();
        let known = is_mapped || self.parse.is_some();
        let (mapped, mapped_parse) = self.mapped.unwrap_or((None, Parse::Valid));
        let parse = match self.parse {
            Some(Parse::Invalid(error)) => Parse::Invalid(error),
            Some(own) if !is_mapped => own,
            _ => mapped_parse,
        };
        // Text that does not parse is no empty field: it says why, not "Fill in this field.".
        let empty = match known {
            true => parse == Parse::Empty,
            false => self.empty,
        };
        // Barred from validation as native constraint validation does: the user cannot fix it (4208).
        let barred = self.disabled || self.readonly;
        // `required` is a rule at submit inside a form; the field's own rules speak first.
        let missing = self.required && empty && !barred && scope.is_some();
        // The rules over what the field edits speak before the parse and the domain's rules.
        let own_rules = match barred {
            true => FieldStatus::Valid,
            false => first_failing(self.rules, mapped).unwrap_or_default(),
        };
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
                    label: summary_label(
                        label.text(),
                        self.aria_label,
                        attribute_text(self.attributes, "aria-label"),
                    ),
                    name: name.clone(),
                    status: worst(own_explicit.clone(), rules.clone()),
                    explicit_error: own_explicit.is_error(),
                    parse,
                    required: self.required,
                    barred,
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
        let description = self.description.unwrap_or(&Caption::None);
        let helper = self.helper.unwrap_or(&Caption::None);
        // An inline control with no visible text drops the empty label column and its gap (4173).
        let solo = self.inline
            && label.is_none()
            && description.is_none()
            && helper.is_none()
            && status.and_then(FieldStatus::message).is_none();

        let mut states = self
            .states
            .and_then(Input::as_ref)
            .cloned()
            .unwrap_or_default()
            .with(self.size.state_name(), true)
            .with(self.radius.size.radius_state_name(), true)
            .with("disabled", self.disabled)
            .with("required", self.required)
            .with("inline", self.inline)
            .with("solo", solo)
            .with("card", self.card);
        if let Some(state) = status_state {
            states = states.active(state);
        }
        let states: Input<States> = states.into();

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

        // The frame and controls inherit it past their own `per_radius` reset.
        let radius: Input<Variables> = variables()
            .with(
                SizeCss::RADIUS.override_var(),
                self.radius.custom_css(SizeCss::RADIUS),
            )
            .into();
        let mut wrapper = use_box()
            .framework_sx(&FIELD_SX)
            .states(&states)
            .variables(&radius);
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
                let settled = match (scope, &name) {
                    (Some(mut scope), Some(name)) => {
                        scope.touch(name);
                        scope.settle_draft(name)
                    }
                    _ => false,
                };
                if !hook.touched.replace(true) || settled {
                    Runtime::current().needs_update(hook.owner);
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
            busy: matches!(status, Some(FieldStatus::Pending)),
            required: self.required,
            inline: self.inline,
            activation,
            states,
            wrapper,
            root,
        }
    }
}

/// The first error of the two, else the first warning; `None` when neither speaks.
fn first_failing(first: Option<FieldStatus>, second: Option<FieldStatus>) -> Option<FieldStatus> {
    match (first, second) {
        (Some(first), _) if first.is_error() => Some(first),
        (_, Some(second)) if second.is_error() => Some(second),
        (Some(first), _) if first.is_warning() => Some(first),
        (first, second) => second.or(first),
    }
}

/// The field's name in the error summary: the label, then `aria_label`, then a spread one;
/// blank text names nothing.
fn summary_label(
    label: Option<&str>,
    aria_label: Option<&str>,
    spread: Option<String>,
) -> Option<String> {
    label
        .into_iter()
        .chain(aria_label)
        .map(str::to_string)
        .chain(spread)
        .find(|text| !text.trim().is_empty())
}

/// Whether a `focusout` inside `root` took focus out of it. The web reads `relatedTarget`; Blitz
/// fires it after the move, so asks where focus is. A WebView cannot answer: any move counts.
pub(in crate::components::form) fn focus_left(
    event: &FocusEvent,
    root: Option<ElementHandle>,
) -> bool {
    let Some(mounted) = root.and_then(|root| root.mounted()) else {
        return true;
    };
    match platform::focus_left(event, &mounted) {
        Some(left) => left,
        None => !platform::reads_dom_synchronously() || !platform::focus_is_in(&mounted),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blank_name_does_not_head_a_summary_line() {
        assert_eq!(summary_label(Some(" "), None, None), None);
        assert_eq!(
            summary_label(Some(" "), Some("Email"), Some("x".into())).as_deref(),
            Some("Email")
        );
        assert_eq!(
            summary_label(None, Some(""), Some("Email".into())).as_deref(),
            Some("Email")
        );
        assert_eq!(
            summary_label(Some("Name"), Some("Email"), None).as_deref(),
            Some("Name")
        );
    }
}
