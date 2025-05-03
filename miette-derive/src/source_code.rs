use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef},
    forward::WhichFn,
    trait_bounds::TypeParamBoundStore,
    utils::{display_pat_members, extract_option, gen_all_variants_with},
};

pub struct SourceCode {
    source_code: syn::Member,
    is_option: bool,
}

impl SourceCode {
    pub fn from_fields(
        fields: &syn::Fields,
        bounds_store: &mut TypeParamBoundStore,
    ) -> syn::Result<Option<Self>> {
        match fields {
            syn::Fields::Named(named) => {
                Self::from_fields_vec(named.named.iter().collect(), bounds_store)
            }
            syn::Fields::Unnamed(unnamed) => {
                Self::from_fields_vec(unnamed.unnamed.iter().collect(), bounds_store)
            }
            syn::Fields::Unit => Ok(None),
        }
    }

    fn from_fields_vec(
        fields: Vec<&syn::Field>,
        bounds_store: &mut TypeParamBoundStore,
    ) -> syn::Result<Option<Self>> {
        for (i, field) in fields.iter().enumerate() {
            for attr in &field.attrs {
                if attr.path().is_ident("source_code") {
                    let is_option = extract_option(&field.ty);

                    let code_ty = is_option.unwrap_or(&field.ty);
                    bounds_store
                        .add_where_predicate(syn::parse_quote!(#code_ty: ::miette::SourceCode));

                    let source_code = if let Some(ident) = field.ident.clone() {
                        syn::Member::Named(ident)
                    } else {
                        syn::Member::Unnamed(syn::Index {
                            index: i as u32,
                            span: field.span(),
                        })
                    };
                    return Ok(Some(SourceCode {
                        source_code,
                        is_option: is_option.is_some(),
                    }));
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn gen_struct(&self, fields: &syn::Fields) -> Option<TokenStream> {
        let (display_pat, _display_members) = display_pat_members(fields);
        let src = &self.source_code;
        let ret = if self.is_option {
            quote! {
                self.#src.as_ref().map(|s| s as _)
            }
        } else {
            quote! {
                Some(&self.#src)
            }
        };

        Some(quote! {
            #[allow(unused_variables)]
            fn source_code(&self) -> std::option::Option<&dyn miette::SourceCode> {
                let Self #display_pat = self;
                #ret
            }
        })
    }

    pub(crate) fn gen_enum(variants: &[DiagnosticDef]) -> Option<TokenStream> {
        gen_all_variants_with(
            variants,
            WhichFn::SourceCode,
            |ident, fields, DiagnosticConcreteArgs { source_code, .. }| {
                let (display_pat, _display_members) = display_pat_members(fields);
                source_code.as_ref().and_then(|source_code| {
                    let field = match &source_code.source_code {
                        syn::Member::Named(ident) => ident.clone(),
                        syn::Member::Unnamed(syn::Index { index, .. }) => {
                            format_ident!("_{}", index)
                        }
                    };
                    let variant_name = ident.clone();
                    let ret = if source_code.is_option {
                        quote! {
                            #field.as_ref().map(|s| s as _)
                        }
                    } else {
                        quote! {
                            std::option::Option::Some(#field)
                        }
                    };
                    match &fields {
                        syn::Fields::Unit => None,
                        _ => Some(quote! {
                            Self::#variant_name #display_pat => #ret,
                        }),
                    }
                })
            },
        )
    }
}
