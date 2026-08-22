//! Derive macros for `libero`. Re-exported from the crate the trait lives in,
//! so one import brings both.

mod slider_value;

use proc_macro::TokenStream;

/// Implements `SliderValue` for an ordered enum of unit variants, which is
/// what makes a `Slider` discrete: the variants in declaration order are the
/// options, and each one's name is its label.
///
/// ```ignore
/// #[derive(Clone, PartialEq, SliderValue)]
/// enum Quality {
///     Low,
///     #[slider(label = "Med")]
///     Medium,
///     High,
/// }
/// ```
#[proc_macro_derive(SliderValue, attributes(slider))]
pub fn slider_value(input: TokenStream) -> TokenStream {
    slider_value::derive(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
