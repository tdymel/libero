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

pub(crate) type Runner = Rc<dyn Fn(CommandName) -> bool>;

/// Drives one [`RichTextEditor`](super::RichTextEditor) from outside: pass it as the
/// editor's `handle`, then run commands from your own buttons and read the state.
/// Reads are reactive. Make one with [`use_rich_text_editor`].
#[derive(Clone, Copy, PartialEq)]
pub struct RichTextHandle {
    status: Signal<Status>,
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
    let runner = use_hook(|| CopyValue::new(None));
    RichTextHandle { status, runner }
}

impl RichTextHandle {
    /// Runs the command `name` on the editor's selection and gives focus back to the text.
    /// `false` when no editor is mounted, the command is unknown, or it changed nothing.
    pub fn run(&self, name: impl Into<CommandName>) -> bool {
        let runner = self.runner.peek().as_ref().map(|(_, run)| run.clone());
        runner.is_some_and(|run| run(name.into()))
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
        }
    }

    pub(crate) fn publish(&self, owner: &str, next: Status) {
        let owns = self
            .runner
            .peek()
            .as_ref()
            .is_some_and(|(own, _)| own == owner);
        if owns && *self.status.peek() != next {
            let mut status = self.status;
            status.set(next);
        }
    }
}
