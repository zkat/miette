use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Token,
};

use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef},
    fmt::{self, Display},
    forward::WhichFn,
    utils::gen_all_variants_with,
};

pub struct Snippets(Vec<Snippet>);

struct Snippet {
    message: Option<Display>,
    highlights: Vec<Highlight>,
    source: syn::Member,
    snippet: syn::Member,
}

struct Highlight {
    label: Option<Display>,
    highlight: syn::Member,
}

struct SnippetAttr {
    source: syn::Member,
    message: Option<Display>,
}

struct HighlightAttr {
    label: Option<Display>,
    snippet: syn::Member,
}

impl Parse for SnippetAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let source = input.parse::<syn::Member>()?;
        let message = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let ident = input.parse::<syn::Ident>()?;
            if ident == "message" {
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
                        Some(display)
                    } else {
                        return Err(syn::Error::new(ident.span(), "Invalid argument to message() sub-attribute. The first argument must be a literal string."));
                    }
                } else {
                    input.parse::<Token![=]>()?;
                    Some(Display {
                        fmt: input.parse()?,
                        args: TokenStream::new(),
                        has_bonus_display: false,
                    })
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "Invalid sub-attribute. Only `message()` is allowed.",
                ));
            }
        } else {
            None
        };
        Ok(SnippetAttr { source, message })
    }
}

impl Parse for HighlightAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let snippet = input.parse::<syn::Member>()?;
        let label = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let ident = input.parse::<syn::Ident>()?;
            if ident == "label" {
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
                        Some(display)
                    } else {
                        return Err(syn::Error::new(ident.span(), "Invalid argument to label() sub-attribute. The first argument must be a literal string."));
                    }
                } else {
                    input.parse::<Token![=]>()?;
                    Some(Display {
                        fmt: input.parse()?,
                        args: TokenStream::new(),
                        has_bonus_display: false,
                    })
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "Invalid sub-attribute. Only `label()` is allowed.",
                ));
            }
        } else {
            None
        };
        Ok(HighlightAttr { snippet, label })
    }
}

impl Snippets {
    pub fn from_fields(fields: &syn::Fields) -> syn::Result<Option<Self>> {
        match fields {
            syn::Fields::Named(named) => Self::from_fields_vec(named.named.iter().collect()),
            syn::Fields::Unnamed(unnamed) => {
                Self::from_fields_vec(unnamed.unnamed.iter().collect())
            }
            syn::Fields::Unit => Ok(None),
        }
    }

    fn from_fields_vec(fields: Vec<&syn::Field>) -> syn::Result<Option<Self>> {
        let mut snippets = HashMap::new();
        // First we collect all the contexts
        for (i, field) in fields.iter().enumerate() {
            for attr in &field.attrs {
                if attr.path.is_ident("snippet") {
                    let snippet = if let Some(ident) = field.ident.clone() {
                        syn::Member::Named(ident)
                    } else {
                        syn::Member::Unnamed(syn::Index {
                            index: i as u32,
                            span: field.span(),
                        })
                    };
                    let SnippetAttr { source, message } = attr.parse_args::<SnippetAttr>()?;
                    // TODO: useful error when source refers to a field that doesn't exist.
                    snippets.insert(
                        snippet.clone(),
                        Snippet {
                            message,
                            highlights: Vec::new(),
                            source,
                            snippet,
                        },
                    );
                }
            }
        }
        // Then we loop again looking for highlights
        for (i, field) in fields.iter().enumerate() {
            for attr in &field.attrs {
                if attr.path.is_ident("highlight") {
                    let HighlightAttr { snippet, label } = attr.parse_args::<HighlightAttr>()?;
                    if let Some(snippet) = snippets.get_mut(&snippet) {
                        let member = if let Some(ident) = field.ident.clone() {
                            syn::Member::Named(ident)
                        } else {
                            syn::Member::Unnamed(syn::Index {
                                index: i as u32,
                                span: field.span(),
                            })
                        };
                        snippet.highlights.push(Highlight {
                            highlight: member,
                            label,
                        });
                    } else {
                        return Err(syn::Error::new(snippet.span(), "Highlight must refer to an existing field with a #[snippet(...)] attribute."));
                    }
                }
            }
        }
        if snippets.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Snippets(snippets.into_values().collect())))
        }
    }

    pub(crate) fn gen_struct(&self, fields: &syn::Fields) -> Option<TokenStream> {
        let snippets = self.0.iter().map(|snippet| {
            // snippet message
            let msg = if let Some(display) = &snippet.message {
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
                let mut display = display.clone();
                display.expand_shorthand(&members);
                let Display { fmt, args, .. } = display;
                quote! {
                    message: std::option::Option::Some(format!(#fmt, #args)),
                }
            } else {
                quote! {
                    message: std::option::Option::None,
                }
            };

            // Source field
            let src_ident = &snippet.source;
            let src_ident = quote! {
                source: &self.#src_ident,
            };

            // Context
            let context = &snippet.snippet;
            let context = quote! {
                context: self.#context.clone().into(),
            };

            // Highlights
            let highlights = snippet.highlights.iter().map(|highlight| {
                let Highlight { highlight, label } = highlight;
                if let Some(Display { fmt, args, .. }) = label {
                    quote! {
                        (
                            std::option::Option::Some(
                                format!(#fmt, #args)
                            ),
                            self.#highlight.clone().into()
                        )
                    }
                } else {
                    quote! {
                        (std::option::Option::None, self.#highlight.clone().into())
                    }
                }
            });
            let highlights = quote! {
                highlights: std::option::Option::Some(vec![
                    #(#highlights),*
                ]),
            };

            // Generate the snippet itself
            quote! {
                miette::DiagnosticSnippet {
                    #msg
                    #src_ident
                    #context
                    #highlights
                }
            }
        });
        Some(quote! {
            #[allow(unused_variables)]
            fn snippets(&self) -> std::option::Option<std::boxed::Box<dyn std::iter::Iterator<Item = miette::DiagnosticSnippet> + '_>> {
                Some(Box::new(vec![
                    #(#snippets),*
                ].into_iter()))
            }
        })
    }

    pub(crate) fn gen_enum(variants: &[DiagnosticDef]) -> Option<TokenStream> {
        gen_all_variants_with(
            variants,
            WhichFn::Snippets,
            |ident, fields, DiagnosticConcreteArgs { snippets, .. }| {
                snippets.as_ref().and_then(|snippets| {
                        let variant_snippets = snippets.0.iter().map(|snippet| {
                            // snippet message
                            let msg = if let Some(display) = &snippet.message {
                            let members: HashSet<syn::Member> = fields.iter().enumerate().map(|(i, field)| {
                                if let Some(ident) = field.ident.as_ref().cloned() {
                                    syn::Member::Named(ident)
                                } else {
                                    syn::Member::Unnamed(syn::Index { index: i as u32, span: field.span() })
                                }
                            }).collect();
                                let mut display = display.clone();
                                display.expand_shorthand(&members);
                                let Display { fmt, args, .. } = display;
                                quote! {
                                    message: std::option::Option::Some(format!(#fmt, #args)),
                                }
                            } else {
                                quote! {
                                    message: std::option::Option::None,
                                }
                            };
                            // Source field
                            let src_ident = match &snippet.source {
                                syn::Member::Named(id) => id.clone(),
                                syn::Member::Unnamed(syn::Index { index, .. }) => {
                                    format_ident!("_{}", index)
                                }
                            };
                            let src_ident = quote! {
                                // TODO: I don't like this. Think about it more and maybe improve protocol?
                                source: #src_ident,
                            };

                            // Context
                            let context = match &snippet.snippet {
                                syn::Member::Named(id) => id.clone(),
                                syn::Member::Unnamed(syn::Index { index, .. }) => {
                                    format_ident!("_{}", index)
                                }
                            };
                            let context = quote! {
                                context: #context.clone().into(),
                            };

                            // Highlights
                            let highlights = snippet.highlights.iter().map(|highlight| {
                                let Highlight { highlight, label } = highlight;
                                let m = match highlight {
                                    syn::Member::Named(id) => id.clone(),
                                    syn::Member::Unnamed(syn::Index { index, .. }) => {
                                        format_ident!("_{}", index)
                                    }
                                };
                                if let Some(Display { fmt, args, ..}) = label {
                                    quote! {
                                        (
                                            std::option::Option::Some(format!(#fmt, #args)),
                                            #m.clone().into()
                                        )
                                    }
                                } else {
                                    quote! {
                                        (std::option::Option::None, #m.clone().into())
                                    }
                                }
                            });
                            let highlights = quote! {
                                highlights: std::option::Option::Some(vec![
                                    #(#highlights),*
                                ]),
                            };

                            // Generate the snippet itself
                            quote! {
                                miette::DiagnosticSnippet {
                                    #msg
                                    #src_ident
                                    #context
                                    #highlights
                                }
                            }
                        });
                        let variant_name = ident.clone();
                        let members = fields.iter().enumerate().map(|(i, field)| {
                            field
                                .ident
                                .as_ref()
                                .cloned()
                                .unwrap_or_else(|| format_ident!("_{}", i))
                        });
                        match &fields {
                            syn::Fields::Unit => None,
                            syn::Fields::Named(_) => Some(quote! {
                                Self::#variant_name { #(#members),* } => std::option::Option::Some(std::boxed::Box::new(vec![
                                    #(#variant_snippets),*
                                ].into_iter())),
                            }),
                            syn::Fields::Unnamed(_) => Some(quote! {
                                Self::#variant_name(#(#members),*) => std::option::Option::Some(Box::new(vec![
                                    #(#variant_snippets),*
                                ].into_iter())),
                            }),
                        }
                    })
            },
        )
    }
}
