//! Layer-neutral helpers, so anything from `css` up to `components` can reach
//! them without an upward dependency.

mod warn;

#[cfg(test)]
pub(crate) use warn::take_warnings;
pub(crate) use warn::warn;
