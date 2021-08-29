use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Fields, Token,
};

use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef},
    utils::{gen_all_variants_with, gen_display_fields_pat},
};
use crate::{
    fmt::{self, Display},
    forward::WhichFn,
};

pub struct Help {
    pub display: Display,
}

impl Parse for Help {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        if ident == "help" {
            let la = input.lookahead1();
            if la.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                let fmt = content.parse()?;
                let args = if content.is_empty() {
                    TokenStream::new()
                } else {
                    fmt::parse_token_expr(&content, false)?
                };
                let display = Display {
                    fmt,
                    args,
                    has_bonus_display: false,
                };
                Ok(Help { display })
            } else {
                input.parse::<Token![=]>()?;
                Ok(Help {
                    display: Display {
                        fmt: input.parse()?,
                        args: TokenStream::new(),
                        has_bonus_display: false,
                    },
                })
            }
        } else {
            Err(syn::Error::new(ident.span(), "not a help"))
        }
    }
}

impl Help {
    pub(crate) fn gen_enum(variants: &[DiagnosticDef]) -> Option<TokenStream> {
        gen_all_variants_with(
            variants,
            WhichFn::Help,
            |ident, fields, DiagnosticConcreteArgs { help, .. }| {
                let mut display = help.as_ref()?.display.clone();
                let display_pat = gen_display_fields_pat(&mut display, fields);
                let Display { fmt, args, .. } = display;
                Some(quote! {
                    Self::#ident #display_pat => std::option::Option::Some(std::boxed::Box::new(format!(#fmt #args))),
                })
            },
        )
    }

    pub(crate) fn gen_struct(&self, fields: &Fields) -> Option<TokenStream> {
        let mut display = self.display.clone();
        let fields_pat = gen_display_fields_pat(&mut display, fields);
        let Display { fmt, args, .. } = display;
        Some(quote! {
            fn help<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                #[allow(unused_variables, deprecated)]
                let Self #fields_pat = self;
                std::option::Option::Some(std::boxed::Box::new(format!(#fmt #args)))
            }
        })
    }
}
