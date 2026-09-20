//! Ready-made [`ThemeSet`]s: the library's own, and nineteen community palettes.
//! Each file names its source: the Kopuz theme pack or the upstream repository.
//!
//! ## The pack's dark-only palettes
//!
//! The light half is the palette's popular light equivalent, invented only when
//! none exists (Maintainer, 2026-09-22):
//!
//! - **Upstream's own**: Dracula (Alucard), Nord (bright ambiance), Gruvbox Classic, Ef Night (ef-day).
//! - **Already in the catalogue**: Ayu Mirage takes [`AYU_LIGHT`], Kanagawa Dragon
//!   [`KANAGAWA_LIGHT`], Gruvbox Soft [`GRUVBOX_LIGHT`].
//! - **Derived**: Vague, Osmium and kettek16 keep the dark accents; only the neutrals
//!   are ours, and each file shows its numbers.
//!
//! ## How a palette is mapped
//!
//! Upstream names colours by purpose, as our roles do, so the mapping is one to one:
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
//! **`warning` has no counterpart**: every ported theme keeps Libero's amber. Ramps
//! and roles are derived by the stylesheet, so a ported theme is ten hex values.
//!
//! Code blocks, `Kbd` and `Tooltip` follow the `muted` ramp (todo 396). Only `Paper`'s
//! background and `CodeDefaults`' syntax hues (per scheme) stay literal.

mod ayu;
mod catppuccin;
mod dracula;
mod ef;
mod everforest;
mod flexoki;
mod github;
mod gruvbox;
mod kanagawa;
mod kettek16;
mod nord;
mod one;
mod osmium;
mod rose_pine;
mod vague;

pub use ayu::{AYU_DARK, AYU_LIGHT, AYU_MIRAGE_DARK};
pub use catppuccin::{CATPPUCCIN_DARK, CATPPUCCIN_LIGHT};
pub use dracula::{DRACULA_DARK, DRACULA_LIGHT};
pub use ef::{EF_NIGHT_DARK, EF_NIGHT_LIGHT};
pub use everforest::{EVERFOREST_DARK, EVERFOREST_LIGHT};
pub use flexoki::{FLEXOKI_DARK, FLEXOKI_LIGHT};
pub use github::{GITHUB_DARK, GITHUB_LIGHT};
pub use gruvbox::{
    GRUVBOX_CLASSIC_DARK, GRUVBOX_CLASSIC_LIGHT, GRUVBOX_DARK, GRUVBOX_LIGHT, GRUVBOX_SOFT_DARK,
};
pub use kanagawa::{KANAGAWA_DARK, KANAGAWA_DRAGON_DARK, KANAGAWA_LIGHT};
pub use kettek16::{KETTEK16_DARK, KETTEK16_LIGHT};
pub use nord::{NORD_DARK, NORD_LIGHT};
pub use one::{ONE_DARK, ONE_LIGHT};
pub use osmium::{OSMIUM_DARK, OSMIUM_LIGHT};
pub use rose_pine::{ROSE_PINE_DARK, ROSE_PINE_LIGHT};
pub use vague::{VAGUE_DARK, VAGUE_LIGHT};

use super::ThemeSet;

impl ThemeSet {
    /// Every shipped set in picker order, ours first. The only reference to the
    /// ported themes, so an app that never touches it does not pay for them.
    ///
    /// Docs: <https://libero-ui.dev/about/theming>
    pub const CATALOGUE: &'static [&'static ThemeSet] = &[
        &ThemeSet::DEFAULT,
        &ThemeSet::AYU,
        &ThemeSet::AYU_MIRAGE,
        &ThemeSet::CATPPUCCIN,
        &ThemeSet::DRACULA,
        &ThemeSet::EF_NIGHT,
        &ThemeSet::EVERFOREST,
        &ThemeSet::FLEXOKI,
        &ThemeSet::GITHUB,
        &ThemeSet::GRUVBOX,
        &ThemeSet::GRUVBOX_CLASSIC,
        &ThemeSet::GRUVBOX_SOFT,
        &ThemeSet::KANAGAWA,
        &ThemeSet::KANAGAWA_DRAGON,
        &ThemeSet::KETTEK16,
        &ThemeSet::NORD,
        &ThemeSet::ONE,
        &ThemeSet::OSMIUM,
        &ThemeSet::ROSE_PINE,
        &ThemeSet::VAGUE,
    ];

    /// The [`CATALOGUE`](Self::CATALOGUE) set labelled `name` (case-sensitive), or `None`.
    ///
    /// ```
    /// # use libero::theme::ThemeSet;
    /// assert_eq!(ThemeSet::from_catalogue("Nord"), Some(&ThemeSet::NORD));
    /// ```
    pub fn from_catalogue(name: &str) -> Option<&'static ThemeSet> {
        Self::CATALOGUE
            .iter()
            .copied()
            .find(|set| set.name() == name)
    }
}
