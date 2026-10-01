use crate::tokens::{Size, SizeCss};

use super::{Sx, ThemeAwareValue};

/// Every CSS property `Sx` knows: variant, CSS name, builder method, and the
/// [`SizeCss`] scale a bare `Size` resolves through. `#[color]` marks a value that is one colour.
macro_rules! properties {
    (@scale) => { None };
    (@scale $scale:expr) => { Some($scale) };
    (@color) => { false };
    (@color color) => { true };
    ($($(#[$kind:ident])? $variant:ident => $css_name:literal, $method:ident $(, $scale:expr)?;)*) => {
        /// A CSS property `Sx` has a builder method for.
        #[repr(u16)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Property {
            $($variant,)*
        }

        impl Property {
            /// The property named `property` in CSS (`"max-height"`); `None` for one
            /// this enum does not list, custom properties included.
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

            /// Whether the value is one colour, which a misspelled theme colour name breaks.
            pub(crate) const fn takes_color(self) -> bool {
                match self {
                    $(Self::$variant => properties!(@color $($kind)?),)*
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

            /// Each property's builder method, beside the property it should set.
            #[cfg(test)]
            const BUILDERS: &'static [(Self, fn(Sx, ThemeAwareValue) -> Sx)] =
                &[$((Self::$variant, |sx, value| sx.$method(value)),)*];
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
    #[color] Background => "background", background;
    #[color] BackgroundColor => "background-color", background_color;
    Width => "width", width, SizeCss::BREAKPOINT;
    Height => "height", height, SizeCss::BREAKPOINT;
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
    MaxHeight => "max-height", max_height, SizeCss::BREAKPOINT;
    Font => "font", font;
    FontFamily => "font-family", font_family;
    FontSize => "font-size", font_size, SizeCss::FONT_SIZE;
    FontWeight => "font-weight", font_weight;
    LetterSpacing => "letter-spacing", letter_spacing;
    LineHeight => "line-height", line_height;
    TextDecoration => "text-decoration", text_decoration;
    Border => "border", border;
    BorderTop => "border-top", border_top;
    BorderRight => "border-right", border_right;
    BorderBottom => "border-bottom", border_bottom;
    BorderLeft => "border-left", border_left;
    #[color] Color => "color", color;
    Flex => "flex", flex;
    FlexGrow => "flex-grow", flex_grow;
    Content => "content", content;
    BorderWidth => "border-width", border_width;
    BorderStyle => "border-style", border_style;
    #[color] BorderColor => "border-color", border_color;
    #[color] BorderRightColor => "border-right-color", border_right_color;
    #[color] BorderBottomColor => "border-bottom-color", border_bottom_color;
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
    #[color] ScrollbarColor => "scrollbar-color", scrollbar_color;
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
    MinHeight => "min-height", min_height, SizeCss::BREAKPOINT;
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
    InlineSize => "inline-size", inline_size, SizeCss::BREAKPOINT;
    BlockSize => "block-size", block_size, SizeCss::BREAKPOINT;
    MinInlineSize => "min-inline-size", min_inline_size, SizeCss::BREAKPOINT;
    MaxInlineSize => "max-inline-size", max_inline_size, SizeCss::BREAKPOINT;
    MinBlockSize => "min-block-size", min_block_size, SizeCss::BREAKPOINT;
    MaxBlockSize => "max-block-size", max_block_size, SizeCss::BREAKPOINT;
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
    #[color] TextDecorationColor => "text-decoration-color", text_decoration_color;
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

    #[color] BorderTopColor => "border-top-color", border_top_color;
    #[color] BorderLeftColor => "border-left-color", border_left_color;
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
    #[color] OutlineColor => "outline-color", outline_color;

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

    #[color] CaretColor => "caret-color", caret_color;
    #[color] AccentColor => "accent-color", accent_color;
    ColorScheme => "color-scheme", color_scheme;
    ForcedColorAdjust => "forced-color-adjust", forced_color_adjust;
    ScrollPadding => "scroll-padding", scroll_padding;
    ScrollPaddingBlock => "scroll-padding-block", scroll_padding_block;
    ScrollPaddingTop => "scroll-padding-top", scroll_padding_top;
    ScrollMarginTop => "scroll-margin-top", scroll_margin_top;
    ScrollMarginBlock => "scroll-margin-block", scroll_margin_block;
    ScrollbarGutter => "scrollbar-gutter", scrollbar_gutter;
    OverscrollBehavior => "overscroll-behavior", overscroll_behavior;

    #[color] Fill => "fill", fill;
    #[color] Stroke => "stroke", stroke;
    StrokeWidth => "stroke-width", stroke_width;
    StrokeLinecap => "stroke-linecap", stroke_linecap;
    StrokeLinejoin => "stroke-linejoin", stroke_linejoin;
    StrokeDasharray => "stroke-dasharray", stroke_dasharray;
    StrokeDashoffset => "stroke-dashoffset", stroke_dashoffset;
}

/// Which `:root` ramp a palette colour resolves through, by its use. Borders, rings
/// and shadows have none: the brand colour already clears 1.4.11's 3:1.
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

/// A declaration's property: a known [`Property`] or any other name, custom properties included.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SxPropertyKey {
    /// A property [`Property`] lists.
    Known(Property),
    /// Any other name, written as is.
    Raw(String),
}

impl SxPropertyKey {
    /// [`Known`](Self::Known) when [`Property::parse`] knows the name, else [`Raw`](Self::Raw).
    pub fn parse(property: impl Into<String>) -> Self {
        let property = property.into();
        match Property::parse(&property) {
            Some(property) => Self::Known(property),
            None => Self::Raw(property),
        }
    }

    /// The CSS name.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Known(property) => property.as_str(),
            Self::Raw(property) => property.as_str(),
        }
    }
}

/// What a nested `Sx` block applies under: a selector, a `data-state` condition, or an at-rule.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SxModifierKey {
    /// A selector for the element: `&` stands for it, else the pattern is appended (`":hover"`).
    Selector(String),
    /// `data-state` words the element must carry, each becoming `[data-state~="word"]`.
    Condition(String),
    /// A `@media (min-width: ..)` rule: breakpoint `size` and wider.
    Breakpoint(Size),
    /// A `@media` rule with this query.
    Media(String),
    /// A `@supports` rule with this condition.
    Supports(String),
    /// A `@container` rule for container `name` (empty for the nearest) and `condition`.
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

    /// The one declaration `builder` emits for `value`, as `name:value`.
    fn declaration(builder: fn(Sx, ThemeAwareValue) -> Sx, value: ThemeAwareValue) -> String {
        let sheet = crate::css::Stylesheet::from(&builder(crate::sx::sx(), value));
        let css = sheet.as_str();
        let body = css[css.find('{').expect("a rule") + 1..css.rfind('}').expect("a rule")].trim();
        body.strip_suffix(';').unwrap_or(body).to_string()
    }

    #[test]
    fn every_builder_method_emits_its_own_declaration() {
        for (property, builder) in Property::BUILDERS {
            assert_eq!(
                declaration(*builder, "inherit".into()),
                format!("{}:inherit", property.as_str())
            );
        }
    }

    /// The scaled builders resolve a bare `Size` through their theme scale.
    #[test]
    fn every_scaled_builder_resolves_a_size() {
        for (property, builder) in Property::BUILDERS {
            if let Some(scale) = property.size_scale() {
                assert_eq!(
                    declaration(*builder, Size::Md.into()),
                    format!("{}:{}", property.as_str(), scale.value(Size::Md))
                );
            }
        }
    }

    /// One scale for every size key (1732): `max_height(Size::Md)` once emitted `max-height:md`.
    #[test]
    fn every_size_key_uses_the_breakpoint_scale() {
        use Property::*;
        for property in [
            Width,
            Height,
            MinWidth,
            MaxWidth,
            MinHeight,
            MaxHeight,
            InlineSize,
            BlockSize,
            MinInlineSize,
            MaxInlineSize,
            MinBlockSize,
            MaxBlockSize,
        ] {
            assert_eq!(
                property.size_scale(),
                Some(SizeCss::BREAKPOINT),
                "{property:?}"
            );
        }
        assert_eq!(
            declaration(|sx, value| sx.max_height(value), Size::Md.into()),
            "max-height:var(--lsx-breakpoint-md)"
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
