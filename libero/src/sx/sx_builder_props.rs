use crate::theme::Size;

use super::{Property, Sx, SxModifierKey, ThemeAwareValue};

impl Sx {
    pub fn background(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Background, value)
    }

    pub fn width(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Width, value)
    }

    pub fn height(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Height, value)
    }

    pub fn padding_top(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::PaddingTop, value)
    }

    pub fn padding_left(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::PaddingLeft, value)
    }

    pub fn padding_right(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::PaddingRight, value)
    }

    pub fn display(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Display, value)
    }

    pub fn margin_left(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::MarginLeft, value)
    }

    pub fn margin_right(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::MarginRight, value)
    }

    pub fn flex_direction(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::FlexDirection, value)
    }

    pub fn flex_wrap(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::FlexWrap, value)
    }

    pub fn align_items(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::AlignItems, value)
    }

    pub fn justify_content(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::JustifyContent, value)
    }

    pub fn gap(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Gap, value)
    }

    pub fn max_width(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::MaxWidth, value)
    }

    pub fn margin(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Margin, value)
    }

    pub fn font_family(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::FontFamily, value)
    }

    pub fn font_size(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::FontSize, value)
    }

    pub fn font_weight(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::FontWeight, value)
    }

    pub fn letter_spacing(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::LetterSpacing, value)
    }

    pub fn line_height(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::LineHeight, value)
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
