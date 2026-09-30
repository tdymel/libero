use dioxus::prelude::*;

use super::activation::Activation;
use crate::{
    components::{
        common::{HtmlTag, Input, Part, States},
        form::FieldPart,
        layout::BoxStyle,
    },
    hooks::ElementHandle,
};

/// The resolved chrome. Pure - no hooks - so a field may render it on one path
/// and not another.
pub(crate) struct PreparedField {
    pub(super) id: String,
    pub(super) states: Input<States>,
    pub(super) text_slots: [bool; 2],
    pub(super) describedby: Option<String>,
    pub(super) invalid: bool,
    pub(super) required: bool,
    pub(super) inline: bool,
    pub(super) labelled_by: bool,
    pub(super) activation: Option<Activation>,
    pub(super) wrapper: BoxStyle,
    pub(super) root: Option<ElementHandle>,
    /// `None` rather than an empty `rsx! {}`: a slot nothing filled costs no
    /// node at all, which is most slots on most fields.
    pub(super) label: Option<Element>,
    pub(super) description: Option<Element>,
    pub(super) helper: Option<Element>,
    pub(super) status: Option<Element>,
}

impl PreparedField {
    /// Size, radius, status, `disabled` and `required`, for the control's own
    /// [`use_box`].
    pub fn states(&self) -> &Input<States> {
        &self.states
    }

    /// The field's id - the control's when the field applies it, and the stem
    /// every caption id is derived from either way.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The label's id, for a control named by `aria-labelledby`.
    pub fn label_id(&self) -> Option<String> {
        self.labelled_by.then(|| format!("{}-label", self.id))
    }

    /// The filled caption slots, joined. What `aria-describedby` should hold.
    pub fn describedby(&self) -> Option<String> {
        self.describedby.clone()
    }

    /// The ids the frame gives its leading and trailing slot, for the slots
    /// [`FieldBuilder::text_slots`] named; `aria-describedby` points at them.
    pub fn slot_ids(&self) -> [Option<String>; 2] {
        let [leading, trailing] = self.text_slots;
        [
            leading.then(|| format!("{}-leading", self.id)),
            trailing.then(|| format!("{}-trailing", self.id)),
        ]
    }

    /// Whether the status is an error, for a control that applies its own
    /// `aria-invalid`.
    pub fn invalid(&self) -> bool {
        self.invalid
    }

    /// The wrapper's element, held for a `labelled_by` field or a Blitz focus watch.
    pub fn root(&self) -> Option<ElementHandle> {
        self.root
    }

    /// The a11y wiring and the `control` slot, onto the control's already-prepared styling - and,
    /// under [`FieldBuilder::activates`], the input's activation.
    pub fn aria(&self, control: BoxStyle) -> BoxStyle {
        // An inline field's input is hidden; its visible wrapper carries the slot.
        let control = control
            .attr(
                "data-slot",
                (!self.inline).then_some(FieldPart::Control.slot()),
            )
            .attr("id", self.id.clone())
            .attr("aria-describedby", self.describedby.clone())
            .attr("aria-invalid", self.invalid.then_some("true"))
            .attr("aria-required", self.required.then_some("true"));
        match &self.activation {
            Some(activation) => activation.wire(control),
            None => control,
        }
    }

    /// Label, description, control, helper, status in the wrapper; an inline
    /// control comes first.
    pub fn render(self, control: Element) -> Element {
        let mut children = Vec::with_capacity(5);
        if self.inline {
            children.push(control);
            children.extend(self.label);
            children.extend(self.description);
        } else {
            children.extend(self.label);
            children.extend(self.description);
            children.push(control);
        }
        children.extend(self.helper);
        children.extend(self.status);

        self.wrapper.render(HtmlTag::Div, Vec::new(), children)
    }
}
