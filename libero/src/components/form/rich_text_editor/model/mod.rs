//! The editor's document core: no dioxus here, so it runs and tests natively.

mod command;
mod doc;
mod edit;
mod editor;
mod mark;
mod markdown;
mod parse;
mod registry;
mod rules;
mod state;
mod structure;
mod syntax;
#[cfg(test)]
mod tests;

pub use command::{
    Action, Builtin, Chord, ChordError, CommandName, Commands, EditFn, KeyPress, Keymap, Record,
};
pub use doc::{Attrs, Block, BlockKind, Content, ContentKind, CustomContent, Doc, Inline, NodeKey};
pub use editor::{Editor, UndoStack};
pub use mark::{Href, Mark, MarkKind, Marks, UnsafeHref};
pub use registry::{NodeRegistry, NodeSpec, Placement, RegistryError, ToMarkdown};
pub use state::{EditorState, Position, Selection};
