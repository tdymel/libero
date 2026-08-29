use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;

use crate::components::form::{FieldStatus, Validators, worst};

/// What a `Form` shares with the fields and fieldsets inside it. A `Fieldset`
/// with no `Form` above it opens one of its own, so its composite rules still
/// reach its fields.
#[derive(Clone, Copy)]
pub(crate) struct FormScope {
    fields: Signal<BTreeMap<usize, FieldEntry>, UnsyncStorage>,
    /// Composite issues by the key of the `Form` or `Fieldset` that raised
    /// them.
    issues: Signal<BTreeMap<usize, Vec<Issue>>>,
    /// Names of fields that lost focus once. Shared, because a composite issue
    /// waits until every field it names is touched.
    touched: Signal<BTreeSet<String>>,
    submitted: Signal<bool>,
    next_key: Signal<usize>,
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
            next_key: Signal::new(0),
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

    pub fn withdraw(&mut self, key: usize) {
        if self.issues.peek().contains_key(&key) {
            self.issues.write().remove(&key);
        }
    }

    /// The worst composite issue naming `name` that may show: after a submit,
    /// or once every field the issue names is touched. Reactive on the issues,
    /// the touched set and the submit.
    pub fn visible_issue(&self, name: &str) -> FieldStatus {
        let submitted = self.submitted();
        let touched = self.touched.read();
        self.issues
            .read()
            .values()
            .flatten()
            .filter(|issue| issue.paths.iter().any(|path| path == name))
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

    /// Tolerates a form that is already gone: a field inside it can drop after
    /// the form's own signals did.
    pub fn unregister(&mut self, key: usize) {
        let present = self
            .fields
            .try_peek()
            .is_ok_and(|fields| fields.contains_key(&key));
        if present {
            self.fields.write_unchecked().remove(&key);
        }
    }

    /// Whether anything blocks a submit: an error on a field, or an error
    /// raised by a composite rule. Warnings never block.
    pub fn has_errors(&self) -> bool {
        self.fields
            .peek()
            .values()
            .any(|field| field.status.is_error())
            || self
                .issues
                .peek()
                .values()
                .flatten()
                .any(|issue| issue.status.is_error())
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

        for issue in self.issues.peek().values().flatten() {
            let FieldStatus::Error(message) = &issue.status else {
                continue;
            };
            items.push(SummaryItem {
                message: message.clone(),
                target: issue.paths.first().and_then(|path| id_of(path)),
            });
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

/// The issues a set of composite rules raises over `value`, every path put
/// under `prefix`.
pub(crate) fn issues_of<V>(rules: &Validators<V>, value: &V, prefix: &str) -> Vec<Issue> {
    rules
        .iter()
        .filter_map(|rule| {
            let status = rule.validate(value);
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
