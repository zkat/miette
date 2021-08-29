use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Token,
};

use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef, DiagnosticDefArgs},
    forward::WhichFn,
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
    pub(crate) fn gen_enum(variants: &[DiagnosticDef]) -> TokenStream {
        let code_pairs = variants
            .iter()
            .map(
                |DiagnosticDef {
                     ident,
                     fields,
                     args,
                 }| {
                    match args {
                        DiagnosticDefArgs::Transparent(forward) => {
                            forward.gen_enum_match_arm(ident, WhichFn::Code)
                        }
                        DiagnosticDefArgs::Concrete(DiagnosticConcreteArgs { code, .. }) => {
                            let code = &code.0;
                            match fields {
                                syn::Fields::Named(_) => {
                                    quote! { Self::#ident { .. } => std::boxed::Box::new(#code), }
                                }
                                syn::Fields::Unnamed(_) => {
                                    quote! { Self::#ident(..) => std::boxed::Box::new(#code), }
                                }
                                syn::Fields::Unit => {
                                    quote! { Self::#ident => std::boxed::Box::new(#code), }
                                }
                            }
                        }
                    }
                },
            );
        quote! {
            fn code<'a>(&'a self) -> std::boxed::Box<dyn std::fmt::Display + 'a> {
                match self {
                    #(#code_pairs)*
                }
            }
        }
    }

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        let code = &self.0;
        Some(quote! {
            fn code<'a>(&'a self) -> std::boxed::Box<dyn std::fmt::Display + 'a> {
                std::boxed::Box::new(#code)
            }
        })
    }
}
