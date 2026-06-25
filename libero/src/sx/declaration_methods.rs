use crate::sx::{static_value::IntoStaticValue, sx::Sx};

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticPropertyId {
    Display,
    Visibility,
    Width,
    MinWidth,
    MaxWidth,
    Height,
    MinHeight,
    MaxHeight,
    BoxSizing,
    AspectRatio,
    Font,
    FontFamily,
    FontSize,
    FontWeight,
    FontStyle,
    LineHeight,
    LetterSpacing,
    TextAlign,
    TextTransform,
    TextDecoration,
    TextOverflow,
    WhiteSpace,
    WordBreak,
    OverflowWrap,
    Color,
    Opacity,
    Background,
    BackgroundColor,
    BackgroundImage,
    BackgroundSize,
    BackgroundPosition,
    BackgroundRepeat,
    Border,
    BorderTop,
    BorderRight,
    BorderBottom,
    BorderLeft,
    BorderColor,
    BorderStyle,
    BorderWidth,
    BorderRadius,
    BorderTopLeftRadius,
    BorderTopRightRadius,
    BorderBottomRightRadius,
    BorderBottomLeftRadius,
    Outline,
    BoxShadow,
    Position,
    Top,
    Right,
    Bottom,
    Left,
    Inset,
    InsetTop,
    InsetRight,
    InsetBottom,
    InsetLeft,
    ZIndex,
    Float,
    Clear,
    Overflow,
    OverflowX,
    OverflowY,
    ClipPath,
    Resize,
    Isolation,
    WillChange,
    Contain,
    ContentVisibility,
    OverscrollBehavior,
    OverscrollBehaviorX,
    OverscrollBehaviorY,
    ScrollBehavior,
    ScrollMargin,
    ScrollMarginTop,
    ScrollMarginRight,
    ScrollMarginBottom,
    ScrollMarginLeft,
    ScrollPadding,
    ScrollPaddingTop,
    ScrollPaddingRight,
    ScrollPaddingBottom,
    ScrollPaddingLeft,
    ScrollSnapType,
    ScrollSnapAlign,
    ScrollSnapStop,
    Flex,
    FlexDirection,
    FlexWrap,
    FlexGrow,
    FlexShrink,
    FlexBasis,
    JustifyContent,
    JustifyItems,
    JustifySelf,
    AlignContent,
    AlignItems,
    AlignSelf,
    PlaceContent,
    PlaceItems,
    PlaceSelf,
    Order,
    Gap,
    RowGap,
    ColumnGap,
    GridTemplateColumns,
    GridTemplateRows,
    GridTemplateAreas,
    GridAutoColumns,
    GridAutoRows,
    GridAutoFlow,
    GridColumn,
    GridRow,
    GridArea,
    Padding,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,
    Margin,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    Columns,
    ColumnCount,
    ColumnWidth,
    ColumnFill,
    ColumnSpan,
    ColumnRule,
    ColumnRuleColor,
    ColumnRuleStyle,
    ColumnRuleWidth,
    ObjectFit,
    ObjectPosition,
    ListStyle,
    ListStyleType,
    ListStylePosition,
    ListStyleImage,
    VerticalAlign,
    CaptionSide,
    TableLayout,
    EmptyCells,
    BorderCollapse,
    BorderSpacing,
    Cursor,
    PointerEvents,
    UserSelect,
    Appearance,
    CaretColor,
    AccentColor,
    Quotes,
    TabSize,
    Direction,
    WritingMode,
    TextOrientation,
    UnicodeBidi,
    Hyphens,
    Transition,
    TransitionProperty,
    TransitionDuration,
    TransitionTimingFunction,
    TransitionDelay,
    Transform,
    TransformOrigin,
    Filter,
    BackdropFilter,
    Mask,
    MaskImage,
    MaskSize,
    MaskPosition,
    MaskRepeat,
    MixBlendMode,
    BackgroundBlendMode,
    Fill,
    Stroke,
    Animation,
    AnimationName,
    AnimationDuration,
    AnimationTimingFunction,
    AnimationDelay,
    AnimationIterationCount,
    AnimationDirection,
    AnimationFillMode,
    Content,
}

impl StaticPropertyId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Display => "display",
            Self::Visibility => "visibility",
            Self::Width => "width",
            Self::MinWidth => "min-width",
            Self::MaxWidth => "max-width",
            Self::Height => "height",
            Self::MinHeight => "min-height",
            Self::MaxHeight => "max-height",
            Self::BoxSizing => "box-sizing",
            Self::AspectRatio => "aspect-ratio",
            Self::Font => "font",
            Self::FontFamily => "font-family",
            Self::FontSize => "font-size",
            Self::FontWeight => "font-weight",
            Self::FontStyle => "font-style",
            Self::LineHeight => "line-height",
            Self::LetterSpacing => "letter-spacing",
            Self::TextAlign => "text-align",
            Self::TextTransform => "text-transform",
            Self::TextDecoration => "text-decoration",
            Self::TextOverflow => "text-overflow",
            Self::WhiteSpace => "white-space",
            Self::WordBreak => "word-break",
            Self::OverflowWrap => "overflow-wrap",
            Self::Color => "color",
            Self::Opacity => "opacity",
            Self::Background => "background",
            Self::BackgroundColor => "background-color",
            Self::BackgroundImage => "background-image",
            Self::BackgroundSize => "background-size",
            Self::BackgroundPosition => "background-position",
            Self::BackgroundRepeat => "background-repeat",
            Self::Border => "border",
            Self::BorderTop => "border-top",
            Self::BorderRight => "border-right",
            Self::BorderBottom => "border-bottom",
            Self::BorderLeft => "border-left",
            Self::BorderColor => "border-color",
            Self::BorderStyle => "border-style",
            Self::BorderWidth => "border-width",
            Self::BorderRadius => "border-radius",
            Self::BorderTopLeftRadius => "border-top-left-radius",
            Self::BorderTopRightRadius => "border-top-right-radius",
            Self::BorderBottomRightRadius => "border-bottom-right-radius",
            Self::BorderBottomLeftRadius => "border-bottom-left-radius",
            Self::Outline => "outline",
            Self::BoxShadow => "box-shadow",
            Self::Position => "position",
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Inset => "inset",
            Self::InsetTop => "inset-top",
            Self::InsetRight => "inset-right",
            Self::InsetBottom => "inset-bottom",
            Self::InsetLeft => "inset-left",
            Self::ZIndex => "z-index",
            Self::Float => "float",
            Self::Clear => "clear",
            Self::Overflow => "overflow",
            Self::OverflowX => "overflow-x",
            Self::OverflowY => "overflow-y",
            Self::ClipPath => "clip-path",
            Self::Resize => "resize",
            Self::Isolation => "isolation",
            Self::WillChange => "will-change",
            Self::Contain => "contain",
            Self::ContentVisibility => "content-visibility",
            Self::OverscrollBehavior => "overscroll-behavior",
            Self::OverscrollBehaviorX => "overscroll-behavior-x",
            Self::OverscrollBehaviorY => "overscroll-behavior-y",
            Self::ScrollBehavior => "scroll-behavior",
            Self::ScrollMargin => "scroll-margin",
            Self::ScrollMarginTop => "scroll-margin-top",
            Self::ScrollMarginRight => "scroll-margin-right",
            Self::ScrollMarginBottom => "scroll-margin-bottom",
            Self::ScrollMarginLeft => "scroll-margin-left",
            Self::ScrollPadding => "scroll-padding",
            Self::ScrollPaddingTop => "scroll-padding-top",
            Self::ScrollPaddingRight => "scroll-padding-right",
            Self::ScrollPaddingBottom => "scroll-padding-bottom",
            Self::ScrollPaddingLeft => "scroll-padding-left",
            Self::ScrollSnapType => "scroll-snap-type",
            Self::ScrollSnapAlign => "scroll-snap-align",
            Self::ScrollSnapStop => "scroll-snap-stop",
            Self::Flex => "flex",
            Self::FlexDirection => "flex-direction",
            Self::FlexWrap => "flex-wrap",
            Self::FlexGrow => "flex-grow",
            Self::FlexShrink => "flex-shrink",
            Self::FlexBasis => "flex-basis",
            Self::JustifyContent => "justify-content",
            Self::JustifyItems => "justify-items",
            Self::JustifySelf => "justify-self",
            Self::AlignContent => "align-content",
            Self::AlignItems => "align-items",
            Self::AlignSelf => "align-self",
            Self::PlaceContent => "place-content",
            Self::PlaceItems => "place-items",
            Self::PlaceSelf => "place-self",
            Self::Order => "order",
            Self::Gap => "gap",
            Self::RowGap => "row-gap",
            Self::ColumnGap => "column-gap",
            Self::GridTemplateColumns => "grid-template-columns",
            Self::GridTemplateRows => "grid-template-rows",
            Self::GridTemplateAreas => "grid-template-areas",
            Self::GridAutoColumns => "grid-auto-columns",
            Self::GridAutoRows => "grid-auto-rows",
            Self::GridAutoFlow => "grid-auto-flow",
            Self::GridColumn => "grid-column",
            Self::GridRow => "grid-row",
            Self::GridArea => "grid-area",
            Self::Padding => "padding",
            Self::PaddingTop => "padding-top",
            Self::PaddingRight => "padding-right",
            Self::PaddingBottom => "padding-bottom",
            Self::PaddingLeft => "padding-left",
            Self::Margin => "margin",
            Self::MarginTop => "margin-top",
            Self::MarginRight => "margin-right",
            Self::MarginBottom => "margin-bottom",
            Self::MarginLeft => "margin-left",
            Self::Columns => "columns",
            Self::ColumnCount => "column-count",
            Self::ColumnWidth => "column-width",
            Self::ColumnFill => "column-fill",
            Self::ColumnSpan => "column-span",
            Self::ColumnRule => "column-rule",
            Self::ColumnRuleColor => "column-rule-color",
            Self::ColumnRuleStyle => "column-rule-style",
            Self::ColumnRuleWidth => "column-rule-width",
            Self::ObjectFit => "object-fit",
            Self::ObjectPosition => "object-position",
            Self::ListStyle => "list-style",
            Self::ListStyleType => "list-style-type",
            Self::ListStylePosition => "list-style-position",
            Self::ListStyleImage => "list-style-image",
            Self::VerticalAlign => "vertical-align",
            Self::CaptionSide => "caption-side",
            Self::TableLayout => "table-layout",
            Self::EmptyCells => "empty-cells",
            Self::BorderCollapse => "border-collapse",
            Self::BorderSpacing => "border-spacing",
            Self::Cursor => "cursor",
            Self::PointerEvents => "pointer-events",
            Self::UserSelect => "user-select",
            Self::Appearance => "appearance",
            Self::CaretColor => "caret-color",
            Self::AccentColor => "accent-color",
            Self::Quotes => "quotes",
            Self::TabSize => "tab-size",
            Self::Direction => "direction",
            Self::WritingMode => "writing-mode",
            Self::TextOrientation => "text-orientation",
            Self::UnicodeBidi => "unicode-bidi",
            Self::Hyphens => "hyphens",
            Self::Transition => "transition",
            Self::TransitionProperty => "transition-property",
            Self::TransitionDuration => "transition-duration",
            Self::TransitionTimingFunction => "transition-timing-function",
            Self::TransitionDelay => "transition-delay",
            Self::Transform => "transform",
            Self::TransformOrigin => "transform-origin",
            Self::Filter => "filter",
            Self::BackdropFilter => "backdrop-filter",
            Self::Mask => "mask",
            Self::MaskImage => "mask-image",
            Self::MaskSize => "mask-size",
            Self::MaskPosition => "mask-position",
            Self::MaskRepeat => "mask-repeat",
            Self::MixBlendMode => "mix-blend-mode",
            Self::BackgroundBlendMode => "background-blend-mode",
            Self::Fill => "fill",
            Self::Stroke => "stroke",
            Self::Animation => "animation",
            Self::AnimationName => "animation-name",
            Self::AnimationDuration => "animation-duration",
            Self::AnimationTimingFunction => "animation-timing-function",
            Self::AnimationDelay => "animation-delay",
            Self::AnimationIterationCount => "animation-iteration-count",
            Self::AnimationDirection => "animation-direction",
            Self::AnimationFillMode => "animation-fill-mode",
            Self::Content => "content",
        }
    }
}

macro_rules! css_declaration_method {
    ($name:ident, $prop_id:ident) => {
        pub const fn $name(self, value: impl const IntoStaticValue) -> Sx<{ N + 1 }, R> {
            self.property(StaticPropertyId::$prop_id, value)
        }
    };
}

impl<const N: usize, const R: usize> Sx<N, R> {
    css_declaration_method!(display, Display);
    css_declaration_method!(visibility, Visibility);
    css_declaration_method!(width, Width);
    css_declaration_method!(min_width, MinWidth);
    css_declaration_method!(max_width, MaxWidth);
    css_declaration_method!(height, Height);
    css_declaration_method!(min_height, MinHeight);
    css_declaration_method!(max_height, MaxHeight);
    css_declaration_method!(box_sizing, BoxSizing);
    css_declaration_method!(aspect_ratio, AspectRatio);
    css_declaration_method!(font, Font);
    css_declaration_method!(font_family, FontFamily);
    css_declaration_method!(font_size, FontSize);
    css_declaration_method!(font_weight, FontWeight);
    css_declaration_method!(font_style, FontStyle);
    css_declaration_method!(line_height, LineHeight);
    css_declaration_method!(letter_spacing, LetterSpacing);
    css_declaration_method!(text_align, TextAlign);
    css_declaration_method!(text_transform, TextTransform);
    css_declaration_method!(text_decoration, TextDecoration);
    css_declaration_method!(text_overflow, TextOverflow);
    css_declaration_method!(white_space, WhiteSpace);
    css_declaration_method!(word_break, WordBreak);
    css_declaration_method!(overflow_wrap, OverflowWrap);
    css_declaration_method!(color, Color);
    css_declaration_method!(opacity, Opacity);
    css_declaration_method!(background, Background);
    css_declaration_method!(background_color, BackgroundColor);
    css_declaration_method!(background_image, BackgroundImage);
    css_declaration_method!(background_size, BackgroundSize);
    css_declaration_method!(background_position, BackgroundPosition);
    css_declaration_method!(background_repeat, BackgroundRepeat);
    css_declaration_method!(border, Border);
    css_declaration_method!(border_top, BorderTop);
    css_declaration_method!(border_right, BorderRight);
    css_declaration_method!(border_bottom, BorderBottom);
    css_declaration_method!(border_left, BorderLeft);
    css_declaration_method!(border_color, BorderColor);
    css_declaration_method!(border_style, BorderStyle);
    css_declaration_method!(border_width, BorderWidth);
    css_declaration_method!(border_radius, BorderRadius);
    css_declaration_method!(border_top_left_radius, BorderTopLeftRadius);
    css_declaration_method!(border_top_right_radius, BorderTopRightRadius);
    css_declaration_method!(border_bottom_right_radius, BorderBottomRightRadius);
    css_declaration_method!(border_bottom_left_radius, BorderBottomLeftRadius);
    css_declaration_method!(outline, Outline);
    css_declaration_method!(box_shadow, BoxShadow);
    css_declaration_method!(position, Position);
    css_declaration_method!(top, Top);
    css_declaration_method!(right, Right);
    css_declaration_method!(bottom, Bottom);
    css_declaration_method!(left, Left);
    css_declaration_method!(inset, Inset);
    css_declaration_method!(inset_top, InsetTop);
    css_declaration_method!(inset_right, InsetRight);
    css_declaration_method!(inset_bottom, InsetBottom);
    css_declaration_method!(inset_left, InsetLeft);
    css_declaration_method!(z_index, ZIndex);
    css_declaration_method!(float, Float);
    css_declaration_method!(clear, Clear);
    css_declaration_method!(overflow, Overflow);
    css_declaration_method!(overflow_x, OverflowX);
    css_declaration_method!(overflow_y, OverflowY);
    css_declaration_method!(clip_path, ClipPath);
    css_declaration_method!(resize, Resize);
    css_declaration_method!(isolation, Isolation);
    css_declaration_method!(will_change, WillChange);
    css_declaration_method!(contain, Contain);
    css_declaration_method!(content_visibility, ContentVisibility);
    css_declaration_method!(overscroll_behavior, OverscrollBehavior);
    css_declaration_method!(overscroll_behavior_x, OverscrollBehaviorX);
    css_declaration_method!(overscroll_behavior_y, OverscrollBehaviorY);
    css_declaration_method!(scroll_behavior, ScrollBehavior);
    css_declaration_method!(scroll_margin, ScrollMargin);
    css_declaration_method!(scroll_margin_top, ScrollMarginTop);
    css_declaration_method!(scroll_margin_right, ScrollMarginRight);
    css_declaration_method!(scroll_margin_bottom, ScrollMarginBottom);
    css_declaration_method!(scroll_margin_left, ScrollMarginLeft);
    css_declaration_method!(scroll_padding, ScrollPadding);
    css_declaration_method!(scroll_padding_top, ScrollPaddingTop);
    css_declaration_method!(scroll_padding_right, ScrollPaddingRight);
    css_declaration_method!(scroll_padding_bottom, ScrollPaddingBottom);
    css_declaration_method!(scroll_padding_left, ScrollPaddingLeft);
    css_declaration_method!(scroll_snap_type, ScrollSnapType);
    css_declaration_method!(scroll_snap_align, ScrollSnapAlign);
    css_declaration_method!(scroll_snap_stop, ScrollSnapStop);
    css_declaration_method!(flex, Flex);
    css_declaration_method!(flex_direction, FlexDirection);
    css_declaration_method!(flex_wrap, FlexWrap);
    css_declaration_method!(flex_grow, FlexGrow);
    css_declaration_method!(flex_shrink, FlexShrink);
    css_declaration_method!(flex_basis, FlexBasis);
    css_declaration_method!(justify_content, JustifyContent);
    css_declaration_method!(justify_items, JustifyItems);
    css_declaration_method!(justify_self, JustifySelf);
    css_declaration_method!(align_content, AlignContent);
    css_declaration_method!(align_items, AlignItems);
    css_declaration_method!(align_self, AlignSelf);
    css_declaration_method!(place_content, PlaceContent);
    css_declaration_method!(place_items, PlaceItems);
    css_declaration_method!(place_self, PlaceSelf);
    css_declaration_method!(order, Order);
    css_declaration_method!(gap, Gap);
    css_declaration_method!(row_gap, RowGap);
    css_declaration_method!(column_gap, ColumnGap);
    css_declaration_method!(grid_template_columns, GridTemplateColumns);
    css_declaration_method!(grid_template_rows, GridTemplateRows);
    css_declaration_method!(grid_template_areas, GridTemplateAreas);
    css_declaration_method!(grid_auto_columns, GridAutoColumns);
    css_declaration_method!(grid_auto_rows, GridAutoRows);
    css_declaration_method!(grid_auto_flow, GridAutoFlow);
    css_declaration_method!(grid_column, GridColumn);
    css_declaration_method!(grid_row, GridRow);
    css_declaration_method!(grid_area, GridArea);
    css_declaration_method!(padding, Padding);
    css_declaration_method!(padding_top, PaddingTop);
    css_declaration_method!(padding_right, PaddingRight);
    css_declaration_method!(padding_bottom, PaddingBottom);
    css_declaration_method!(padding_left, PaddingLeft);
    css_declaration_method!(margin, Margin);
    css_declaration_method!(margin_top, MarginTop);
    css_declaration_method!(margin_right, MarginRight);
    css_declaration_method!(margin_bottom, MarginBottom);
    css_declaration_method!(margin_left, MarginLeft);
    css_declaration_method!(columns, Columns);
    css_declaration_method!(column_count, ColumnCount);
    css_declaration_method!(column_width, ColumnWidth);
    css_declaration_method!(column_fill, ColumnFill);
    css_declaration_method!(column_span, ColumnSpan);
    css_declaration_method!(column_rule, ColumnRule);
    css_declaration_method!(column_rule_color, ColumnRuleColor);
    css_declaration_method!(column_rule_style, ColumnRuleStyle);
    css_declaration_method!(column_rule_width, ColumnRuleWidth);
    css_declaration_method!(object_fit, ObjectFit);
    css_declaration_method!(object_position, ObjectPosition);
    css_declaration_method!(list_style, ListStyle);
    css_declaration_method!(list_style_type, ListStyleType);
    css_declaration_method!(list_style_position, ListStylePosition);
    css_declaration_method!(list_style_image, ListStyleImage);
    css_declaration_method!(vertical_align, VerticalAlign);
    css_declaration_method!(caption_side, CaptionSide);
    css_declaration_method!(table_layout, TableLayout);
    css_declaration_method!(empty_cells, EmptyCells);
    css_declaration_method!(border_collapse, BorderCollapse);
    css_declaration_method!(border_spacing, BorderSpacing);
    css_declaration_method!(cursor, Cursor);
    css_declaration_method!(pointer_events, PointerEvents);
    css_declaration_method!(user_select, UserSelect);
    css_declaration_method!(appearance, Appearance);
    css_declaration_method!(caret_color, CaretColor);
    css_declaration_method!(accent_color, AccentColor);
    css_declaration_method!(quotes, Quotes);
    css_declaration_method!(tab_size, TabSize);
    css_declaration_method!(direction, Direction);
    css_declaration_method!(writing_mode, WritingMode);
    css_declaration_method!(text_orientation, TextOrientation);
    css_declaration_method!(unicode_bidi, UnicodeBidi);
    css_declaration_method!(hyphens, Hyphens);
    css_declaration_method!(transition, Transition);
    css_declaration_method!(transition_property, TransitionProperty);
    css_declaration_method!(transition_duration, TransitionDuration);
    css_declaration_method!(transition_timing_function, TransitionTimingFunction);
    css_declaration_method!(transition_delay, TransitionDelay);
    css_declaration_method!(transform, Transform);
    css_declaration_method!(transform_origin, TransformOrigin);
    css_declaration_method!(filter, Filter);
    css_declaration_method!(backdrop_filter, BackdropFilter);
    css_declaration_method!(mask, Mask);
    css_declaration_method!(mask_image, MaskImage);
    css_declaration_method!(mask_size, MaskSize);
    css_declaration_method!(mask_position, MaskPosition);
    css_declaration_method!(mask_repeat, MaskRepeat);
    css_declaration_method!(mix_blend_mode, MixBlendMode);
    css_declaration_method!(background_blend_mode, BackgroundBlendMode);
    css_declaration_method!(fill, Fill);
    css_declaration_method!(stroke, Stroke);
    css_declaration_method!(animation, Animation);
    css_declaration_method!(animation_name, AnimationName);
    css_declaration_method!(animation_duration, AnimationDuration);
    css_declaration_method!(animation_timing_function, AnimationTimingFunction);
    css_declaration_method!(animation_delay, AnimationDelay);
    css_declaration_method!(animation_iteration_count, AnimationIterationCount);
    css_declaration_method!(animation_direction, AnimationDirection);
    css_declaration_method!(animation_fill_mode, AnimationFillMode);
    css_declaration_method!(content, Content);
}
