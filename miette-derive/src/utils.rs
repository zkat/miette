use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

pub(crate) enum MemberOrString {
    Member(syn::Member),
    String(syn::LitStr),
}

impl ToTokens for MemberOrString {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use MemberOrString::*;
        match self {
            Member(member) => member.to_tokens(tokens),
            String(string) => string.to_tokens(tokens),
        }
    }
}

impl Parse for MemberOrString {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) || lookahead.peek(syn::LitInt) {
            Ok(MemberOrString::Member(input.parse()?))
        } else if lookahead.peek(syn::LitStr) {
            Ok(MemberOrString::String(input.parse()?))
        } else {
            Err(syn::Error::new(
                input.span(),
                "Expected a string or a field reference.",
            ))
        }
    }
}

use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef},
    forward::WhichFn,
};

pub(crate) fn gen_all_variants_with(
    variants: &[DiagnosticDef],
    which_fn: WhichFn,
    mut f: impl FnMut(&syn::Ident, &syn::Fields, &DiagnosticConcreteArgs) -> Option<TokenStream>,
) -> Option<TokenStream> {
    let pairs = variants
        .iter()
        .map(|def| {
            def.args
                .forward_or_override_enum(&def.ident, which_fn, |concrete| {
                    f(&def.ident, &def.fields, concrete)
                })
        })
        .flatten()
        .collect::<Vec<_>>();
    if pairs.is_empty() {
        return None;
    }
    let signature = which_fn.signature();
    let catchall = which_fn.catchall_arm();
    Some(quote! {
        #signature {
            #[allow(unused_variables, deprecated)]
            match self {
                #(#pairs)*
                #catchall
            }
        }
    })
}

use crate::fmt::Display;
use std::collections::HashSet;

pub(crate) fn gen_unused_pat(fields: &syn::Fields) -> TokenStream {
    match fields {
        syn::Fields::Named(_) => quote! { { .. } },
        syn::Fields::Unnamed(_) => quote! { ( .. ) },
        syn::Fields::Unit => quote! {},
    }
}

pub(crate) fn gen_display_fields_pat(display: &mut Display, fields: &syn::Fields) -> TokenStream {
    let members: HashSet<syn::Member> = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            if let Some(ident) = field.ident.as_ref().cloned() {
                syn::Member::Named(ident)
            } else {
                syn::Member::Unnamed(syn::Index {
                    index: i as u32,
                    span: field.span(),
                })
            }
        })
        .collect();

    display.expand_shorthand(&members);

    let member_idents = fields.iter().enumerate().map(|(i, field)| {
        field
            .ident
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format_ident!("_{}", i))
    });
    match fields {
        syn::Fields::Named(_) => quote! {
            { #(#member_idents),* }
        },
        syn::Fields::Unnamed(_) => quote! {
            ( #(#member_idents),* )
        },
        syn::Fields::Unit => quote! {},
    }
}
