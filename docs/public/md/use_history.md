# History

Crate: `libero`
Import: `use libero::hooks::{HistoryHandle, UndoHistory, use_history};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/history/mod.rs>
Index: [index.md](index.md) lists every other page
Description: Undo and redo over snapshots of a value, with rapid changes grouped into one step by time or size.

`use_history(initial: impl FnOnce() -> UndoHistory<T>, group_ms: u64) -> HistoryHandle<T>`
keeps snapshots of a value to step back and forth through. `push` records a
step of its own; `merge` folds rapid changes, such as typing, into one step.
`undo`, `redo` and `reset` move through them, and `value`, `can_undo` and
`can_redo` read them reactively.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, TextField},
    hooks::{UndoHistory, use_history},
};

#[component]
fn UndoableNote() -> Element {
    let note = use_history(|| UndoHistory::new(String::new()), 500);

    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            // Ctrl+Z and Ctrl+Shift+Z (Cmd on macOS) anywhere in the note's controls.
            onkeydown: move |event: KeyboardEvent| {
                let modifiers = event.modifiers();
                let Key::Character(key) = event.key() else { return };
                if !(modifiers.ctrl() || modifiers.meta()) || !key.eq_ignore_ascii_case("z") {
                    return;
                }
                event.prevent_default();
                if modifiers.shift() { note.redo(); } else { note.undo(); }
            },
            TextField {
                label: "Note",
                value: note.value().to_string(),
                oninput: move |next| note.merge(next),
            }
            Flex { direction: "row", gap: "sm",
                Button {
                    variant: "outlined",
                    disabled: !note.can_undo(),
                    focusable_when_disabled: true,
                    onclick: move |_| { note.undo(); },
                    "Undo"
                }
                Button {
                    variant: "outlined",
                    disabled: !note.can_redo(),
                    focusable_when_disabled: true,
                    onclick: move |_| { note.redo(); },
                    "Redo"
                }
                Button {
                    variant: "text",
                    onclick: move |_| note.reset(String::new()),
                    "Clear"
                }
            }
        }
    }
}
```

Without the hook, for a model that has no dioxus in it:

```rust
use libero::hooks::UndoHistory;

let mut history = UndoHistory::new(0).with_cap(20);
history.merge(1);
history.merge(2);
history.seal();
history.push(3);

history.undo();
assert_eq!(**history.present(), 2);
history.undo();
assert_eq!(**history.present(), 0);
```

## Grouping

A group of merges closes on whichever comes first: `group_ms` without a merge,
its size limit, or a `seal`, push, undo or redo. `UndoHistory` itself has no dioxus
in it: build one with `UndoHistory::new(value).with_cap(50).with_group_max(20)`
(defaults 200 steps and 50 changes per group) and use it on its own or hand it
to the hook. Each snapshot is held in an `Rc`, so cloning a history or a value
is cheap.

## Accessibility

### Libero handles

- Nothing on screen: the hook keeps snapshots, your controls show them.

### You must

- Offer undo and redo from the keyboard as well, as the demo does with Ctrl+Z
  and Ctrl+Shift+Z (Cmd on macOS) on the element around the field; a button
  alone leaves a keyboard user tabbing away from the field.
- Keep the Undo and Redo buttons focusable in the tab order, even at the end of
  the stack (`focusable_when_disabled`), and named by their text or
  `aria-label`.

### Example

A notes field with `use_history`: Ctrl+Z and Ctrl+Shift+Z undo and redo in the
field, and the Undo and Redo buttons stay in the tab order with
`focusable_when_disabled` at either end of the stack.

## API

```rust,ignore
pub fn use_history<T: 'static>(initial: impl FnOnce() -> UndoHistory<T>, group_ms: u64) -> HistoryHandle<T>
```

| `HistoryHandle<T>` method | Does |
|---|---|
| `value() -> Rc<T>` | The current snapshot. Reactive. |
| `can_undo()`, `can_redo()` | Whether there is a step to move to. Reactive. |
| `read(\|history\| ..)` | Reads the whole `UndoHistory`. Reactive. |
| `push(value)` | Records a step of its own; drops the redo steps. |
| `merge(value)` | Folds `value` into the open group, or opens one; drops the redo steps. |
| `seal()` | Closes the open group now, e.g. at a word end or on blur. |
| `undo() -> bool`, `redo() -> bool` | Closes the open group, then moves one step; `false` at the end. |
| `reset(value)` | Starts over at `value`, nothing to undo or redo. |

| `UndoHistory<T>` | Does |
|---|---|
| `new(value)` | At `value`, 200 undo steps, 50 changes per group. |
| `with_cap(n)` | Keeps at most `n` undo steps; the oldest drop first. |
| `with_group_max(n)` | Lets a group merge at most `n` changes. |
| `present() -> &Rc<T>` | The current snapshot. |

`UndoHistory` has the same `push`, `merge`, `seal`, `undo`, `redo`, `reset`,
`can_undo` and `can_redo` as the handle. `group_ms` 0 never closes a group by
time, nor does a target without a timer, such as a server render. Sealing a
group renders nothing; every other change renders the readers. Call the hook
unconditionally, in the same order every render.
