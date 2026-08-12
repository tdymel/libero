use crate::theme::Size;

use super::{
    declaration::{Property, SxModifierKey},
    sx::Sx,
};

impl Sx {
    pub fn background(self, value: impl Into<String>) -> Self {
        self.with_known_property(Property::Background, value)
    }

    pub fn width(self, value: impl Into<String>) -> Self {
        self.with_known_property(Property::Width, value)
    }

    pub fn height(self, value: impl Into<String>) -> Self {
        self.with_known_property(Property::Height, value)
    }

    pub fn padding_top(self, value: impl Into<String>) -> Self {
        self.with_known_property(Property::PaddingTop, value)
    }

    pub fn display(self, value: impl Into<String>) -> Self {
        self.with_known_property(Property::Display, value)
    }

    pub fn flex_direction(self, value: impl Into<String>) -> Self {
        self.with_known_property(Property::FlexDirection, value)
    }

    pub fn flex_wrap(self, value: impl Into<String>) -> Self {
        self.with_known_property(Property::FlexWrap, value)
    }

    pub fn align_items(self, value: impl Into<String>) -> Self {
        self.with_known_property(Property::AlignItems, value)
    }

    pub fn justify_content(self, value: impl Into<String>) -> Self {
        self.with_known_property(Property::JustifyContent, value)
    }

    pub fn gap(self, value: impl Into<String>) -> Self {
        self.with_known_property(Property::Gap, value)
    }

    pub fn hover(self, nested: Sx) -> Self {
        self.selector(":hover", nested)
    }

    pub fn focus(self, nested: Sx) -> Self {
        self.selector(":focus", nested)
    }

    pub fn when(self, condition: impl Into<String>, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Condition(condition.into()), nested)
    }

    pub fn selector(self, selector: impl Into<String>, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Selector(selector.into()), nested)
    }

    pub fn breakpoint(self, breakpoint: Size, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Breakpoint(breakpoint), nested)
    }
}
