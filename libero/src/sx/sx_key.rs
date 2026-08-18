use crate::theme::Size;

use super::{Sx, ThemeAwareValue};

/// The single source of truth for every CSS property `Sx` knows: enum
/// variant, its CSS name, and its `Sx` builder method. Adding a property is
/// one line here.
macro_rules! properties {
    ($($variant:ident => $css_name:literal, $method:ident;)*) => {
        #[repr(u16)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Property {
            $($variant,)*
        }

        impl Property {
            pub fn parse(property: &str) -> Option<Self> {
                match property {
                    $($css_name => Some(Self::$variant),)*
                    _ => None,
                }
            }

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $css_name,)*
                }
            }

            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant,)*];
        }

        impl Sx {
            $(
                pub fn $method(self, value: impl Into<ThemeAwareValue>) -> Self {
                    self.with_known_property(Property::$variant, value)
                }
            )*
        }
    };
}

properties! {
    Background => "background", background;
    Width => "width", width;
    Height => "height", height;
    Padding => "padding", padding;
    PaddingTop => "padding-top", padding_top;
    PaddingLeft => "padding-left", padding_left;
    PaddingRight => "padding-right", padding_right;
    PaddingBottom => "padding-bottom", padding_bottom;
    MarginLeft => "margin-left", margin_left;
    MarginRight => "margin-right", margin_right;
    MarginTop => "margin-top", margin_top;
    MarginBottom => "margin-bottom", margin_bottom;
    Margin => "margin", margin;
    Display => "display", display;
    FlexDirection => "flex-direction", flex_direction;
    FlexWrap => "flex-wrap", flex_wrap;
    AlignItems => "align-items", align_items;
    JustifyContent => "justify-content", justify_content;
    Gap => "gap", gap;
    GridTemplateColumns => "grid-template-columns", grid_template_columns;
    GridColumn => "grid-column", grid_column;
    MaxWidth => "max-width", max_width;
    MaxHeight => "max-height", max_height;
    FontFamily => "font-family", font_family;
    FontSize => "font-size", font_size;
    FontWeight => "font-weight", font_weight;
    LetterSpacing => "letter-spacing", letter_spacing;
    LineHeight => "line-height", line_height;
    TextDecoration => "text-decoration", text_decoration;
    Border => "border", border;
    BorderTop => "border-top", border_top;
    BorderRight => "border-right", border_right;
    BorderBottom => "border-bottom", border_bottom;
    BorderLeft => "border-left", border_left;
    Color => "color", color;
    Flex => "flex", flex;
    FlexGrow => "flex-grow", flex_grow;
    Content => "content", content;
    BorderWidth => "border-width", border_width;
    BorderStyle => "border-style", border_style;
    BorderColor => "border-color", border_color;
    BorderRightColor => "border-right-color", border_right_color;
    BorderBottomColor => "border-bottom-color", border_bottom_color;
    FlexShrink => "flex-shrink", flex_shrink;
    AlignSelf => "align-self", align_self;
    WhiteSpace => "white-space", white_space;
    UserSelect => "user-select", user_select;
    Position => "position", position;
    Top => "top", top;
    Right => "right", right;
    Bottom => "bottom", bottom;
    Left => "left", left;
    Overflow => "overflow", overflow;
    OverflowX => "overflow-x", overflow_x;
    OverflowY => "overflow-y", overflow_y;
    ScrollbarWidth => "scrollbar-width", scrollbar_width;
    ScrollbarColor => "scrollbar-color", scrollbar_color;
    Clip => "clip", clip;
    ObjectFit => "object-fit", object_fit;
    BorderRadius => "border-radius", border_radius;
    Cursor => "cursor", cursor;
    Opacity => "opacity", opacity;
    PointerEvents => "pointer-events", pointer_events;
    ListStyle => "list-style", list_style;
    MinWidth => "min-width", min_width;
    MinHeight => "min-height", min_height;
    AspectRatio => "aspect-ratio", aspect_ratio;
    Outline => "outline", outline;
    OutlineOffset => "outline-offset", outline_offset;
    Inset => "inset", inset;
    ZIndex => "z-index", z_index;
    Transition => "transition", transition;
    BackdropFilter => "backdrop-filter", backdrop_filter;
    BoxShadow => "box-shadow", box_shadow;
    TextAlign => "text-align", text_align;
    ScrollMargin => "scroll-margin", scroll_margin;
    Transform => "transform", transform;
    Visibility => "visibility", visibility;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_property_parses_back_from_its_css_name() {
        for property in Property::ALL {
            assert_eq!(Property::parse(property.as_str()), Some(*property));
        }
    }

    #[test]
    fn a_builder_method_sets_its_own_property() {
        assert_eq!(
            SxPropertyKey::parse("z-index"),
            SxPropertyKey::Known(Property::ZIndex)
        );
    }
}
