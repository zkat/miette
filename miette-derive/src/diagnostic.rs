use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, Token};

use crate::code::Code;
use crate::diagnostic_arg::DiagnosticArg;
use crate::help::Help;
use crate::severity::Severity;
use crate::snippets::Snippets;

pub enum Diagnostic {
    Struct {
        fields: syn::Fields,
        ident: syn::Ident,
        generics: syn::Generics,
        code: Code,
        severity: Option<Severity>,
        help: Option<Help>,
        snippets: Option<Snippets>,
    },
    Enum {
        ident: syn::Ident,
        generics: syn::Generics,
        variants: Vec<DiagnosticVariant>,
    },
}

pub struct DiagnosticVariant {
    pub ident: syn::Ident,
    pub fields: syn::Fields,
    pub code: Code,
    pub severity: Option<Severity>,
    pub help: Option<Help>,
    pub snippets: Option<Snippets>,
}

impl Diagnostic {
    pub fn from_derive_input(input: DeriveInput) -> Result<Self, syn::Error> {
        Ok(match input.data {
            syn::Data::Struct(data_struct) => {
                if let Some(attr) = input.attrs.iter().find(|x| x.path.is_ident("diagnostic")) {
                    let args = attr.parse_args_with(
                        Punctuated::<DiagnosticArg, Token![,]>::parse_terminated,
                    )?;
                    let mut code = None;
                    let mut severity = None;
                    let mut help = None;
                    for arg in args {
                        match arg {
                            DiagnosticArg::Code(new_code) => {
                                // TODO: error on multiple?
                                code = Some(new_code);
                            }
                            DiagnosticArg::Severity(sev) => {
                                severity = Some(sev);
                            }
                            DiagnosticArg::Help(hl) => help = Some(hl),
                        }
                    }
                    let snippets = Snippets::from_fields(&data_struct.fields)?;
                    let ident = input.ident.clone();
                    Diagnostic::Struct {
                        fields: data_struct.fields,
                        ident: input.ident,
                        generics: input.generics,
                        code: code.ok_or_else(|| {
                            syn::Error::new(ident.span(), "Diagnostic code is required.")
                        })?,
                        help,
                        severity,
                        snippets,
                    }
                } else {
                    // Also handle when there's multiple `#[diagnostic]` attrs?
                    return Err(syn::Error::new(
                        input.ident.span(),
                        "#[diagnostic] attribute is required when deriving Diagnostic.",
                    ));
                }
            }
            syn::Data::Enum(syn::DataEnum { variants, .. }) => {
                let mut vars = Vec::new();
                for var in variants {
                    if let Some(attr) = var.attrs.iter().find(|x| x.path.is_ident("diagnostic")) {
                        let args = attr.parse_args_with(
                            Punctuated::<DiagnosticArg, Token![,]>::parse_terminated,
                        )?;
                        let mut code = None;
                        let mut severity = None;
                        let mut help = None;
                        for arg in args {
                            match arg {
                                DiagnosticArg::Code(new_code) => {
                                    // TODO: error on multiple?
                                    code = Some(new_code);
                                }
                                DiagnosticArg::Severity(sev) => {
                                    severity = Some(sev);
                                }
                                DiagnosticArg::Help(hl) => {
                                    help = Some(hl);
                                }
                            }
                        }
                        let snippets = Snippets::from_fields(&var.fields)?;
                        let ident = input.ident.clone();
                        vars.push(DiagnosticVariant {
                            ident: var.ident,
                            fields: var.fields,
                            code: code.ok_or_else(|| {
                                syn::Error::new(ident.span(), "Diagnostic code is required.")
                            })?,
                            help,
                            severity,
                            snippets,
                        });
                    } else {
                        // Also handle when there's multiple `#[diagnostic]` attrs?
                        return Err(syn::Error::new(
                            var.ident.span(),
                            "#[diagnostic] attribute is required on all enum variants when deriving Diagnostic.",
                        ));
                    }
                }
                Diagnostic::Enum {
                    ident: input.ident,
                    generics: input.generics,
                    variants: vars,
                }
            }
            syn::Data::Union(_) => {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "Can't derive Diagnostic for Unions",
                ))
            }
        })
    }

    pub fn gen(&self) -> TokenStream {
        match self {
            Self::Struct {
                fields,
                ident,
                generics,
                code,
                severity,
                help,
                snippets,
            } => {
                let (impl_generics, ty_generics, where_clause) = &generics.split_for_impl();
                let code_body = code.gen_struct();
                let help_body = help.as_ref().and_then(|x| x.gen_struct(fields));
                let sev_body = severity.as_ref().and_then(|x| x.gen_struct());
                let snip_body = snippets.as_ref().and_then(|x| x.gen_struct());

                quote! {
                    impl #impl_generics miette::Diagnostic for #ident #ty_generics #where_clause {
                        #code_body
                        #help_body
                        #sev_body
                        #snip_body
                    }
                }
            }
            Self::Enum {
                ident,
                generics,
                variants,
            } => {
                let (impl_generics, ty_generics, where_clause) = &generics.split_for_impl();
                let code_body = Code::gen_enum(variants);
                let help_body = Help::gen_enum(variants);
                let sev_body = Severity::gen_enum(variants);
                let snip_body = Snippets::gen_enum(variants);

                quote! {
                    impl #impl_generics miette::Diagnostic for #ident #ty_generics #where_clause {
                        #code_body
                        #help_body
                        #sev_body
                        #snip_body
                    }
                }
            }
        }
    }
}
