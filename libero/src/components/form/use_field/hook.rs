use std::{cell::Cell, rc::Rc};

use dioxus::{
    core::{ScopeId, current_scope_id},
    prelude::*,
};

use crate::components::form::{Binding, Disabled, FieldName, FieldStatus, FormScope, Validators};

/// What a field keeps across renders for validation and binding. `touched`
/// re-renders its owner by hand; nothing else reads it.
pub(crate) struct FieldHook {
    pub(super) touched: Cell<bool>,
    /// The form's reset generation this field last rendered at.
    pub(super) generation: Cell<u32>,
    /// Whether a `name` that does not fit the form's value was warned about.
    pub(super) warned: Cell<bool>,
    pub(super) owner: ScopeId,
    pub(super) form: Option<(FormScope, usize)>,
    pub(super) binding: Binding,
    pub(super) disabled: Option<Disabled>,
}

impl FieldHook {
    pub(super) fn new() -> Rc<Self> {
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

/// Resolves a field's `name` against the enclosing `Form` or `Fieldset`: its
/// full name, and its bound value. A hook; hand it to [`FieldBuilder::bound`].
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
    pub(super) hook: Rc<FieldHook>,
    /// The name, kept only while it binds.
    pub(super) name: Option<FieldName<T>>,
    pub(super) full: Option<String>,
    /// What the user last entered, for a field whose value lives nowhere else.
    pub(super) entered: Signal<Option<T>>,
    /// Whether `entered` is a value source: not with a handler or a binding.
    pub(super) owns: bool,
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

    /// What the field calls with its next value: the `handler`, else a bound
    /// write, else a note in [`entered`](Self::entered).
    pub fn emit(&self, handler: Option<EventHandler<T>>) -> Option<impl Fn(T) + Clone + 'static> {
        let setter = self.setter();
        let entered = self.owns.then_some(self.entered);
        if handler.is_none() && setter.is_none() && entered.is_none() {
            return None;
        }
        Some(move |next: T| {
            // Copied out, so the closure stays an `Fn`.
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

    /// What the user last entered, for a field with no handler and no binding,
    /// so its rules don't judge `T::default()` for ever.
    pub fn entered(&self) -> Option<T> {
        self.owns.then(|| self.entered.cloned()).flatten()
    }

    /// The status the field's rules give `value`, else what was entered. Reads
    /// nothing without rules, so no re-render per keystroke.
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
