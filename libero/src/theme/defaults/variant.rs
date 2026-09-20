use crate::str_enum::str_enum;

str_enum! {
    /// Material 3's five emphasis levels, in descending order, plus a gradient
    /// fill. Shared by every component with variant chrome.
    pub enum Variant {
        #[default]
        Filled = "filled",
        Tonal = "tonal" | "filled-tonal",
        Elevated = "elevated",
        Outlined = "outlined" | "outline",
        /// The lowest emphasis; `text`, M3's button name, parses too.
        Standard = "standard" | "text",
        /// Not M3's: filled with the theme's gradient, or the call site's `gradient`.
        Gradient = "gradient",
    }
}
