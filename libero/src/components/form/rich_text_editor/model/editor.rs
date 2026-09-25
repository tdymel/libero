//! [`Editor`]: the live state plus its undo history, and the one place commands run.

use std::rc::Rc;

use crate::hooks::UndoHistory;

use super::command::{Action, CommandName, Commands, KeyPress, Keymap, Record};
use super::doc::Doc;
use super::state::{EditorState, Selection};

/// What the editor needs from an undo stack: [`UndoHistory`] here, `use_history`'s
/// handle in the view.
pub trait UndoStack {
    fn present(&self) -> Rc<EditorState>;
    fn push(&mut self, state: EditorState);
    fn merge(&mut self, state: EditorState);
    fn seal(&mut self);
    fn undo(&mut self) -> bool;
    fn redo(&mut self) -> bool;
    fn reset(&mut self, state: EditorState);
    fn can_undo(&self) -> bool;
    fn can_redo(&self) -> bool;
}

impl UndoStack for UndoHistory<EditorState> {
    fn present(&self) -> Rc<EditorState> {
        UndoHistory::present(self).clone()
    }

    fn push(&mut self, state: EditorState) {
        UndoHistory::push(self, state);
    }

    fn merge(&mut self, state: EditorState) {
        UndoHistory::merge(self, state);
    }

    fn seal(&mut self) {
        UndoHistory::seal(self);
    }

    fn undo(&mut self) -> bool {
        UndoHistory::undo(self)
    }

    fn redo(&mut self) -> bool {
        UndoHistory::redo(self)
    }

    fn reset(&mut self, state: EditorState) {
        UndoHistory::reset(self, state);
    }

    fn can_undo(&self) -> bool {
        UndoHistory::can_undo(self)
    }

    fn can_redo(&self) -> bool {
        UndoHistory::can_redo(self)
    }
}

/// The live state and its history. Selection moves change only the live state;
/// edits also record a snapshot, so undo restores the doc and the caret after it.
#[derive(Clone, Debug)]
pub struct Editor<H = UndoHistory<EditorState>> {
    state: EditorState,
    history: H,
}

impl Editor {
    #[allow(dead_code, reason = "tests; the view runs on `use_history`")]
    pub fn new(doc: Doc) -> Self {
        let state = EditorState::new(doc);
        let history = UndoHistory::new(state.clone());
        Self { state, history }
    }
}

impl<H: UndoStack> Editor<H> {
    /// An editor over `history`, starting at its present state.
    pub fn with_history(history: H) -> Self {
        let state = (*history.present()).clone();
        Self { state, history }
    }

    pub fn state(&self) -> &EditorState {
        &self.state
    }

    pub fn doc(&self) -> &Doc {
        &self.state.doc
    }

    #[allow(dead_code, reason = "for the command handle, todo in plan 1159")]
    pub fn history(&self) -> &H {
        &self.history
    }

    /// Replaces the doc from outside (a new controlled value): history starts over.
    pub fn reset(&mut self, doc: Doc) {
        self.state = EditorState::new(doc);
        self.history.reset(self.state.clone());
    }

    /// Moves the selection; the next typing starts a new undo group.
    pub fn select(&mut self, selection: Selection) {
        self.state.select(selection);
        self.history.seal();
    }

    /// Runs `edit` on the state and records it as `record` says.
    pub fn apply(&mut self, record: Record, edit: impl FnOnce(&mut EditorState) -> bool) -> bool {
        let mut next = self.state.clone();
        if !edit(&mut next) {
            return false;
        }
        let doc_changed = next.doc != self.state.doc;
        self.state = next;
        match (record, doc_changed) {
            (Record::Step, true) => self.history.push(self.state.clone()),
            (Record::Merge, true) => self.history.merge(self.state.clone()),
            _ => {}
        }
        true
    }

    /// Typed text: one undo group while typing goes on. A completed Markdown
    /// shortcut is a step of its own, so undo brings the typed syntax back.
    pub fn type_text(&mut self, text: &str) -> bool {
        if !self.apply(Record::Merge, |state| state.insert_text(text)) {
            return false;
        }
        if text.chars().count() == 1 {
            self.apply(Record::Step, EditorState::apply_input_rules);
        }
        true
    }

    pub fn undo(&mut self) -> bool {
        self.step(H::undo)
    }

    pub fn redo(&mut self) -> bool {
        self.step(H::redo)
    }

    fn step(&mut self, step: fn(&mut H) -> bool) -> bool {
        if !step(&mut self.history) {
            return false;
        }
        self.state = (*self.history.present()).clone();
        true
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Runs the command `name`. `false` when it is unknown or changed nothing, so
    /// the view can let the key through (Tab outside a list moves focus).
    pub fn run(&mut self, commands: &Commands, name: impl Into<CommandName>) -> bool {
        let Some(action) = commands.get(&name.into()).cloned() else {
            return false;
        };
        match action {
            Action::Edit(edit, record) => self.apply(record, |state| edit(state)),
            Action::Undo => self.undo(),
            Action::Redo => self.redo(),
            Action::View => false,
        }
    }

    /// Runs the command bound to `press`. `false` when none is bound or it did nothing.
    #[allow(
        dead_code,
        reason = "tests; the view routes keys through its own dialogs"
    )]
    pub fn handle_key(
        &mut self,
        keymap: &Keymap,
        commands: &Commands,
        press: &KeyPress,
        apple: bool,
    ) -> bool {
        match keymap.command_for(press, apple).cloned() {
            Some(name) => self.run(commands, name),
            None => false,
        }
    }
}
