use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, States,
        form::{Caption, FieldStatus},
        layout::{BoxStyle, use_box},
    },
    hooks::use_root_id,
    sx::{StaticSx, Sx, sx},
    theme::{FieldDefaults, Size},
};

/// The wrapper owns the look of all four text slots, addressed by tag and by
/// `data-slot`. Giving each caption its own `use_box` would cost four more
/// stylesheet registrations per field, and none of them takes styling props.
static FIELD_SX: StaticSx = StaticSx::new(|| {
    FieldDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .selector("& > [data-slot]", sx().color("grey.7"))
        .selector(
            "& label > [data-slot='required']",
            sx().color("error.7").margin_left("2px"),
        )
        // Not `opacity`: the control dims itself, and two stacked opacities
        // multiply.
        .when(
            "disabled",
            sx().color("grey.6")
                .selector("& > [data-slot]", sx().color("grey.6")),
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

/// The chrome every field stacks around its control: the label, the
/// description, the helper text, the status message, and the a11y wiring that
/// ties them to the control.
///
/// Prepare-style, like [`use_box`]: libero's own fields pay no extra component
/// scope for it. The field keeps its control - its element, its type, its
/// events and its frame; the helper never touches those.
///
/// `use_` because [`FieldBuilder::prepare`] calls [`use_root_id`] and
/// [`use_box`]. Build it in the component body and return it - never inside an
/// `if`, a `match` arm, or after an early `return`.
pub(crate) fn use_field<'a>() -> FieldBuilder<'a> {
    FieldBuilder::default()
}

pub(crate) struct FieldBuilder<'a> {
    label: Option<&'a Caption>,
    description: Option<&'a Caption>,
    helper: Option<&'a Caption>,
    status: Option<&'a Input<FieldStatus>>,
    required: bool,
    disabled: bool,
    size: Size,
    radius: Size,
    class: Option<&'a Input<ClassList>>,
    sx: Option<&'a Input<Sx>>,
    states: Option<&'a Input<States>>,
    attributes: &'a [Attribute],
}

impl Default for FieldBuilder<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            label: None,
            description: None,
            helper: None,
            status: None,
            required: false,
            disabled: false,
            size: Size::Md,
            radius: Size::Sm,
            class: None,
            sx: None,
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

    #[inline]
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    #[inline]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    #[inline]
    pub fn states(mut self, states: &'a Input<States>) -> Self {
        self.states = Some(states);
        self
    }

    /// The caller's attributes, read for an `id` to adopt and for an
    /// `aria-describedby` that outranks ours.
    #[inline]
    pub fn attributes(mut self, attributes: &'a [Attribute]) -> Self {
        self.attributes = attributes;
        self
    }

    /// Resolves the chrome. **This is the hook** - see [`use_field`].
    pub fn prepare(self) -> PreparedField {
        let id = use_root_id(self.attributes);
        let id_value = id();

        let status = self.status.and_then(Input::as_ref);
        let status_state = status.and_then(FieldStatus::state);

        let mut states = self
            .states
            .and_then(Input::as_ref)
            .cloned()
            .unwrap_or_default()
            .with(self.size.state_name(), true)
            .with(self.radius.radius_state_name(), true)
            .with("disabled", self.disabled)
            .with("required", self.required);
        if let Some(state) = status_state {
            states = states.active(state);
        }
        let states: Input<States> = states.into();

        let label = self.label.unwrap_or(&Caption::None);
        let description = self.description.unwrap_or(&Caption::None);
        let helper = self.helper.unwrap_or(&Caption::None);

        // Only a `Text` caption is named: markup the caller built is theirs to
        // describe, and the status message is always text.
        let described = [
            ("description", description.text().is_some()),
            ("helper", helper.text().is_some()),
            ("status", status.and_then(FieldStatus::message).is_some()),
        ];
        let describedby = match caller_names_the_description(self.attributes) {
            true => None,
            false => join_ids(&id_value, described),
        };

        let mut wrapper = use_box().framework_sx(&FIELD_SX).states(&states);
        if let Some(class) = self.class {
            wrapper = wrapper.class(class);
        }
        if let Some(sx) = self.sx {
            wrapper = wrapper.sx(sx);
        }
        let wrapper = wrapper.prepare();

        PreparedField {
            label: label_node(&id_value, label, self.required),
            description: slot_node("description", &id_value, description),
            helper: slot_node("helper", &id_value, helper),
            status: status_node(&id_value, status),
            id: id_value,
            describedby,
            invalid: matches!(status, Some(FieldStatus::Error(_))),
            required: self.required,
            states,
            wrapper,
        }
    }
}

/// The resolved chrome. Pure - no hooks - so a field may render it on one path
/// and not another.
pub(crate) struct PreparedField {
    id: String,
    states: Input<States>,
    describedby: Option<String>,
    invalid: bool,
    required: bool,
    wrapper: BoxStyle,
    /// `None` rather than an empty `rsx! {}`: a slot nothing filled costs no
    /// node at all, which is most slots on most fields.
    label: Option<Element>,
    description: Option<Element>,
    helper: Option<Element>,
    status: Option<Element>,
}

impl PreparedField {
    /// Size, radius, status, `disabled` and `required`, for the control's own
    /// [`use_box`].
    pub fn states(&self) -> &Input<States> {
        &self.states
    }

    /// The a11y wiring, onto the control's already-prepared styling.
    pub fn aria(&self, control: BoxStyle) -> BoxStyle {
        control
            .attr("id", self.id.clone())
            .attr("aria-describedby", self.describedby.clone())
            .attr("aria-invalid", self.invalid.then_some("true"))
            .attr("aria-required", self.required.then_some("true"))
    }

    /// Label, description, control, helper, status - in that order, inside the
    /// wrapper. A field wanting another layout takes the parts instead.
    pub fn render(self, control: Element) -> Element {
        let mut children = Vec::with_capacity(5);
        children.extend(self.label);
        children.extend(self.description);
        children.push(control);
        children.extend(self.helper);
        children.extend(self.status);

        self.wrapper.render(HtmlTag::Div, Vec::new(), children)
    }
}

/// A caller's own `aria-describedby` wins outright - ours is dropped rather
/// than merged, so the caption props become purely visual.
fn caller_names_the_description(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.name == "aria-describedby")
}

fn join_ids(id: &str, slots: [(&'static str, bool); 3]) -> Option<String> {
    let present = || slots.iter().filter(|(_, present)| *present);
    let capacity: usize = present()
        .map(|(slot, _)| id.len() + slot.len() + 2)
        .sum::<usize>();
    if capacity == 0 {
        return None;
    }

    let mut names = String::with_capacity(capacity - 1);
    for (slot, _) in present() {
        if !names.is_empty() {
            names.push(' ');
        }
        names.push_str(id);
        names.push('-');
        names.push_str(slot);
    }
    Some(names)
}

fn caption_content(caption: &Caption) -> Element {
    match caption {
        Caption::None => rsx! {},
        Caption::Text(text) => rsx! { "{text}" },
        Caption::Node(node) => node.clone(),
    }
}

fn label_node(id: &str, label: &Caption, required: bool) -> Option<Element> {
    if label.is_none() {
        return None;
    }

    let content = caption_content(label);
    Some(rsx! {
        label { r#for: "{id}",
            {content}
            // `aria-required` already tells AT; the asterisk is decoration.
            if required {
                span { "aria-hidden": "true", "data-slot": "required", "*" }
            }
        }
    })
}

fn slot_node(slot: &'static str, id: &str, caption: &Caption) -> Option<Element> {
    if caption.is_none() {
        return None;
    }

    // Markup is not named, so it is not in `aria-describedby` either.
    let named = caption.text().is_some().then(|| format!("{id}-{slot}"));
    let content = caption_content(caption);
    Some(rsx! {
        span { "data-slot": slot, id: named, {content} }
    })
}

fn status_node(id: &str, status: Option<&FieldStatus>) -> Option<Element> {
    let message = status.and_then(FieldStatus::message)?;

    Some(rsx! {
        span { "data-slot": "status", id: "{id}-status", "{message}" }
    })
}
