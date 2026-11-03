/// `AvatarGroup`'s overflow chip. `{n}` is the hidden count, `{names}` the
/// known hidden names joined by `, `.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarLabels {
    /// The chip's visible text: `+3`.
    pub count: &'static str,
    /// The chip's accessible name: `3 more`. Its tooltip describes it with the names.
    pub more: &'static str,
    /// The tooltip when `total` counts more people than are listed: `Grace and others`.
    pub others: &'static str,
}

impl AvatarLabels {
    pub const ENGLISH: Self = Self {
        count: "+{n}",
        more: "{n} more",
        others: "{names} and others",
    };

    pub const GERMAN: Self = Self {
        count: "+{n}",
        more: "{n} weitere",
        others: "{names} und weitere",
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::localization::fill;

    /// Todo 5598: the chip's accessible name keeps the hidden count bare.
    #[test]
    fn the_chip_name_has_no_group_separator() {
        let holes: &[(&str, &dyn std::fmt::Display)] = &[("n", &1234)];
        assert_eq!(fill(AvatarLabels::ENGLISH.more, holes), "1234 more");
        assert_eq!(fill(AvatarLabels::GERMAN.more, holes), "1234 weitere");
    }
}
