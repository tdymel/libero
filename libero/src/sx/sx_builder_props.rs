use crate::theme::Size;

use super::{declaration::Property, sx_builder::SxBuilder, sx_modifier::SxModifier};

impl SxBuilder {
    pub const fn background(self, value: &'static str) -> Self {
        self.with_known_property(Property::Background, value)
    }

    pub const fn width(self, value: &'static str) -> Self {
        self.with_known_property(Property::Width, value)
    }

    pub const fn height(self, value: &'static str) -> Self {
        self.with_known_property(Property::Height, value)
    }

    pub const fn padding_top(self, value: &'static str) -> Self {
        self.with_known_property(Property::PaddingTop, value)
    }

    pub const fn hover(self, nested: SxBuilder) -> Self {
        self.selector(":hover", nested)
    }

    pub const fn focus(self, nested: SxBuilder) -> Self {
        self.selector(":focus", nested)
    }

    pub const fn when(self, condition: &'static str, nested: SxBuilder) -> Self {
        self.modifier(SxModifier::Condition(condition), nested)
    }

    pub const fn selector(self, selector: &'static str, nested: SxBuilder) -> Self {
        self.modifier(SxModifier::Selector(selector), nested)
    }

    pub const fn breakpoint(self, breakpoint: Size, nested: SxBuilder) -> Self {
        self.modifier(SxModifier::Breakpoint(breakpoint), nested)
    }
}
