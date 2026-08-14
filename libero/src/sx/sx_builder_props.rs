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

    pub fn padding(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Padding, value)
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

    pub fn margin_top(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::MarginTop, value)
    }

    pub fn margin_bottom(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::MarginBottom, value)
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

    pub fn text_decoration(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::TextDecoration, value)
    }

    pub fn border(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Border, value)
    }

    pub fn border_top(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::BorderTop, value)
    }

    pub fn border_right(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::BorderRight, value)
    }

    pub fn border_bottom(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::BorderBottom, value)
    }

    pub fn border_left(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::BorderLeft, value)
    }

    pub fn color(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Color, value)
    }

    pub fn flex(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Flex, value)
    }

    pub fn flex_grow(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::FlexGrow, value)
    }

    pub fn content(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Content, value)
    }

    pub fn border_width(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::BorderWidth, value)
    }

    pub fn border_style(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::BorderStyle, value)
    }

    pub fn border_color(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::BorderColor, value)
    }

    pub fn border_right_color(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::BorderRightColor, value)
    }

    pub fn border_bottom_color(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::BorderBottomColor, value)
    }

    pub fn flex_shrink(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::FlexShrink, value)
    }

    pub fn align_self(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::AlignSelf, value)
    }

    pub fn white_space(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::WhiteSpace, value)
    }

    pub fn user_select(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::UserSelect, value)
    }

    pub fn position(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Position, value)
    }

    pub fn overflow(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Overflow, value)
    }

    pub fn clip(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Clip, value)
    }

    pub fn object_fit(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::ObjectFit, value)
    }

    pub fn border_radius(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::BorderRadius, value)
    }

    pub fn cursor(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Cursor, value)
    }

    pub fn opacity(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::Opacity, value)
    }

    pub fn pointer_events(self, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_known_property(Property::PointerEvents, value)
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
