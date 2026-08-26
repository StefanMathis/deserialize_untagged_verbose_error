use proc_macro::{self};
use proc_macro_error::proc_macro_error;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Attribute, Generics, Index, ItemEnum, Token, Type, parse_macro_input};

/**
The macro this crate revolves around is essentially a specialized version
of the `Deserialize` macro from [serde](https://serde.rs), combined with
the [`untagged`](https://serde.rs/enum-representations.html#untagged)
enum representation. Unlike Serde's implementation, however, it reports a
detailed error for every variant that was attempted when deserialization fails.

Please see the
[crate-level documentation](https://crates.io/crates/deserialize_untagged_verbose_error)
for more details and examples.
 */
#[proc_macro_derive(DeserializeUntaggedVerboseError, attributes(serde))]
#[proc_macro_error]
pub fn deserialize_untagged_verbose_error(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    struct UnnamedFieldInfo {
        ty: Type,
        attrs: Vec<Attribute>,
    }

    impl UnnamedFieldInfo {
        fn has_attrs(&self) -> bool {
            !self.attrs.is_empty()
        }
    }

    struct NamedFieldInfo {
        ident: Ident,
        ty: Type,
        attrs: Vec<Attribute>,
    }

    enum VariantKind {
        Unit,
        Unnamed(Vec<UnnamedFieldInfo>),
        Named(Vec<NamedFieldInfo>),
    }

    struct VariantInfo {
        ident: Ident,
        attrs: Vec<Attribute>,
        kind: VariantKind,
    }

    impl VariantInfo {
        fn unnamed_fields_have_attrs(&self) -> bool {
            match &self.kind {
                VariantKind::Unnamed(fields) => fields.iter().any(UnnamedFieldInfo::has_attrs),
                _ => false,
            }
        }

        fn generate_constructor(&self, enum_name: &Ident) -> proc_macro2::TokenStream {
            let ident = &self.ident;

            match &self.kind {
                VariantKind::Unit => {
                    quote! {
                        #enum_name::#ident
                    }
                }
                VariantKind::Unnamed(fields) => {
                    if fields.len() == 1 && self.unnamed_fields_have_attrs() {
                        quote! {
                            #enum_name::#ident(__var.0)
                        }
                    } else {
                        match fields.len() {
                            // Tuple variants with one name need to be special-cased,
                            // since __var.0 does not exist for them. "let fields"
                            // in the general case below therefore fails for them.
                            1 => {
                                quote! {
                                    #enum_name::#ident(__var)
                                }
                            }

                            _ => {
                                let indices = (0..fields.len()).map(Index::from);

                                let fields = indices.map(|index| {
                                    quote! { __var.#index }
                                });

                                quote! {
                                    #enum_name::#ident(#(#fields),*)
                                }
                            }
                        }
                    }
                }
                VariantKind::Named(fields) => {
                    let fields = fields.iter().map(|fields_info| {
                        let ident = &fields_info.ident;
                        quote! {
                            #ident: __var.#ident
                        }
                    });

                    quote! {
                        #enum_name::#ident {
                            #(#fields),*
                        }
                    }
                }
            }
        }

        /// Generates a helper struct for struct tuple variants.
        fn generate_helper_struct(
            &self,
            enum_name: &Ident,
            enum_generics: &Generics,
            enum_attrs: &[Attribute],
        ) -> proc_macro2::TokenStream {
            let should_generate = match &self.kind {
                VariantKind::Named(_) => true,
                VariantKind::Unnamed(fields) => fields.iter().any(UnnamedFieldInfo::has_attrs),
                VariantKind::Unit => false,
            };

            if !should_generate {
                return TokenStream::new();
            }

            match &self.kind {
                VariantKind::Named(fields) => {
                    let helper_struct =
                        Ident::new(&self.helper_struct_name(enum_name), self.ident.span());
                    let helper_generics = self.helper_struct_generics(enum_generics);

                    let field_names = fields
                        .iter()
                        .map(|named_field_info| &named_field_info.ident);
                    let field_types = fields.iter().map(|named_field_info| &named_field_info.ty);
                    let field_attrs = fields.iter().map(|field| &field.attrs);

                    quote! {
                        #[allow(non_camel_case_types)]
                        #[derive(
                            deserialize_untagged_verbose_error::__serde::Deserialize
                        )]
                        #(#enum_attrs)*
                        struct #helper_struct #helper_generics {
                            #(
                                #(#field_attrs)*
                                #field_names: #field_types,
                            )*
                        }
                    }
                }
                VariantKind::Unnamed(fields) => {
                    let helper_struct =
                        Ident::new(&self.helper_struct_name(enum_name), self.ident.span());
                    let helper_generics = self.helper_struct_generics(enum_generics);

                    let field_types = fields.iter().map(|field_info| &field_info.ty);
                    let field_attrs = fields.iter().map(|field_info| &field_info.attrs);

                    quote! {
                        #[allow(non_camel_case_types)]
                        #[derive(
                            deserialize_untagged_verbose_error::__serde::Deserialize
                        )]
                        struct #helper_struct #helper_generics(
                            #(
                                #(#field_attrs)*
                                #field_types,
                            )*
                        );
                    }
                }
                VariantKind::Unit => TokenStream::new(),
            }
        }

        /// Convert the generics of the original enum into a TokenStream.
        fn helper_struct_generics(&self, generics: &Generics) -> TokenStream {
            let params = generics.params.iter().map(|param| match param {
                syn::GenericParam::Type(param) => {
                    let ident = &param.ident;
                    quote! { #ident }
                }
                syn::GenericParam::Lifetime(param) => {
                    let lifetime = &param.lifetime;
                    quote! { #lifetime }
                }
                syn::GenericParam::Const(param) => {
                    let ident = &param.ident;
                    quote! { #ident }
                }
            });

            // Create the expected Rust generics syntax. The order will be correct
            // by default (lifetimes before types) because the input generics from
            // the enum are already in correct order.
            quote! { <#(#params),*> }
        }

        fn deserialize_type(
            &self,
            enum_name: &Ident,
            enum_generics: &Generics,
        ) -> proc_macro2::TokenStream {
            match &self.kind {
                VariantKind::Unit => {
                    quote! { () }
                }
                VariantKind::Unnamed(fields) => {
                    if self.unnamed_fields_have_attrs() {
                        let helper_ident =
                            Ident::new(&self.helper_struct_name(enum_name), self.ident.span());

                        let helper_generics = self.helper_struct_generics(enum_generics);

                        quote! { #helper_ident #helper_generics }
                    } else {
                        match fields.as_slice() {
                            [unnamed_field_info] => {
                                let ty = &unnamed_field_info.ty;
                                quote! { #ty }
                            }
                            _ => {
                                let ty = fields.iter().map(|field| &field.ty);
                                quote! { (#(#ty,)*) }
                            }
                        }
                    }
                }
                VariantKind::Named(_) => {
                    let helper_ident =
                        Ident::new(&self.helper_struct_name(enum_name), self.ident.span());
                    let helper_generics = self.helper_struct_generics(enum_generics);
                    quote! { #helper_ident #helper_generics }
                }
            }
        }

        fn helper_struct_name(&self, enum_name: &Ident) -> String {
            format!(
                "__DeserializeUntaggedVerboseError_{}_{}",
                enum_name, self.ident
            )
        }

        fn generate_deserialize_attempt(
            &self,
            enum_name: &Ident,
            enum_generics: &Generics,
        ) -> proc_macro2::TokenStream {
            let deserialize_type = self.deserialize_type(enum_name, enum_generics);
            let constructor = self.generate_constructor(enum_name);
            let ident = &self.ident;

            let tuple_length_check = match &self.kind {
                VariantKind::Unnamed(fields) => {
                    let expected_len = fields.len();

                    quote! {
                        match &__content {
                            deserialize_untagged_verbose_error::__serde_value::Value::Seq(__seq)
                                if __seq.len() != #expected_len =>
                            {
                                let __elem = &mut __errors[__counter];
                                __elem.write((
                                    stringify!(#ident),
                                    __D::Error::custom(format!(
                                        "invalid length {}, expected a tuple of size {}",
                                        __seq.len(),
                                        #expected_len
                                    )),
                                ));
                                __counter += 1;
                            }

                            _ => {
                                match <#deserialize_type as deserialize_untagged_verbose_error::__serde::Deserialize<'de>>::deserialize(
                                    deserialize_untagged_verbose_error::__serde_value::ValueDeserializer::new(
                                        __content.clone()
                                    )
                                ) {
                                    Ok(__var) => return Ok(#constructor),
                                    Err(__error) => {
                                        let __elem = &mut __errors[__counter];
                                        __elem.write((stringify!(#ident), __error));
                                        __counter += 1;
                                    },
                                }
                            }
                        }
                    }
                }
                _ => {
                    quote! {
                        match <#deserialize_type as deserialize_untagged_verbose_error::__serde::Deserialize<'de>>::deserialize(
                            deserialize_untagged_verbose_error::__serde_value::ValueDeserializer::new(
                                __content.clone()
                            )
                        ) {
                            Ok(__var) => return Ok(#constructor),
                            Err(__error) => {
                                let __elem = &mut __errors[__counter];
                                __elem.write((stringify!(#ident), __error));
                                __counter += 1;
                            },
                        }
                    }
                }
            };

            tuple_length_check
        }
    }

    let item_enum = parse_macro_input!(input as ItemEnum);

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
        });
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

    let variants: Vec<VariantInfo> = item_enum
        .variants
        .iter()
        .map(|variant| {
            let kind = match &variant.fields {
                syn::Fields::Unit => VariantKind::Unit,

                syn::Fields::Unnamed(fields) => VariantKind::Unnamed(
                    fields
                        .unnamed
                        .iter()
                        .map(|field| UnnamedFieldInfo {
                            ty: field.ty.clone(),
                            attrs: field.attrs.clone(),
                        })
                        .collect(),
                ),

                syn::Fields::Named(fields) => VariantKind::Named(
                    fields
                        .named
                        .iter()
                        .map(|field| NamedFieldInfo {
                            ident: field.ident.clone().expect("must contain ident"),
                            ty: field.ty.clone(),
                            attrs: field.attrs.clone(),
                        })
                        .collect(),
                ),
            };

            VariantInfo {
                ident: variant.ident.clone(),
                attrs: variant.attrs.clone(),
                kind,
            }
        })
        .collect();

    let number_variants = item_enum.variants.len();
    let indices: Vec<Index> = (0..number_variants).map(|i| Index::from(i)).collect();

    let deserialize_attempts = variants
        .iter()
        .map(|variant| variant.generate_deserialize_attempt(&item_enum_name, &item_enum.generics));

    let helper_structs = variants.iter().map(|variant| {
        variant.generate_helper_struct(&item_enum_name, &item_enum.generics, &item_enum.attrs)
    });

    return TokenStream::from(quote! {
        impl #impl_de_generics deserialize_untagged_verbose_error::__serde::de::Deserialize<'de>
            for #item_enum_name #impl_generics #where_clause
        {
            fn deserialize<__D>(__deserializer: __D) -> Result<Self, __D::Error>
            where
                __D: deserialize_untagged_verbose_error::__serde::de::Deserializer<'de>,
            {
                use deserialize_untagged_verbose_error::__serde::de::Error;

                #(#helper_structs)*

                let __content: deserialize_untagged_verbose_error::__serde_value::Value =
                    deserialize_untagged_verbose_error::__serde::Deserialize::deserialize(
                        __deserializer
                    )?;

                let mut __errors:
                    [::std::mem::MaybeUninit<(&'static str, __D::Error)>; #number_variants]
                    = [const {
                        ::std::mem::MaybeUninit::uninit()
                    }; #number_variants];

                // __counter is used within the interpolated deserialize_attempts TokenStream
                let mut __counter: usize = 0;

                #(#deserialize_attempts)*

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

                Err(__D::Error::custom(
                    deserialize_untagged_verbose_error::UntaggedEnumDeError {
                        enum_name: stringify!(#item_enum_name),
                        errors: __errors_init,
                    },
                ))
            }
        }
    })
    .into();
}
