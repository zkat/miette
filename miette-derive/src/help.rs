use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Token,
};

use crate::diagnostic::{Diagnostic, DiagnosticVariant};

pub struct Help {
    pub fmt: String,
    pub args: Vec<syn::Expr>,
}

impl Parse for Help {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        if ident == "help" {
            let la = input.lookahead1();
            if la.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                let str = content.parse::<syn::LitStr>()?;
                Ok(Help {
                    fmt: str.value(),
                    args: Vec::new(),
                })
            } else {
                input.parse::<Token![=]>()?;
                Ok(Help {
                    fmt: input.parse::<syn::LitStr>()?.value(),
                    args: Vec::new(),
                })
            }
        } else {
            Err(syn::Error::new(ident.span(), "not a help"))
        }
    }
}
impl Help {
    pub(crate) fn gen_enum(
        _diag: &Diagnostic,
        variants: &[DiagnosticVariant],
    ) -> Option<TokenStream> {
        let help_pairs = variants
            .iter()
            .filter(|v| v.help.is_some())
            .map(
                |DiagnosticVariant {
                     ref ident,
                     ref help,
                     ref fields,
                     ..
                 }| {
                     let help = &help.as_ref().unwrap();
                     let fmt = &help.fmt;
                     let args = help.args.iter().map(|arg| quote! { #arg, });
                    match fields {
                        syn::Fields::Named(_) => {
                            quote! { Self::#ident{..} => std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #(#args),*))), }
                        }
                        syn::Fields::Unnamed(_) => {
                            quote! { Self::#ident(..) => std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #(#args),*))), }
                        }
                        syn::Fields::Unit =>
                            quote! { Self::#ident => std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #(#args),*))), },
                    }
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

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        let fmt = &self.fmt;
        let args = &self.args;
        Some(quote! {
            fn help<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #(#args),*)))
            }
        })
    }
}
