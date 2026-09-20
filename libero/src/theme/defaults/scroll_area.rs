use crate::str_enum::str_enum;

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
