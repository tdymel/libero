//! [`RichTextHandle`]: a caller's toolbar or code runs an editor's commands and reads its state.

use std::rc::Rc;

use dioxus::prelude::*;

use super::model::{BlockKind, CommandName, EditorState, MarkKind};

/// The kind of list the caret is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ListKind {
    Bullet,
    Ordered,
}

/// What the editor shows its toolbar, published for the handle.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Status {
    active: Vec<MarkKind>,
    block: BlockKind,
    list: Option<ListKind>,
    quote: bool,
    can_undo: bool,
    can_redo: bool,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            active: Vec::new(),
            block: BlockKind::Paragraph,
            list: None,
            quote: false,
            can_undo: false,
            can_redo: false,
        }
    }
}

const MARKS: [MarkKind; 6] = [
    MarkKind::Bold,
    MarkKind::Italic,
    MarkKind::Underline,
    MarkKind::Strike,
    MarkKind::Code,
    MarkKind::Link,
];

impl Status {
    pub(crate) fn of(state: &EditorState, can_undo: bool, can_redo: bool) -> Self {
        Self {
            active: MARKS
                .into_iter()
                .filter(|kind| state.is_active(*kind))
                .collect(),
            block: state.block_kind().clone(),
            list: state.list_kind().map(|ordered| match ordered {
                true => ListKind::Ordered,
                false => ListKind::Bullet,
            }),
            quote: state.in_quote(),
            can_undo,
            can_redo,
        }
    }
}

pub(crate) type EditFnMut<'a> = &'a mut dyn FnMut(&mut EditorState) -> bool;

/// How a mounted editor runs a command, and a one-off edit.
#[derive(Clone)]
pub(crate) struct Runner {
    pub run: Rc<dyn Fn(CommandName) -> bool>,
    pub edit: Rc<dyn Fn(EditFnMut<'_>) -> bool>,
    pub clear: Rc<dyn Fn() -> bool>,
    pub focus: Rc<dyn Fn() -> bool>,
}

/// Drives one [`RichTextEditor`](super::RichTextEditor) from outside: pass it as the
/// editor's `handle`, then run commands from your own buttons and read the state.
/// Reads are reactive. Make one with [`use_rich_text_editor`].
#[derive(Clone, Copy, PartialEq)]
pub struct RichTextHandle {
    status: Signal<Status>,
    state: Signal<Option<Rc<EditorState>>>,
    runner: CopyValue<Option<(String, Runner)>>,
}

/// A handle for one [`RichTextEditor`](super::RichTextEditor).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, RichTextEditor, rich_text::{Builtin, MarkKind, use_rich_text_editor}};
/// # fn app() -> Element {
/// let editor = use_rich_text_editor();
/// rsx! {
///     Button {
///         aria_pressed: editor.is_active(MarkKind::Bold),
///         onclick: move |_| { editor.run(Builtin::Bold); },
///         "Bold"
///     }
///     RichTextEditor { label: "Comment", toolbar: false, handle: editor }
/// }
/// # }
/// ```
pub fn use_rich_text_editor() -> RichTextHandle {
    let status = use_signal(Status::default);
    let state = use_signal(|| None);
    let runner = use_hook(|| CopyValue::new(None));
    RichTextHandle {
        status,
        state,
        runner,
    }
}

impl RichTextHandle {
    fn runner(&self) -> Option<Runner> {
        self.runner
            .peek()
            .as_ref()
            .map(|(_, runner)| runner.clone())
    }

    /// Runs the command `name` on the editor's selection. When it changed the document, focus
    /// goes to the text and into view; a dialog command (Link, Shortcuts) returns focus to the
    /// text when its dialog closes. `false` when no editor is mounted, the command is unknown, or it changed nothing.
    pub fn run(&self, name: impl Into<CommandName>) -> bool {
        self.runner()
            .is_some_and(|runner| (runner.run)(name.into()))
    }

    /// Runs `f` on the editor's state as one undo step, e.g. to insert the mention a user
    /// picked. When `f` returns `true`, focus goes to the text and into view. `false` when
    /// no editor is mounted or `f` returned `false`.
    ///
    /// ```rust
    /// # use libero::components::rich_text::RichTextHandle;
    /// fn shout(editor: RichTextHandle) {
    ///     editor.edit(|state| state.insert_text("!"));
    /// }
    /// ```
    pub fn edit(&self, f: impl FnOnce(&mut EditorState) -> bool) -> bool {
        let Some(runner) = self.runner() else {
            return false;
        };
        let mut f = Some(f);
        (runner.edit)(&mut |state| f.take().is_some_and(|f| f(state)))
    }

    /// Empties the editor to one blank paragraph, e.g. after a send, and keeps focus in
    /// the text. Undo starts over and `onchange` fires. `false` when no editor is mounted
    /// (or it is read-only) or it was empty already.
    ///
    /// ```rust
    /// # use dioxus::prelude::*;
    /// # use libero::components::{RichTextEditor, rich_text::use_rich_text_editor};
    /// # fn app() -> Element {
    /// let editor = use_rich_text_editor();
    /// rsx! {
    ///     RichTextEditor {
    ///         label: "Message",
    ///         handle: editor,
    ///         onsubmit: move |doc| {
    ///             // Send `doc`, then:
    ///             editor.clear();
    ///         },
    ///     }
    /// }
    /// # }
    /// ```
    pub fn clear(&self) -> bool {
        self.runner().is_some_and(|runner| (runner.clear)())
    }

    /// Puts focus in the text, at the caret, and scrolls it into view. `false` when no editor is mounted or it
    /// is read-only or disabled.
    pub fn focus(&self) -> bool {
        self.runner().is_some_and(|runner| (runner.focus)())
    }

    /// Whether the document holds only blank text, no rules or inline nodes. Reactive;
    /// `true` while no editor is mounted.
    pub fn is_empty(&self) -> bool {
        self.state
            .read()
            .as_deref()
            .is_none_or(|state| state.doc.is_empty())
    }

    /// The document's plain text, one line per block, as `onsubmit` and `max_length`
    /// see it. Reactive; empty while no editor is mounted.
    pub fn plain_text(&self) -> String {
        self.with_state(|state| state.doc.plain_text())
            .unwrap_or_default()
    }

    /// Reads the editor's live state, reactively: re-runs on every edit and caret move.
    /// `None` while no editor is mounted.
    pub fn with_state<R>(&self, f: impl FnOnce(&EditorState) -> R) -> Option<R> {
        self.state.read().as_deref().map(f)
    }

    /// Whether `mark` covers the selection; `false` while no editor is mounted.
    pub fn is_active(&self, mark: MarkKind) -> bool {
        self.status.read().active.contains(&mark)
    }

    /// The kind of block the caret is in; `Paragraph` while no editor is mounted.
    pub fn block_kind(&self) -> BlockKind {
        self.status.read().block.clone()
    }

    /// The list the caret is in; `None` while no editor is mounted.
    pub fn list_kind(&self) -> Option<ListKind> {
        self.status.read().list
    }

    /// Whether the caret is in a quote; `false` while no editor is mounted.
    pub fn in_quote(&self) -> bool {
        self.status.read().quote
    }

    /// Whether there is an edit to undo; `false` while no editor is mounted.
    pub fn can_undo(&self) -> bool {
        self.status.read().can_undo
    }

    /// Whether there is an undone edit to redo; `false` while no editor is mounted.
    pub fn can_redo(&self) -> bool {
        self.status.read().can_redo
    }

    /// The editor `owner` takes the handle over; the last one mounted wins.
    pub(crate) fn attach(&self, owner: &str, run: Runner) {
        let mut runner = self.runner;
        #[cfg(debug_assertions)]
        if runner
            .peek()
            .as_ref()
            .is_some_and(|(other, _)| other != owner)
        {
            crate::utils::warn(
                "RichTextHandle: a second RichTextEditor took this handle over; one handle drives one editor",
            );
        }
        runner.set(Some((owner.to_string(), run)));
    }

    pub(crate) fn detach(&self, owner: &str) {
        let mut runner = self.runner;
        if runner.peek().as_ref().is_some_and(|(own, _)| own == owner) {
            runner.set(None);
            let mut status = self.status;
            status.set(Status::default());
            let mut state = self.state;
            state.set(None);
        }
    }

    pub(crate) fn publish(&self, owner: &str, next: Status, live: &EditorState) {
        let owns = self
            .runner
            .peek()
            .as_ref()
            .is_some_and(|(own, _)| own == owner);
        if !owns {
            return;
        }
        if *self.status.peek() != next {
            let mut status = self.status;
            status.set(next);
        }
        if self.state.peek().as_deref() != Some(live) {
            let mut state = self.state;
            state.set(Some(Rc::new(live.clone())));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use dioxus::core::{ScopeId, current_scope_id};

    use super::*;
    use crate::{
        LiberoProvider,
        components::form::{RichTextEditor, rich_text::Builtin},
    };

    thread_local! {
        static CALLER: Cell<Option<(RichTextHandle, ScopeId)>> = const { Cell::new(None) };
    }

    /// Todo 1979: the editor owns the values a command reads, so the caller's call runs as the editor.
    #[test]
    fn the_handle_runs_as_the_editor_not_its_caller() {
        let mut dom = VirtualDom::new(|| {
            let editor = use_rich_text_editor();
            use_hook(|| CALLER.set(Some((editor, current_scope_id()))));
            rsx! { LiberoProvider { RichTextEditor { label: "Comment", handle: editor } } }
        });
        dom.rebuild_in_place();
        let (handle, caller) = CALLER.get().expect("the caller rendered");

        let mut ran_in = None;
        let edited = dom.in_scope(caller, || {
            handle.edit(|state| {
                ran_in = Some(current_scope_id());
                state.insert_text("a")
            })
        });
        assert!(edited);
        assert_ne!(ran_in, Some(caller));
        assert!(dom.in_scope(caller, || handle.run(Builtin::Undo)));
    }
}
