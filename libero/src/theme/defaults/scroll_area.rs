use crate::str_enum::str_enum;

str_enum! {
    /// Which axes show a scrollbar / allow overflow.
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
    /// `Scroll` behaves as `Hover` - nothing here fades on an idle timeout.
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
    /// Maps directly to the CSS `scrollbar-width` keyword.
    #[state_prefix = "size"]
    pub enum ScrollbarSize {
        #[default]
        Thin = "thin",
        Auto = "auto",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollAreaDefaults {
    pub scrollbars: ScrollAxis,
    pub visibility: ScrollbarVisibility,
    pub size: ScrollbarSize,
}
