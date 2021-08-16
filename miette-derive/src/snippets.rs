use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use syn::parse::{Parse, ParseStream};

use crate::diagnostic::{Diagnostic, DiagnosticVariant};

pub struct Snippets(Vec<Snippet>);

struct Snippet {
    message: String,
    highlights: Vec<Highlight>,
    // TODO: These two should be special expressions a-la-thiserror. This won't work for enums either.
    source_field: syn::Ident,
    context_field: syn::Ident,
}

struct Highlight {
    label: String,
    // TODO: This should be a special expression a-la-thiserror.
    highlight_field: syn::Ident,
}

struct ContextAttr {
    source: syn::Ident,
    message: String,
}

impl Parse for ContextAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

struct HighlightAttr {
    context: syn::Ident,
    label: String,
}

impl Parse for HighlightAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

impl Snippets {
    pub fn from_fields(fields: &syn::Fields) -> Option<Self> {
        match fields {
            syn::Fields::Named(named) => Self::from_named_fields(named),
            syn::Fields::Unnamed(unnamed) => Self::from_unnamed_fields(unnamed),
            syn::Fields::Unit => None,
        }
    }

    fn from_named_fields(fields: &syn::FieldsNamed) -> Option<Self> {
        let mut snippets = HashMap::new();
        // First we collect all the contexts
        for field in &fields.named {
            for attr in &field.attrs {
                if attr.path.is_ident("context") {
                    // TODO: parse these two from attr
                    let source = syn::Ident::new("foo", Span::call_site());
                    let message = String::new();
                    // TODO: useful error when source refers to a field that doesn't exist.
                    snippets.insert(
                        source.clone(),
                        Snippet {
                            message,
                            highlights: Vec::new(),
                            source_field: source.clone(),
                            context_field: field
                                .ident
                                .clone()
                                .expect("MIETTE BUG: named fields should have idents?"),
                        },
                    );
                }
            }
        }
        // Then we loop again looking for highlights
        for field in &fields.named {
            for attr in &field.attrs {
                if attr.path.is_ident("highlight") {
                    // TODO: parse these two from attr
                    let context = syn::Ident::new("foo", Span::call_site());
                    let label = String::new();
                    if let Some(snippet) = snippets.get_mut(&context) {
                        snippet.highlights.push(Highlight {
                            label,
                            highlight_field: field
                                .ident
                                .clone()
                                .expect("MIETTE BUG: named fields should have idents?"),
                        });
                    } else {
                        // TODO: useful error when context refers to a field that isn't a highlight.
                        todo!()
                    }
                }
            }
        }
        if snippets.is_empty() {
            None
        } else {
            Some(Snippets(snippets.into_values().collect()))
        }
    }

    fn from_unnamed_fields(_fields: &syn::FieldsUnnamed) -> Option<Self> {
        None
    }

    pub(crate) fn gen_struct(&self) -> Option<TokenStream> {
        None
    }

    pub(crate) fn gen_enum(
        _diag: &Diagnostic,
        _variants: &[DiagnosticVariant],
    ) -> Option<TokenStream> {
        None
    }
}
