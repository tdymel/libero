//! Layer-neutral helpers, so anything from `css` up to `components` can reach
//! them without an upward dependency.

mod data_url;
mod warn;

pub(crate) use data_url::bytes_data_url;
pub use data_url::data_url;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub(crate) use data_url::encode_base64;

pub(crate) use warn::warn;
#[cfg(test)]
pub(crate) use warn::{take_warnings, warnings_of};
