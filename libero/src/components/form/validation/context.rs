use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
};

use dioxus::{
    core::{Runtime, ScopeId, current_scope_id},
    prelude::*,
};

use crate::{
    components::form::{FieldStatus, Validators, worst},
    utils::warn,
};

/// What a `Form` shares with the fields and fieldsets inside it. A lone
/// `Fieldset` opens its own.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct FormScope {
    fields: Signal<BTreeMap<usize, FieldEntry>, UnsyncStorage>,
    /// Composite issues by the key of the `Form` or `Fieldset` that raised them.
    issues: Signal<BTreeMap<usize, Vec<Issue>>>,
    /// Names of fields that lost focus once; a composite issue waits for all it names.
    touched: Signal<BTreeSet<String>>,
    submitted: Signal<bool>,
    /// Bumped by a reset; a field drops its own touched flag when it moved.
    generation: Signal<u32>,
    /// Bumped by every submit and every reset, not only the first submit.
    settled: Signal<u32>,
    /// Bumped by every reset, native reset button included.
    resets: Signal<u32>,
    next_key: Signal<usize>,
    /// The scope that created the signals above. Cleanup runs inside it, or dioxus
    /// warns of a copy value used outside its owner.
    owner: ScopeId,
}

/// One failing composite rule.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Issue {
    /// Full paths of the fields it concerns; empty when it names none.
    pub paths: Vec<String>,
    pub status: FieldStatus,
}

/// A registered field as the summary needs it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FieldEntry {
    pub id: String,
    pub label: Option<String>,
    pub name: Option<String>,
    /// The field's own status - its explicit `status` and its rules - whether
    /// or not it shows yet. Composite issues are kept apart.
    pub status: FieldStatus,
    /// The explicit `status` alone is an error: a server's answer, which the summary follows (2573).
    pub explicit_error: bool,
    /// The field's scope, which a reset re-renders.
    pub owner: ScopeId,
}

/// One line of the error summary.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SummaryItem {
    pub message: String,
    /// The id of the field to jump to, if the line concerns one.
    pub target: Option<String>,
}

impl FormScope {
    pub fn new() -> Self {
        Self {
            fields: Signal::new(BTreeMap::new()),
            issues: Signal::new(BTreeMap::new()),
            touched: Signal::new(BTreeSet::new()),
            submitted: Signal::new(false),
            generation: Signal::new(0),
            settled: Signal::new(0),
            resets: Signal::new(0),
            next_key: Signal::new(0),
            owner: current_scope_id(),
        }
    }

    pub fn key(&mut self) -> usize {
        let key = *self.next_key.peek();
        *self.next_key.write_unchecked() = key + 1;
        key
    }

    /// Reactive: every field re-renders on the first submit.
    pub fn submitted(&self) -> bool {
        (self.submitted)()
    }

    pub fn submit(&mut self) {
        if !*self.submitted.peek() {
            self.submitted.set(true);
        }
    }

    /// Runs drop-time cleanup as the scope that created these signals.
    pub fn in_owner(&self, cleanup: impl FnOnce()) {
        in_owner(self.owner, cleanup);
    }

    /// Which reset the form is at. Not reactive - a reset re-renders the fields.
    pub fn generation(&self) -> u32 {
        *self.generation.peek()
    }

    /// How many submits and resets the form has seen. Reactive, so what a
    /// field shows only while it is edited - a revealed password - can end.
    pub fn settled(&self) -> u32 {
        (self.settled)()
    }

    /// Counts a submit or a reset - see [`settled`](Self::settled).
    pub fn settle(&mut self) {
        self.settled += 1;
    }

    /// How many resets the form has seen, native ones included. Reactive, so
    /// what a field tracks from input events alone - a count - can start over.
    pub fn resets(&self) -> u32 {
        (self.resets)()
    }

    /// Counts a reset of the form's controls, and settles.
    pub fn count_reset(&mut self) {
        self.settle();
        self.resets += 1;
    }

    /// Back to pristine: nothing touched, not submitted, and every registered
    /// field re-rendered so it drops its own touched flag.
    pub fn reset(&mut self) {
        self.count_reset();
        if !self.touched.peek().is_empty() {
            self.touched.write().clear();
        }
        if *self.submitted.peek() {
            self.submitted.set(false);
        }
        *self.generation.write_unchecked() += 1;
        let runtime = Runtime::current();
        for field in self.fields.peek().values() {
            runtime.needs_update(field.owner);
        }
    }

    pub fn touch(&mut self, name: &str) {
        if !self.touched.peek().contains(name) {
            self.touched.write().insert(name.to_string());
        }
    }

    /// Replaces the issues one `Form` or `Fieldset` raised. Written during its
    /// render, so it only writes on a change - the raiser never reads it back.
    pub fn raise(&mut self, key: usize, issues: Vec<Issue>) {
        let unchanged = match self.issues.peek().get(&key) {
            Some(current) => *current == issues,
            None => issues.is_empty(),
        };
        if !unchanged {
            self.issues.write().insert(key, issues);
        }
    }

    /// Called on drop - see `owner`.
    pub fn withdraw(&mut self, key: usize) {
        let mut issues = self.issues;
        in_owner(self.owner, || {
            if issues
                .try_peek()
                .is_ok_and(|issues| issues.contains_key(&key))
            {
                issues.write().remove(&key);
            }
        });
    }

    /// The worst composite issue naming `name` that may show: after a submit,
    /// or once every field it names is touched. Reactive, but on `touched` only while
    /// an issue names `name`: a blur elsewhere re-renders no other field.
    pub fn visible_issue(&self, name: &str) -> FieldStatus {
        let issues = self.issues.read();
        let mut naming = issues
            .values()
            .flatten()
            .filter(|issue| issue.paths.iter().any(|path| path == name))
            .peekable();
        if naming.peek().is_none() {
            return FieldStatus::Valid;
        }
        let submitted = self.submitted();
        let touched = self.touched.read();
        naming
            .filter(|issue| submitted || issue.paths.iter().all(|path| touched.contains(path)))
            .fold(FieldStatus::Valid, |status, issue| {
                worst(status, issue.status.clone())
            })
    }

    /// The worst composite issue raised by `key` that names no field - what a
    /// `Fieldset` shows in its own status slot.
    pub fn unnamed_issue(&self, key: usize) -> FieldStatus {
        self.issues
            .read()
            .get(&key)
            .into_iter()
            .flatten()
            .filter(|issue| issue.paths.is_empty())
            .fold(FieldStatus::Valid, |status, issue| {
                worst(status, issue.status.clone())
            })
    }

    /// Whether any field at or under `prefix` is touched. Reactive.
    pub fn touched_under(&self, prefix: &str) -> bool {
        self.touched.read().iter().any(|name| {
            prefix.is_empty() || name == prefix || name.starts_with(&format!("{prefix}."))
        })
    }

    /// Kept in a signal nobody subscribes to during render: only the submit
    /// handler reads it, so a field updating its entry re-renders nothing.
    pub fn register(&mut self, key: usize, entry: FieldEntry) {
        if self.fields.peek().get(&key) != Some(&entry) {
            self.fields.write_unchecked().insert(key, entry);
        }
    }

    /// Tolerates a form already gone. Forgets the touched name unless another
    /// field carries it, so a remount waits for its own blur.
    pub fn unregister(&mut self, key: usize) {
        let fields = self.fields;
        let mut touched = self.touched;
        in_owner(self.owner, || {
            let present = fields
                .try_peek()
                .is_ok_and(|fields| fields.contains_key(&key));
            if !present {
                return;
            }
            let name = fields
                .write_unchecked()
                .remove(&key)
                .and_then(|entry| entry.name);
            let Some(name) = name else {
                return;
            };
            let shared = fields
                .peek()
                .values()
                .any(|field| field.name.as_deref() == Some(name.as_str()));
            if !shared
                && touched
                    .try_peek()
                    .is_ok_and(|touched| touched.contains(&name))
            {
                touched.write().remove(&name);
            }
        });
    }

    /// Whether anything blocks a submit: an error on a field, or an error
    /// raised by a composite rule. Warnings never block.
    pub fn has_errors(&self) -> bool {
        errors_in(&self.fields.peek(), &self.issues.peek())
    }

    /// [`has_errors`](Self::has_errors), but subscribing the caller to every
    /// field's status and every composite issue.
    pub fn has_errors_tracked(&self) -> bool {
        errors_in(&self.fields.read(), &self.issues.read())
    }

    /// Whether a field's explicit `status` is an error, subscribing the caller to every field.
    pub fn explicit_error_tracked(&self) -> bool {
        self.fields
            .read()
            .values()
            .any(|field| field.explicit_error)
    }

    /// One line per field error, in registration order, then one per
    /// composite error - each linked to the first field it names.
    pub fn summary(&self) -> Vec<SummaryItem> {
        let fields = self.fields.peek();
        let id_of = |name: &str| {
            fields
                .values()
                .find(|field| field.name.as_deref() == Some(name))
                .map(|field| field.id.clone())
        };

        let mut items: Vec<SummaryItem> = fields
            .values()
            .filter_map(|field| {
                let message = field.status.is_error().then(|| field.status.message())??;
                Some(SummaryItem {
                    message: match &field.label {
                        Some(label) => format!("{label}: {message}"),
                        None => message.to_string(),
                    },
                    target: Some(field.id.clone()),
                })
            })
            .collect();

        for (key, issues) in self.issues.peek().iter() {
            for issue in issues {
                let FieldStatus::Error(message) = &issue.status else {
                    continue;
                };
                let item = match issue.paths.first() {
                    Some(path) => SummaryItem {
                        message: message.clone(),
                        target: id_of(path),
                    },
                    // A `Fieldset`'s own rule: its entry shares the key, so the line goes to the group.
                    None => match fields.get(key) {
                        Some(group) => SummaryItem {
                            message: match &group.label {
                                Some(label) => format!("{label}: {message}"),
                                None => message.clone(),
                            },
                            target: Some(group.id.clone()),
                        },
                        None => SummaryItem {
                            message: message.clone(),
                            target: None,
                        },
                    },
                };
                items.push(item);
            }
        }
        items
    }

    /// Debug builds warn about a composite rule naming a path no registered
    /// field carries - almost always a field missing its `name`.
    pub fn warn_unknown_paths(&self) {
        if !cfg!(debug_assertions) {
            return;
        }
        let fields = self.fields.peek();
        for issue in self.issues.peek().values().flatten() {
            for path in &issue.paths {
                if !fields
                    .values()
                    .any(|field| field.name.as_deref() == Some(path.as_str()))
                {
                    warn!(
                        "a validation rule names `{path}`, but no field inside the form has that `name`"
                    );
                }
            }
        }
    }
}

fn errors_in(fields: &BTreeMap<usize, FieldEntry>, issues: &BTreeMap<usize, Vec<Issue>>) -> bool {
    fields.values().any(|field| field.status.is_error())
        || issues
            .values()
            .flatten()
            .any(|issue| issue.status.is_error())
}

/// Runs drop-time cleanup as the scope that owns the signals; with no runtime
/// left (a whole `VirtualDom` dropping) `try_peek` already tolerates it.
fn in_owner(owner: ScopeId, cleanup: impl FnOnce()) {
    match Runtime::try_current() {
        Some(runtime) => runtime.in_scope(owner, cleanup),
        None => cleanup(),
    }
}

/// The issues a set of composite rules raises over `value`, every path put
/// under `prefix`. A blank message warns in debug once per `warned`.
pub(crate) fn issues_of<V>(
    rules: &Validators<V>,
    value: &V,
    prefix: &str,
    warned: &Cell<bool>,
) -> Vec<Issue> {
    rules
        .iter()
        .filter_map(|rule| {
            let status = rule.validate(value);
            let blank = status.message().is_some_and(|text| text.trim().is_empty());
            if blank && !warned.replace(true) {
                warn(
                    "A composite rule message is blank, so the group is invalid with nothing \
                     to read. Give it a text, or use `FieldStatus::Valid`.",
                );
            }
            (!status.is_valid()).then(|| Issue {
                paths: rule
                    .targets()
                    .iter()
                    .map(|path| super::path::join(prefix, path))
                    .collect(),
                status,
            })
        })
        .collect()
}
