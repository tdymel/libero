use std::{any::Any, rc::Rc};

use dioxus::prelude::*;

use super::path::{Step, join, resolve, resolve_mut};

/// A form's value with its type erased, so a field typed only by its own
/// value can read and write its place in it.
pub(crate) trait Source {
    fn read_value(&self, reader: &mut dyn FnMut(&dyn Any));
    fn write_value(&self, writer: &mut dyn FnMut(&mut dyn Any));
}

impl<V: 'static> Source for Signal<V> {
    /// Subscribes the reader's scope, like any signal read.
    fn read_value(&self, reader: &mut dyn FnMut(&dyn Any)) {
        reader(&*self.read());
    }

    fn write_value(&self, writer: &mut dyn FnMut(&mut dyn Any)) {
        writer(&mut *self.clone().write());
    }
}

/// What a `Form` or `Fieldset` shares so the fields inside bind by `name`: the
/// value, where inside it this scope sits, and the prefix names post under.
#[derive(Clone, Default)]
pub(crate) struct Binding {
    source: Option<Rc<dyn Source>>,
    steps: Vec<Step>,
    prefix: String,
}

impl Binding {
    pub fn root(source: Option<Rc<dyn Source>>) -> Self {
        Self {
            source,
            steps: Vec::new(),
            prefix: String::new(),
        }
    }

    /// The scope a `Fieldset` opens at `path`. Without steps of its own - a
    /// plain string path - only the names move, and the fieldset's fields
    /// cannot bind.
    pub fn narrow(&self, path: &str, steps: Option<&[Step]>) -> Self {
        Self {
            source: steps.and(self.source.clone()),
            steps: [&self.steps, steps.unwrap_or_default()].concat(),
            prefix: join(&self.prefix, path),
        }
    }

    /// A fieldset with a value of its own: names still move under `path`,
    /// but the fields read and write that value.
    pub fn rebased(&self, path: &str, source: Rc<dyn Source>) -> Self {
        Self {
            source: Some(source),
            steps: Vec::new(),
            prefix: join(&self.prefix, path),
        }
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn is_bound(&self) -> bool {
        self.source.is_some()
    }

    /// Runs `reader` on the value at `steps`, when it is a `T`. `None` when
    /// nothing is bound or the path does not fit. Subscribes the caller.
    pub fn with<T: 'static, R>(&self, steps: &[Step], reader: impl FnOnce(&T) -> R) -> Option<R> {
        let source = self.source.as_ref()?;
        let mut reader = Some(reader);
        let mut result = None;
        source.read_value(&mut |root| {
            let value = resolve(root, &self.steps)
                .and_then(|scope| resolve(scope, steps))
                .and_then(|value| value.downcast_ref::<T>());
            if let (Some(value), Some(reader)) = (value, reader.take()) {
                result = Some(reader(value));
            }
        });
        result
    }

    /// Puts `next` at `steps`. `false` when nothing is bound or the path does
    /// not fit.
    pub fn set<T: 'static>(&self, steps: &[Step], next: T) -> bool {
        let Some(source) = self.source.as_ref() else {
            return false;
        };
        let mut next = Some(next);
        source.write_value(&mut |root| {
            let slot = resolve_mut(root, &self.steps)
                .and_then(|scope| resolve_mut(scope, steps))
                .and_then(|value| value.downcast_mut::<T>());
            if let (Some(slot), Some(next)) = (slot, next.take()) {
                *slot = next;
            }
        });
        next.is_none()
    }
}

/// What a disabled `Fieldset` shares with everything inside it, nested
/// fieldsets included. A signal, so fields whose props compare equal still
/// follow a toggle.
#[derive(Clone, Copy)]
pub(crate) struct Disabled(pub Signal<bool>);
