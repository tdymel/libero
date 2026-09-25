use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    activation::Activation,
    hook::{Bound, FieldHook},
};
use crate::{
    components::{
        common::{ClassList, Input, Part, Parts, States, parts_source},
        form::{Caption, FieldStatus},
    },
    hooks::{ElementHandle, SxSource},
    sx::Sx,
    theme::Size,
};

/// The chrome every field stacks around its control, and its a11y wiring.
/// A hook: `prepare` calls hooks, so never build it conditionally.
pub(crate) fn use_field<'a>() -> FieldBuilder<'a> {
    FieldBuilder::default()
}

pub(crate) struct FieldBuilder<'a> {
    pub(super) label: Option<&'a Caption>,
    pub(super) description: Option<&'a Caption>,
    pub(super) helper: Option<&'a Caption>,
    pub(super) status: Option<&'a Input<FieldStatus>>,
    pub(super) rules: Option<FieldStatus>,
    pub(super) name: Option<&'a str>,
    pub(super) hook: Option<Rc<FieldHook>>,
    pub(super) required: bool,
    pub(super) required_word: Option<&'static str>,
    pub(super) disabled: bool,
    pub(super) inline: bool,
    pub(super) card: bool,
    pub(super) labelled_by: bool,
    pub(super) group: bool,
    pub(super) label_with_id: bool,
    pub(super) text_slots: [bool; 2],
    pub(super) activation: Option<Activation>,
    pub(super) size: Size,
    pub(super) radius: Size,
    pub(super) class: Option<&'a Input<ClassList>>,
    pub(super) sx: Option<&'a Input<Sx>>,
    pub(super) parts: Option<SxSource<'a>>,
    pub(super) states: Option<&'a Input<States>>,
    pub(super) attributes: &'a [Attribute],
}

impl Default for FieldBuilder<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            label: None,
            description: None,
            helper: None,
            status: None,
            rules: None,
            name: None,
            hook: None,
            required: false,
            required_word: None,
            disabled: false,
            inline: false,
            card: false,
            labelled_by: false,
            group: false,
            label_with_id: false,
            text_slots: [false; 2],
            activation: None,
            size: Size::Md,
            radius: Size::Sm,
            class: None,
            sx: None,
            parts: None,
            states: None,
            attributes: &[],
        }
    }
}

impl<'a> FieldBuilder<'a> {
    #[inline]
    pub fn label(mut self, label: &'a Caption) -> Self {
        self.label = Some(label);
        self
    }

    #[inline]
    pub fn description(mut self, description: &'a Caption) -> Self {
        self.description = Some(description);
        self
    }

    #[inline]
    pub fn helper(mut self, helper: &'a Caption) -> Self {
        self.helper = Some(helper);
        self
    }

    #[inline]
    pub fn status(mut self, status: &'a Input<FieldStatus>) -> Self {
        self.status = Some(status);
        self
    }

    /// What the field's own `validate` rules say, or `None` when it has none.
    /// Shown once the field has lost focus or its form was submitted.
    #[inline]
    pub fn rules(mut self, rules: Option<FieldStatus>) -> Self {
        self.rules = rules;
        self
    }

    /// The name composite rules address the field by, for a field whose
    /// `name` is a prop rather than an attribute.
    #[inline]
    pub fn name(mut self, name: Option<&'a str>) -> Self {
        self.name = name;
        self
    }

    /// The field's `name`, resolved by [`use_bound`] - its full name and the
    /// hook `use_bound` already took, so the field pays for one.
    #[inline]
    pub fn bound<T>(mut self, bound: &'a Bound<T>) -> Self {
        self.name = bound.name();
        self.hook = Some(bound.hook.clone());
        self
    }

    #[inline]
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Says `word` in the label's accessible name when required, for a
    /// control `aria-required` is not allowed on (`role="slider"`).
    #[inline]
    pub fn required_in_name(mut self, word: &'static str) -> Self {
        self.required_word = Some(word);
        self
    }

    #[inline]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Puts the control beside the label instead of under it, for a field
    /// with no frame.
    #[inline]
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    /// Draws the inline layout as a card whose every click activates the
    /// control. Needs [`inline`](Self::inline) and [`activates`](Self::activates).
    #[inline]
    pub fn card(mut self, card: bool) -> Self {
        self.card = card;
        self
    }

    /// Names the control by `aria-labelledby`, for one `for` cannot name; the
    /// ids are handed out by [`PreparedField::label_id`].
    #[inline]
    pub fn labelled_by(mut self) -> Self {
        self.labelled_by = true;
        self
    }

    /// The label names a group of controls, as a `<legend>` does: a click on
    /// it focuses none of them.
    #[inline]
    pub fn names_group(mut self) -> Self {
        self.group = true;
        self
    }

    /// Gives a `<label for>` an id too, for a popup the control owns that
    /// the label must also name - `Autocomplete`'s listbox.
    #[inline]
    pub fn label_with_id(mut self) -> Self {
        self.label_with_id = true;
        self
    }

    /// Which frame slots describe the control too - a prefix, a counter. The
    /// caller says so (`describe_leading`/`_trailing`): an icon's name must not.
    #[inline]
    pub fn text_slots(mut self, leading: bool, trailing: bool) -> Self {
        self.text_slots = [leading, trailing];
        self
    }

    /// A checkbox or radio whose checked state Rust owns: label clicks, Space
    /// and input clicks all run `activate`. See [`Activation`].
    #[inline]
    pub fn activates(mut self, element: ElementHandle, activate: impl Fn() + 'static) -> Self {
        self.activation = Some(Activation::new(element, activate));
        self
    }

    /// Enter activates as well as Space, for a switch outside a `Form`; inside
    /// one it is left to the browser, which submits.
    #[inline]
    pub fn enter_activates(mut self, enter: bool) -> Self {
        if let Some(activation) = &mut self.activation {
            activation.enter = enter;
        }
        self
    }

    #[inline]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    #[inline]
    pub fn radius(mut self, radius: Size) -> Self {
        self.radius = radius;
        self
    }

    #[inline]
    pub fn class(mut self, class: &'a Input<ClassList>) -> Self {
        self.class = Some(class);
        self
    }

    #[inline]
    pub fn sx(mut self, sx: &'a Input<Sx>) -> Self {
        self.sx = Some(sx);
        self
    }

    /// The `parts` prop, merged into the wrapper's `sx`.
    #[inline]
    pub fn parts<P: Part>(mut self, parts: &'a Input<Parts<P>>) -> Self {
        self.parts = parts_source(parts);
        self
    }

    #[inline]
    pub fn states(mut self, states: &'a Input<States>) -> Self {
        self.states = Some(states);
        self
    }

    /// The caller's attributes, read for an `id` to adopt.
    #[inline]
    pub fn attributes(mut self, attributes: &'a [Attribute]) -> Self {
        self.attributes = attributes;
        self
    }
}
