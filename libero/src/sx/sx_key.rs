use crate::tokens::{Size, SizeCss};

use super::{Sx, ThemeAwareValue};

/// Every CSS property `Sx` knows: variant, CSS name, builder method, and the
/// [`SizeCss`] scale a bare `Size` resolves through. Adding a property is one
/// line here.
macro_rules! properties {
    (@scale) => { None };
    (@scale $scale:expr) => { Some($scale) };
    ($($variant:ident => $css_name:literal, $method:ident $(, $scale:expr)?;)*) => {
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

            /// `None` keeps the size's own name (`"md"`).
            pub(crate) const fn size_scale(self) -> Option<SizeCss> {
                match self {
                    $(Self::$variant => properties!(@scale $($scale)?),)*
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
    BackgroundColor => "background-color", background_color;
    Width => "width", width;
    Height => "height", height;
    Padding => "padding", padding, SizeCss::SPACING;
    PaddingTop => "padding-top", padding_top, SizeCss::SPACING;
    PaddingLeft => "padding-left", padding_left, SizeCss::SPACING;
    PaddingRight => "padding-right", padding_right, SizeCss::SPACING;
    PaddingBottom => "padding-bottom", padding_bottom, SizeCss::SPACING;
    MarginLeft => "margin-left", margin_left, SizeCss::SPACING;
    MarginRight => "margin-right", margin_right, SizeCss::SPACING;
    MarginTop => "margin-top", margin_top, SizeCss::SPACING;
    MarginBottom => "margin-bottom", margin_bottom, SizeCss::SPACING;
    Margin => "margin", margin, SizeCss::SPACING;
    Display => "display", display;
    FlexDirection => "flex-direction", flex_direction;
    FlexWrap => "flex-wrap", flex_wrap;
    AlignItems => "align-items", align_items;
    JustifyContent => "justify-content", justify_content;
    Gap => "gap", gap, SizeCss::SPACING;
    GridTemplateColumns => "grid-template-columns", grid_template_columns;
    GridTemplateRows => "grid-template-rows", grid_template_rows;
    GridTemplateAreas => "grid-template-areas", grid_template_areas;
    GridColumn => "grid-column", grid_column;
    GridRow => "grid-row", grid_row;
    GridArea => "grid-area", grid_area;
    GridAutoRows => "grid-auto-rows", grid_auto_rows;
    GridAutoFlow => "grid-auto-flow", grid_auto_flow;
    AlignContent => "align-content", align_content;
    RowGap => "row-gap", row_gap, SizeCss::SPACING;
    ColumnGap => "column-gap", column_gap, SizeCss::SPACING;
    MaxWidth => "max-width", max_width, SizeCss::BREAKPOINT;
    MaxHeight => "max-height", max_height;
    Font => "font", font;
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
    Appearance => "appearance", appearance;
    TouchAction => "touch-action", touch_action;
    Position => "position", position;
    Top => "top", top, SizeCss::SPACING;
    Right => "right", right, SizeCss::SPACING;
    Bottom => "bottom", bottom, SizeCss::SPACING;
    Left => "left", left, SizeCss::SPACING;
    Overflow => "overflow", overflow;
    OverflowX => "overflow-x", overflow_x;
    OverflowY => "overflow-y", overflow_y;
    ContainerType => "container-type", container_type;
    ContainerName => "container-name", container_name;
    ScrollbarWidth => "scrollbar-width", scrollbar_width;
    ScrollbarColor => "scrollbar-color", scrollbar_color;
    Clip => "clip", clip;
    ClipPath => "clip-path", clip_path;
    ObjectFit => "object-fit", object_fit;
    BorderRadius => "border-radius", border_radius, SizeCss::RADIUS;
    BorderTopLeftRadius => "border-top-left-radius", border_top_left_radius, SizeCss::RADIUS;
    BorderTopRightRadius => "border-top-right-radius", border_top_right_radius, SizeCss::RADIUS;
    BorderBottomLeftRadius => "border-bottom-left-radius", border_bottom_left_radius, SizeCss::RADIUS;
    BorderBottomRightRadius => "border-bottom-right-radius", border_bottom_right_radius, SizeCss::RADIUS;
    Cursor => "cursor", cursor;
    Opacity => "opacity", opacity;
    PointerEvents => "pointer-events", pointer_events;
    ListStyle => "list-style", list_style;
    MinWidth => "min-width", min_width, SizeCss::BREAKPOINT;
    MinHeight => "min-height", min_height;
    Resize => "resize", resize;
    AspectRatio => "aspect-ratio", aspect_ratio;
    Outline => "outline", outline;
    OutlineOffset => "outline-offset", outline_offset;
    Inset => "inset", inset, SizeCss::SPACING;
    ZIndex => "z-index", z_index;
    Transition => "transition", transition;
    BackdropFilter => "backdrop-filter", backdrop_filter;
    BoxShadow => "box-shadow", box_shadow, SizeCss::SHADOW;
    TextAlign => "text-align", text_align;
    TextOverflow => "text-overflow", text_overflow;
    TextTransform => "text-transform", text_transform;
    ScrollMargin => "scroll-margin", scroll_margin;
    ScrollPaddingInline => "scroll-padding-inline", scroll_padding_inline;
    ScrollBehavior => "scroll-behavior", scroll_behavior;
    ScrollSnapType => "scroll-snap-type", scroll_snap_type;
    ScrollSnapAlign => "scroll-snap-align", scroll_snap_align;
    ScrollSnapStop => "scroll-snap-stop", scroll_snap_stop;
    OverscrollBehaviorX => "overscroll-behavior-x", overscroll_behavior_x;
    OverscrollBehaviorY => "overscroll-behavior-y", overscroll_behavior_y;
    Transform => "transform", transform;
    Animation => "animation", animation;
    AnimationDirection => "animation-direction", animation_direction;
    AnimationPlayState => "animation-play-state", animation_play_state;
    Visibility => "visibility", visibility;
    BorderCollapse => "border-collapse", border_collapse;
    VerticalAlign => "vertical-align", vertical_align;

    BoxSizing => "box-sizing", box_sizing;
    Float => "float", float;
    Clear => "clear", clear;
    Isolation => "isolation", isolation;
    Contain => "contain", contain;
    ContentVisibility => "content-visibility", content_visibility;
    WillChange => "will-change", will_change;
    All => "all", all;
    InlineSize => "inline-size", inline_size;
    BlockSize => "block-size", block_size;
    MinInlineSize => "min-inline-size", min_inline_size, SizeCss::BREAKPOINT;
    MaxInlineSize => "max-inline-size", max_inline_size, SizeCss::BREAKPOINT;
    MinBlockSize => "min-block-size", min_block_size;
    MaxBlockSize => "max-block-size", max_block_size;
    MarginInline => "margin-inline", margin_inline, SizeCss::SPACING;
    MarginInlineStart => "margin-inline-start", margin_inline_start, SizeCss::SPACING;
    MarginInlineEnd => "margin-inline-end", margin_inline_end, SizeCss::SPACING;
    MarginBlock => "margin-block", margin_block, SizeCss::SPACING;
    MarginBlockStart => "margin-block-start", margin_block_start, SizeCss::SPACING;
    MarginBlockEnd => "margin-block-end", margin_block_end, SizeCss::SPACING;
    PaddingInline => "padding-inline", padding_inline, SizeCss::SPACING;
    PaddingInlineStart => "padding-inline-start", padding_inline_start, SizeCss::SPACING;
    PaddingInlineEnd => "padding-inline-end", padding_inline_end, SizeCss::SPACING;
    PaddingBlock => "padding-block", padding_block, SizeCss::SPACING;
    PaddingBlockStart => "padding-block-start", padding_block_start, SizeCss::SPACING;
    PaddingBlockEnd => "padding-block-end", padding_block_end, SizeCss::SPACING;
    InsetInline => "inset-inline", inset_inline, SizeCss::SPACING;
    InsetInlineStart => "inset-inline-start", inset_inline_start, SizeCss::SPACING;
    InsetInlineEnd => "inset-inline-end", inset_inline_end, SizeCss::SPACING;
    InsetBlock => "inset-block", inset_block, SizeCss::SPACING;
    InsetBlockStart => "inset-block-start", inset_block_start, SizeCss::SPACING;
    InsetBlockEnd => "inset-block-end", inset_block_end, SizeCss::SPACING;

    FlexBasis => "flex-basis", flex_basis;
    FlexFlow => "flex-flow", flex_flow;
    Order => "order", order;
    JustifyItems => "justify-items", justify_items;
    JustifySelf => "justify-self", justify_self;
    PlaceItems => "place-items", place_items;
    PlaceContent => "place-content", place_content;
    PlaceSelf => "place-self", place_self;
    Grid => "grid", grid;
    GridTemplate => "grid-template", grid_template;
    GridAutoColumns => "grid-auto-columns", grid_auto_columns;
    GridColumnStart => "grid-column-start", grid_column_start;
    GridColumnEnd => "grid-column-end", grid_column_end;
    GridRowStart => "grid-row-start", grid_row_start;
    GridRowEnd => "grid-row-end", grid_row_end;
    Columns => "columns", columns;
    ColumnCount => "column-count", column_count;
    ColumnWidth => "column-width", column_width;
    BreakInside => "break-inside", break_inside;
    BreakBefore => "break-before", break_before;
    BreakAfter => "break-after", break_after;

    FontStyle => "font-style", font_style;
    FontVariant => "font-variant", font_variant;
    FontVariantNumeric => "font-variant-numeric", font_variant_numeric;
    FontFeatureSettings => "font-feature-settings", font_feature_settings;
    FontVariationSettings => "font-variation-settings", font_variation_settings;
    FontStretch => "font-stretch", font_stretch;
    TextRendering => "text-rendering", text_rendering;
    TextIndent => "text-indent", text_indent;
    TextShadow => "text-shadow", text_shadow;
    TextWrap => "text-wrap", text_wrap;
    TextDecorationLine => "text-decoration-line", text_decoration_line;
    TextDecorationColor => "text-decoration-color", text_decoration_color;
    TextDecorationStyle => "text-decoration-style", text_decoration_style;
    TextDecorationThickness => "text-decoration-thickness", text_decoration_thickness;
    TextUnderlineOffset => "text-underline-offset", text_underline_offset;
    WordSpacing => "word-spacing", word_spacing;
    WordBreak => "word-break", word_break;
    OverflowWrap => "overflow-wrap", overflow_wrap;
    Hyphens => "hyphens", hyphens;
    TabSize => "tab-size", tab_size;
    Direction => "direction", direction;
    UnicodeBidi => "unicode-bidi", unicode_bidi;
    WritingMode => "writing-mode", writing_mode;
    ListStyleType => "list-style-type", list_style_type;
    ListStylePosition => "list-style-position", list_style_position;
    CounterReset => "counter-reset", counter_reset;
    CounterIncrement => "counter-increment", counter_increment;

    BackgroundImage => "background-image", background_image;
    BackgroundPosition => "background-position", background_position;
    BackgroundSize => "background-size", background_size;
    BackgroundRepeat => "background-repeat", background_repeat;
    BackgroundClip => "background-clip", background_clip;
    BackgroundOrigin => "background-origin", background_origin;
    BackgroundAttachment => "background-attachment", background_attachment;
    ObjectPosition => "object-position", object_position;
    ImageRendering => "image-rendering", image_rendering;

    BorderTopColor => "border-top-color", border_top_color;
    BorderLeftColor => "border-left-color", border_left_color;
    BorderTopWidth => "border-top-width", border_top_width;
    BorderRightWidth => "border-right-width", border_right_width;
    BorderBottomWidth => "border-bottom-width", border_bottom_width;
    BorderLeftWidth => "border-left-width", border_left_width;
    BorderInline => "border-inline", border_inline;
    BorderInlineStart => "border-inline-start", border_inline_start;
    BorderInlineEnd => "border-inline-end", border_inline_end;
    BorderBlock => "border-block", border_block;
    BorderBlockStart => "border-block-start", border_block_start;
    BorderBlockEnd => "border-block-end", border_block_end;
    BorderStartStartRadius => "border-start-start-radius", border_start_start_radius, SizeCss::RADIUS;
    BorderStartEndRadius => "border-start-end-radius", border_start_end_radius, SizeCss::RADIUS;
    BorderEndStartRadius => "border-end-start-radius", border_end_start_radius, SizeCss::RADIUS;
    BorderEndEndRadius => "border-end-end-radius", border_end_end_radius, SizeCss::RADIUS;
    BorderSpacing => "border-spacing", border_spacing;
    TableLayout => "table-layout", table_layout;
    OutlineWidth => "outline-width", outline_width;
    OutlineStyle => "outline-style", outline_style;
    OutlineColor => "outline-color", outline_color;

    Filter => "filter", filter;
    MixBlendMode => "mix-blend-mode", mix_blend_mode;
    Mask => "mask", mask;
    MaskImage => "mask-image", mask_image;
    TransformOrigin => "transform-origin", transform_origin;
    Translate => "translate", translate;
    Rotate => "rotate", rotate;
    Scale => "scale", scale;
    Perspective => "perspective", perspective;
    BackfaceVisibility => "backface-visibility", backface_visibility;
    TransitionProperty => "transition-property", transition_property;
    TransitionDuration => "transition-duration", transition_duration;
    TransitionTimingFunction => "transition-timing-function", transition_timing_function;
    TransitionDelay => "transition-delay", transition_delay;
    AnimationName => "animation-name", animation_name;
    AnimationDuration => "animation-duration", animation_duration;
    AnimationTimingFunction => "animation-timing-function", animation_timing_function;
    AnimationDelay => "animation-delay", animation_delay;
    AnimationIterationCount => "animation-iteration-count", animation_iteration_count;
    AnimationFillMode => "animation-fill-mode", animation_fill_mode;

    CaretColor => "caret-color", caret_color;
    AccentColor => "accent-color", accent_color;
    ColorScheme => "color-scheme", color_scheme;
    ForcedColorAdjust => "forced-color-adjust", forced_color_adjust;
    ScrollPadding => "scroll-padding", scroll_padding;
    ScrollPaddingBlock => "scroll-padding-block", scroll_padding_block;
    ScrollPaddingTop => "scroll-padding-top", scroll_padding_top;
    ScrollMarginTop => "scroll-margin-top", scroll_margin_top;
    ScrollMarginBlock => "scroll-margin-block", scroll_margin_block;
    ScrollbarGutter => "scrollbar-gutter", scrollbar_gutter;
    OverscrollBehavior => "overscroll-behavior", overscroll_behavior;

    Fill => "fill", fill;
    Stroke => "stroke", stroke;
    StrokeWidth => "stroke-width", stroke_width;
    StrokeLinecap => "stroke-linecap", stroke_linecap;
    StrokeLinejoin => "stroke-linejoin", stroke_linejoin;
    StrokeDasharray => "stroke-dasharray", stroke_dasharray;
    StrokeDashoffset => "stroke-dashoffset", stroke_dashoffset;
}

/// What a palette colour is about to be used for, which decides which of the
/// three ramps at `:root` it resolves through. See [`crate::tokens::ColorValue`].
///
/// Only the properties that put a colour *under text* or *in text* have a
/// role. A border, a ring or a shadow keeps the brand colour: 1.4.11 asks 3:1
/// of those, which the palette already clears, and moving them would repaint
/// every outline in the library for nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorRole {
    Text,
    Fill,
}

impl Property {
    pub(crate) const fn color_role(self) -> Option<ColorRole> {
        match self {
            Self::Color => Some(ColorRole::Text),
            Self::Background | Self::BackgroundColor => Some(ColorRole::Fill),
            _ => None,
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
    Media(String),
    Container { name: String, condition: String },
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

    /// Every builder method emits its own CSS name; the scaled ones resolve a
    /// bare `Size` through their theme scale.
    #[test]
    fn every_builder_method_emits_its_declaration() {
        macro_rules! assert_emits {
            ($value:expr => $expected:literal: $($method:ident => $css:literal,)*) => {
                let css = crate::css::Stylesheet::from(&crate::sx::sx()$(.$method($value))*)
                    .as_str()
                    .to_string();
                $(assert!(
                    css.contains(concat!($css, ":", $expected, ";")),
                    "{} missing from {css}",
                    $css
                );)*
            };
        }

        assert_emits!("inherit" => "inherit":
            box_sizing => "box-sizing", float => "float", clear => "clear",
            isolation => "isolation", contain => "contain",
            content_visibility => "content-visibility", will_change => "will-change",
            all => "all", inline_size => "inline-size", block_size => "block-size",
            min_block_size => "min-block-size", max_block_size => "max-block-size",
            flex_basis => "flex-basis", flex_flow => "flex-flow", order => "order",
            justify_items => "justify-items", justify_self => "justify-self",
            place_items => "place-items", place_content => "place-content",
            place_self => "place-self", grid => "grid", grid_template => "grid-template",
            grid_auto_columns => "grid-auto-columns",
            grid_column_start => "grid-column-start", grid_column_end => "grid-column-end",
            grid_row_start => "grid-row-start", grid_row_end => "grid-row-end",
            columns => "columns", column_count => "column-count",
            column_width => "column-width", break_inside => "break-inside",
            break_before => "break-before", break_after => "break-after",
            font_style => "font-style", font_variant => "font-variant",
            font_variant_numeric => "font-variant-numeric",
            font_feature_settings => "font-feature-settings",
            font_variation_settings => "font-variation-settings",
            font_stretch => "font-stretch", text_rendering => "text-rendering",
            text_indent => "text-indent", text_shadow => "text-shadow",
            text_wrap => "text-wrap", text_decoration_line => "text-decoration-line",
            text_decoration_color => "text-decoration-color",
            text_decoration_style => "text-decoration-style",
            text_decoration_thickness => "text-decoration-thickness",
            text_underline_offset => "text-underline-offset",
            word_spacing => "word-spacing", word_break => "word-break",
            overflow_wrap => "overflow-wrap", hyphens => "hyphens", tab_size => "tab-size",
            direction => "direction", unicode_bidi => "unicode-bidi",
            writing_mode => "writing-mode", list_style_type => "list-style-type",
            list_style_position => "list-style-position",
            counter_reset => "counter-reset", counter_increment => "counter-increment",
            background_image => "background-image",
            background_position => "background-position",
            background_size => "background-size", background_repeat => "background-repeat",
            background_clip => "background-clip", background_origin => "background-origin",
            background_attachment => "background-attachment",
            object_position => "object-position", image_rendering => "image-rendering",
            border_top_color => "border-top-color", border_left_color => "border-left-color",
            border_top_width => "border-top-width",
            border_right_width => "border-right-width",
            border_bottom_width => "border-bottom-width",
            border_left_width => "border-left-width", border_inline => "border-inline",
            border_inline_start => "border-inline-start",
            border_inline_end => "border-inline-end", border_block => "border-block",
            border_block_start => "border-block-start",
            border_block_end => "border-block-end", border_spacing => "border-spacing",
            table_layout => "table-layout", outline_width => "outline-width",
            outline_style => "outline-style", outline_color => "outline-color",
            filter => "filter", mix_blend_mode => "mix-blend-mode", mask => "mask",
            mask_image => "mask-image", transform_origin => "transform-origin",
            translate => "translate", rotate => "rotate", scale => "scale",
            perspective => "perspective", backface_visibility => "backface-visibility",
            transition_property => "transition-property",
            transition_duration => "transition-duration",
            transition_timing_function => "transition-timing-function",
            transition_delay => "transition-delay", animation_name => "animation-name",
            animation_duration => "animation-duration",
            animation_timing_function => "animation-timing-function",
            animation_delay => "animation-delay",
            animation_iteration_count => "animation-iteration-count",
            animation_fill_mode => "animation-fill-mode", caret_color => "caret-color",
            accent_color => "accent-color", color_scheme => "color-scheme",
            forced_color_adjust => "forced-color-adjust", scroll_padding => "scroll-padding",
            scroll_padding_block => "scroll-padding-block",
            scroll_padding_top => "scroll-padding-top",
            scroll_margin_top => "scroll-margin-top",
            scroll_margin_block => "scroll-margin-block",
            scrollbar_gutter => "scrollbar-gutter",
            overscroll_behavior => "overscroll-behavior", fill => "fill", stroke => "stroke",
            stroke_width => "stroke-width", stroke_linecap => "stroke-linecap",
            stroke_linejoin => "stroke-linejoin", stroke_dasharray => "stroke-dasharray",
            stroke_dashoffset => "stroke-dashoffset",
        );
        assert_emits!(Size::Md => "var(--lsx-spacing-md)":
            margin_inline => "margin-inline", margin_inline_start => "margin-inline-start",
            margin_inline_end => "margin-inline-end", margin_block => "margin-block",
            margin_block_start => "margin-block-start", margin_block_end => "margin-block-end",
            padding_inline => "padding-inline",
            padding_inline_start => "padding-inline-start",
            padding_inline_end => "padding-inline-end", padding_block => "padding-block",
            padding_block_start => "padding-block-start",
            padding_block_end => "padding-block-end", inset_inline => "inset-inline",
            inset_inline_start => "inset-inline-start",
            inset_inline_end => "inset-inline-end", inset_block => "inset-block",
            inset_block_start => "inset-block-start", inset_block_end => "inset-block-end",
            top => "top", right => "right", bottom => "bottom", left => "left",
            inset => "inset",
        );
        assert_emits!(Size::Md => "var(--lsx-shadow-md)": box_shadow => "box-shadow",);
        assert_emits!(Size::Md => "var(--lsx-breakpoint-md)":
            min_inline_size => "min-inline-size", max_inline_size => "max-inline-size",
        );
        assert_emits!(Size::Md => "var(--lsx-radius-md)":
            border_start_start_radius => "border-start-start-radius",
            border_start_end_radius => "border-start-end-radius",
            border_end_start_radius => "border-end-start-radius",
            border_end_end_radius => "border-end-end-radius",
        );
    }

    #[test]
    fn a_builder_method_sets_its_own_property() {
        assert_eq!(
            SxPropertyKey::parse("z-index"),
            SxPropertyKey::Known(Property::ZIndex)
        );
    }
}
