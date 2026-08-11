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

    pub const fn display(self, value: &'static str) -> Self {
        self.with_known_property(Property::Display, value)
    }

    pub const fn flex_direction(self, value: &'static str) -> Self {
        self.with_known_property(Property::FlexDirection, value)
    }

    pub const fn flex_wrap(self, value: &'static str) -> Self {
        self.with_known_property(Property::FlexWrap, value)
    }

    pub const fn align_items(self, value: &'static str) -> Self {
        self.with_known_property(Property::AlignItems, value)
    }

    pub const fn justify_content(self, value: &'static str) -> Self {
        self.with_known_property(Property::JustifyContent, value)
    }

    pub const fn gap(self, value: &'static str) -> Self {
        self.with_known_property(Property::Gap, value)
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
