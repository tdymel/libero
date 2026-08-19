//! Layer-neutral helpers, so anything from `css` up to `components` can reach
//! them without an upward dependency.

mod warn;

pub(crate) use warn::warn;
