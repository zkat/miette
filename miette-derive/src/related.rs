use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef},
    forward::WhichFn,
    utils::{display_pat_members, gen_all_variants_with},
};

pub struct Relateds(Vec<Related>);

pub struct Related(syn::Member);

impl Relateds {
    pub(crate) fn from_fields(fields: &syn::Fields) -> syn::Result<Option<Self>> {
        match fields {
            syn::Fields::Named(named) => Self::from_fields_vec(named.named.iter().collect()),
            syn::Fields::Unnamed(unnamed) => {
                Self::from_fields_vec(unnamed.unnamed.iter().collect())
            }
            syn::Fields::Unit => Ok(None),
        }
    }

    fn from_fields_vec(fields: Vec<&syn::Field>) -> syn::Result<Option<Self>> {
        let mut relateds = Vec::new();
        for (i, field) in fields.iter().enumerate() {
            for attr in &field.attrs {
                if attr.path.is_ident("related") {
                    let related = if let Some(ident) = field.ident.clone() {
                        syn::Member::Named(ident)
                    } else {
                        syn::Member::Unnamed(syn::Index {
                            index: i as u32,
                            span: field.span(),
                        })
                    };
                    relateds.push(Related(related));
                }
            }
        }
        if relateds.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Relateds(relateds)))
        }
    }

    pub(crate) fn gen_enum(variants: &[DiagnosticDef]) -> Option<TokenStream> {
        gen_all_variants_with(
            variants,
            WhichFn::Related,
            |ident, fields, DiagnosticConcreteArgs { related, .. }| {
                let (display_pat, _display_members) = display_pat_members(fields);
                related.as_ref().map(|relateds| {
                    let relateds = relateds.0.iter().map(|related| {
                        let rel = match &related.0 {
                            syn::Member::Named(ident) => ident.clone(),
                            syn::Member::Unnamed(syn::Index { index, .. }) => {
                                format_ident!("_{}", index)
                            }
                        };
                        quote! { #rel }
                    });
                    quote! {
                        Self::#ident #display_pat => {
                            std::option::Option::Some(std::boxed::Box::new(
                                vec![
                                    #(#relateds),*
                                ].into_iter().map(|x| -> &dyn Diagnostic { &*x as &Diagnostic })
                            ))
                        }
                    }
                })
            },
        )
    }

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        if self.0.is_empty() {
            None
        } else {
            let relateds = self.0.iter().map(|related| {
                let rel = &related.0;
                quote! { &self.#rel }
            });
            Some(quote! {
                fn related<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::iter::Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
                    std::option::Option::Some(std::boxed::Box::new(
                        vec![
                            #(#relateds),*
                        ].into_iter().map(|x| -> &dyn Diagnostic { &*x })
                    ))
                }
            })
        }
    }
}
