use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Fields, Token,
};

use crate::{fmt::{self, Display}, forward::WhichFn};
use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef, DiagnosticDefArgs},
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
                    content.parse::<Token![,]>()?;
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
        let help_pairs = variants
            .iter()
            .map(
                |DiagnosticDef {
                     ident,
                     fields,
                     args,
                     ..
                 }| {
                     match args {
                         DiagnosticDefArgs::Transparent(forward) => {
                             Some(forward.gen_enum_match_arm(ident, WhichFn::Help))
                         }
                         DiagnosticDefArgs::Concrete(DiagnosticConcreteArgs { help, .. }) => {
                             let mut display = help.as_ref()?.display.clone();
                             let member_idents = fields.iter().enumerate().map(|(i, field)| {
                                 field
                                     .ident
                                     .as_ref()
                                     .cloned()
                                     .unwrap_or_else(|| format_ident!("_{}", i))
                             });
                             let members: HashSet<syn::Member> = fields.iter().enumerate().map(|(i, field)| {
                                 if let Some(ident) = field.ident.as_ref().cloned() {
                                     syn::Member::Named(ident)
                                 } else {
                                     syn::Member::Unnamed(syn::Index { index: i as u32, span: field.span() })
                                 }
                             }).collect();
                             display.expand_shorthand(&members);
                             let Display { fmt, args, .. } = display;
                             Some(match fields {
                                 syn::Fields::Named(_) => {
                                     quote! { Self::#ident{ #(#member_idents),* } => std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #args))), }
                                 }
                                 syn::Fields::Unnamed(_) => {
                                     quote! { Self::#ident( #(#member_idents),* ) => std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #args))), }
                                 }
                                 syn::Fields::Unit =>
                                     quote! { Self::#ident => std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #args))), },
                             })
                         }
                     }
                 },
            )
            .flatten()
            .collect::<Vec<_>>();
        if help_pairs.is_empty() {
            None
        } else {
            Some(quote! {
                fn help<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                    #[allow(unused_variables, deprecated)]
                    match self {
                        #(#help_pairs)*
                        _ => None,
                    }
                }
            })
        }
    }

    pub(crate) fn gen_struct(&self, fields: &Fields) -> Option<TokenStream> {
        let mut display = self.display.clone();
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
        let members = members.iter();
        let Display { fmt, args, .. } = display;
        let fields_pat = match fields {
            Fields::Named(_) => quote! {
                let Self { #(#members),* } = self;
            },
            Fields::Unnamed(_) => {
                let vars = members.map(|member| {
                    if let syn::Member::Unnamed(member) = member {
                        format_ident!("_{}", member)
                    } else {
                        unreachable!()
                    }
                });
                quote! {
                    let Self(#(#vars),*) = self;
                }
            }
            Fields::Unit => quote! {},
        };
        Some(quote! {
            fn help<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                #[allow(unused_variables, deprecated)]
                #fields_pat
                std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #args)))
            }
        })
    }
}
