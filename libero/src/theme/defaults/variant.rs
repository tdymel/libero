use crate::str_enum::str_enum;

str_enum! {
    /// Material 3's five emphasis levels, in descending order. Named after
    /// its button styles, and shared by every component with variant chrome.
    pub enum Variant {
        #[default]
        Filled = "filled",
        Tonal = "tonal" | "filled-tonal",
        Elevated = "elevated",
        Outlined = "outlined" | "outline",
        /// M3's name for the lowest-emphasis arm. `text`, its name on a
        /// button, parses too.
        Standard = "standard" | "text",
    }
}
