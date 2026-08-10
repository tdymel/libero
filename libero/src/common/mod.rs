mod const_str;
mod const_vec;
mod str;

pub use const_str::ConstStr;
pub use const_vec::{ConstReadBuffer, ConstVec};
pub(crate) use str::{eq, starts_with};
