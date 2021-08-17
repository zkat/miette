use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Token,
};

use crate::diagnostic::{Diagnostic, DiagnosticVariant};

pub struct Snippets(Vec<Snippet>);

struct Snippet {
    message: Option<String>,
    highlights: Vec<Highlight>,
    source_name: Option<String>,
    // TODO: These two should be special expressions a-la-thiserror. This won't work for enums either.
    source: syn::Ident,
    context: syn::Ident,
}

struct Highlight {
    highlight: syn::Ident,
    label: Option<String>,
}

struct SnippetAttr {
    source: syn::Ident,
    source_name: Option<String>,
    message: Option<String>,
}

impl Parse for SnippetAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punc = Punctuated::<syn::Expr, Token![,]>::parse_terminated(input)?;
        let span = punc.span();
        let mut iter = punc.into_iter();
        let source = if let Some(syn::Expr::Path(syn::ExprPath { path, .. })) = iter.next() {
            if let Some(ident) = path.get_ident() {
                ident.clone()
            } else {
                return Err(syn::Error::new(
                    span,
                    "Source must be an identifier that refers to a Source for this snippet.",
                ));
            }
        } else {
            return Err(syn::Error::new(
                span,
                "Source must be an identifier that refers to a Source for this snippet.",
            ));
        };
        let src_name = iter
            .next()
            .map(|m| {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(str),
                    ..
                }) = m
                {
                    Ok(str.value())
                } else {
                    Err(syn::Error::new(
                        m.span(),
                        "Only literal strings are supported as snippet source names.",
                    ))
                }
            })
            .transpose()?;

        let message = iter
            .next()
            .map(|m| {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(str),
                    ..
                }) = m
                {
                    Ok(str.value())
                } else {
                    Err(syn::Error::new(
                        m.span(),
                        "Only literal strings are supported as snippet context messages.",
                    ))
                }
            })
            .transpose()?;
        Ok(SnippetAttr {
            source,
            source_name: src_name,
            message,
        })
    }
}

struct HighlightAttr {
    context: syn::Ident,
    label: Option<String>,
}

impl Parse for HighlightAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punc = Punctuated::<syn::Expr, Token![,]>::parse_terminated(input)?;
        let span = punc.span();
        let mut iter = punc.into_iter();
        let context = if let Some(syn::Expr::Path(syn::ExprPath { path, .. })) = iter.next() {
            if let Some(ident) = path.get_ident() {
                ident.clone()
            } else {
                return Err(syn::Error::new(
                    span,
                    "Context must be an identifier that refers to a .",
                ));
            }
        } else {
            return Err(syn::Error::new(
                span,
                "Context must be an identifier that refers to a Source for this snippet.",
            ));
        };
        let label = iter
            .next()
            .map(|m| {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(str),
                    ..
                }) = m
                {
                    Ok(str.value())
                } else {
                    Err(syn::Error::new(
                        m.span(),
                        "Only literal strings are supported as snippet context messages.",
                    ))
                }
            })
            .transpose()?;
        Ok(HighlightAttr { context, label })
    }
}

impl Snippets {
    pub fn from_fields(fields: &syn::Fields) -> syn::Result<Option<Self>> {
        match fields {
            syn::Fields::Named(named) => Self::from_named_fields(named),
            syn::Fields::Unnamed(unnamed) => Self::from_unnamed_fields(unnamed),
            syn::Fields::Unit => Ok(None),
        }
    }

    fn from_named_fields(fields: &syn::FieldsNamed) -> syn::Result<Option<Self>> {
        let mut snippets = HashMap::new();
        // First we collect all the contexts
        for field in &fields.named {
            for attr in &field.attrs {
                if attr.path.is_ident("snippet") {
                    let field_ident = field
                        .ident
                        .clone()
                        .expect("MIETTE BUG: named fields should have idents");
                    let SnippetAttr {
                        source,
                        message,
                        source_name,
                    } = attr.parse_args::<SnippetAttr>()?;
                    // TODO: useful error when source refers to a field that doesn't exist.
                    snippets.insert(
                        field_ident.clone(),
                        Snippet {
                            message,
                            highlights: Vec::new(),
                            source_name,
                            source,
                            context: field_ident,
                        },
                    );
                }
            }
        }
        // Then we loop again looking for highlights
        for field in &fields.named {
            for attr in &field.attrs {
                if attr.path.is_ident("highlight") {
                    let HighlightAttr { context, label } = attr.parse_args::<HighlightAttr>()?;
                    if let Some(snippet) = snippets.get_mut(&context) {
                        snippet.highlights.push(Highlight {
                            label,
                            highlight: field
                                .ident
                                .clone()
                                .expect("MIETTE BUG: named fields should have idents?"),
                        });
                    } else {
                        return Err(syn::Error::new(context.span(), "Highlight must refer to an existing field with a #[snippet(...)] attribute."));
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

    fn from_unnamed_fields(_fields: &syn::FieldsUnnamed) -> syn::Result<Option<Self>> {
        Ok(None)
    }

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        let snippets = self.0.iter().map(|snippet| {
            // snippet message
            let msg = snippet
                .message
                .as_ref()
                .map(|msg| {
                    quote! {
                        message: std::option::Option::Some(#msg.into()),
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
                // TODO: I don't like this. Think about it more and maybe improve protocol?
                source: self.#src_ident.clone(),
            };

            // Source name
            let src_name = &snippet.source_name;
            let src_name = quote! {
                source_name: #src_name.into(),
            };

            // Context
            let context = &snippet.context;
            let context = quote! {
                context: self.#context.clone(),
            };

            // Highlights
            let highlights = snippet.highlights.iter().map(|highlight| {
                let Highlight { highlight, label } = highlight;
                quote! {
                    (#label.into(), self.#highlight.clone())
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
                    #src_name
                    #src_ident
                    #context
                    #highlights
                }
            }
        });
        Some(quote! {
            fn snippets(&self) -> std::option::Option<std::boxed::Box<dyn std::iter::Iterator<Item = miette::DiagnosticSnippet>>> {
                Some(Box::new(vec![
                    #(#snippets),*
                ].into_iter()))
            }
        })
    }

    pub(crate) fn gen_enum(
        _diag: &Diagnostic,
        _variants: &[DiagnosticVariant],
    ) -> Option<TokenStream> {
        None
    }
}
