use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Token,
};

use crate::diagnostic::{Diagnostic, DiagnosticVariant};

pub struct Severity(pub syn::Path);

impl Parse for Severity {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        if ident == "severity" {
            let la = input.lookahead1();
            if la.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                let la = content.lookahead1();
                if la.peek(syn::LitStr) {
                    let str = content.parse::<syn::LitStr>()?;
                    Ok(Severity(str.parse()?))
                } else {
                    let path = content.parse::<syn::Path>()?;
                    Ok(Severity(path))
                }
            } else {
                input.parse::<Token![=]>()?;
                Ok(Severity(input.parse::<syn::LitStr>()?.parse()?))
            }
        } else {
            Err(syn::Error::new(
                ident.span(),
                "not a severity level.",
            ))
        }
    }
}
impl Severity {
    pub(crate) fn gen_enum(
        _diag: &Diagnostic,
        variants: &[DiagnosticVariant],
    ) -> Option<TokenStream> {
        let sev_pairs = variants
            .iter()
            .filter(|v| v.severity.is_some())
            .map(
                |DiagnosticVariant {
                     ident, severity, ..
                 }| {
                     let severity = &severity.as_ref().unwrap().0;
                     quote! { Self::#ident => std::option::Option::Some(miette::Severity::#severity), }
                },
            )
            .collect::<Vec<_>>();
        if sev_pairs.is_empty() {
            None
        } else {
            Some(quote! {
               fn severity(&self) -> std::option::Option<miette::Severity> {
                   match self {
                        #(#sev_pairs)*
                        _ => None,
                   }
               }
            })
        }
    }

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        let sev = &self.0;
        Some(quote! {
            fn severity(&self) -> std::option::Option<miette::Severity> {
                Some(miette::Severity::#sev)
            }
        })
    }
}
