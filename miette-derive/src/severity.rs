use darling::{ast::Fields, error::Error as DarlingError, FromMeta};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Lit, LitStr, Meta, NestedMeta, Path};

use crate::{Diagnostic, DiagnosticField, DiagnosticVariant};

#[derive(Debug)]
pub struct Severity(pub Path);

impl FromMeta for Severity {
    fn from_string(arg: &str) -> Result<Self, DarlingError> {
        Ok(Severity(LitStr::new(arg, Span::call_site()).parse()?))
    }

    fn from_list(items: &[NestedMeta]) -> Result<Self, DarlingError> {
        match &items[0] {
            NestedMeta::Meta(Meta::Path(p)) => Ok(Severity(p.clone())),
            NestedMeta::Lit(Lit::Str(sev)) => Ok(Severity(sev.parse()?)),
            _ => Err(DarlingError::custom(
                "invalid severity format. Only literal names and string literals are accepted",
            )),
        }
    }
}

impl Severity {
    pub(crate) fn gen_enum(
        _diag: &Diagnostic,
        variants: &[&DiagnosticVariant],
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

    pub(crate) fn gen_struct(diag: &Diagnostic, _fields: &Fields<&DiagnosticField>) -> Option<TokenStream> {
        diag.severity.as_ref().map(|sev| {
            let sev = &sev.0;
            quote! {
                fn severity(&self) -> std::option::Option<miette::Severity> {
                    Some(miette::Severity::#sev)
                }
            }
        })
    }
}
