use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Token,
};

use crate::{diagnostic::{DiagnosticConcreteArgs, DiagnosticDef, DiagnosticDefArgs}, forward::WhichFn};

pub struct Severity(pub syn::Ident);

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
                    let sev = get_severity(&str.value(), str.span())?;
                    Ok(Severity(syn::Ident::new(&sev, str.span())))
                } else {
                    let ident = content.parse::<syn::Ident>()?;
                    let sev = get_severity(&ident.to_string(), ident.span())?;
                    Ok(Severity(syn::Ident::new(&sev, ident.span())))
                }
            } else {
                input.parse::<Token![=]>()?;
                let str = input.parse::<syn::LitStr>()?;
                let sev = get_severity(&str.value(), str.span())?;
                Ok(Severity(syn::Ident::new(&sev, str.span())))
            }
        } else {
            Err(syn::Error::new(
                ident.span(),
                "MIETTE BUG: not a severity option",
            ))
        }
    }
}

fn get_severity(input: &str, span: Span) -> syn::Result<String> {
    match input.to_lowercase().as_ref() {
        "error" | "err" => Ok("Error".into()),
        "warning" | "warn" => Ok("Warning".into()),
        "advice" | "adv" | "info" => Ok("Advice".into()),
        _ => Err(syn::Error::new(
            span,
            "Invalid severity level. Only Error, Warning, and Advice are supported.",
        )),
    }
}

impl Severity {
    pub(crate) fn gen_enum(variants: &[DiagnosticDef]) -> Option<TokenStream> {
        let sev_pairs = variants
            .iter()
            .map(
                |DiagnosticDef {
                     ident, fields, args
                 }| {
                     match args {
                         DiagnosticDefArgs::Transparent(forward) => {
                             Some(forward.gen_enum_match_arm(ident, WhichFn::Severity))
                         }
                         DiagnosticDefArgs::Concrete(DiagnosticConcreteArgs { severity, .. }) => {
                             let severity = &severity.as_ref()?.0;
                             let fields = match fields {
                                 syn::Fields::Named(_) => quote! { { .. } },
                                 syn::Fields::Unnamed(_) => quote! { (..) },
                                 syn::Fields::Unit => quote!{},
                             };
                             Some(quote! { Self::#ident #fields => std::option::Option::Some(miette::Severity::#severity), })
                         }
                     }
                },
            )
            .flatten()
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
