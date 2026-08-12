use crate::theme::Size;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Property {
    Background,
    Width,
    Height,
    PaddingTop,
    PaddingLeft,
    PaddingRight,
    MarginLeft,
    MarginRight,
    Display,
    FlexDirection,
    FlexWrap,
    AlignItems,
    JustifyContent,
    Gap,
    MaxWidth,
}

impl Property {
    pub fn parse(property: &str) -> Option<Self> {
        match property {
            "background" => Some(Self::Background),
            "width" => Some(Self::Width),
            "height" => Some(Self::Height),
            "padding-top" => Some(Self::PaddingTop),
            "padding-left" => Some(Self::PaddingLeft),
            "padding-right" => Some(Self::PaddingRight),
            "margin-left" => Some(Self::MarginLeft),
            "margin-right" => Some(Self::MarginRight),
            "display" => Some(Self::Display),
            "flex-direction" => Some(Self::FlexDirection),
            "flex-wrap" => Some(Self::FlexWrap),
            "align-items" => Some(Self::AlignItems),
            "justify-content" => Some(Self::JustifyContent),
            "gap" => Some(Self::Gap),
            "max-width" => Some(Self::MaxWidth),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Width => "width",
            Self::Height => "height",
            Self::PaddingTop => "padding-top",
            Self::PaddingLeft => "padding-left",
            Self::PaddingRight => "padding-right",
            Self::MarginLeft => "margin-left",
            Self::MarginRight => "margin-right",
            Self::Display => "display",
            Self::FlexDirection => "flex-direction",
            Self::FlexWrap => "flex-wrap",
            Self::AlignItems => "align-items",
            Self::JustifyContent => "justify-content",
            Self::Gap => "gap",
            Self::MaxWidth => "max-width",
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
