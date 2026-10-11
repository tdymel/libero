//! Editable cells: a column's setter, the commit that hands the caller an edited
//! clone of the row, and the editor drawn in the cell meanwhile.

use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{cell_value::EditKind, column::Column};
use crate::{
    components::common::Glyph,
    context::IconSlot,
    hooks::{ElementHandle, use_element},
    platform::ElementApi,
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
pub struct EditCtx {
    pub value: String,
    /// The editor's name: "Edit Age, Ana".
    pub label: String,
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

/// The editor in an editing cell: Enter or leaving it commits, Escape or the cancel
/// button (on a touch screen, where no Escape is) puts the value back. `commit`
/// takes the text and whether the cell gets the focus back (not after a blur), and
/// answers whether it took the text; one it did not keeps the editor open.
#[component]
pub(super) fn CellEditor(
    value: String,
    label: String,
    kind: EditKind,
    cancel_label: &'static str,
    commit: Callback<(String, bool), bool>,
    cancel: Callback<()>,
) -> Element {
    let mut draft = use_signal(|| value.clone());
    let mut invalid = use_signal(|| false);
    // Closed once: the blur after Enter or Escape commits nothing more.
    let mut done = use_hook(|| CopyValue::new(false));
    let input: ElementHandle = use_element();
    let mut focused = use_hook(|| CopyValue::new(false));
    use_effect(move || {
        if input.is_mounted() && !*focused.peek() {
            focused.set(true);
            let _ = input.focus();
        }
    });
    // `(save, blurred)`.
    let close = use_callback(move |(save, blurred): (bool, bool)| {
        if *done.peek() {
            return;
        }
        if !save {
            done.set(true);
            cancel.call(());
        } else if commit.call((draft.peek().clone(), !blurred)) {
            done.set(true);
        } else {
            invalid.set(true);
        }
    });
    rsx! {
        div {
            "data-cell-editor": true,
            // The row's click and the cell's double click are not the editor's.
            onclick: move |event: MouseEvent| event.stop_propagation(),
            ondoubleclick: move |event: MouseEvent| event.stop_propagation(),
            onpointerdown: move |event: PointerEvent| event.stop_propagation(),
            input {
                r#type: "text",
                inputmode: (kind == EditKind::Number).then_some("decimal"),
                aria_label: label,
                aria_invalid: invalid().then_some("true"),
                value: draft(),
                onmounted: input.mount(),
                oninput: move |event: FormEvent| {
                    draft.set(event.value());
                    if *invalid.peek() {
                        invalid.set(false);
                    }
                },
                onkeydown: move |event: KeyboardEvent| {
                    if event.is_composing() {
                        return;
                    }
                    let save = match event.key() {
                        Key::Enter => true,
                        Key::Escape => false,
                        _ => return,
                    };
                    event.prevent_default();
                    event.stop_propagation();
                    close.call((save, false));
                },
                onblur: move |_| close.call((true, true)),
                ..input.attributes(),
            }
            button {
                r#type: "button",
                "data-cell-editor-cancel": true,
                aria_label: cancel_label,
                // Keeps the focus in the field, whose blur would commit first.
                onmousedown: move |event: MouseEvent| event.prevent_default(),
                onclick: move |_| close.call((false, false)),
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
    fn option_empty_text_is_none() {
        let nick = column("Nick")
            .value(|p: &Person| p.nick.clone())
            .editable(|p: &mut Person, nick: Option<String>| p.nick = nick);
        assert_eq!(commit_edit(&ana(), &nick, "").unwrap().unwrap().nick, None);
    }
}
