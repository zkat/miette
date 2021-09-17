use proc_macro2::TokenStream;
use quote::quote;
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
                Some(quote! {
                    Self::#ident #fields => {
                        let mut __relateds = std::alloc::Vec::new();
                        std::option::Option::Some(std::boxed::Box(
                            __relateds.iter().map(|x| -> dyn Diagnostic { &*x })
                        ))
                    }
                })
            },
        )
    }

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        Some(quote! {
            fn related<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::iter::Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
                None
            }
        })
    }
}
