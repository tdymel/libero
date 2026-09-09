//! Ready-made [`ThemeSet`]s: the library's own, and nineteen community
//! palettes.
//!
//! Ayu, Catppuccin, Everforest, Gruvbox, One and Rosé Pine are light/dark
//! pairs straight from the Kopuz theme pack; Flexoki, GitHub and Kanagawa
//! were taken from their upstream repositories, named in each file.
//!
//! ## The pack's dark-only palettes
//!
//! Ten of the pack's palettes are dark-only, and a `ThemeSet` designates
//! both halves. Their dark half is the pack's; the light half follows the
//! Maintainer's rule (2026-09-22): **take the palette's popular light
//! equivalent, and invent one only when there is none.** Each file names
//! its source:
//!
//! - **Upstream's own light theme** - Dracula (Alucard, from the Dracula
//!   spec), Nord (the bright ambiance Nord's docs describe, as Helix ships
//!   it), Gruvbox Classic (`morhetz/gruvbox` light), Ef Night (ef-day, its
//!   counterpart by name in `protesilaos/ef-themes`).
//! - **A light half the catalogue already has** - Ayu Mirage takes
//!   [`AYU_LIGHT`], Kanagawa Dragon [`KANAGAWA_LIGHT`] (Lotus), Gruvbox Soft
//!   [`GRUVBOX_LIGHT`] (Light Soft): each is upstream's one light theme for
//!   that family.
//! - **Derived, because none exists** - Vague, Osmium and kettek16. The
//!   light half keeps the dark half's five accents, since the stylesheet
//!   derives every role's text and fill steps against the page anyway;
//!   `ink` is the dark page; the page is the dark `text` mixed 80% toward
//!   white and the card the same mixed 60%; `muted` is the dark
//!   `text-muted` moved toward the ink or toward white until it sits as far
//!   from the light page, by contrast ratio, as it does from the dark one.
//!   Only the neutrals are ours, and each file shows its numbers.
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
//! **`warning` has no counterpart** - the Kopuz pack carries none, and the
//! palettes that do name a yellow are kept on ours for one catalogue - so
//! every ported theme keeps Libero's amber. The shade ramps, the text and
//! fill roles and the `muted` ramp are all derived from the four colours
//! above by [`super::stylesheet`], so a ported theme is ten hex values and
//! nothing else.
//!
//! Code blocks, `Kbd` and `Tooltip` draw on steps of the `muted` ramp, so they
//! follow the palette (todo 396). Two `*Defaults` still hold literal colours:
//! `Paper`'s background, per palette because the palettes name it, and the
//! syntax-token hues of `CodeDefaults`, which follow the *scheme* and are
//! walked until they read on the palette's code block.

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
    /// Every set the library ships, in the order a picker should list them:
    /// ours first, then the ported palettes alphabetically.
    ///
    /// It is what the docs site's theme picker is built from, and it is the
    /// only thing that references the ported themes - so an app that never
    /// touches it does not pay for them.
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

    /// The set `name` labels, or `None`. Case-sensitive, and the names are
    /// the ones in [`CATALOGUE`](Self::CATALOGUE).
    pub fn from_catalogue(name: &str) -> Option<&'static ThemeSet> {
        Self::CATALOGUE
            .iter()
            .copied()
            .find(|set| set.name() == name)
    }
}
