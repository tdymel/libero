use crate::theme::Size;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Property {
    Background,
    Width,
    Height,
    Padding,
    PaddingTop,
    PaddingLeft,
    PaddingRight,
    MarginLeft,
    MarginRight,
    Margin,
    Display,
    FlexDirection,
    FlexWrap,
    AlignItems,
    JustifyContent,
    Gap,
    MaxWidth,
    FontFamily,
    FontSize,
    FontWeight,
    LetterSpacing,
    LineHeight,
    TextDecoration,
    Border,
    BorderTop,
    BorderRight,
    BorderBottom,
    BorderLeft,
    Color,
    Flex,
    FlexGrow,
    Content,
    BorderWidth,
    BorderStyle,
    BorderColor,
    FlexShrink,
    AlignSelf,
    WhiteSpace,
    UserSelect,
}

impl Property {
    pub fn parse(property: &str) -> Option<Self> {
        match property {
            "background" => Some(Self::Background),
            "width" => Some(Self::Width),
            "height" => Some(Self::Height),
            "padding" => Some(Self::Padding),
            "padding-top" => Some(Self::PaddingTop),
            "padding-left" => Some(Self::PaddingLeft),
            "padding-right" => Some(Self::PaddingRight),
            "margin-left" => Some(Self::MarginLeft),
            "margin-right" => Some(Self::MarginRight),
            "margin" => Some(Self::Margin),
            "display" => Some(Self::Display),
            "flex-direction" => Some(Self::FlexDirection),
            "flex-wrap" => Some(Self::FlexWrap),
            "align-items" => Some(Self::AlignItems),
            "justify-content" => Some(Self::JustifyContent),
            "gap" => Some(Self::Gap),
            "max-width" => Some(Self::MaxWidth),
            "font-family" => Some(Self::FontFamily),
            "font-size" => Some(Self::FontSize),
            "font-weight" => Some(Self::FontWeight),
            "letter-spacing" => Some(Self::LetterSpacing),
            "line-height" => Some(Self::LineHeight),
            "text-decoration" => Some(Self::TextDecoration),
            "border" => Some(Self::Border),
            "border-top" => Some(Self::BorderTop),
            "border-right" => Some(Self::BorderRight),
            "border-bottom" => Some(Self::BorderBottom),
            "border-left" => Some(Self::BorderLeft),
            "color" => Some(Self::Color),
            "flex" => Some(Self::Flex),
            "flex-grow" => Some(Self::FlexGrow),
            "content" => Some(Self::Content),
            "border-width" => Some(Self::BorderWidth),
            "border-style" => Some(Self::BorderStyle),
            "border-color" => Some(Self::BorderColor),
            "flex-shrink" => Some(Self::FlexShrink),
            "align-self" => Some(Self::AlignSelf),
            "white-space" => Some(Self::WhiteSpace),
            "user-select" => Some(Self::UserSelect),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Width => "width",
            Self::Height => "height",
            Self::Padding => "padding",
            Self::PaddingTop => "padding-top",
            Self::PaddingLeft => "padding-left",
            Self::PaddingRight => "padding-right",
            Self::MarginLeft => "margin-left",
            Self::MarginRight => "margin-right",
            Self::Margin => "margin",
            Self::Display => "display",
            Self::FlexDirection => "flex-direction",
            Self::FlexWrap => "flex-wrap",
            Self::AlignItems => "align-items",
            Self::JustifyContent => "justify-content",
            Self::Gap => "gap",
            Self::MaxWidth => "max-width",
            Self::FontFamily => "font-family",
            Self::FontSize => "font-size",
            Self::FontWeight => "font-weight",
            Self::LetterSpacing => "letter-spacing",
            Self::LineHeight => "line-height",
            Self::TextDecoration => "text-decoration",
            Self::Border => "border",
            Self::BorderTop => "border-top",
            Self::BorderRight => "border-right",
            Self::BorderBottom => "border-bottom",
            Self::BorderLeft => "border-left",
            Self::Color => "color",
            Self::Flex => "flex",
            Self::FlexGrow => "flex-grow",
            Self::Content => "content",
            Self::BorderWidth => "border-width",
            Self::BorderStyle => "border-style",
            Self::BorderColor => "border-color",
            Self::FlexShrink => "flex-shrink",
            Self::AlignSelf => "align-self",
            Self::WhiteSpace => "white-space",
            Self::UserSelect => "user-select",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SxPropertyKey {
    Known(Property),
    Raw(String),
}

impl SxPropertyKey {
    pub fn parse(property: impl Into<String>) -> Self {
        let property = property.into();
        match Property::parse(&property) {
            Some(property) => Self::Known(property),
            None => Self::Raw(property),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Known(property) => property.as_str(),
            Self::Raw(property) => property.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SxModifierKey {
    Selector(String),
    Condition(String),
    Breakpoint(Size),
}
