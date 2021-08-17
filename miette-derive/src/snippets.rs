use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Token,
};

use crate::diagnostic::DiagnosticVariant;

pub struct Snippets(Vec<Snippet>);

struct Snippet {
    message: Option<MemberOrString>,
    highlights: Vec<Highlight>,
    source: syn::Member,
    snippet: syn::Member,
}

struct Highlight {
    highlight: syn::Member,
}

struct SnippetAttr {
    source: syn::Member,
    message: Option<MemberOrString>,
}

struct HighlightAttr {
    snippet: syn::Member,
}

enum MemberOrString {
    Member(syn::Member),
    String(syn::LitStr),
}

impl ToTokens for MemberOrString {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use MemberOrString::*;
        match self {
            Member(member) => member.to_tokens(tokens),
            String(string) => string.to_tokens(tokens),
        }
    }
}

impl Parse for MemberOrString {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) || lookahead.peek(syn::LitInt) {
            Ok(MemberOrString::Member(input.parse()?))
        } else if lookahead.peek(syn::LitStr) {
            Ok(MemberOrString::String(input.parse()?))
        } else {
            Err(syn::Error::new(
                input.span(),
                "Expected a string or a field reference.",
            ))
        }
    }
}

impl Parse for SnippetAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punc = Punctuated::<MemberOrString, Token![,]>::parse_terminated(input)?;
        let span = input.span();
        let mut iter = punc.into_iter();
        let source = match iter.next() {
            Some(MemberOrString::Member(member)) => member,
            _ => {
                return Err(syn::Error::new(
                    span,
                    "Source must be an identifier that refers to a Source for this snippet.",
                ))
            }
        };
        let message = iter.next();
        Ok(SnippetAttr { source, message })
    }
}

impl Parse for HighlightAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punc = Punctuated::<MemberOrString, Token![,]>::parse_terminated(input)?;
        let span = input.span();
        let mut iter = punc.into_iter();
        let snippet =
            match iter.next() {
                Some(MemberOrString::Member(member)) => member,
                _ => return Err(syn::Error::new(
                    span,
                    "must be an identifier that refers to something with a #[snippet] attribute.",
                )),
            };
        Ok(HighlightAttr { snippet })
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
                    let HighlightAttr { snippet } = attr.parse_args::<HighlightAttr>()?;
                    if let Some(snippet) = snippets.get_mut(&snippet) {
                        let member = if let Some(ident) = field.ident.clone() {
                            syn::Member::Named(ident)
                        } else {
                            syn::Member::Unnamed(syn::Index {
                                index: i as u32,
                                span: field.span(),
                            })
                        };
                        snippet.highlights.push(Highlight { highlight: member });
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

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        let snippets = self.0.iter().map(|snippet| {
            // snippet message
            let msg = snippet
                .message
                .as_ref()
                .map(|msg| match msg {
                    MemberOrString::String(str) => {
                        quote! {
                            message: std::option::Option::Some(#str),
                        }
                    }
                    MemberOrString::Member(m) => {
                        quote! {
                            message: std::option::Option::Some(self.#m.as_ref()),
                        }
                    }
                })
                .unwrap_or_else(|| {
                    quote! {
                        message: std::option::Option::None,
                    }
                });

            // Source field
            let src_ident = &snippet.source;
            let src_ident = quote! {
                source: &self.#src_ident,
            };

            // Context
            let context = &snippet.snippet;
            let context = quote! {
                context: &self.#context,
            };

            // Highlights
            let highlights = snippet.highlights.iter().map(|highlight| {
                let Highlight { highlight } = highlight;
                quote! {
                    &self.#highlight
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

    pub(crate) fn gen_enum(variants: &[DiagnosticVariant]) -> Option<TokenStream> {
        let variant_arms = variants.iter().map(|variant| {
            variant.snippets.as_ref().map(|snippets| {
                let variant_snippets = snippets.0.iter().map(|snippet| {
                    // snippet message
                    let msg = snippet
                        .message
                        .as_ref()
                        .map(|msg| match msg {
                            MemberOrString::String(str) => {
                                quote! {
                                    message: std::option::Option::Some(#str),
                                }
                            }
                            MemberOrString::Member(m) => {
                                let m = match m {
                                    syn::Member::Named(id) => id.clone(),
                                    syn::Member::Unnamed(syn::Index { index, .. }) => {
                                        format_ident!("_{}", index)
                                    }
                                };
                                quote! {
                                    message: std::option::Option::Some(#m.as_ref()),
                                }
                            }
                        })
                        .unwrap_or_else(|| {
                            quote! {
                                message: std::option::Option::None,
                            }
                        });

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
                        context: #context,
                    };

                    // Highlights
                    let highlights = snippet.highlights.iter().map(|highlight| {
                        let Highlight { highlight } = highlight;
                        let m = match highlight {
                            syn::Member::Named(id) => id.clone(),
                            syn::Member::Unnamed(syn::Index { index, .. }) => {
                                format_ident!("_{}", index)
                            }
                        };
                        quote! {
                            #m
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
                let variant_name = variant.ident.clone();
                let members = variant.fields.iter().enumerate().map(|(i, field)| {
                    field
                        .ident
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| format_ident!("_{}", i))
                });
                match &variant.fields {
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
        });
        Some(quote! {
            #[allow(unused_variables)]
            fn snippets(&self) -> std::option::Option<std::boxed::Box<dyn std::iter::Iterator<Item = miette::DiagnosticSnippet> + '_>> {
                match self {
                    #(#variant_arms)*
                    _ => std::option::Option::None,
                }
            }
        })
    }
}
