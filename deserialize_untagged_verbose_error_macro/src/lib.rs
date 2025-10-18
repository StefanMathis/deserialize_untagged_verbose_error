use proc_macro::{self, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Index, ItemEnum, Token, Type, parse_macro_input};

/**
The macro this crate revolves around. It is essentially a specialized version
of the `Deserialize` macro from [serde](https://serde.rs), which can only be
applied to enums where every variant is a tuple struct with one field. It behaves
in the same way as a combination of `Deserialize` and the
[`untagged`](https://serde.rs/enum-representations.html#untagged) attribute,
but returns a much more detailed (and verbose!) error in case of failure.

Please see the
[crate-level documentation](https://docs.rs/deserialize_untagged_verbose_error/0.1.0/deserialize_untagged_verbose_error/)
for more details and examples.
 */
#[proc_macro_derive(
    DeserializeUntaggedVerboseError,
    attributes(deserialize_untagged_verbose_error)
)]
#[proc_macro_error]
pub fn deserialize_untagged_verbose_error(input: TokenStream) -> TokenStream {
    let item_enum = parse_macro_input!(input as ItemEnum);
    let item_span = item_enum.span();

    // Preallocate the error vector with the right capacity
    let number_variants = item_enum.variants.len();

    // Adjust the generics
    let generics_de = {
        let mut generics_de = item_enum.generics.clone();

        // Add serde::de::Deserialize<'_> as trait bound to all types
        let mut serde_path = Punctuated::new();
        serde_path.push(syn::PathSegment {
            ident: Ident::new("serde", Span::call_site()),
            arguments: syn::PathArguments::None,
        });
        serde_path.push(syn::PathSegment {
            ident: Ident::new("de", Span::call_site()),
            arguments: syn::PathArguments::None,
        });

        let mut lifetimes = Punctuated::new();
        lifetimes.push(syn::GenericArgument::Lifetime(syn::Lifetime::new(
            "'de",
            Span::call_site(),
        )));

        serde_path.push(syn::PathSegment {
            ident: Ident::new("Deserialize", Span::call_site()),
            arguments: syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Token![<](Span::call_site()),
                args: lifetimes,
                gt_token: Token![>](Span::call_site()),
            }),
        });

        let de_bound = syn::TypeParamBound::Trait(syn::TraitBound {
            paren_token: None,
            modifier: syn::TraitBoundModifier::None,
            lifetimes: None,
            path: syn::Path {
                leading_colon: None,
                segments: serde_path,
            },
        }); //
        generics_de.type_params_mut().for_each(|type_param| {
            type_param.bounds.push(de_bound.clone());
        });

        // Adjust the generics of the enum by adding the deserializer lifetime
        let de_lifetime = syn::LifetimeParam {
            attrs: Vec::new(),
            lifetime: syn::Lifetime::new("'de", Span::call_site()),
            colon_token: None,
            bounds: Punctuated::new(),
        };
        generics_de.params = generics_de
            .params
            .into_iter()
            .chain(Some(syn::GenericParam::Lifetime(de_lifetime)))
            .collect();
        generics_de
    };

    let (impl_de_generics, _, where_clause) = generics_de.split_for_impl();
    let (impl_generics, _, _) = item_enum.generics.split_for_impl();
    let item_enum_name = item_enum.ident.clone();
    let mut variants: Vec<Ident> = Vec::new();
    for variant in item_enum.variants.iter() {
        variants.push(variant.ident.clone());
    }
    let mut item_names: Vec<syn::Type> = Vec::new();
    for variant in item_enum.variants.iter() {
        match &variant.fields {
            syn::Fields::Unnamed(fields_unnamed) => match fields_unnamed.unnamed.first() {
                Some(field) => {
                    /*
                    A type "Struct" taken from the field are used in the generated deserializing functions like this:
                    Struct::deserialize(...)
                    For a generic type "Struct<T>", the following token stream would be produced:
                    Struct<T>::deserialize(...)
                    This is not valid rust syntax, the correct token stream is:
                    Struct::<T>::deserialize(...)
                    Hence, we need to insert a leading double colon before "<".
                     */
                    let type_item = field.ty.clone();
                    let type_item = if let Type::Path(mut path) = type_item {
                        for segment in path.path.segments.iter_mut() {
                            match &mut segment.arguments {
                                syn::PathArguments::AngleBracketed(bracketed) => {
                                    // This adds the leading colon
                                    let path_sep = syn::token::PathSep {
                                        spans: [Span::call_site(), Span::call_site()],
                                    };
                                    bracketed.colon2_token = Some(path_sep);
                                }
                                _ => (),
                            }
                        }
                        Type::Path(path)
                    } else {
                        type_item
                    };

                    item_names.push(type_item);
                }
                None => abort!(
                    item_span,
                    "expected a single unnamed type, such as Some(T)".to_string()
                ),
            },
            _ => abort!(
                item_span,
                "expected a single unnamed type, such as Some(T)".to_string()
            ),
        }
        variants.push(variant.ident.clone());
    }

    let indices: Vec<Index> = (0..number_variants).map(|i| Index::from(i)).collect();

    return TokenStream::from(quote! {

        impl #impl_de_generics deserialize_untagged_verbose_error::__serde::de::Deserialize<'de> for #item_enum_name #impl_generics #where_clause {
            fn deserialize<__D>(__deserializer: __D) -> Result<Self, __D::Error>
            where
                __D: deserialize_untagged_verbose_error::__serde::de::Deserializer<'de>,
            {
                use deserialize_untagged_verbose_error::__serde::de::Error;

                let __content: deserialize_untagged_verbose_error::__serde_value::Value = deserialize_untagged_verbose_error::__serde::Deserialize::deserialize(__deserializer)?;

                let mut __errors: [::std::mem::MaybeUninit<(&'static str, __D::Error)>; #number_variants] = [const {::std::mem::MaybeUninit::uninit()}; #number_variants];
                let mut __counter: usize = 0;

                #(match #item_names::deserialize(deserialize_untagged_verbose_error::__serde_value::ValueDeserializer::new(__content.clone())) {
                    Ok(__var) => return Ok(#item_enum_name::#variants(__var)),
                    Err(__error) => {
                        let __elem = &mut __errors[__counter];
                        __elem.write((stringify!(#variants), __error));
                        __counter += 1;
                    },
                })*

                /*
                SAFETY: At this point, all elements of __errors have been
                initialized, since all variants have been tried. Furthermore,
                __errors is never needed again, so we can move all elements out of it.
                 */
                let __errors_init: [(&'static str, __D::Error); #number_variants] = unsafe {
                    [
                        #(std::ptr::read(&__errors[#indices]).assume_init()),*
                    ]
                };

                return Err(__D::Error::custom(
                    deserialize_untagged_verbose_error::UntaggedEnumDeError{
                        enum_name: stringify!(#item_enum_name),
                        errors: __errors_init,
                    },
                ));
            }
        }
    });
}
