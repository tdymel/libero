mod a11y_doc;
mod demo;
mod doc_page;
mod doc_section;
mod prop_doc;
mod tldr;

pub use a11y_doc::{A11yDoc, A11yPanel, a11y};
#[cfg(test)]
pub use demo::DemoCode;
pub use demo::{Child, Control, Demo, DemoValues, UNSET, Wrap, indent, or_unset};
pub use doc_page::DocPage;
pub use doc_section::DocSection;
pub use prop_doc::{PropGroup, PropertyTable, prop, props};
use tldr::Tldr;
