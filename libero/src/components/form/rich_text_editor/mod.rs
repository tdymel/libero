//! The rich text editor (epic 1159): a pure model and the view over it.

mod dialogs;
mod handle;
mod input;
pub(crate) mod model;
mod node_view;
mod offsets;
mod render;
mod surface;
mod toolbar;
mod view;

pub use view::{RichTextEditor, RichTextEditorProps};

/// The document a [`RichTextEditor`] edits and the commands, keys and nodes it runs.
pub mod rich_text {
    pub use super::handle::{ListKind, RichTextHandle, use_rich_text_editor};
    pub use super::input::EditorInput;
    pub use super::model::{
        Action, Attrs, Block, BlockKind, Builtin, Chord, ChordError, CommandName, Commands,
        Content, ContentKind, CustomContent, Doc, EditFn, EditorState, Href, Inline, KeyPress,
        Keymap, Mark, MarkKind, Marks, NodeKey, NodeRegistry, NodeSpec, Placement, Position,
        Record, RegistryError, Selection, ToMarkdown, UnsafeHref,
    };
    pub use super::node_view::{NodeViewProps, NodeViews};
    pub use super::toolbar::RichTextTool;
    pub use super::view::{RichTextEditor, RichTextEditorProps};
}
