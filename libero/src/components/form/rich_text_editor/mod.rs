//! The rich text editor (epic 1159): a pure model and the view over it.

mod input;
pub(crate) mod model;
mod offsets;
mod render;
mod surface;
mod view;

pub use view::{RichTextEditor, RichTextEditorProps};

/// The document a [`RichTextEditor`] edits and the commands, keys and nodes it runs.
pub mod rich_text {
    pub use super::model::{
        Action, Attrs, Block, BlockKind, Builtin, Chord, ChordError, CommandName, Commands,
        Content, ContentKind, CustomContent, Doc, EditFn, EditorState, Href, Inline, KeyPress,
        Keymap, Mark, MarkKind, Marks, NodeKey, NodeRegistry, NodeSpec, Placement, Position,
        Record, RegistryError, Selection, ToMarkdown, UnsafeHref,
    };
    pub use super::view::{RichTextEditor, RichTextEditorProps};
}
