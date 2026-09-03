use std::{cell::Cell, rc::Rc};

use dioxus::{
    core::{AttributeValue, Runtime, ScopeId, current_scope_id},
    prelude::*,
};

use crate::{
    components::{
        ClassList, HtmlTag, Input, States,
        form::{Binding, Caption, Disabled, FieldEntry, FieldName, FieldStatus, FormScope, worst},
        layout::{BoxStyle, use_box},
    },
    hooks::{ElementHandle, use_root_id},
    platform::ElementApi,
    sx::{StaticSx, Sx, sx},
    theme::{FIELD_FRAME_GAP, FieldDefaults, Size},
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
        // A control with no frame - a checkbox, a radio - sits beside its
        // label instead of under it. The control takes column one of the
        // first row; the label and every caption take column two, so the
        // captions line up under the label rather than under the box.
        // A control that fills its width can only fill the wrapper's, so the
        // wrapper has to fill its parent too - a centring parent would
        // otherwise shrink it to the control's natural width.
        .when("full-width", sx().width("100%"))
        .when(
            "inline",
            sx().display("grid")
                .grid_template_columns("auto 1fr")
                .align_items("center")
                .column_gap(FIELD_FRAME_GAP.value())
                .selector("& > label, & > [data-slot]", sx().grid_column("2")),
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
    rules: Option<FieldStatus>,
    name: Option<&'a str>,
    hook: Option<Rc<FieldHook>>,
    required: bool,
    disabled: bool,
    inline: bool,
    labelled_by: bool,
    activation: Option<Activation>,
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
            rules: None,
            name: None,
            hook: None,
            required: false,
            disabled: false,
            inline: false,
            labelled_by: false,
            activation: None,
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

    /// Names the control by `aria-labelledby` rather than `<label for>`, for a
    /// control `for` cannot name: an element with a `role`, or one another
    /// component owns. The field then hands the ids out instead of applying
    /// them - see [`PreparedField::label_id`].
    #[inline]
    pub fn labelled_by(mut self) -> Self {
        self.labelled_by = true;
        self
    }

    /// Makes the field's control a checkbox or a radio whose checked state
    /// Rust owns: `activate` is what a click on the label, Space and a
    /// cancelled click on the input all do. [`PreparedField::aria`] wires the
    /// input's side and attaches `element` to it; the label focuses it.
    ///
    /// No activation reaches the native control, so nothing restores its
    /// `checked` - see [`Activation`] for why that is the whole fix.
    #[inline]
    pub fn activates(mut self, element: ElementHandle, activate: impl Fn() + 'static) -> Self {
        self.activation = Some(Activation::new(element, activate));
        self
    }

    /// Enter activates as well as Space, for a `role="switch"`. A bare
    /// checkbox or radio takes Space alone, on every platform.
    #[inline]
    pub fn enter_activates(mut self) -> Self {
        if let Some(activation) = &mut self.activation {
            activation.enter = true;
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
        let validated = self.rules.is_some();
        let rules = self.rules.unwrap_or_default();

        if let Some((mut scope, key)) = hook.form {
            let generation = scope.generation();
            if hook.generation.replace(generation) != generation {
                hook.touched.set(false);
            }
            scope.register(
                key,
                FieldEntry {
                    id: id_value.clone(),
                    label: label.text().map(|text| text.to_string()),
                    name: name.clone(),
                    status: worst(explicit.clone(), rules.clone()),
                    owner: hook.owner,
                },
            );
        }

        // Rules wait for the first blur or submit; an explicit status - a
        // server's answer - never waits.
        let revealed = hook.touched.get() || scope.is_some_and(|scope| scope.submitted());
        let composite = match (scope, &name) {
            (Some(scope), Some(name)) => scope.visible_issue(name),
            _ => FieldStatus::Valid,
        };
        let shown = worst(
            worst(explicit, if revealed { rules } else { FieldStatus::Valid }),
            composite,
        );

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
            .with("inline", self.inline);
        if let Some(state) = status_state {
            states = states.active(state);
        }
        let states: Input<States> = states.into();

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
        let mut wrapper = wrapper.prepare();
        // `focusout` bubbles, so one listener on the wrapper sees the control
        // lose focus whatever it is - and leaves a caller's own `onblur` alone.
        if validated || scope.is_some() {
            let hook = hook.clone();
            wrapper = wrapper.event("onfocusout", move |_: FocusEvent| {
                if !hook.touched.replace(true) {
                    Runtime::current().needs_update(hook.owner);
                }
                if let (Some(mut scope), Some(name)) = (scope, &name) {
                    scope.touch(name);
                }
            });
        }

        PreparedField {
            label: label_node(
                &id_value,
                label,
                self.required,
                self.labelled_by,
                self.activation.clone(),
            ),
            labelled_by: self.labelled_by && !label.is_none(),
            description: slot_node("description", &id_value, description),
            helper: slot_node("helper", &id_value, helper),
            status: status_node(&id_value, status),
            id: id_value,
            describedby,
            invalid: matches!(status, Some(FieldStatus::Error(_))),
            required: self.required,
            inline: self.inline,
            activation: self.activation,
            states,
            wrapper,
        }
    }
}

/// What a field keeps across renders for validation and binding. Nothing
/// outside the field reads `touched`, so it re-renders its owner by hand rather
/// than through a signal.
pub(crate) struct FieldHook {
    touched: Cell<bool>,
    /// The form's reset generation this field last rendered at.
    generation: Cell<u32>,
    /// Whether a `name` that does not fit the form's value was warned about.
    warned: Cell<bool>,
    owner: ScopeId,
    form: Option<(FormScope, usize)>,
    binding: Binding,
    disabled: Option<Disabled>,
}

impl FieldHook {
    fn new() -> Rc<Self> {
        let form = try_consume_context::<FormScope>().map(|mut scope| (scope, scope.key()));
        Rc::new(Self {
            touched: Cell::new(false),
            generation: Cell::new(form.map_or(0, |(scope, _)| scope.generation())),
            warned: Cell::new(false),
            owner: current_scope_id(),
            form,
            binding: try_consume_context::<Binding>().unwrap_or_default(),
            disabled: try_consume_context::<Disabled>(),
        })
    }
}

/// Resolves a field's `name` against the `Form` or `Fieldset` around it: the
/// full name it posts as, and - for a name built from a path, on a field with
/// no handler of its own - its value inside the form's value.
///
/// **This is a hook**, and it is the one [`FieldBuilder::prepare`] would take:
/// hand the result to [`FieldBuilder::bound`].
pub(crate) fn use_bound<T: Clone + 'static>(name: &FieldName<T>, controlled: bool) -> Bound<T> {
    let hook = use_hook(FieldHook::new);
    let full = (!name.is_empty())
        .then(|| crate::components::form::validation_join(hook.binding.prefix(), name.as_str()));
    let active = !controlled && name.steps().is_some() && hook.binding.is_bound();
    Bound {
        name: active.then(|| name.clone()),
        hook,
        full,
    }
}

pub(crate) struct Bound<T> {
    hook: Rc<FieldHook>,
    /// The name, kept only while it binds.
    name: Option<FieldName<T>>,
    full: Option<String>,
}

impl<T> Bound<T> {
    /// The full name the field posts as - under every enclosing fieldset.
    pub fn name(&self) -> Option<&str> {
        self.full.as_deref()
    }

    /// The field's own `disabled`, or'd with every disabled `Fieldset` around
    /// it - a native `<fieldset disabled>` wins over a `false` too. Subscribes
    /// the field to the group's state.
    pub fn disabled(&self, own: Option<bool>) -> bool {
        own.unwrap_or(false) || self.hook.disabled.is_some_and(|Disabled(group)| group())
    }

    /// Whether the form's value drives the field.
    pub fn is_bound(&self) -> bool {
        self.name.is_some()
    }
}

impl<T: Clone + 'static> Bound<T> {
    /// The field's value inside the form's value, when bound. Subscribes the
    /// field to the form's value.
    pub fn value(&self) -> Option<T> {
        let name = self.name.as_ref()?;
        let value = self.hook.binding.with(name.steps()?, T::clone);
        if value.is_none() && !self.hook.warned.replace(true) {
            warn!(
                "the field named `{name}` holds a `{}`, which is not what that path names inside its form's value - the field is left unbound",
                std::any::type_name::<T>()
            );
        }
        value
    }

    /// Writes into the form's value, when bound.
    pub fn setter(&self) -> Option<Setter<T>> {
        self.name.clone().map(|name| Setter {
            hook: self.hook.clone(),
            name,
        })
    }

    /// What the field calls with its next value: the caller's `handler`, or,
    /// bound and without one, a write into the form's value.
    pub fn emit(&self, handler: Option<EventHandler<T>>) -> Option<impl Fn(T) + Clone + 'static> {
        let setter = self.setter();
        if handler.is_none() && setter.is_none() {
            return None;
        }
        Some(move |next: T| match (&handler, &setter) {
            (Some(handler), _) => handler.call(next),
            (None, Some(setter)) => setter.set(next),
            (None, None) => {}
        })
    }
}

/// A bound field's write into its form's value.
pub(crate) struct Setter<T> {
    hook: Rc<FieldHook>,
    name: FieldName<T>,
}

impl<T> Clone for Setter<T> {
    fn clone(&self) -> Self {
        Self {
            hook: self.hook.clone(),
            name: self.name.clone(),
        }
    }
}

impl<T: 'static> Setter<T> {
    pub fn set(&self, next: T) {
        self.hook
            .binding
            .set(self.name.steps().unwrap_or_default(), next);
    }
}

impl Drop for FieldHook {
    fn drop(&mut self) {
        if let Some((mut scope, key)) = self.form {
            scope.unregister(key);
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
    inline: bool,
    labelled_by: bool,
    activation: Option<Activation>,
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

    /// The field's id - the control's when the field applies it, and the stem
    /// every caption id is derived from either way.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The label's own id, for a control named by `aria-labelledby`. `None`
    /// when there is no label, or when the label names the control by `for`.
    pub fn label_id(&self) -> Option<String> {
        self.labelled_by.then(|| format!("{}-label", self.id))
    }

    /// The filled caption slots, joined. What `aria-describedby` should hold.
    pub fn describedby(&self) -> Option<String> {
        self.describedby.clone()
    }

    /// Whether the status is an error, for a control that applies its own
    /// `aria-invalid`.
    pub fn invalid(&self) -> bool {
        self.invalid
    }

    /// The a11y wiring, onto the control's already-prepared styling - and,
    /// under [`FieldBuilder::activates`], the input's activation.
    pub fn aria(&self, control: BoxStyle) -> BoxStyle {
        let control = control
            .attr("id", self.id.clone())
            .attr("aria-describedby", self.describedby.clone())
            .attr("aria-invalid", self.invalid.then_some("true"))
            .attr("aria-required", self.required.then_some("true"));
        match &self.activation {
            Some(activation) => activation.wire(control),
            None => control,
        }
    }

    /// Label, description, control, helper, status - in that order, inside the
    /// wrapper. Under [`FieldBuilder::inline`] the control comes first
    /// instead, and the grid puts it beside the label.
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

/// A checkable control's activation, taken off the browser.
///
/// The browser flips `checked` before a click is dispatched, and when the
/// click is cancelled it restores the old value at the *end* of the dispatch.
/// dioxus re-renders synchronously inside that dispatch, so a caller that
/// honours `onchange` has its new `checked` written and then overwritten, and
/// the next render sees nothing to write: the look moves, the property,
/// `:checked`, AT and the form post do not. So nothing is left to activate the
/// input natively: the label's click is cancelled, and Space - and Enter, for
/// a switch - is answered on `keydown`, which stops the activation outright.
///
/// A field gets it through [`FieldBuilder::activates`]; a control whose label
/// is its own - `Chip` - wires both halves itself.
#[derive(Clone)]
pub(crate) struct Activation {
    element: ElementHandle,
    activate: Rc<dyn Fn()>,
    enter: bool,
}

impl Activation {
    pub(crate) fn new(element: ElementHandle, activate: impl Fn() + 'static) -> Self {
        Self {
            element,
            activate: Rc::new(activate),
            enter: false,
        }
    }

    /// The label's half. Cancelling the label's click cancels its activation
    /// behaviour, which is the whole of "forward this click to the control":
    /// the input gets no click at all. The focus the forwarding gave it goes
    /// too, so it is given back by hand.
    pub(crate) fn label_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let activation = self.clone();
        move |event| {
            event.prevent_default();
            (activation.activate)();
            let _ = activation.element.focus();
        }
    }

    /// The input's half: attaches the element, and takes Space - and Enter,
    /// for a switch - and whatever click still reaches it.
    pub(crate) fn wire(&self, control: BoxStyle) -> BoxStyle {
        let (keydown, keyup) = (self.clone(), self.enter);
        let (click, input) = (self.activate.clone(), self.activate.clone());
        control
            .element(&self.element)
            // What still reaches the input as a click - assistive tech's
            // default action, a script's `click()` - is cancelled and taken
            // in Rust, which is the old half-fix: correct while `checked`
            // does not change, clobbered when it does.
            .event("onclick", move |event: Event<MouseData>| {
                event.prevent_default();
                click();
            })
            // Blitz forwards a `<label>` click to its input as a default
            // action that emits `input`, never `click`. On the web nothing
            // activates the input any more, so this never fires there; both
            // compute the same next value, so firing twice is a no-op.
            .event("oninput", move |_: FormEvent| input())
            .event("onkeydown", move |event: Event<KeyboardData>| {
                if activates(&event, keydown.enter) {
                    event.prevent_default();
                    if !event.is_auto_repeating() {
                        (keydown.activate)();
                    }
                }
            })
            // A browser that clicks on `keyup` rather than checking the
            // cancelled `keydown` would otherwise activate a second time.
            .event("onkeyup", move |event: Event<KeyboardData>| {
                if activates(&event, keyup) {
                    event.prevent_default();
                }
            })
    }
}

fn activates(event: &KeyboardData, enter: bool) -> bool {
    match event.key() {
        Key::Character(ref c) => c == " ",
        Key::Enter => enter,
        _ => false,
    }
}

/// A caller's own `aria-describedby` wins outright - ours is dropped rather
/// than merged, so the caption props become purely visual.
fn attribute_text(attributes: &[Attribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .rev()
        .find_map(|attribute| match (attribute.name, &attribute.value) {
            (found, AttributeValue::Text(value)) if found == name => Some(value.clone()),
            _ => None,
        })
}

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

fn label_node(
    id: &str,
    label: &Caption,
    required: bool,
    labelled_by: bool,
    activation: Option<Activation>,
) -> Option<Element> {
    if label.is_none() {
        return None;
    }

    // `for` names a labelable element. A control that is not one - a `role` on
    // a span, or an element another component owns - is named the other way
    // round, by an id the control points at.
    let (named, points_at) = match labelled_by {
        true => (Some(format!("{id}-label")), None),
        false => (None, Some(id.to_string())),
    };
    let content = caption_content(label);
    // `aria-required` already tells AT; the asterisk is decoration.
    let asterisk = required.then(|| {
        rsx! {
            span { "aria-hidden": "true", "data-slot": "required", "*" }
        }
    });
    // Two arms, because a listener cannot be optional and most labels need
    // none.
    Some(match activation {
        Some(activation) => rsx! {
            label {
                id: named,
                r#for: points_at,
                onclick: activation.label_click(),
                {content}
                {asterisk}
            }
        },
        None => rsx! {
            label { id: named, r#for: points_at,
                {content}
                {asterisk}
            }
        },
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
