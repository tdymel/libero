/// A `NavLink` with nested links.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavLinkLabels {
    /// Names the button that shows the nested links. The link's own name
    /// follows it, so it reads "Show links, Docs".
    pub show_links: &'static str,
}

impl NavLinkLabels {
    pub const ENGLISH: Self = Self {
        show_links: "Show links",
    };

    pub const GERMAN: Self = Self {
        show_links: "Links anzeigen",
    };
}
