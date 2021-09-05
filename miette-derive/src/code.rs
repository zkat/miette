use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Token,
};

use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef, DiagnosticDefArgs},
    utils::forward_to_single_field_variant,
};

#[derive(Debug)]
pub struct Code(pub String);

impl Parse for Code {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        if ident == "code" {
            let la = input.lookahead1();
            if la.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                let la = content.lookahead1();
                if la.peek(syn::LitStr) {
                    let str = content.parse::<syn::LitStr>()?;
                    Ok(Code(str.value()))
                } else {
                    let path = content.parse::<syn::Path>()?;
                    Ok(Code(
                        path.segments
                            .iter()
                            .map(|s| s.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::"),
                    ))
                }
            } else {
                input.parse::<Token![=]>()?;
                Ok(Code(input.parse::<syn::LitStr>()?.value()))
            }
        } else {
            Err(syn::Error::new(ident.span(), "diagnostic code is required. Use #[diagnostic(code = ...)] or #[diagnostic(code(...))] to define one."))
        }
    }
}

impl Code {
    pub(crate) fn gen_enum(variants: &[DiagnosticDef]) -> Option<TokenStream> {
        let code_pairs = variants.iter().map(
            |DiagnosticDef {
                 ident,
                 fields,
                 args,
             }| {
                match args {
                    DiagnosticDefArgs::Transparent => {
                        Some(forward_to_single_field_variant(ident, fields, quote! { code() }))
                    }
                    DiagnosticDefArgs::Concrete(DiagnosticConcreteArgs { code, .. }) => {
                        let code = &code.as_ref()?.0;
                        Some(match fields {
                            syn::Fields::Named(_) => {
                                quote! { Self::#ident { .. } => std::option::Option::Some(std::boxed::Box::new(#code)), }
                            }
                            syn::Fields::Unnamed(_) => {
                                quote! { Self::#ident(..) => std::option::Option::Some(std::boxed::Box::new(#code)), }
                            }
                            syn::Fields::Unit => {
                                quote! { Self::#ident => std::option::Option::Some(std::boxed::Box::new(#code)), }
                            }
                        })
                    }
                }
            },
        );
        Some(quote! {
            fn code<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                match self {
                    #(#code_pairs)*
                    _ => std::option::Option::None,
                }
            }
        })
    }

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        let code = &self.0;
        Some(quote! {
            fn code<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                std::option::Option::Some(std::boxed::Box::new(#code))
            }
        })
    }
}
