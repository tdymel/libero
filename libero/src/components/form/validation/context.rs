use std::{
    any::Any,
    cell::Cell,
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};

use dioxus::{
    core::{Runtime, ScopeId, current_scope_id},
    prelude::*,
};

use crate::{
    components::form::{FieldError, FieldStatus, Parse, Validators, worst},
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
    /// What mapped fields were typed, by full name: kept apart from the fields,
    /// so invalid text outlives an unmounted field.
    drafts: Signal<HashMap<String, Draft>, UnsyncStorage>,
    /// Text a field reading its own (a date) refused, by full name.
    typed: Signal<HashMap<String, Typed>, UnsyncStorage>,
    /// Paths a composite rule names that no field had registered when it was due:
    /// the form judges it again once one does.
    waiting: Signal<BTreeSet<String>, UnsyncStorage>,
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

/// A mapped field's typed text, shown in place of the stored value's while
/// that value is still what the text wrote.
#[derive(Clone)]
pub(crate) struct Draft {
    pub text: Rc<dyn Any>,
    pub wrote: Rc<dyn Any>,
    pub parse: Parse,
}

/// Typed text a date field could not take, and why.
#[derive(Clone)]
pub(crate) struct Typed {
    pub text: String,
    pub error: Rc<dyn FieldError>,
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
    /// Whether a mapped field's text is a domain value; `Valid` for any other field.
    pub parse: Parse,
    pub required: bool,
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
            drafts: Signal::new_maybe_sync(HashMap::new()),
            typed: Signal::new_maybe_sync(HashMap::new()),
            waiting: Signal::new_maybe_sync(BTreeSet::new()),
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
        if !self.drafts.peek().is_empty() {
            self.drafts.write_unchecked().clear();
        }
        if !self.typed.peek().is_empty() {
            self.typed.write_unchecked().clear();
        }
        let runtime = Runtime::current();
        for field in self.fields.peek().values() {
            runtime.needs_update(field.owner);
        }
    }

    /// The draft typed under `name`. Not reactive: the store write that comes
    /// with every draft re-renders the field.
    pub fn draft(&self, name: &str) -> Option<Draft> {
        self.drafts.peek().get(name).cloned()
    }

    pub fn keep_draft(&mut self, name: &str, draft: Draft) {
        self.drafts
            .write_unchecked()
            .insert(name.to_string(), draft);
    }

    pub fn drop_draft(&mut self, name: &str) {
        if self.drafts.peek().contains_key(name) {
            self.drafts.write_unchecked().remove(name);
        }
    }

    /// On blur: a draft that parsed (or is blank) gives way to the stored
    /// value's own text; invalid text stays. Whether one went.
    pub fn settle_draft(&mut self, name: &str) -> bool {
        let settles = self
            .drafts
            .peek()
            .get(name)
            .is_some_and(|draft| !draft.parse.is_invalid());
        if settles {
            self.drafts.write_unchecked().remove(name);
        }
        settles
    }

    /// Refused text kept under `name`, for a remount.
    pub fn typed(&self, name: &str) -> Option<Typed> {
        self.typed.peek().get(name).cloned()
    }

    pub fn keep_typed(&mut self, name: &str, typed: Typed) {
        self.typed.write_unchecked().insert(name.to_string(), typed);
    }

    pub fn drop_typed(&mut self, name: &str) {
        if self.typed.peek().contains_key(name) {
            self.typed.write_unchecked().remove(name);
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
        let previous = self.fields.peek().get(&key).cloned();
        if previous.as_ref() != Some(&entry) {
            // The form judged its composite rules on the old state, or without this field.
            let waited = entry.name.as_ref().is_some_and(|name| {
                self.waiting.peek().contains(name) && self.waiting.write_unchecked().remove(name)
            });
            let changed = previous.is_some_and(|previous| !previous.parse.same_kind(&entry.parse));
            if waited || changed {
                Runtime::current().needs_update(self.owner);
            }
            self.fields.write_unchecked().insert(key, entry);
        }
    }

    /// A mapped field's parse state as the user typed, ahead of its render: the
    /// form re-judges its composite rules with the store write that comes with it.
    pub fn set_parse(&mut self, name: &str, parse: Parse) {
        let mut fields = self.fields.write_unchecked();
        let named = fields
            .values_mut()
            .filter(|field| field.name.as_deref() == Some(name));
        for field in named {
            field.parse = parse.clone();
        }
    }

    /// Whether a composite rule naming `paths` may run: each field it names holds a
    /// domain value, or nothing and is optional. An invalid field never gets a second,
    /// misleading error, and a rule never sees a required field's missing value.
    /// A rule none of whose paths a field carries yet waits for one.
    pub fn ready(&self, paths: &[String]) -> bool {
        let fields = self.fields.peek();
        let missing: Vec<&String> = paths
            .iter()
            .filter(|path| {
                !fields
                    .values()
                    .any(|field| field.name.as_deref() == Some(path.as_str()))
            })
            .collect();
        let none = !paths.is_empty() && missing.len() == paths.len();
        let waiting = self.waiting;
        for path in missing {
            if !waiting.peek().contains(path) {
                waiting.write_unchecked().insert(path.clone());
            }
        }
        if none {
            return false;
        }
        fields.values().all(|field| {
            let named = field
                .name
                .as_deref()
                .is_some_and(|name| paths.iter().any(|path| path == name));
            !named
                || match field.parse {
                    Parse::Valid => true,
                    Parse::Empty => !field.required,
                    Parse::Invalid(_) => false,
                }
        })
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
    /// composite error - each linked to the first mounted field it names, or to
    /// the first field when it names none.
    pub fn summary(&self) -> Vec<SummaryItem> {
        let fields = self.fields.peek();
        let named = |name: &str| {
            fields
                .values()
                .find(|field| field.name.as_deref() == Some(name))
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
                if !blocks(&fields, issue) {
                    continue;
                }
                let item = if issue.paths.is_empty() {
                    // A `Fieldset`'s own rule: its entry shares the key, so the line goes to the
                    // group. A `Form`'s own rule has no entry and goes to the first field.
                    match fields.get(key) {
                        Some(group) => SummaryItem {
                            message: match &group.label {
                                Some(label) => format!("{label}: {message}"),
                                None => message.clone(),
                            },
                            target: Some(group.id.clone()),
                        },
                        None => SummaryItem {
                            message: message.clone(),
                            target: fields.values().next().map(|field| field.id.clone()),
                        },
                    }
                } else {
                    let target = issue.paths.iter().find_map(|path| named(path));
                    SummaryItem {
                        message: match target.and_then(|field| field.label.as_ref()) {
                            Some(label) => format!("{label}: {message}"),
                            None => message.clone(),
                        },
                        target: target.map(|field| field.id.clone()),
                    }
                };
                items.push(item);
            }
        }
        items
    }

    /// Debug builds warn about a composite rule naming a path no registered
    /// field carries - almost always a field missing its `name`. Such a rule
    /// blocks nothing while none of its fields is mounted.
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
            .any(|issue| issue.status.is_error() && blocks(fields, issue))
}

/// A rule naming only unmounted fields concerns nothing on screen, like a hidden field's own rules.
fn blocks(fields: &BTreeMap<usize, FieldEntry>, issue: &Issue) -> bool {
    issue.paths.is_empty()
        || issue.paths.iter().any(|path| {
            fields
                .values()
                .any(|field| field.name.as_deref() == Some(path.as_str()))
        })
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
/// under `prefix`. A rule whose fields are not [`ready`](FormScope::ready) is
/// not run. A blank message warns in debug once per `warned`.
pub(crate) fn issues_of<V>(
    rules: &Validators<V>,
    value: &V,
    prefix: &str,
    warned: &Cell<bool>,
    scope: &FormScope,
) -> Vec<Issue> {
    rules
        .iter()
        .filter_map(|rule| {
            let paths: Vec<String> = rule
                .targets()
                .iter()
                .map(|path| super::path::join(prefix, path))
                .collect();
            if !scope.ready(&paths) {
                return None;
            }
            let status = rule.validate(value);
            let blank = status.message().is_some_and(|text| text.trim().is_empty());
            if blank && !warned.replace(true) {
                warn(
                    "A composite rule message is blank, so the group is invalid with nothing \
                     to read. Give it a text, or use `FieldStatus::Valid`.",
                );
            }
            (!status.is_valid()).then_some(Issue { paths, status })
        })
        .collect()
}
