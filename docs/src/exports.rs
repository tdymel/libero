//! What libero exports, read from its source by `build.rs`.

include!(concat!(env!("OUT_DIR"), "/components.rs"));

#[cfg(test)]
mod tests {
    use super::COMPONENTS;

    #[test]
    fn the_count_takes_every_kind_of_component() {
        // A `#[component]` fn, a hand-written props fn, a generic one and a sub-component.
        for name in ["Button", "Container", "Select", "RangeSlider", "TreeItem"] {
            assert!(COMPONENTS.contains(&name), "{name} is missing");
        }
        // Crate-only ones stay out.
        for name in ["Modal", "Calendar", "CloseIcon"] {
            assert!(!COMPONENTS.contains(&name), "{name} is not exported");
        }
    }
}
