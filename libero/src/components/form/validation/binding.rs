use std::{any::Any, rc::Rc};

use dioxus::{prelude::*, stores::scope::SelectorScope};

use super::path::{Step, StepKey, join, resolve, resolve_mut};

/// A form's value, type-erased. The path comes in two parts (scope, field) to
/// spare a joined copy per render.
pub(crate) trait Source {
    /// Subscribes the caller to the value at the path and everything under it.
    fn read_value(&self, path: [&[Step]; 2], reader: &mut dyn FnMut(&dyn Any));
    /// Re-renders whoever reads the value at the path, under it, or the whole
    /// of anything above it - never a sibling.
    fn write_value(&self, path: [&[Step]; 2], writer: &mut dyn FnMut(&mut dyn Any));
}

impl<V: 'static> Source for Store<V> {
    fn read_value(&self, path: [&[Step]; 2], reader: &mut dyn FnMut(&dyn Any)) {
        selector_at(self, path).track();
        reader(&*self.peek());
    }

    fn write_value(&self, path: [&[Step]; 2], writer: &mut dyn FnMut(&mut dyn Any)) {
        let at = selector_at(self, path);
        at.mark_dirty();
        writer(&mut *at.write_untracked());
    }
}

/// The store's subscription node for the path.
fn selector_at<V: 'static>(store: &Store<V>, path: [&[Step]; 2]) -> SelectorScope<WriteSignal<V>> {
    path.iter()
        .flat_map(|steps| steps.iter())
        .fold(*store.selector(), |scope, step| match step.key {
            StepKey::Index(index) => scope.child_unmapped(index),
            StepKey::Name(name) => scope.hash_child_unmapped(&name),
        })
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

    /// The scope a `Fieldset` opens at `path`. A plain string path moves only
    /// the names; its fields cannot bind.
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
        self.with_slot(steps, |slot| slot.downcast_ref::<T>().map(reader))
            .flatten()
    }

    /// [`with`](Self::with), on the value at `steps` whatever its type.
    pub fn with_slot<R>(&self, steps: &[Step], reader: impl FnOnce(&dyn Any) -> R) -> Option<R> {
        let source = self.source.as_ref()?;
        let mut reader = Some(reader);
        let mut result = None;
        source.read_value([&self.steps, steps], &mut |root| {
            let value = resolve(root, &self.steps).and_then(|scope| resolve(scope, steps));
            if let (Some(value), Some(reader)) = (value, reader.take()) {
                result = Some(reader(value));
            }
        });
        result
    }

    /// Puts `next` at `steps`. `false` when nothing is bound or the path does
    /// not fit.
    pub fn set<T: 'static>(&self, steps: &[Step], next: T) -> bool {
        self.set_slot(steps, |slot| {
            slot.downcast_mut::<T>().map(|slot| *slot = next)
        })
        .flatten()
        .is_some()
    }

    /// [`set`](Self::set), through `writer` on the value at `steps` whatever its type.
    pub fn set_slot<R>(&self, steps: &[Step], writer: impl FnOnce(&mut dyn Any) -> R) -> Option<R> {
        let source = self.source.as_ref()?;
        let mut writer = Some(writer);
        let mut result = None;
        source.write_value([&self.steps, steps], &mut |root| {
            let slot = resolve_mut(root, &self.steps).and_then(|scope| resolve_mut(scope, steps));
            if let (Some(slot), Some(writer)) = (slot, writer.take()) {
                result = Some(writer(slot));
            }
        });
        result
    }
}

/// A disabled `Fieldset`'s state for everything inside. A signal, so memoized
/// fields still follow a toggle.
#[derive(Clone, Copy)]
pub(crate) struct Disabled(pub Signal<bool>);

/// A required `Fieldset`'s state for the fields inside; a field's own `required` wins.
#[derive(Clone, Copy)]
pub(crate) struct Required(pub Signal<bool>);

/// A field's own `required`, else the nearest enclosing `Fieldset`'s. No hook: callable anywhere.
pub(crate) fn fieldset_required(own: Option<bool>) -> bool {
    own.unwrap_or_else(|| try_consume_context::<Required>().is_some_and(|Required(group)| group()))
}

/// Whether an enclosing `Fieldset` is disabled. A hook, for a control that is no field.
pub(crate) fn use_fieldset_disabled() -> bool {
    use_hook(try_consume_context::<Disabled>).is_some_and(|Disabled(group)| group())
}
