use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Field, Fields, Result, parse2};

pub(crate) fn derive(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let name = &input.ident;
    let vis = &input.vis;
    let paths = format_ident!("{name}Fields");

    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            name,
            "Fields can only be derived for a struct - the paths are its named fields",
        ));
    };
    let Fields::Named(named) = &data.fields else {
        return Err(Error::new_spanned(
            name,
            "Fields needs named fields - a path is spelled with the field's name",
        ));
    };
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &input.generics,
            "Fields cannot be derived for a generic type yet",
        ));
    }

    let methods = named
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().expect("a named field");
            let field_vis = &field.vis;
            let ty = &field.ty;
            let key = ident.to_string();
            let doc = format!("The path of `{key}`.");
            // Plain `fn` items, not closures: a closure's elided lifetimes do
            // not tie its `&dyn Any` result to its argument.
            let step = quote! {
                {
                    fn get(value: &dyn ::std::any::Any) -> ::std::option::Option<&dyn ::std::any::Any> {
                        value
                            .downcast_ref::<#name>()
                            .map(|parent| &parent.#ident as &dyn ::std::any::Any)
                    }
                    fn get_mut(
                        value: &mut dyn ::std::any::Any,
                    ) -> ::std::option::Option<&mut dyn ::std::any::Any> {
                        value
                            .downcast_mut::<#name>()
                            .map(|parent| &mut parent.#ident as &mut dyn ::std::any::Any)
                    }
                    ::libero::components::Step { get, get_mut }
                }
            };
            Ok(match is_nested(field)? {
                true => quote! {
                    #[doc = #doc]
                    #field_vis fn #ident(&self) -> <#ty as ::libero::components::Fields>::Paths<R> {
                        <#ty as ::libero::components::Fields>::paths(
                            self.base.child::<#ty>(#key, #step),
                        )
                    }
                },
                false => quote! {
                    #[doc = #doc]
                    #field_vis fn #ident(&self) -> ::libero::components::FieldPath<R, #ty> {
                        self.base.child(#key, #step)
                    }
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let struct_doc = format!("Typed field paths of [`{name}`], rooted at `R`.");
    Ok(quote! {
        #[doc = #struct_doc]
        #vis struct #paths<R> {
            base: ::libero::components::FieldPath<R, #name>,
        }

        impl<R> #paths<R> {
            /// The path of this struct itself - the prefix every field here
            /// sits under. What a `Fieldset` takes as its `path`.
            pub fn path(&self) -> ::libero::components::FieldPath<R, #name> {
                self.base.clone()
            }

            #(#methods)*
        }

        impl<R> ::std::convert::From<#paths<R>> for ::std::string::String {
            fn from(paths: #paths<R>) -> Self {
                paths.base.into()
            }
        }

        /// `path: Order::FIELDS.address()` on a `Fieldset`.
        impl<R> ::std::convert::From<#paths<R>> for ::libero::components::FieldName<#name> {
            fn from(paths: #paths<R>) -> Self {
                paths.base.into()
            }
        }

        impl<R> ::std::fmt::Display for #paths<R> {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.base, formatter)
            }
        }

        impl ::libero::components::Fields for #name {
            type Paths<R> = #paths<R>;

            fn paths<R>(base: ::libero::components::FieldPath<R, #name>) -> #paths<R> {
                #paths { base }
            }
        }

        impl #name {
            /// Typed paths of this struct's fields, for `name` and `.on(..)`.
            pub const FIELDS: #paths<#name> = #paths {
                base: ::libero::components::FieldPath::new(""),
            };
        }
    })
}

/// `#[fields(nested)]`: the field's type derives `Fields` too, so its paths
/// continue under this one.
fn is_nested(field: &Field) -> Result<bool> {
    let mut nested = false;
    for attribute in field.attrs.iter().filter(|a| a.path().is_ident("fields")) {
        attribute.parse_nested_meta(|meta| match meta.path.is_ident("nested") {
            true => {
                nested = true;
                Ok(())
            }
            false => Err(meta.error("unknown `fields` setting - only `nested` is understood")),
        })?;
    }
    Ok(nested)
}
