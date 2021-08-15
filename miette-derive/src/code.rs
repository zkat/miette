use std::fmt::Display;

use darling::{ast::Fields, error::Error as DarlingError, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Lit, Meta, NestedMeta};

use crate::{Diagnostic, DiagnosticField, DiagnosticVariant};

#[derive(Debug)]
pub struct Code(String);

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromMeta for Code {
    fn from_string(arg: &str) -> Result<Self, DarlingError> {
        Ok(Code(arg.into()))
    }

    fn from_list(items: &[NestedMeta]) -> Result<Self, DarlingError> {
        match &items[0] {
            NestedMeta::Meta(Meta::Path(p)) => Ok(Code(
                p.segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::"),
            )),
            NestedMeta::Lit(Lit::Str(code)) => Ok(Code(code.value())),
            _ => Err(DarlingError::custom(
                "invalid code format. Only path::style and string literals are accepted",
            )),
        }
    }
}

impl Code {
    pub(crate) fn gen_enum(
        _diag: &Diagnostic,
        variants: &[&DiagnosticVariant],
    ) -> Option<TokenStream> {
        let code_pairs = variants.iter().map(
            |DiagnosticVariant {
                 ref ident,
                 ref code,
                 ..
             }| {
                let code = code.to_string();
                quote! { Self::#ident => std::boxed::Box::new(#code), }
            },
        );
        Some(quote! {
            fn code<'a>(&'a self) -> std::boxed::Box<dyn std::fmt::Display + 'a> {
                match self {
                    #(#code_pairs)*
                }
            }
        })
    }

    pub(crate) fn gen_struct(
        diag: &Diagnostic,
        _fields: &Fields<&DiagnosticField>,
    ) -> Option<TokenStream> {
        let code = diag
            .code
            .as_ref()
            .expect("`code` attribute is required for diagnostics.")
            .to_string();
        Some(quote! {
            fn code<'a>(&'a self) -> std::boxed::Box<dyn std::fmt::Display + 'a> {
                std::boxed::Box::new(#code)
            }
        })
    }
}
