use crate::str_enum::str_enum;

str_enum! {
    /// Material 3's five emphasis levels, in descending order, plus a
    /// gradient fill. Named after M3's button styles, and shared by every
    /// component with variant chrome.
    pub enum Variant {
        #[default]
        Filled = "filled",
        Tonal = "tonal" | "filled-tonal",
        Elevated = "elevated",
        Outlined = "outlined" | "outline",
        /// M3's name for the lowest-emphasis arm. `text`, its name on a
        /// button, parses too.
        Standard = "standard" | "text",
        /// Not M3's: a filled look in the theme's gradient, or the call
        /// site's `gradient` where the component takes one.
        Gradient = "gradient",
    }
}
