use std::{
    cell::{Cell, OnceCell},
    rc::Rc,
};

use dioxus::{
    core::{ScopeId, current_scope_id},
    prelude::*,
};

use crate::components::form::{
    Adapt, Binding, Checking, Disabled, Draft, FieldName, FieldStatus, FormScope, Parse,
    Validators, locale, word, worst,
};

/// What a field keeps across renders for validation and binding. `touched`
/// re-renders its owner by hand; nothing else reads it.
pub(crate) struct FieldHook {
    pub(super) touched: Cell<bool>,
    /// The form's reset generation this field last rendered at.
    pub(super) generation: Cell<u32>,
    /// Whether a `name` that does not fit the form's value was warned about.
    pub(super) warned: Cell<bool>,
    /// Whether a blank status message was warned about (todo 2342).
    pub(super) warned_blank: Cell<bool>,
    pub(super) owner: ScopeId,
    pub(super) form: Option<(FormScope, usize)>,
    pub(super) binding: Binding,
    pub(super) disabled: Option<Disabled>,
    /// A mapped name's async rules, made on the first render that has some.
    checking: OnceCell<Rc<Checking>>,
}

impl FieldHook {
    fn checking(&self) -> &Rc<Checking> {
        self.checking
            .get_or_init(|| Checking::new(self.owner, self.form))
    }

    pub(super) fn new() -> Rc<Self> {
        let form = try_consume_context::<FormScope>().map(|mut scope| (scope, scope.key()));
        Rc::new(Self {
            touched: Cell::new(false),
            generation: Cell::new(form.map_or(0, |(scope, _)| scope.generation())),
            warned: Cell::new(false),
            warned_blank: Cell::new(false),
            owner: current_scope_id(),
            form,
            binding: try_consume_context::<Binding>().unwrap_or_default(),
            disabled: try_consume_context::<Disabled>(),
            checking: OnceCell::new(),
        })
    }
}

/// Resolves a field's `name` against the enclosing `Form` or `Fieldset`: its
/// full name, and its bound value. A hook; hand it to `FieldBuilder::bound`.
pub(crate) fn use_bound<T: Clone + 'static>(name: &FieldName<T>, controlled: bool) -> Bound<T> {
    let hook = use_hook(FieldHook::new);
    let entered = use_hook(|| Signal::new(None::<(u32, T)>));
    let full = (!name.is_empty())
        .then(|| crate::components::form::validation_join(hook.binding.prefix(), name.as_str()));
    let active = !controlled && name.steps().is_some() && hook.binding.is_bound();
    let mapped = active
        .then(|| name.adapter().zip(name.steps()))
        .flatten()
        .and_then(|(adapter, steps)| read_mapped(&hook, &**adapter, steps, full.as_deref()));
    Bound {
        name: active.then(|| name.clone()),
        hook,
        full,
        entered,
        owns: !controlled && !active,
        mapped,
    }
}

/// A mapped field's text and state: its draft while the store still holds
/// what the draft wrote, else the stored value's own text.
fn read_mapped<T: Clone + 'static>(
    hook: &FieldHook,
    adapter: &dyn Adapt<T>,
    steps: &[crate::components::form::Step],
    full: Option<&str>,
) -> Option<Mapped<T>> {
    let scope = hook.form.map(|(scope, _)| scope).zip(full);
    hook.binding
        .with_slot(steps, |slot| {
            let draft = scope.and_then(|(scope, full)| scope.draft(full));
            let current = draft
                .as_ref()
                .filter(|draft| adapter.holds(slot, &*draft.wrote))
                .and_then(|draft| {
                    Some((draft.text.downcast_ref::<T>()?.clone(), draft.parse.clone()))
                });
            if let (None, Some(_), Some((mut scope, full))) = (&current, &draft, scope) {
                // Set from elsewhere - a reset, code, a load: the stored value speaks again.
                scope.drop_draft(full);
            }
            let (shown, parse) = match current {
                Some(current) => current,
                None => (adapter.read(slot)?, adapter.state(slot)),
            };
            let status = match &parse {
                Parse::Invalid(error) => Some(FieldStatus::Error(word(&**error))),
                Parse::Valid => adapter.check(slot, locale()),
                Parse::Empty => None,
            };
            // The async rules ask only about a value the other rules let through.
            let asks = parse == Parse::Valid && !status.as_ref().is_some_and(FieldStatus::is_error);
            let status = match (adapter.checks(), asks) {
                (false, _) => status,
                (true, true) => {
                    let answer = adapter.check_async(slot, hook.checking(), locale());
                    match (status, answer) {
                        (Some(status), Some(answer)) => Some(worst(status, answer)),
                        (status, answer) => status.or(answer),
                    }
                }
                (true, false) => {
                    hook.checking().idle();
                    status
                }
            };
            Some(Mapped {
                shown,
                parse,
                status,
            })
        })
        .flatten()
}

/// What a mapped field shows and where its text stands.
pub(crate) struct Mapped<T> {
    shown: T,
    pub(super) parse: Parse,
    /// The parse error, else the `.rules` over the domain value.
    pub(crate) status: Option<FieldStatus>,
}

pub(crate) struct Bound<T> {
    pub(super) hook: Rc<FieldHook>,
    /// The name, kept only while it binds.
    pub(super) name: Option<FieldName<T>>,
    pub(super) full: Option<String>,
    /// What the user last entered, for a field whose value lives nowhere else, with the
    /// form's reset generation it was entered at: a reset forgets it (4377).
    pub(super) entered: Signal<Option<(u32, T)>>,
    /// Whether `entered` is a value source: not with a handler or a binding.
    pub(super) owns: bool,
    /// For a name mapped to a domain type.
    pub(crate) mapped: Option<Mapped<T>>,
}

impl<T> Bound<T> {
    /// The full name the field posts as - under every enclosing fieldset.
    pub fn name(&self) -> Option<&str> {
        self.full.as_deref()
    }

    /// The field's own `disabled`, or'd with every enclosing disabled `Fieldset`.
    pub fn disabled(&self, own: Option<bool>) -> bool {
        own.unwrap_or(false) || self.hook.disabled.is_some_and(|Disabled(group)| group())
    }

    /// Whether the form's value drives the field.
    pub fn is_bound(&self) -> bool {
        self.name.is_some()
    }

    /// A mapped name's parse error or domain rules, and its parse state.
    pub fn mapped_state(&self) -> Option<super::builder::MappedState> {
        self.mapped
            .as_ref()
            .map(|mapped| (mapped.status.clone(), mapped.parse.clone()))
    }

    /// What an edit calls: drops an error set from code on the field's name.
    pub fn edited(&self) -> impl Fn() + Clone + 'static {
        let form = self.hook.form.map(|(scope, _)| scope);
        let full = self.full.clone();
        move || clear_set_error(form, full.as_deref())
    }
}

/// The form's reset generation; 0 outside a form.
fn generation(form: Option<(FormScope, usize)>) -> u32 {
    form.map_or(0, |(scope, _)| scope.generation())
}

fn clear_set_error(form: Option<FormScope>, full: Option<&str>) {
    if let (Some(mut scope), Some(full)) = (form, full) {
        scope.clear_error(full);
    }
}

impl<T: Clone + 'static> Bound<T> {
    /// The field's value inside the form's value, when bound. Subscribes the
    /// field to the form's value.
    pub fn value(&self) -> Option<T> {
        if let Some(mapped) = &self.mapped {
            return Some(mapped.shown.clone());
        }
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
            full: self.full.clone(),
        })
    }

    /// What the field calls with its next value: the `handler`, else a bound
    /// write, else a note in [`entered`](Self::entered).
    pub fn emit(&self, handler: Option<EventHandler<T>>) -> Option<impl Fn(T) + Clone + 'static> {
        let setter = self.setter();
        let entered = self.owns.then_some(self.entered);
        if handler.is_none() && setter.is_none() && entered.is_none() {
            return None;
        }
        let edited = self.edited();
        let form = self.hook.form;
        Some(move |next: T| {
            // Copied out, so the closure stays an `Fn`.
            if let Some(mut entered) = entered {
                entered.set(Some((generation(form), next.clone())));
            }
            edited();
            match (&handler, &setter) {
                (Some(handler), _) => handler.call(next),
                (None, Some(setter)) => setter.set(next),
                (None, None) => {}
            }
        })
    }

    /// Clears what was entered, so a form reset draws `value` again.
    pub(crate) fn forget_entered(&self) -> impl Fn() + Copy + 'static {
        let entered = self.entered;
        move || {
            let mut entered = entered;
            if entered.peek().is_some() {
                entered.set(None);
            }
        }
    }

    /// What the user last entered, for a field with no handler and no binding,
    /// so its rules don't judge `T::default()` for ever.
    pub fn entered(&self) -> Option<T> {
        if !self.owns {
            return None;
        }
        // Subscribes, so a field the form does not re-render (a `Chip`) hears a reset too.
        if let Some((scope, _)) = self.hook.form {
            scope.resets();
        }
        match &*self.entered.read() {
            Some((at, value)) if *at == generation(self.hook.form) => Some(value.clone()),
            _ => None,
        }
    }

    /// Whether a `required` field inside a form holds nothing: `shown` is what it draws, else what
    /// was entered. Reads nothing otherwise; a value it cannot see (a handler, no `value`) is filled.
    pub fn is_empty(
        &self,
        required: bool,
        shown: impl FnOnce() -> Option<T>,
        empty: impl Fn(&T) -> bool,
    ) -> bool {
        if !required || self.hook.form.is_none() {
            return false;
        }
        if let Some(mapped) = &self.mapped {
            return mapped.parse == Parse::Empty;
        }
        match shown().or_else(|| self.entered()) {
            Some(value) => empty(&value),
            None => self.owns,
        }
    }

    /// The status the field's rules give `value`, else what was entered. Reads
    /// nothing without rules, so no re-render per keystroke; none for a value it cannot see.
    pub fn check(&self, rules: &Validators<T>, value: Option<T>) -> Option<FieldStatus>
    where
        T: Default,
    {
        if rules.is_empty() || (!self.owns && value.is_none()) {
            return None;
        }
        Some(rules.validate(&value.or_else(|| self.entered()).unwrap_or_default()))
    }
}

/// A bound field's write into its form's value.
pub(crate) struct Setter<T> {
    hook: Rc<FieldHook>,
    name: FieldName<T>,
    full: Option<String>,
}

impl<T> Clone for Setter<T> {
    fn clone(&self) -> Self {
        Self {
            hook: self.hook.clone(),
            name: self.name.clone(),
            full: self.full.clone(),
        }
    }
}

impl<T: Clone + 'static> Setter<T> {
    pub fn set(&self, next: T) {
        clear_set_error(self.hook.form.map(|(scope, _)| scope), self.full.as_deref());
        let steps = self.name.steps().unwrap_or_default();
        let Some(adapter) = self.name.adapter() else {
            self.hook.binding.set(steps, next);
            return;
        };
        // Parsed into the store; the text itself is kept as typed.
        let written = self
            .hook
            .binding
            .set_slot(steps, |slot| adapter.write(slot, next.clone()))
            .flatten();
        if let (Some((parse, wrote)), Some((mut scope, _)), Some(full)) =
            (written, self.hook.form, &self.full)
        {
            scope.set_parse(full, parse.clone());
            let text = Rc::new(next);
            scope.keep_draft(full, Draft { text, wrote, parse });
        }
    }
}

impl Drop for FieldHook {
    fn drop(&mut self) {
        if let Some(checking) = self.checking.get() {
            checking.release();
        }
        if let Some((mut scope, key)) = self.form {
            scope.unregister(key);
        }
    }
}
