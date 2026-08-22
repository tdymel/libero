use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Expr, Fields, Ident, Lit, Result, Variant, parse2};

pub(crate) fn derive(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let name = &input.ident;

    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            &input.ident,
            "TabValue can only be derived for an enum - the tab strip is its variants",
        ));
    };
    if data.variants.is_empty() {
        return Err(Error::new_spanned(
            &input.ident,
            "TabValue needs at least one variant to show a tab for",
        ));
    }

    let variants: Vec<&Ident> = data
        .variants
        .iter()
        .map(|variant| match variant.fields {
            Fields::Unit => Ok(&variant.ident),
            _ => Err(Error::new_spanned(
                variant,
                "TabValue needs unit variants - a variant with fields is not one tab",
            )),
        })
        .collect::<Result<_>>()?;
    let labels: Vec<String> = data.variants.iter().map(label_of).collect::<Result<_>>()?;

    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &input.generics,
            "TabValue cannot be derived for a generic type - its tabs are a `&'static [Self]`",
        ));
    }

    Ok(quote! {
        impl ::libero::components::TabValue for #name {
            fn options() -> &'static [Self] {
                &[#(Self::#variants),*]
            }

            fn label(&self) -> ::std::string::String {
                match self {
                    #(Self::#variants => #labels,)*
                }
                .to_string()
            }
        }
    })
}

/// `#[tab(label = "..")]`, else the variant's own name.
fn label_of(variant: &Variant) -> Result<String> {
    let mut label = None;
    for attribute in variant.attrs.iter().filter(|a| a.path().is_ident("tab")) {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("label") {
                return Err(meta.error("unknown `tab` option - only `label` is understood"));
            }
            match meta.value()?.parse()? {
                Expr::Lit(literal) => match literal.lit {
                    Lit::Str(text) => {
                        label = Some(text.value());
                        Ok(())
                    }
                    _ => Err(meta.error("`label` takes a string")),
                },
                _ => Err(meta.error("`label` takes a string literal")),
            }
        })?;
    }
    Ok(label.unwrap_or_else(|| variant.ident.to_string()))
}
