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

pub enum Url {
    Display(Display),
    DocsRs,
}

impl Parse for Url {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        if ident == "url" {
            let la = input.lookahead1();
            if la.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                if content.peek(syn::LitStr) {
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
                    Ok(Url::Display(display))
                } else {
                    let option = content.parse::<syn::Ident>()?;
                    if option == "docsrs" {
                        Ok(Url::DocsRs)
                    } else {
                        Err(syn::Error::new(option.span(), "Invalid argument to url() sub-attribute. It must be either a string or a plain `docsrs` identifier"))
                    }
                }
            } else {
                input.parse::<Token![=]>()?;
                Ok(Url::Display(Display {
                    fmt: input.parse()?,
                    args: TokenStream::new(),
                    has_bonus_display: false,
                }))
            }
        } else {
            Err(syn::Error::new(ident.span(), "not a url"))
        }
    }
}

impl Url {
    pub(crate) fn gen_enum(
        enum_name: &syn::Ident,
        variants: &[DiagnosticDef],
    ) -> Option<TokenStream> {
        let url_pairs = variants.iter().map(|variant| {
            let DiagnosticDef { ident, fields, args: def_args } = variant;
            match def_args {
                DiagnosticDefArgs::Transparent(forward) => {
                    Some(forward.gen_enum_match_arm(ident, WhichFn::Url))
                }
                DiagnosticDefArgs::Concrete(DiagnosticConcreteArgs { ref url, .. }) => {
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
                    let (fmt, args) = match url.as_ref()? {
                        // fall through to `_ => None` below
                        Url::Display(display) => {
                            let mut display = display.clone();
                            display.expand_shorthand(&members);
                            let Display { fmt, args, .. } = display;
                            (fmt.value(), args)
                        }
                        Url::DocsRs => {
                            let fmt = "https://docs.rs/{crate_name}/{crate_version}/{crate_name}/{item_path}".into();
                            let item_path = format!("enum.{}.html#variant.{}", enum_name, ident);
                            let args = quote! {
                                crate_name=env!("CARGO_PKG_NAME"),
                                crate_version=env!("CARGO_PKG_VERSION"),
                                item_path=#item_path
                            };
                            (fmt, args)
                        }
                    };
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
         })
        .flatten()
        .collect::<Vec<_>>();
        if url_pairs.is_empty() {
            None
        } else {
            Some(quote! {
                fn url<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                    #[allow(unused_variables, deprecated)]
                    match self {
                        #(#url_pairs)*
                        _ => None,
                    }
                }
            })
        }
    }

    pub(crate) fn gen_struct(
        &self,
        struct_name: &syn::Ident,
        fields: &Fields,
    ) -> Option<TokenStream> {
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
        let (fmt, args) = match self {
            Url::Display(display) => {
                let mut display = display.clone();
                display.expand_shorthand(&members);
                let Display { fmt, args, .. } = display;
                (fmt.value(), args)
            }
            Url::DocsRs => {
                let fmt =
                    "https://docs.rs/{crate_name}/{crate_version}/{crate_name}/{item_path}".into();
                let item_path = format!("struct.{}.html", struct_name);
                let args = quote! {
                    crate_name=env!("CARGO_PKG_NAME"),
                    crate_version=env!("CARGO_PKG_VERSION"),
                    item_path=#item_path
                };
                (fmt, args)
            }
        };
        let members = members.iter();
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
            fn url<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                #[allow(unused_variables, deprecated)]
                #fields_pat
                std::option::Option::Some(std::boxed::Box::new(format!(#fmt, #args)))
            }
        })
    }
}
