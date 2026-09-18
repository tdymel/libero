use std::{cell::Cell, rc::Rc};

use dioxus::{
    core::{AttributeValue, Runtime, ScopeId, current_scope_id},
    prelude::*,
};

use crate::{
    components::{
        common::{ClassList, HtmlTag, Input, States, input_from_str},
        form::{
            Binding, Caption, Disabled, FieldEntry, FieldName, FieldStatus, FormScope, Validators,
            worst,
        },
        layout::{BoxStyle, use_box},
    },
    hooks::{ElementHandle, use_root_id, use_silent_focus_out},
    platform::{ElementApi, nested_interactive, next_task},
    sx::{StaticSx, Sx, sx},
    theme::{
        ChoiceVariant, FIELD_CARD_PADDING, FIELD_FRAME_GAP, FieldDefaults, PAPER_BACKGROUND,
        PAPER_BORDER_COLOR, PAPER_RADIUS, Size,
    },
};

input_from_str!(ChoiceVariant);

impl From<ChoiceVariant> for Input<ChoiceVariant> {
    fn from(value: ChoiceVariant) -> Self {
        Input::Value(value)
    }
}

/// The wrapper owns the look of all four text slots, addressed by tag and by
/// `data-slot`. Giving each caption its own `use_box` would cost four more
/// stylesheet registrations per field, and none of them takes styling props.
static FIELD_SX: StaticSx = StaticSx::new(|| {
    FieldDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .selector("& > [data-slot]", sx().color("muted.7"))
        .selector(
            "& label > [data-slot='required']",
            sx().color("error.7")
                .margin_left("2px")
                .rtl(sx().margin_left("0").margin_right("2px")),
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
        // A card is the inline layout drawn as a surface, and the whole of it
        // is the hit area. Paper's tokens, not a paint of its own. No
        // `--lsx-focus-contrast`: the only ring is the card's own, and it sits
        // outside the card, on whatever the card sits on.
        .when(
            "card",
            sx().background(PAPER_BACKGROUND.value())
                .border(format!("1px solid {}", PAPER_BORDER_COLOR.value()))
                .border_radius(PAPER_RADIUS.value())
                .cursor("pointer")
                // A card stretched by its row keeps its content at the top
                // rather than spreading the rows over the extra height.
                .align_content("start")
                .per_size(|size| sx().padding(FIELD_CARD_PADDING.value(size)))
                // The ring moves from the control to the card: the control
                // turns static under `card`, so its ring overlay covers this
                // box instead, out by the border as an outline would sit.
                .position("relative")
                .selector(
                    "& [data-state~=\"card\"] > [data-ring]",
                    sx().inset("-1px").border_radius(PAPER_RADIUS.value()),
                )
                .when("disabled", sx().cursor("not-allowed")),
        )
        // Not `opacity`: the control dims itself, and two stacked opacities
        // multiply.
        .when(
            "disabled",
            sx().color("muted.6")
                .selector("& > [data-slot]", sx().color("muted.6")),
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
    card: bool,
    labelled_by: bool,
    group: bool,
    label_with_id: bool,
    text_slots: [bool; 2],
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

    /// Draws the inline layout as a card: a bordered surface whose every
    /// click activates the control. Needs [`inline`](Self::inline) and
    /// [`activates`](Self::activates); without an activation the card is
    /// only drawn.
    #[inline]
    pub fn card(mut self, card: bool) -> Self {
        self.card = card;
        self
    }

    /// Names the control by `aria-labelledby` rather than `<label for>`, for a
    /// control `for` cannot name: an element with a `role`, or one another
    /// component owns. The field then hands the ids out instead of applying
    /// them - see [`PreparedField::label_id`].
    ///
    /// A click on the label focuses the control, as `<label for>` would.
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

    #[inline]
    pub fn states(mut self, states: &'a Input<States>) -> Self {
        self.states = Some(states);
        self
    }

    /// The caller's attributes, read for an `id` to adopt. A caller's own
    /// `aria-describedby` is not read here: it is merged with the field's in
    /// `styling_attributes`, so neither side loses its ids.
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
        let mut wrapper = wrapper.prepare();
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
        let own = use_hook(|| focuses.then(ElementHandle::new));
        let root = silent.or(own);
        if let Some(element) = &root {
            wrapper = wrapper.element(element);
        }
        // `focusout` bubbles, so one listener on the wrapper sees the control
        // lose focus whatever it is - and leaves a caller's own `onblur` alone.
        if touches {
            wrapper = wrapper.event("onfocusout", move |_: FocusEvent| touch());
        }

        PreparedField {
            label: label_node(
                &id_value,
                label,
                self.required,
                self.labelled_by,
                self.label_with_id,
                activation.clone(),
                root.filter(|_| focuses),
            ),
            labelled_by: (self.labelled_by || self.label_with_id) && !label.is_none(),
            description: slot_node("description", &id_value, description),
            helper: slot_node("helper", &id_value, helper),
            status: status_node(&id_value, status),
            id: id_value,
            text_slots: self.text_slots,
            describedby,
            invalid: matches!(status, Some(FieldStatus::Error(_))),
            required: self.required,
            inline: self.inline,
            activation,
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
    let entered = use_hook(|| Signal::new(None::<T>));
    let full = (!name.is_empty())
        .then(|| crate::components::form::validation_join(hook.binding.prefix(), name.as_str()));
    let active = !controlled && name.steps().is_some() && hook.binding.is_bound();
    Bound {
        name: active.then(|| name.clone()),
        hook,
        full,
        entered,
        owns: !controlled && !active,
    }
}

pub(crate) struct Bound<T> {
    hook: Rc<FieldHook>,
    /// The name, kept only while it binds.
    name: Option<FieldName<T>>,
    full: Option<String>,
    /// What the user last entered, for a field whose value lives nowhere else.
    /// `None` until the first edit.
    entered: Signal<Option<T>>,
    /// Whether `entered` is a value source at all: with a handler or a binding
    /// the value belongs to the caller or the form, and this is never read.
    owns: bool,
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
    /// bound and without one, a write into the form's value - and, with
    /// neither, a note of what the user entered, which is then the field's
    /// only value. See [`entered`](Self::entered).
    pub fn emit(&self, handler: Option<EventHandler<T>>) -> Option<impl Fn(T) + Clone + 'static> {
        let setter = self.setter();
        let entered = self.owns.then_some(self.entered);
        if handler.is_none() && setter.is_none() && entered.is_none() {
            return None;
        }
        Some(move |next: T| {
            // Copied out, so the closure stays an `Fn` - a handler a field
            // holds across renders cannot be `FnMut`.
            if let Some(mut entered) = entered {
                entered.set(Some(next.clone()));
            }
            match (&handler, &setter) {
                (Some(handler), _) => handler.call(next),
                (None, Some(setter)) => setter.set(next),
                (None, None) => {}
            }
        })
    }

    /// What the user last entered, for a field with no handler and no place in
    /// a form's value. Without it such a field has no value at all: its rules
    /// would judge `T::default()` for ever, and inside a `Form` that default
    /// cancels every submit.
    ///
    /// A control the browser keeps no state for - a checkbox, a switch - also
    /// *renders* from it, and so becomes usable uncontrolled. One that keeps
    /// its own text does not: writing the text back would move the caret.
    pub fn entered(&self) -> Option<T> {
        self.owns.then(|| self.entered.cloned()).flatten()
    }

    /// The status the field's own rules give it, over the value they must
    /// judge: the form's value when bound, else `value`, else what the user
    /// entered. Nothing is read without rules, so a field that has none never
    /// re-renders on a keystroke.
    pub fn check(&self, rules: &Validators<T>, value: Option<T>) -> Option<FieldStatus>
    where
        T: Default,
    {
        if rules.is_empty() {
            return None;
        }
        Some(rules.validate(&value.or_else(|| self.entered()).unwrap_or_default()))
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
    text_slots: [bool; 2],
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
    /// when there is no label, or when the label names the control by `for`
    /// alone - see [`FieldBuilder::label_with_id`].
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
/// input natively: the label's click is cancelled, and Space is answered on
/// `keydown`, which stops the activation outright. Enter is left to the
/// browser, which submits the form around it; outside a `Form`, a switch and
/// a segmented control take it too ([`enter_activates`](Self::enter_activates)).
///
/// A field gets it through [`FieldBuilder::activates`]; a control whose label
/// is its own - `Chip` - wires both halves itself, and one that renders its
/// inputs by hand - `SegmentedControl` - takes the input's handlers one by one.
#[derive(Clone)]
pub(crate) struct Activation {
    /// Attached to the input by [`wire`](Self::wire), and focused by the label.
    element: Option<ElementHandle>,
    /// Focuses the input instead, for a control with no handle per input.
    focus: Option<Rc<dyn Fn()>>,
    activate: Rc<dyn Fn()>,
    enter: bool,
    /// Inside a card, whose own click activates too: the label's and the
    /// input's clicks stop where they are, or they would activate twice.
    card: bool,
}

impl Activation {
    pub(crate) fn new(element: ElementHandle, activate: impl Fn() + 'static) -> Self {
        Self {
            element: Some(element),
            focus: None,
            activate: Rc::new(activate),
            enter: false,
            card: false,
        }
    }

    /// For inputs rendered in a loop, which cannot each take a handle: the
    /// label calls `focus` where it would focus the handle.
    pub(crate) fn focusing(focus: impl Fn() + 'static, activate: impl Fn() + 'static) -> Self {
        Self {
            element: None,
            focus: Some(Rc::new(focus)),
            activate: Rc::new(activate),
            enter: false,
            card: false,
        }
    }

    /// Enter activates as well as Space: a switch or segmented control
    /// outside a `Form` or raw `<form>`, where Enter has nothing to submit
    /// (todos 648, 660).
    pub(crate) fn enter_activates(mut self, enter: bool) -> Self {
        self.enter = enter;
        self
    }

    fn focus(&self) {
        match (&self.focus, &self.element) {
            (Some(focus), _) => focus(),
            (None, Some(element)) => {
                let _ = element.focus();
            }
            (None, None) => {}
        }
    }

    /// The label's half. Cancelling the label's click cancels its activation
    /// behaviour, which is the whole of "forward this click to the control":
    /// the input gets no click at all. The focus the forwarding gave it goes
    /// too, so it is given back by hand.
    pub(crate) fn label_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let activation = self.clone();
        move |event| {
            if nested_interactive(&event, CLICK_BOUNDARY) {
                return;
            }
            event.prevent_default();
            if activation.card {
                event.stop_propagation();
            }
            (activation.activate)();
            activation.focus();
        }
    }

    /// A card's half: a click anywhere on the card that neither the label nor
    /// the input took. Not cancelled - a card is a `div` and has no default
    /// action of its own to cancel.
    fn card_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let activation = self.clone();
        move |event| {
            if nested_interactive(&event, CLICK_BOUNDARY) {
                return;
            }
            (activation.activate)();
            activation.focus();
        }
    }

    /// The input's half: attaches the element, and takes Space and whatever
    /// click still reaches it.
    pub(crate) fn wire(&self, control: BoxStyle) -> BoxStyle {
        let keydown = self.clone();
        let control = match &self.element {
            Some(element) => control.element(element),
            None => control,
        };
        control
            .event("onclick", self.input_click())
            .event("oninput", self.input_input())
            .event("onkeydown", move |event: Event<KeyboardData>| {
                keydown.keydown(&event);
            })
            .event("onkeyup", self.input_keyup())
    }

    /// What still reaches the input as a click - assistive tech's default
    /// action, a click on the input itself - is cancelled, and taken in Rust
    /// only once the dispatch is over. The cancelled activation restores the
    /// old `checked` at the end of the dispatch, so a re-render inside it
    /// would be overwritten; after it, the new `checked` is the last write.
    pub(crate) fn input_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let (activate, card) = (self.activate.clone(), self.card);
        move |event| {
            event.prevent_default();
            if card {
                event.stop_propagation();
            }
            let activate = activate.clone();
            spawn(async move {
                next_task().await;
                activate();
            });
        }
    }

    /// Blitz forwards a `<label>` click to its input as a default action that
    /// emits `input`, never `click`. On the web nothing activates the input
    /// any more, so this never fires there; both compute the same next value,
    /// so firing twice is a no-op.
    pub(crate) fn input_input(&self) -> impl FnMut(FormEvent) + 'static {
        let activate = self.activate.clone();
        move |_| activate()
    }

    /// Space - and Enter, outside a `Form` - answered on `keydown`, which
    /// stops the activation outright. `true` when the key was one of those, so
    /// a control with keys of its own knows it is handled.
    pub(crate) fn keydown(&self, event: &Event<KeyboardData>) -> bool {
        if !activates(event, self.enter) {
            return false;
        }
        event.prevent_default();
        if !event.is_auto_repeating() {
            (self.activate)();
        }
        true
    }

    /// A browser that clicks on `keyup` rather than checking the cancelled
    /// `keydown` would otherwise activate a second time.
    pub(crate) fn input_keyup(&self) -> impl FnMut(Event<KeyboardData>) + 'static {
        let enter = self.enter;
        move |event| {
            if activates(&event, enter) {
                event.prevent_default();
            }
        }
    }
}

/// What a label or a card leaves to a link or button nested in it: the
/// label, or the card's wrapper - not the control `<span>`, which carries
/// `card` too.
const CLICK_BOUNDARY: &str = "label, div[data-state~=\"card\"]";

fn activates(event: &KeyboardData, enter: bool) -> bool {
    match event.key() {
        Key::Character(ref c) => c == " ",
        Key::Enter => enter,
        _ => false,
    }
}

fn attribute_text(attributes: &[Attribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .rev()
        .find_map(|attribute| match (attribute.name, &attribute.value) {
            (found, AttributeValue::Text(value)) if found == name => Some(value.clone()),
            _ => None,
        })
}

pub(super) fn join_ids<const N: usize>(
    id: &str,
    slots: [(&'static str, bool); N],
) -> Option<String> {
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

pub(super) fn caption_content(caption: &Caption) -> Element {
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
    with_id: bool,
    activation: Option<Activation>,
    focuses: Option<ElementHandle>,
) -> Option<Element> {
    if label.is_none() {
        return None;
    }

    // `for` names a labelable element. A control that is not one - a `role` on
    // a span, or an element another component owns - is named the other way
    // round, by an id the control points at.
    let named = (labelled_by || with_id).then(|| format!("{id}-label"));
    let points_at = (!labelled_by).then(|| id.to_string());
    let content = caption_content(label);
    // `aria-required` already tells AT; the asterisk is decoration.
    let asterisk = required.then(|| {
        rsx! {
            span { "aria-hidden": "true", "data-slot": "required", "*" }
        }
    });
    // One arm per listener, because a listener cannot be optional and most
    // labels need none.
    Some(match (activation, focuses) {
        (Some(activation), _) => rsx! {
            label {
                id: named,
                r#for: points_at,
                onclick: activation.label_click(),
                {content}
                {asterisk}
            }
        },
        (None, Some(root)) => rsx! {
            label {
                id: named.clone(),
                onclick: focus_labelled(root, named.unwrap_or_default()),
                {content}
                {asterisk}
            }
        },
        (None, None) => rsx! {
            label { id: named, r#for: points_at,
                {content}
                {asterisk}
            }
        },
    })
}

/// The label's click for a control it names by id: focuses the tab stop that
/// id names - the control, or its first input (`PinField`'s group).
fn focus_labelled(root: ElementHandle, label_id: String) -> impl FnMut(Event<MouseData>) + 'static {
    let named = format!("[aria-labelledby~=\"{label_id}\"]");
    let selector = format!(
        "{named}[tabindex=\"0\"], {named} input:not([type=\"hidden\"]):not(:disabled):not([tabindex=\"-1\"])"
    );
    move |event| {
        if nested_interactive(&event, CLICK_BOUNDARY) {
            return;
        }
        if let Ok(control) = root.query_selector(&selector) {
            let _ = control.focus();
        }
    }
}

pub(super) fn slot_node(slot: &'static str, id: &str, caption: &Caption) -> Option<Element> {
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

pub(super) fn status_node(id: &str, status: Option<&FieldStatus>) -> Option<Element> {
    let message = status.and_then(FieldStatus::message)?;

    Some(rsx! {
        span { "data-slot": "status", id: "{id}-status", "{message}" }
    })
}
