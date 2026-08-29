//! Derive macros for `libero`. Re-exported from the crate the trait lives in,
//! so one import brings both.

mod fields;
mod options;
mod slider_value;

use proc_macro::TokenStream;

/// Emits typed field paths for a struct: `Signup::FIELDS.address().zip()` is
/// a `FieldPath<Signup, String>` spelling `address.zip`, and a typo does not
/// compile. Mark a field whose type derives `Fields` too with
/// `#[fields(nested)]`.
///
/// ```ignore
/// #[derive(Fields)]
/// struct Signup {
///     email: String,
///     #[fields(nested)]
///     address: Address,
/// }
/// ```
#[proc_macro_derive(Fields, attributes(fields))]
pub fn fields(input: TokenStream) -> TokenStream {
    fields::derive(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

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

/// Implements `Options` for an enum of unit variants: the variants in
/// declaration order are the choices, and each one's name is its label.
///
/// ```ignore
/// #[derive(Clone, PartialEq, Options)]
/// enum Section {
///     Account,
///     #[option(label = "Admin area")]
///     Admin,
/// }
/// ```
#[proc_macro_derive(Options, attributes(option))]
pub fn options(input: TokenStream) -> TokenStream {
    options::derive(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
