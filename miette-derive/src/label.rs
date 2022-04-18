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
    utils::{display_pat_members, gen_all_variants_with},
};

pub struct Labels(Vec<Label>);

struct Label {
    label: Option<Display>,
    optional: bool,
    span: syn::Member,
}

struct LabelAttr {
    label: Option<Display>,
    optional: bool,
}

impl Parse for LabelAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let la = input.lookahead1();
        let (label, optional) = if la.peek(syn::token::Paren) {
            // #[label("{}", x)]
            let content;
            parenthesized!(content in input);
            let optional = if content.peek(syn::Ident) {
                let ident = content.parse::<syn::Ident>()?;
                if ident == "optional" {
                    if content.peek(syn::Token![,]) {
                        content.parse::<syn::Token![,]>()?;
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if content.peek(syn::LitStr) {
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
                (Some(display), optional)
            } else if optional {
                (None, optional)
            } else {
                return Err(syn::Error::new(input.span(), "Invalid argument to label() attribute. The first argument must be a literal string or the identifier `optional`"));
            }
        } else if la.peek(Token![=]) {
            // #[label = "blabla"]
            input.parse::<Token![=]>()?;
            (
                Some(Display {
                    fmt: input.parse()?,
                    args: TokenStream::new(),
                    has_bonus_display: false,
                }),
                false,
            )
        } else {
            (None, false)
        };
        Ok(LabelAttr { label, optional })
    }
}

impl Labels {
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
        let mut labels = Vec::new();
        for (i, field) in fields.iter().enumerate() {
            for attr in &field.attrs {
                if attr.path.is_ident("label") {
                    let span = if let Some(ident) = field.ident.clone() {
                        syn::Member::Named(ident)
                    } else {
                        syn::Member::Unnamed(syn::Index {
                            index: i as u32,
                            span: field.span(),
                        })
                    };
                    let LabelAttr { label, optional } =
                        syn::parse2::<LabelAttr>(attr.tokens.clone())?;
                    labels.push(Label {
                        label,
                        span,
                        optional,
                    });
                }
            }
        }
        if labels.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Labels(labels)))
        }
    }

    pub(crate) fn gen_struct(&self, fields: &syn::Fields) -> Option<TokenStream> {
        let (display_pat, display_members) = display_pat_members(fields);
        let labels = self.0.iter().map(|highlight| {
            let Label {
                span,
                label,
                optional,
            } = highlight;
            let var = quote! { __miette_internal_var };
            if let Some(display) = label {
                let (fmt, args) = display.expand_shorthand_cloned(&display_members);
                if *optional {
                    quote! {
                        self.#span.clone().map(|#var|
                            miette::LabeledSpan::new_with_span(
                                std::option::Option::Some(format!(#fmt #args)),
                                #var,
                        ))
                    }
                } else {
                    quote! {
                        Some(miette::LabeledSpan::new_with_span(
                            std::option::Option::Some(format!(#fmt #args)),
                            self.#span.clone(),
                        ))
                    }
                }
            } else if *optional {
                quote! {
                    self.#span.clone().map(|#var|
                        miette::LabeledSpan::new_with_span(
                            std::option::Option::None,
                            #var,
                    ))
                }
            } else {
                quote! {
                    Some(miette::LabeledSpan::new_with_span(
                        std::option::Option::None,
                        self.#span.clone(),
                    ))
                }
            }
        });
        Some(quote! {
            #[allow(unused_variables)]
            fn labels(&self) -> std::option::Option<std::boxed::Box<dyn std::iter::Iterator<Item = miette::LabeledSpan> + '_>> {
                let Self #display_pat = self;
                std::option::Option::Some(Box::new(vec![
                    #(#labels),*
                ].into_iter().filter(Option::is_some).map(Option::unwrap)))
            }
        })
    }

    pub(crate) fn gen_enum(variants: &[DiagnosticDef]) -> Option<TokenStream> {
        gen_all_variants_with(
            variants,
            WhichFn::Labels,
            |ident, fields, DiagnosticConcreteArgs { labels, .. }| {
                let (display_pat, display_members) = display_pat_members(fields);
                labels.as_ref().and_then(|labels| {
                let variant_labels = labels.0.iter().map(|label| {
                    let Label { span, label, optional } = label;
                    let field = match &span {
                        syn::Member::Named(ident) => ident.clone(),
                        syn::Member::Unnamed(syn::Index { index, .. }) => {
                            format_ident!("_{}", index)
                        }
                    };
                    let var = quote! { __miette_internal_var };
                    if let Some(display) = label {
                        let (fmt, args) = display.expand_shorthand_cloned(&display_members);
                        if *optional {
                            quote! {
                                #field.clone().map(|#var|
                                    miette::LabeledSpan::new_with_span(
                                        std::option::Option::Some(format!(#fmt #args)),
                                        #var,
                                ))
                            }
                        } else {
                            quote! {
                                Some(miette::LabeledSpan::new_with_span(
                                    std::option::Option::Some(format!(#fmt #args)),
                                    #field.clone(),
                                ))
                            }
                        }
                    } else if *optional {
                        quote! {
                            #field.clone().map(|#var|
                                miette::LabeledSpan::new_with_span(
                                    std::option::Option::None,
                                    #var,
                            ))
                        }
                    } else {
                        quote! {
                            Some(miette::LabeledSpan::new_with_span(
                                std::option::Option::None,
                                #field.clone(),
                            ))
                        }
                    }
                });
                let variant_name = ident.clone();
                match &fields {
                    syn::Fields::Unit => None,
                    _ => Some(quote! {
                        Self::#variant_name #display_pat => std::option::Option::Some(std::boxed::Box::new(vec![
                            #(#variant_labels),*
                        ].into_iter().filter(Option::is_some).map(Option::unwrap))),
                    }),
                }
                })
            },
        )
    }
}
