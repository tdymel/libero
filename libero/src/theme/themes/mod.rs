//! Ready-made [`ThemeSet`]s: the library's own, and six community palettes.
//!
//! Every ported set is a real light/dark **pair** from its upstream palette,
//! because a `ThemeSet` designates both halves. Ten of the palettes in the
//! pack these came from are dark-only and are not here (Maintainer's call,
//! 2026-09-20): a set whose light half we invented would be our palette
//! wearing somebody else's name.
//!
//! ## How a palette is mapped
//!
//! Upstream palettes name colours by *what they are for*, which is the same
//! shape as our roles, so the mapping is one to one:
//!
//! | Ours | Theirs | Why |
//! |---|---|---|
//! | `surface` | `bg` | The page. |
//! | `ink` | `text` | What text is set in. |
//! | `neutral` | `text` | Ours is the text-dark role a plain control takes. |
//! | `muted` | `text-muted` | Distance from the page, at its quiet-text step. |
//! | `primary` | `accent` | |
//! | `secondary` | `highlight` | |
//! | `info` | `accent-soft` | |
//! | `success` | `progress` | |
//! | `error` | `danger` | |
//! | `paper.background` | `raised` | The card drawn on the page. |
//!
//! **`warning` has no counterpart** - none of these palettes carries one - so
//! every ported theme keeps Libero's amber. The shade ramps, the text and
//! fill roles and the `muted` ramp are all derived from the four colours
//! above by [`super::stylesheet`], so a ported theme is ten hex values and
//! nothing else.
//!
//! The five `*Defaults` that hold literal CSS colours follow the *scheme*,
//! not the palette: a ported dark theme takes `CodeDefaults::DARK` and the
//! rest, a ported light one takes their `DEFAULT`. Only `Paper`'s background
//! is per-palette, because the palettes name it.

mod ayu;
mod catppuccin;
mod everforest;
mod gruvbox;
mod one;
mod rose_pine;

pub use ayu::{AYU_DARK, AYU_LIGHT};
pub use catppuccin::{CATPPUCCIN_DARK, CATPPUCCIN_LIGHT};
pub use everforest::{EVERFOREST_DARK, EVERFOREST_LIGHT};
pub use gruvbox::{GRUVBOX_DARK, GRUVBOX_LIGHT};
pub use one::{ONE_DARK, ONE_LIGHT};
pub use rose_pine::{ROSE_PINE_DARK, ROSE_PINE_LIGHT};

use super::ThemeSet;

impl ThemeSet {
    /// Every set the library ships, in the order a picker should list them:
    /// ours first, then the ported palettes alphabetically.
    ///
    /// It is what the docs site's theme picker is built from, and it is the
    /// only thing that references the ported themes - so an app that never
    /// touches it does not pay for them.
    pub const CATALOGUE: &'static [&'static ThemeSet] = &[
        &ThemeSet::DEFAULT,
        &ThemeSet::AYU,
        &ThemeSet::CATPPUCCIN,
        &ThemeSet::EVERFOREST,
        &ThemeSet::GRUVBOX,
        &ThemeSet::ONE,
        &ThemeSet::ROSE_PINE,
    ];

    /// The set `name` labels, or `None`. Case-sensitive, and the names are
    /// the ones in [`CATALOGUE`](Self::CATALOGUE).
    pub fn from_catalogue(name: &str) -> Option<&'static ThemeSet> {
        Self::CATALOGUE
            .iter()
            .copied()
            .find(|set| set.name() == name)
    }
}
