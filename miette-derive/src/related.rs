use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef},
    forward::WhichFn,
    trait_bounds::TypeParamBoundStore,
    utils::{display_pat_members, gen_all_variants_with},
};

pub struct Related(syn::Member);

impl Related {
    pub(crate) fn from_fields(
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
                if attr.path().is_ident("related") {
                    let related = if let Some(ident) = field.ident.clone() {
                        syn::Member::Named(ident)
                    } else {
                        syn::Member::Unnamed(syn::Index {
                            index: i as u32,
                            span: field.span(),
                        })
                    };
                    // this is somewhat hacky and only supports concrete types for the #[related] type
                    // ittself but supports generics for the arguments, i.e. Vec<T> where T is generic.
                    //
                    // I think that this is a current limitation of the design of the Diagnostic trait,
                    // since we'd need bounds on the method and we can't do that (to refer to the lifetime)
                    //
                    // Someone smarter than me might be able to figure out a better solution (?)
                    let ty = &field.ty;
                    bounds_store.add_where_predicate(syn::parse_quote!(
                        <#ty as ::std::iter::IntoIterator>::Item: ::miette::Diagnostic + 'static
                    ));
                    return Ok(Some(Related(related)));
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn gen_enum(variants: &[DiagnosticDef]) -> Option<TokenStream> {
        gen_all_variants_with(
            variants,
            WhichFn::Related,
            |ident, fields, DiagnosticConcreteArgs { related, .. }| {
                let (display_pat, _display_members) = display_pat_members(fields);
                related.as_ref().map(|related| {
                    let rel = match &related.0 {
                        syn::Member::Named(ident) => ident.clone(),
                        syn::Member::Unnamed(syn::Index { index, .. }) => {
                            format_ident!("_{}", index)
                        }
                    };
                    quote! {
                        Self::#ident #display_pat => {
                            std::option::Option::Some(std::boxed::Box::new(
                                #rel.iter().map(|x| -> &(dyn miette::Diagnostic) { &*x })
                            ))
                        }
                    }
                })
            },
        )
    }

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        let rel = &self.0;
        Some(quote! {
            fn related<'a>(&'a self) -> std::option::Option<
                std::boxed::Box<dyn std::iter::Iterator<Item = &'a dyn miette::Diagnostic> + 'a>
            > {
                use ::core::borrow::Borrow;
                std::option::Option::Some(std::boxed::Box::new(
                        self.#rel.iter().map(|x| -> &(dyn miette::Diagnostic) { &*x.borrow() })
                ))
            }
        })
    }
}
