use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const FIELD_LABEL_FONT_SIZE: SizeCss = SizeCss::new("--lsx-field-label-font-size-");
pub const FIELD_CAPTION_FONT_SIZE: SizeCss = SizeCss::new("--lsx-field-caption-font-size-");
pub const FIELD_FONT_SIZE: SizeCss = SizeCss::new("--lsx-field-font-size-");
pub const FIELD_HEIGHT: SizeCss = SizeCss::new("--lsx-field-height-");
pub const FIELD_PADDING_Y: SizeCss = SizeCss::new("--lsx-field-padding-y-");
pub const FIELD_PADDING_X: SizeCss = SizeCss::new("--lsx-field-padding-x-");
pub const FIELD_GAP: CssVar = CssVar::new("--lsx-field-gap");
pub const FIELD_FRAME_GAP: CssVar = CssVar::new("--lsx-field-frame-gap");
pub const FIELD_CARD_PADDING: SizeCss = SizeCss::new("--lsx-field-card-padding-");

/// Matches WebKit on iOS and iPadOS only.
const IOS_ONLY: &str = "(-webkit-touch-callout: none)";

/// Lifts a text input's `font_size` to 16px on iOS, which zooms into a smaller one.
pub(crate) fn ios_text_floor_sx(font_size: impl std::fmt::Display) -> Sx {
    sx().supports(IOS_ONLY, sx().font_size(format!("max(16px, {font_size})")))
}

/// A field wrapper's caption slots, not the label, the frame or the control.
/// `:where` keeps the specificity of a bare `[data-slot]`.
pub(crate) const FIELD_CAPTIONS: &str =
    "& > [data-slot]:where(:not([data-slot='label'], [data-slot='frame'], [data-slot='control']))";

str_enum! {
    /// How a checkable field draws its wrapper. `Card` makes the whole field a
    /// bordered surface and its hit area.
    pub enum ChoiceVariant {
        #[default]
        Plain = "plain",
        Card = "card",
    }
}

/// One size step of a field: the slots around the control, and its frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldSizeLevel {
    /// The caption above the control.
    pub label_font_size: &'static str,
    /// Description, helper and status share one scale.
    pub caption_font_size: &'static str,
    /// The control's own text.
    pub font_size: &'static str,
    /// Floor for a single-line field, so every field in a form lines up.
    pub height: &'static str,
    /// Drives the frame's height once the control wraps (a `Textarea`).
    pub padding_y: &'static str,
    pub padding_x: &'static str,
}

/// Theme defaults shared by every field, set on [`Theme`](crate::theme::Theme).
///
/// A field's own `*Defaults` hold only what differs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldDefaults {
    /// Vertical gap between the slots.
    pub gap: &'static str,
    /// Horizontal gap between the frame's leading, control and trailing.
    pub frame_gap: &'static str,
    pub sizes: Sizes<FieldSizeLevel>,
    /// Inner padding of a field drawn as a card, per size step.
    pub card_paddings: Sizes<&'static str>,
}

impl FieldDefaults {
    pub const DEFAULT: Self = Self {
        gap: "4px",
        frame_gap: "8px",
        sizes: Sizes::new(
            FieldSizeLevel {
                label_font_size: "0.6875rem",
                caption_font_size: "0.6875rem",
                font_size: "0.75rem",
                height: "28px",
                padding_y: "4px",
                padding_x: "8px",
            },
            FieldSizeLevel {
                label_font_size: "0.75rem",
                caption_font_size: "0.75rem",
                font_size: "0.8125rem",
                height: "32px",
                padding_y: "5px",
                padding_x: "10px",
            },
            // The type floor (todo 2707): 16px for what the user types, 14px for its label.
            FieldSizeLevel {
                label_font_size: "0.875rem",
                caption_font_size: "0.75rem",
                font_size: "1rem",
                height: "36px",
                padding_y: "6px",
                padding_x: "12px",
            },
            FieldSizeLevel {
                label_font_size: "0.9375rem",
                caption_font_size: "0.8125rem",
                font_size: "1.0625rem",
                height: "40px",
                padding_y: "7px",
                padding_x: "14px",
            },
            FieldSizeLevel {
                label_font_size: "1rem",
                caption_font_size: "0.875rem",
                font_size: "1.125rem",
                height: "44px",
                padding_y: "8px",
                padding_x: "16px",
            },
            FieldSizeLevel {
                label_font_size: "1.0625rem",
                caption_font_size: "0.9375rem",
                font_size: "1.25rem",
                height: "48px",
                padding_y: "9px",
                padding_x: "18px",
            },
        ),
        card_paddings: Sizes::new("8px", "10px", "12px", "14px", "16px", "18px"),
    };

    /// Per-slot typography, addressed from the wrapper: a class per slot would
    /// cost four stylesheet registrations per field.
    fn size_sx(size: Size) -> Sx {
        sx().selector(
            "& > label",
            sx().font_size(FIELD_LABEL_FONT_SIZE.value(size)),
        )
        .selector(
            FIELD_CAPTIONS,
            sx().font_size(FIELD_CAPTION_FONT_SIZE.value(size)),
        )
    }

    pub fn theme_vars() -> Sx {
        sx().gap(FIELD_GAP.value()).per_size(Self::size_sx)
    }

    fn frame_size_sx(size: Size) -> Sx {
        let padding_y = FIELD_PADDING_Y.value(size);
        sx().font_size(FIELD_FONT_SIZE.value(size))
            .and(ios_text_floor_sx(FIELD_FONT_SIZE.value(size)))
            .min_height(FIELD_HEIGHT.value(size))
            .padding_top(padding_y.clone())
            .padding_bottom(padding_y.clone())
            .padding_left(FIELD_PADDING_X.value(size))
            .padding_right(FIELD_PADDING_X.value(size))
            // A native `<select>` fills the frame's height, so a press on the padding opens it.
            .selector(
                "& > select",
                sx().align_self("stretch")
                    .margin_top(format!("calc(-1 * {padding_y})"))
                    .margin_bottom(format!("calc(-1 * {padding_y})"))
                    .padding_top(padding_y.clone())
                    .padding_bottom(padding_y),
            )
    }

    fn frame_radius_sx(radius: Size) -> Sx {
        sx().border_radius(SizeCss::RADIUS.overridable(radius))
    }

    pub fn frame_theme_vars() -> Sx {
        sx().gap(FIELD_FRAME_GAP.value())
            .per_size(Self::frame_size_sx)
            .per_radius(Self::frame_radius_sx)
    }
}

impl ToCssDeclarations for FieldDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![
            FIELD_GAP.declare(self.gap),
            FIELD_FRAME_GAP.declare(self.frame_gap),
        ];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(FIELD_LABEL_FONT_SIZE.declare(size, level.label_font_size));
            declarations.push(FIELD_CAPTION_FONT_SIZE.declare(size, level.caption_font_size));
            declarations.push(FIELD_FONT_SIZE.declare(size, level.font_size));
            declarations.push(FIELD_HEIGHT.declare(size, level.height));
            declarations.push(FIELD_PADDING_Y.declare(size, level.padding_y));
            declarations.push(FIELD_PADDING_X.declare(size, level.padding_x));
            declarations.push(FIELD_CARD_PADDING.declare(size, self.card_paddings.get(size)));
        }
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;
    use crate::sx::StaticSx;

    fn rem(length: &str) -> f32 {
        length.trim_end_matches("rem").parse().unwrap()
    }

    /// Todo 2707 raised md to the type floor; the steps above it moved along.
    #[test]
    fn the_text_and_label_rise_strictly_from_a_16px_and_14px_md() {
        let level = |size| FieldDefaults::DEFAULT.sizes.get(size);
        assert_eq!(rem(level(Size::Md).font_size), 1.0);
        assert_eq!(rem(level(Size::Md).label_font_size), 0.875);
        for pair in Size::ALL.windows(2) {
            let (low, high) = (level(pair[0]), level(pair[1]));
            assert!(rem(low.font_size) < rem(high.font_size), "{pair:?}");
            assert!(
                rem(low.label_font_size) < rem(high.label_font_size),
                "{pair:?}"
            );
        }
    }

    #[test]
    fn ios_lifts_every_size_to_16px_so_focus_does_not_zoom() {
        static PROBE: StaticSx = StaticSx::new(FieldDefaults::frame_theme_vars);
        let css = Stylesheet::from(&PROBE);
        let css = css.as_str();
        let supports = css
            .find("@supports (-webkit-touch-callout: none)")
            .unwrap_or_else(|| panic!("no iOS query: {css}"));
        let lifts = css[supports..].matches("max(16px, ").count();
        assert_eq!(lifts, Size::ALL.len(), "{css}");
    }

    #[test]
    fn the_floor_keeps_the_text_at_16px_or_the_given_size_on_ios_only() {
        static PROBE: StaticSx = StaticSx::new(|| sx().and(ios_text_floor_sx("1em")));
        let css = Stylesheet::from(&PROBE);
        let css = css.as_str();
        assert!(
            css.contains("@supports (-webkit-touch-callout: none)"),
            "{css}"
        );
        assert!(css.contains("font-size:max(16px, 1em)"), "{css}");
    }
}
