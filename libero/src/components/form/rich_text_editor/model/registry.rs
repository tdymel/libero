//! The node registry's data side: which node types exist, their content and codecs.
//! The view adds a component per type; built-in types are registered the same way.

use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

use super::doc::{Attrs, Block, BlockKind, CustomContent, Doc, Inline, visit};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Block,
    Inline,
}

/// Writes a node as Markdown: its attrs and, for a container, its content already written.
pub type ToMarkdown = Rc<dyn Fn(&Attrs, &str) -> String>;

#[derive(Clone)]
pub struct NodeSpec {
    pub name: String,
    pub placement: Placement,
    /// A block's content; ignored for inline nodes, which are atoms.
    pub content: CustomContent,
    pub default_attrs: Attrs,
    /// Without one, an inline node writes nothing and a block writes its content.
    pub to_markdown: Option<ToMarkdown>,
    pub builtin: bool,
}

impl fmt::Debug for NodeSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeSpec")
            .field("name", &self.name)
            .field("placement", &self.placement)
            .field("content", &self.content)
            .field("builtin", &self.builtin)
            .finish_non_exhaustive()
    }
}

impl NodeSpec {
    pub fn inline(name: impl Into<String>) -> Self {
        Self::new(name, Placement::Inline, CustomContent::Atom)
    }

    pub fn block(name: impl Into<String>, content: CustomContent) -> Self {
        Self::new(name, Placement::Block, content)
    }

    fn new(name: impl Into<String>, placement: Placement, content: CustomContent) -> Self {
        Self {
            name: name.into(),
            placement,
            content,
            default_attrs: Attrs::new(),
            to_markdown: None,
            builtin: false,
        }
    }

    pub fn attr(mut self, name: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.default_attrs.insert(name.into(), value.into());
        self
    }

    pub fn markdown(mut self, write: impl Fn(&Attrs, &str) -> String + 'static) -> Self {
        self.to_markdown = Some(Rc::new(write));
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegistryError {
    /// The name is taken by a built-in node.
    Builtin(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin(name) => write!(f, "{name:?} is a built-in node"),
        }
    }
}

impl std::error::Error for RegistryError {}

#[derive(Clone, Debug)]
pub struct NodeRegistry {
    specs: BTreeMap<String, NodeSpec>,
}

impl Default for NodeRegistry {
    fn default() -> Self {
        let mut specs = BTreeMap::new();
        for (name, content) in Self::BUILTIN {
            let mut spec = NodeSpec::block(*name, *content);
            spec.builtin = true;
            specs.insert(name.to_string(), spec);
        }
        Self { specs }
    }
}

impl NodeRegistry {
    /// The built-in block types, by the `type` they serialise as.
    pub const BUILTIN: &[(&str, CustomContent)] = &[
        ("paragraph", CustomContent::Inline),
        ("heading", CustomContent::Inline),
        ("code_block", CustomContent::Inline),
        ("quote", CustomContent::Blocks),
        ("list", CustomContent::Blocks),
        ("list_item", CustomContent::Blocks),
        ("rule", CustomContent::Atom),
    ];

    /// Adds or replaces a caller's node type. Built-in names are refused.
    pub fn register(&mut self, spec: NodeSpec) -> Result<&mut Self, RegistryError> {
        if self
            .specs
            .get(&spec.name)
            .is_some_and(|known| known.builtin)
        {
            return Err(RegistryError::Builtin(spec.name));
        }
        self.specs.insert(spec.name.clone(), spec);
        Ok(self)
    }

    pub fn get(&self, name: &str) -> Option<&NodeSpec> {
        self.specs.get(name)
    }

    pub fn specs(&self) -> impl Iterator<Item = &NodeSpec> {
        self.specs.values()
    }

    /// A new block of a registered block type, with its default attrs under `attrs`.
    pub fn new_block(&self, doc: &mut Doc, name: &str, attrs: Attrs) -> Option<Block> {
        let spec = self
            .get(name)
            .filter(|spec| spec.placement == Placement::Block && !spec.builtin)?;
        let mut merged = spec.default_attrs.clone();
        merged.extend(attrs);
        let kind = BlockKind::Custom {
            name: spec.name.clone(),
            attrs: merged,
            content: spec.content,
        };
        let block = match spec.content {
            CustomContent::Blocks => {
                let paragraph = doc.leaf(BlockKind::Paragraph, Vec::new());
                doc.container(kind, vec![paragraph])
            }
            _ => doc.leaf(kind, Vec::new()),
        };
        Some(block)
    }

    /// A new inline node of a registered inline type.
    pub fn new_inline(&self, name: &str, attrs: Attrs) -> Option<Inline> {
        let spec = self
            .get(name)
            .filter(|spec| spec.placement == Placement::Inline)?;
        let mut merged = spec.default_attrs.clone();
        merged.extend(attrs);
        Some(Inline::Node {
            name: spec.name.clone(),
            attrs: merged,
        })
    }

    /// The custom node names `doc` uses that are not registered here.
    pub fn unknown_in(&self, doc: &Doc) -> Vec<String> {
        let mut unknown = Vec::new();
        let mut note = |name: &str| {
            if self.get(name).is_none() && !unknown.iter().any(|known| known == name) {
                unknown.push(name.to_string());
            }
        };
        visit(&doc.blocks, &mut |block| {
            if let BlockKind::Custom { name, .. } = &block.kind {
                note(name);
            }
            for inline in block.inlines() {
                if let Inline::Node { name, .. } = inline {
                    note(name);
                }
            }
        });
        unknown
    }
}
