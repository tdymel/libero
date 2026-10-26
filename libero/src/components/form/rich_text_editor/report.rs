//! What the editor does with each [`Report`] its surface script sends.

use dioxus::prelude::*;

use super::input::SubmitOn;
use super::model::{Doc, EditorState, KeyPress, NodeKey, NodeRegistry, Record, Selection};
use super::surface::{Caret, Report};
use super::view::{LiveEditor, clip_of, enter_code, model_position, reconcile};

/// The editor's state and actions a report reaches; the actions are the view's closures.
pub(super) struct Reporter<Edit, Changed, BeforeInput, KeyDown, NoteLimit, TrySubmit> {
    pub editor: CopyValue<LiveEditor>,
    pub registry: CopyValue<Option<NodeRegistry>>,
    pub last_caret: CopyValue<Option<Caret>>,
    pub wants_caret: CopyValue<bool>,
    pub caret_tick: Signal<u32>,
    pub revision: Signal<u32>,
    pub syncing: CopyValue<bool>,
    pub composing: CopyValue<bool>,
    pub can_edit: CopyValue<bool>,
    pub generation: Signal<u32>,
    pub focused: Signal<bool>,
    pub submit: CopyValue<Option<(EventHandler<Doc>, SubmitOn)>>,
    pub edit: Edit,
    pub changed: Changed,
    pub before_input: BeforeInput,
    pub key_down: KeyDown,
    pub note_limit: NoteLimit,
    pub try_submit: TrySubmit,
}

impl<Edit, Changed, BeforeInput, KeyDown, NoteLimit, TrySubmit>
    Reporter<Edit, Changed, BeforeInput, KeyDown, NoteLimit, TrySubmit>
where
    Edit: FnMut(&dyn Fn(&mut LiveEditor) -> bool, bool) -> bool,
    Changed: FnMut(Option<Doc>, bool),
    BeforeInput: FnMut(&str, Option<String>, &str) -> bool,
    KeyDown: FnMut(KeyPress) -> bool,
    NoteLimit: FnMut(),
    TrySubmit: FnMut(&KeyPress) -> bool,
{
    /// Applies `report`; the answer is the Markdown a reported selection copies in a WebView.
    pub fn handle(&mut self, report: Report) -> Option<String> {
        if report.caret.is_some() && *self.last_caret.peek() != report.caret {
            self.last_caret.set(report.caret);
            if *self.wants_caret.peek() {
                self.caret_tick += 1;
            }
        }
        let mut clip = None;
        if let Some((anchor_key, anchor, head_key, head)) = report.selection
            && (report.press || !*self.syncing.peek())
            && !*self.composing.peek()
        {
            let selection = {
                let live = self.editor.peek();
                model_position(&live, anchor_key, anchor)
                    .zip(model_position(&live, head_key, head))
                    .map(|(anchor, head)| Selection::range(anchor, head))
            };
            if let Some(selection) = selection {
                if selection != self.editor.peek().state().selection {
                    self.editor.write().select(selection);
                    self.revision += 1;
                }
                clip = clip_of(&self.editor.peek(), &self.registry.peek());
            }
        }
        if let Some((from_key, from, to_key, to, text)) = report.replace
            && *self.can_edit.peek()
        {
            let range = {
                let live = self.editor.peek();
                model_position(&live, from_key, from).zip(model_position(&live, to_key, to))
            };
            if let Some((from, to)) = range {
                (self.edit)(
                    &|live| {
                        live.apply(Record::Step, |state| {
                            state.select(Selection::range(from, to));
                            state.delete_selection();
                            state.insert_text(&text);
                            true
                        })
                    },
                    false,
                );
            }
        }
        if let Some((input_type, data, key)) = report.input
            && *self.can_edit.peek()
        {
            (self.before_input)(&input_type, data, &key);
        }
        if let Some((key, code, ctrl, meta, alt, shift)) = report.key
            && *self.can_edit.peek()
            && !*self.composing.peek()
        {
            (self.key_down)(KeyPress {
                key,
                code,
                ctrl,
                meta,
                alt,
                shift,
            });
        }
        if report.synced {
            self.syncing.set(false);
        }
        if let Some((key, text)) = report.text {
            let sends = self
                .submit
                .peek()
                .is_some_and(|(_, on)| on == SubmitOn::Enter)
                && *self.can_edit.peek()
                && !report.shift;
            let soft = report.shift;
            let editor = self.editor;
            let entered = reconcile(editor, NodeKey(key), &text, sends, soft, &mut self.changed);
            // Enter in a composed word arrives as its "\n": send after the word is in.
            if entered && !(self.try_submit)(&KeyPress::new("Enter")) {
                reconcile(editor, NodeKey(key), &text, false, soft, &mut self.changed);
            }
            (self.note_limit)();
            self.generation += 1;
        }
        if let Some((key, at_end)) = report.code
            && *self.can_edit.peek()
        {
            enter_code(self.editor, NodeKey(key), at_end);
            (self.changed)(None, true);
        }
        if (report.exit || report.end) && *self.can_edit.peek() {
            let leave = if report.exit {
                EditorState::exit_code
            } else {
                EditorState::exit_end
            };
            let before = self.editor.peek().doc().clone();
            if self.editor.write().apply(Record::Step, leave) {
                self.focused.set(true);
                (self.changed)(Some(before), true);
            }
        }
        clip
    }
}
