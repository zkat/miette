use darling::{ast::Fields, error::Error as DarlingError, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Lit, NestedMeta};

use crate::{Diagnostic, DiagnosticField, DiagnosticVariant};

#[derive(Debug)]
pub struct Help {
    pub fmt: String,
    pub args: Vec<NestedMeta>,
}

impl FromMeta for Help {
    fn from_string(arg: &str) -> Result<Help, DarlingError> {
        Ok(Help {
            fmt: arg.into(),
            args: Vec::new(),
        })
    }

    fn from_list(items: &[NestedMeta]) -> Result<Help, DarlingError> {
        match &items.get(0) {
            Some(NestedMeta::Lit(Lit::Str(fmt))) => Ok(Help {
                fmt: fmt.value(),
                args: items[1..]
                    .iter()
                    .map(|item| match item {
                        NestedMeta::Meta(_) => Err(DarlingError::custom(
                            "Only literals are supported for now. Sorry :("
                        )),
                        NestedMeta::Lit(_) => Ok(item.clone()),
                    })
                    .collect::<Result<Vec<_>, DarlingError>>()?,
            }),
            None => Err(DarlingError::custom("Help format string is required")),
            _ => Err(DarlingError::custom(
                "First argument must be a literal format string",
            )),
        }
    }
}

impl Help {
    pub(crate) fn gen_enum(
        _diag: &Diagnostic,
        variants: &[&DiagnosticVariant],
    ) -> Option<TokenStream> {
        let help_pairs = variants
            .iter()
            .filter(|v| v.help.is_some())
            .map(
                |DiagnosticVariant {
                     ref ident,
                     ref help,
                     ..
                 }| {
                     let help = &help.as_ref().unwrap();
                     let fmt = &help.fmt;
                     let args = help.args.iter().map(|arg| quote! { #arg, });
                     quote! { Self::#ident => std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #(#args),*))), }
                 },
            )
            .collect::<Vec<_>>();
        if help_pairs.is_empty() {
            None
        } else {
            Some(quote! {
                fn help<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                    match self {
                        #(#help_pairs)*
                        _ => None,
                    }
                }
            })
        }
    }

    pub(crate) fn gen_struct(
        diag: &Diagnostic,
        _fields: &Fields<&DiagnosticField>,
    ) -> Option<TokenStream> {
        diag.help.as_ref().map(|h| {
            let fmt = &h.fmt;
            let args = &h.args;
            quote! {
                fn help<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                    std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #(#args),*)))
                }
            }
        })
    }
}
