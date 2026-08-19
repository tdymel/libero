//! Crate-wide helpers that belong to no layer in particular, so anything
//! from `css` up to `components` can reach them without an upward
//! dependency (see CLAUDE.md's layered architecture).

mod warn;

pub(crate) use warn::warn;
