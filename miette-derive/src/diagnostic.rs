use proc_macro2::TokenStream;
use quote::{format_ident, quote};
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
        args: DiagnosticDefArgs,
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

#[derive(Default)]
pub struct DiagnosticConcreteArgs {
    pub code: Option<Code>,
    pub severity: Option<Severity>,
    pub help: Option<Help>,
    pub snippets: Option<Snippets>,
    pub url: Option<Url>,
}

impl DiagnosticConcreteArgs {
    fn parse(
        _ident: &syn::Ident,
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
        let snippets = Snippets::from_fields(fields)?;
        let concrete = DiagnosticConcreteArgs {
            code,
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
                    let args =
                        DiagnosticDefArgs::parse(&input.ident, &data_struct.fields, attr, true)?;
                    Diagnostic::Struct {
                        fields: data_struct.fields,
                        ident: input.ident,
                        generics: input.generics,
                        args,
                    }
                } else {
                    Diagnostic::Struct {
                        fields: data_struct.fields,
                        ident: input.ident,
                        generics: input.generics,
                        args: DiagnosticDefArgs::Concrete(Default::default()),
                    }
                }
            }
            syn::Data::Enum(syn::DataEnum { variants, .. }) => {
                let mut vars = Vec::new();
                for var in variants {
                    if let Some(attr) = var.attrs.iter().find(|x| x.path.is_ident("diagnostic")) {
                        let args = DiagnosticDefArgs::parse(&var.ident, &var.fields, attr, true)?;
                        vars.push(DiagnosticDef {
                            ident: var.ident,
                            fields: var.fields,
                            args,
                        });
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
                match args {
                    DiagnosticDefArgs::Transparent => {
                        if fields.iter().len() != 1 {
                            return quote! {
                                compile_error!("you can only use #[diagnostic(transparent)] on a struct with exactly one field");
                            };
                        }
                        let field = fields
                            .iter()
                            .next()
                            .expect("MIETTE BUG: thought we knew we had exactly one field");
                        let field_name = field
                            .ident
                            .clone()
                            .unwrap_or_else(|| format_ident!("unnamed"));
                        let matcher = match fields {
                            syn::Fields::Named(_) => quote! { let Self { #field_name } = self; },
                            syn::Fields::Unnamed(_) => quote! { let Self(#field_name) = self; },
                            syn::Fields::Unit => {
                                unreachable!("MIETTE BUG: thought we knew we had exactly one field")
                            }
                        };

                        quote! {
                            impl #impl_generics miette::Diagnostic for #ident #ty_generics #where_clause {
                                fn code<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                                    #matcher
                                    #field_name.code()
                                }
                                fn help<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                                    #matcher
                                    #field_name.help()
                                }
                                fn url<'a>(&'a self) -> std::option::Option<std::boxed::Box<dyn std::fmt::Display + 'a>> {
                                    #matcher
                                    #field_name.url()
                                }
                                fn severity(&self) -> std::option::Option<miette::Severity> {
                                    #matcher
                                    #field_name.severity()
                                }
                                fn snippets(&self) -> std::option::Option<std::boxed::Box<dyn std::iter::Iterator<Item = miette::DiagnosticSnippet> + '_>> {
                                    #matcher
                                    #field_name.snippets()
                                }
                            }
                        }
                    }
                    DiagnosticDefArgs::Concrete(concrete) => {
                        let code_body = concrete.code.as_ref().and_then(|x| x.gen_struct());
                        let help_body = concrete.help.as_ref().and_then(|x| x.gen_struct(fields));
                        let sev_body = concrete.severity.as_ref().and_then(|x| x.gen_struct());
                        let snip_body = concrete
                            .snippets
                            .as_ref()
                            .and_then(|x| x.gen_struct(fields));
                        let url_body = concrete
                            .url
                            .as_ref()
                            .and_then(|x| x.gen_struct(ident, fields));

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
