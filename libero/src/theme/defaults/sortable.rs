/// Where a dropped item's slide into its slot starts, a `transform`; set inline per item.
pub(crate) const SORTABLE_SETTLE_FROM: &str = "--lsx-sortable-settle";
pub(crate) const SORTABLE_SETTLE: &str = "lsx-sortable-settle";

/// Only without reduced motion: an inline `animation` naming missing keyframes does nothing.
pub(crate) const SORTABLE_KEYFRAMES: &str = concat!(
    "@media (prefers-reduced-motion: no-preference){",
    "@keyframes lsx-sortable-settle{from{transform:var(--lsx-sortable-settle);}}",
    "}",
);
