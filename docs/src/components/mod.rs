mod a11y_doc;
mod demo;
mod doc_page;
mod doc_section;
mod icon_catalogue;
mod pictogram_note;
mod prop_doc;

pub use a11y_doc::{A11yDoc, A11yPanel, a11y};
#[cfg(test)]
pub use demo::DemoCode;
pub use demo::{
    Child, Control, Demo, DemoFile, DemoValues, UNSET, Wrap, align_of, delay_of, gradient_controls,
    gradient_value, indent, not_gradient_variant, or_unset, side_of,
};
pub use doc_page::{DocPage, ExtraTab};
pub use doc_section::{DocSection, SectionLink};
pub use icon_catalogue::IconCatalogue;
pub use pictogram_note::PictogramNote;
pub use prop_doc::{
    PartsPanel, PropGroup, PropertyTable, disabled_prop, prop, props, prose, readonly_prop,
    required_prop, status_prop,
};
