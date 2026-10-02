use crate::{str_enum::str_enum, theme::CssVar};

/// How far the area scrolls, in px, on the layer the drawn bars ride.
pub(crate) const SCROLL_AREA_RANGE_Y: CssVar = CssVar::new("--lsx-scroll-area-range-y");
/// Negative under RTL, where the content moves right as it scrolls on.
pub(crate) const SCROLL_AREA_RANGE_X: CssVar = CssVar::new("--lsx-scroll-area-range-x");
/// How far a drawn thumb travels along its track, in px; negative for an RTL x thumb.
pub(crate) const SCROLL_AREA_THUMB_TRAVEL: CssVar = CssVar::new("--lsx-scroll-area-thumb-travel");

/// Keeps the drawn bars' layers still and moves their thumbs while the content
/// scrolls, on the compositor. Without scroll timelines, inline styles do it.
pub(crate) const SCROLL_AREA_KEYFRAMES: &str = concat!(
    "@keyframes lsx-scroll-area-pin-y{from{translate:0 0;}",
    "to{translate:0 var(--lsx-scroll-area-range-y);}}",
    "@keyframes lsx-scroll-area-pin-x{from{translate:0 0;}",
    "to{translate:var(--lsx-scroll-area-range-x) 0;}}",
    "@keyframes lsx-scroll-area-thumb-y{from{translate:0 0;}",
    "to{translate:0 var(--lsx-scroll-area-thumb-travel);}}",
    "@keyframes lsx-scroll-area-thumb-x{from{translate:0 0;}",
    "to{translate:var(--lsx-scroll-area-thumb-travel) 0;}}",
    "@supports (animation-timeline:scroll()){",
    "[data-slot='scrollbars']:not([data-empty]){animation:lsx-scroll-area-pin-y linear both;",
    "animation-timeline:scroll(nearest block);}",
    "[data-scrollbars-x]{animation:lsx-scroll-area-pin-x linear both;",
    "animation-timeline:scroll(nearest inline);}",
    "[data-scrollbars-x]>[data-orientation='vertical']>[data-slot='thumb']{",
    "animation:lsx-scroll-area-thumb-y linear both;animation-timeline:scroll(nearest block);}",
    "[data-scrollbars-x]>[data-orientation='horizontal']>[data-slot='thumb']{",
    "animation:lsx-scroll-area-thumb-x linear both;animation-timeline:scroll(nearest inline);}}"
);

str_enum! {
    /// Which axes allow overflow and show a scrollbar.
    #[state_prefix = "axis"]
    pub enum ScrollAxis {
        #[default]
        Vertical = "vertical",
        Horizontal = "horizontal",
        Both = "both",
        None = "none",
    }
}

str_enum! {
    /// `Scroll` behaves as `Hover`: nothing fades on an idle timeout.
    #[state_prefix = "visible"]
    pub enum ScrollbarVisibility {
        #[default]
        Always = "always",
        Hover = "hover",
        Hidden = "hidden",
        Scroll = "scroll",
    }
}

str_enum! {
    /// The CSS `scrollbar-width` keyword.
    #[state_prefix = "size"]
    pub enum ScrollbarSize {
        #[default]
        Thin = "thin",
        Auto = "auto",
    }
}

/// Theme defaults for `ScrollArea`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollAreaDefaults {
    pub scrollbars: ScrollAxis,
    pub visibility: ScrollbarVisibility,
    pub size: ScrollbarSize,
    /// Rows a `Virtualize` keeps beyond each edge of the viewport.
    pub overscan: usize,
}

impl ScrollAreaDefaults {
    pub const DEFAULT: Self = Self {
        scrollbars: ScrollAxis::Vertical,
        visibility: ScrollbarVisibility::Always,
        size: ScrollbarSize::Thin,
        overscan: 4,
    };
}
