//! Editable cells: a column's setter, the commit that hands the caller an edited
//! clone of the row, and the editor drawn in the cell meanwhile.

use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{cell_focus::Move, cell_value::EditKind, column::Column};
use crate::{
    components::{accessibility::VisuallyHidden, common::Glyph},
    context::IconSlot,
    hooks::{ElementHandle, use_element, use_silent_focus_out_of},
    platform::{ElementApi, is_tab},
};

/// A committed cell edit: a clone of `data[index]` with the new value set. The
/// handler stores it; `Table` keeps no copy.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{RowUpdate, Table, column};
/// #[derive(Clone, PartialEq)]
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// #[component]
/// fn Users() -> Element {
///     let mut users = use_signal(|| vec![User { name: "Ana".into(), age: 31 }]);
///     rsx! {
///         Table {
///             aria_label: "Users",
///             data: users(),
///             columns: vec![
///                 column("Name").value(|u: &User| u.name.clone()).row_header(),
///                 column("Age")
///                     .value(|u: &User| u.age)
///                     .editable(|u: &mut User, age: u32| u.age = age),
///             ],
///             row_key: |u: &User| u.name.clone(),
///             onrowupdate: move |update: RowUpdate<User>| users.write()[update.index] = update.row,
///         }
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct RowUpdate<T> {
    /// The row's `row_key`, else its index.
    pub key: String,
    /// Its index in `data`.
    pub index: usize,
    /// The edited column's header.
    pub column: String,
    pub row: T,
}

/// What a custom [`editor`](super::Column::editor) gets: the text it starts from,
/// its name, and the commit and cancel that close it.
#[derive(Clone, PartialEq)]
#[non_exhaustive]
pub struct EditCtx {
    pub value: String,
    /// The editor's name: "Edit Age, Ana".
    pub label: String,
    /// Why the last commit was refused, shown under the editor and announced: mark
    /// the field invalid with it. Cleared when the editor closes.
    pub error: Option<String>,
    /// The id of the element showing `error`, which exists only while `error` is `Some`:
    /// set the field's `aria-describedby` to it then, and not otherwise.
    pub error_id: String,
    /// Call it when the text changed: it clears `error`, as typing does in the built-in editor.
    pub changed: Callback<()>,
    /// Takes the new text; one that does not parse or validate keeps the editor open.
    pub commit: Callback<String>,
    pub cancel: Callback<()>,
}

type EditSet<T> = Rc<dyn Fn(&mut T, &str) -> Option<()>>;
type EditCheck<T> = Rc<dyn Fn(&T, &str) -> Result<(), String>>;

/// An editable column's parts; `set` parses the text and sets it, `None` when it does not parse.
pub(super) struct ColumnEdit<T> {
    pub kind: EditKind,
    pub set: EditSet<T>,
    pub validate: Option<EditCheck<T>>,
    pub editor: Option<Rc<dyn Fn(EditCtx) -> Element>>,
}

// Hand-written: a derive would demand `T: Clone`.
impl<T> Clone for ColumnEdit<T> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            set: self.set.clone(),
            validate: self.validate.clone(),
            editor: self.editor.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum EditError {
    /// Not a value of the column's type.
    Parse,
    /// The column's `validate` said no, with its message.
    Invalid(String),
}

/// `row` with `text` set in `column`; `Ok(None)` when the text is the value's own.
pub(super) fn commit_edit<T: Clone>(
    row: &T,
    column: &Column<T>,
    text: &str,
) -> Result<Option<T>, EditError> {
    let Some(edit) = &column.edit else {
        return Ok(None);
    };
    if text == (column.value_text)(row) {
        return Ok(None);
    }
    let mut next = row.clone();
    (edit.set)(&mut next, text).ok_or(EditError::Parse)?;
    if let Some(validate) = &edit.validate {
        validate(row, text).map_err(EditError::Invalid)?;
    }
    Ok(Some(next))
}

/// How a committed or cancelled editor closes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Leave {
    /// Focus went elsewhere: it stays there.
    Blur,
    /// The cell takes the focus back, as the field goes with it.
    Stay,
    /// The stop moves on: Enter down, Shift+Enter up, Tab right, Shift+Tab left.
    To(Move),
}

/// What a key does in the editor: whether to save, and how to leave. `None` types on.
fn key_exit(key: &Key, tab: bool, shift: bool) -> Option<(bool, Leave)> {
    match key {
        Key::Enter => Some((true, Leave::To(if shift { Move::Up } else { Move::Down }))),
        Key::Escape => Some((false, Leave::Stay)),
        _ if tab => Some((
            true,
            Leave::To(if shift { Move::Left } else { Move::Right }),
        )),
        _ => None,
    }
}

/// One close of the editor, given whether it `done` already: saves or cancels once
/// and answers whether it is done now. A commit that is refused leaves it open.
fn settle(done: bool, save: bool, commit: impl FnOnce() -> bool, cancel: impl FnOnce()) -> bool {
    if done {
        return true;
    }
    if save {
        return commit();
    }
    cancel();
    true
}

/// The editor in an editing cell: Enter, Tab or leaving it commits, Escape or the
/// cancel button (on a touch screen, where no Escape is) puts the value back.
/// `commit` takes the text and how to leave, and answers whether it took the text;
/// one it did not keeps the editor open, with `error`, until the next input; on a
/// blur it closes the editor and answers `true`, the old value back.
#[component]
pub(super) fn CellEditor(
    value: String,
    label: String,
    kind: EditKind,
    cancel_label: &'static str,
    error: Option<String>,
    error_id: String,
    /// The form the text takes, for a kind that has no obvious one.
    hint: Option<&'static str>,
    commit: Callback<(String, Leave), bool>,
    cancel: Callback<()>,
    changed: Callback<()>,
) -> Element {
    let mut draft = use_signal(|| value.clone());
    // Closed once: the blur after Enter or Escape commits nothing more.
    let mut done = use_hook(|| CopyValue::new(false));
    let input: ElementHandle = use_element();
    let mut focused = use_hook(|| CopyValue::new(false));
    use_effect(move || {
        if input.is_mounted() && !*focused.peek() {
            focused.set(true);
            let _ = input.focus();
            let _ = input.select();
        }
    });
    let close = use_callback(move |(save, leave): (bool, Leave)| {
        let closed = settle(
            *done.peek(),
            save,
            || commit.call((draft.peek().clone(), leave)),
            || cancel.call(()),
        );
        done.set(closed);
    });
    // Blitz focuses a pressed button without a `blur` here (todo 3263).
    use_silent_focus_out_of(input, move || close.call((true, Leave::Blur)));
    let invalid = error.is_some();
    let hint_id = hint.map(|_| format!("{error_id}-hint"));
    let described = [invalid.then(|| error_id.clone()), hint_id.clone()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    rsx! {
        div {
            "data-cell-editor": true,
            // The row's click and the cell's double click are not the editor's.
            onclick: move |event: MouseEvent| event.stop_propagation(),
            ondoubleclick: move |event: MouseEvent| event.stop_propagation(),
            onpointerdown: move |event: PointerEvent| event.stop_propagation(),
            if let (Some(hint), Some(id)) = (hint, hint_id) {
                VisuallyHidden { id, "{hint}" }
            }
            input {
                r#type: "text",
                inputmode: (kind == EditKind::Number).then_some("decimal"),
                aria_label: label,
                aria_invalid: invalid.then_some("true"),
                aria_describedby: (!described.is_empty()).then(|| described.join(" ")),
                placeholder: hint,
                value: draft(),
                onmounted: input.mount(),
                oninput: move |event: FormEvent| {
                    draft.set(event.value());
                    if invalid {
                        changed.call(());
                    }
                },
                onkeydown: move |event: KeyboardEvent| {
                    if event.is_composing() {
                        return;
                    }
                    let shift = event.modifiers().shift();
                    let Some((save, leave)) = key_exit(&event.key(), is_tab(&event), shift) else {
                        return;
                    };
                    event.prevent_default();
                    event.stop_propagation();
                    close.call((save, leave));
                },
                onblur: move |_| close.call((true, Leave::Blur)),
                ..input.attributes(),
            }
            button {
                r#type: "button",
                "data-cell-editor-cancel": true,
                aria_label: cancel_label,
                // Keeps the focus in the field, whose blur would commit first.
                onmousedown: move |event: MouseEvent| event.prevent_default(),
                onclick: move |_| close.call((false, Leave::Stay)),
                Glyph { slot: IconSlot::Close, icon: lucide::x::outlined }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::column::column, *};

    #[derive(Clone, Debug, PartialEq)]
    struct Person {
        name: String,
        age: u32,
        nick: Option<String>,
    }

    fn ana() -> Person {
        Person {
            name: "Ana".into(),
            age: 31,
            nick: Some("An".into()),
        }
    }

    fn age() -> Column<Person> {
        column("Age")
            .value(|p: &Person| p.age)
            .editable(|p: &mut Person, age: u32| p.age = age)
    }

    #[test]
    fn a_valid_text_applies_the_setter() {
        let next = commit_edit(&ana(), &age(), " 42 ").unwrap().unwrap();
        assert_eq!(next.age, 42);
        assert_eq!(next.name, "Ana");
    }

    #[test]
    fn a_bad_number_is_an_error() {
        assert_eq!(commit_edit(&ana(), &age(), "old"), Err(EditError::Parse));
        let checked = age().validate(|_: &Person, age: &u32| match *age <= 150 {
            true => Ok(()),
            false => Err("Too old".into()),
        });
        assert_eq!(
            commit_edit(&ana(), &checked, "200"),
            Err(EditError::Invalid("Too old".into()))
        );
    }

    #[test]
    fn an_unchanged_text_is_no_update() {
        assert_eq!(commit_edit(&ana(), &age(), "31"), Ok(None));
        // The value's own text, not a `format`ted one.
        let formatted = age().format(|p: &Person| format!("{} years", p.age));
        assert_eq!(commit_edit(&ana(), &formatted, "31"), Ok(None));
    }

    #[test]
    fn enter_and_tab_save_and_move_escape_cancels() {
        let exit = |key: Key, tab, shift| key_exit(&key, tab, shift);
        assert_eq!(
            exit(Key::Enter, false, false),
            Some((true, Leave::To(Move::Down)))
        );
        assert_eq!(
            exit(Key::Enter, false, true),
            Some((true, Leave::To(Move::Up)))
        );
        assert_eq!(
            exit(Key::Tab, true, false),
            Some((true, Leave::To(Move::Right)))
        );
        assert_eq!(
            exit(Key::Tab, true, true),
            Some((true, Leave::To(Move::Left)))
        );
        assert_eq!(exit(Key::Escape, false, false), Some((false, Leave::Stay)));
        assert_eq!(exit(Key::Character("a".into()), false, false), None);
        assert_eq!(exit(Key::ArrowLeft, false, false), None);
    }

    #[test]
    fn a_refused_commit_stays_open_and_a_closed_editor_acts_once() {
        let (mut commits, mut cancels) = (0, 0);
        let mut run = |done, save, accept| {
            settle(
                done,
                save,
                || {
                    commits += 1;
                    accept
                },
                || cancels += 1,
            )
        };
        assert!(!run(false, true, false), "a refused commit keeps it open");
        assert!(run(false, true, true), "an accepted commit closes it");
        assert!(
            run(true, true, true),
            "a blur after Enter does nothing more"
        );
        assert!(run(true, false, true), "nor after Escape");
        assert!(run(false, false, false), "Escape closes it");
        assert_eq!((commits, cancels), (2, 1));
    }

    #[test]
    fn option_empty_text_is_none() {
        let nick = column("Nick")
            .value(|p: &Person| p.nick.clone())
            .editable(|p: &mut Person, nick: Option<String>| p.nick = nick);
        assert_eq!(commit_edit(&ana(), &nick, "").unwrap().unwrap().nick, None);
    }
}
