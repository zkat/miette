use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, Token};

use crate::code::Code;
use crate::diagnostic_arg::DiagnosticArg;
use crate::help::Help;
use crate::severity::Severity;
use crate::snippets::Snippets;
use crate::url::Url;

pub enum Diagnostic {
    Struct {
        generics: syn::Generics,
        ident: syn::Ident,
        fields: syn::Fields,
        // Nobody needs a transparent wrapper struct for another thing that already
        // implements diagnostic, surely.
        args: DiagnosticConcreteArgs,
    },
    Enum {
        ident: syn::Ident,
        generics: syn::Generics,
        variants: Vec<DiagnosticDef>,
    },
}

pub struct DiagnosticDef {
    pub ident: syn::Ident,
    pub fields: syn::Fields,
    pub args: DiagnosticDefArgs,
}

pub enum DiagnosticDefArgs {
    Transparent,
    Concrete(DiagnosticConcreteArgs),
}

pub struct DiagnosticConcreteArgs {
    pub code: Code,
    pub severity: Option<Severity>,
    pub help: Option<Help>,
    pub snippets: Option<Snippets>,
    pub url: Option<Url>,
}

impl DiagnosticConcreteArgs {
    fn parse<'a>(
        ident: &syn::Ident,
        fields: &syn::Fields,
        attr: &syn::Attribute,
        args: impl Iterator<Item = DiagnosticArg>,
    ) -> Result<Self, syn::Error> {
        let mut code = None;
        let mut severity = None;
        let mut help = None;
        let mut url = None;
        for arg in args {
            match arg {
                DiagnosticArg::Transparent => {
                    return Err(syn::Error::new_spanned(attr, "transparent not allowed"));
                }
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
                DiagnosticArg::Url(u) => {
                    url = Some(u);
                }
            }
        }
        let snippets = Snippets::from_fields(&fields)?;
        let concrete = DiagnosticConcreteArgs {
            code: code
                .ok_or_else(|| syn::Error::new(ident.span(), "Diagnostic code is required."))?,
            help,
            severity,
            snippets,
            url,
        };
        Ok(concrete)
    }
}

impl DiagnosticDefArgs {
    fn parse(
        ident: &syn::Ident,
        fields: &syn::Fields,
        attr: &syn::Attribute,
        allow_transparent: bool,
    ) -> Result<Self, syn::Error> {
        let args =
            attr.parse_args_with(Punctuated::<DiagnosticArg, Token![,]>::parse_terminated)?;
        if allow_transparent
            && args.len() == 1
            && matches!(args.first(), Some(DiagnosticArg::Transparent))
        {
            return Ok(Self::Transparent);
        } else if args.iter().any(|x| matches!(x, DiagnosticArg::Transparent)) {
            return Err(syn::Error::new_spanned(
                attr,
                if allow_transparent {
                    "diagnostic(transparent) not allowed in combination with other args"
                } else {
                    "diagnostic(transparent) not allowed here"
                },
            ));
        }
        let args = args
            .into_iter()
            .filter(|x| !matches!(x, DiagnosticArg::Transparent));
        let concrete = DiagnosticConcreteArgs::parse(ident, fields, attr, args)?;
        Ok(DiagnosticDefArgs::Concrete(concrete))
    }
}

impl Diagnostic {
    pub fn from_derive_input(input: DeriveInput) -> Result<Self, syn::Error> {
        Ok(match input.data {
            syn::Data::Struct(data_struct) => {
                if let Some(attr) = input.attrs.iter().find(|x| x.path.is_ident("diagnostic")) {
                    let args = attr.parse_args_with(
                        Punctuated::<DiagnosticArg, Token![,]>::parse_terminated,
                    )?;
                    let concrete = DiagnosticConcreteArgs::parse(
                        &input.ident,
                        &data_struct.fields,
                        attr,
                        args.into_iter(),
                    )?;
                    Diagnostic::Struct {
                        fields: data_struct.fields,
                        ident: input.ident,
                        generics: input.generics,
                        args: concrete,
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
                        let args = DiagnosticDefArgs::parse(&var.ident, &var.fields, &attr, true)?;
                        vars.push(DiagnosticDef {
                            ident: var.ident,
                            fields: var.fields,
                            args,
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
                ident,
                fields,
                generics,
                args,
            } => {
                let (impl_generics, ty_generics, where_clause) = &generics.split_for_impl();
                let code_body = args.code.gen_struct();
                let help_body = args.help.as_ref().and_then(|x| x.gen_struct(fields));
                let sev_body = args.severity.as_ref().and_then(|x| x.gen_struct());
                let snip_body = args.snippets.as_ref().and_then(|x| x.gen_struct(fields));
                let url_body = args.url.as_ref().and_then(|x| x.gen_struct(ident, fields));

                quote! {
                    impl #impl_generics miette::Diagnostic for #ident #ty_generics #where_clause {
                        #code_body
                        #help_body
                        #sev_body
                        #snip_body
                        #url_body
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
                let url_body = Url::gen_enum(ident, variants);
                quote! {
                    impl #impl_generics miette::Diagnostic for #ident #ty_generics #where_clause {
                        #code_body
                        #help_body
                        #sev_body
                        #snip_body
                        #url_body
                    }
                }
            }
        }
    }
}
