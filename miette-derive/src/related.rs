use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef},
    forward::WhichFn,
    utils::{display_pat_members, gen_all_variants_with},
};

pub struct Related(Vec<syn::Member>);

impl Related {
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
        let mut members = Vec::new();
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

                    members.push(related);
                }
            }
        }

        if members.is_empty() {
            return Ok(None);
        }
        
        Ok(Some(Self(members)))
    }

    pub(crate) fn gen_enum(variants: &[DiagnosticDef]) -> Option<TokenStream> {
        gen_all_variants_with(
            variants,
            WhichFn::Related,
            |ident, fields, DiagnosticConcreteArgs { related, .. }| {
                let (display_pat, _display_members) = display_pat_members(fields);
                related.as_ref().map(|related| {
                    assert_ne!(related.0.len(), 0);

                    let mut rels = related.0.iter().map(|rel| {
                        match rel {
                            syn::Member::Named(ident) => ident.clone(),
                            syn::Member::Unnamed(syn::Index { index, .. }) => {
                                format_ident!("_{}", index)
                            }
                        }
                    });

                    let rel = rels.next().unwrap();

                    quote! {
                        Self::#ident #display_pat => {
                            std::option::Option::Some(std::boxed::Box::new(
                                #rel.iter().map(|x| -> &(dyn miette::Diagnostic) { &*x })
                                    #(.chain(#rels.iter().map(|x| -> &(dyn miette::Diagnostic) { &*x })))*
                            ))
                        }
                    }
                })
            },
        )
    }

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        assert_ne!(self.0.len(), 0);

        let rel = &self.0[0];
        let rels = &self.0[1..];
        
        Some(quote! {
            fn related<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::iter::Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
                use ::core::borrow::Borrow;
                std::option::Option::Some(std::boxed::Box::new(
                    self.#rel.iter().map(|x| -> &(dyn miette::Diagnostic) { &*x.borrow() })
                    #(.chain(self.#rels.iter().map(|x| -> &(dyn miette::Diagnostic) { &*x.borrow() })))*
                ))
            }
        })
    }
}
