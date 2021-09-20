use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, Token};

use crate::code::Code;
use crate::diagnostic_arg::DiagnosticArg;
use crate::forward::{Forward, WhichFn};
use crate::help::Help;
use crate::label::Labels;
use crate::severity::Severity;
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
    Transparent(Forward),
    Concrete(Box<DiagnosticConcreteArgs>),
}

impl DiagnosticDefArgs {
    pub(crate) fn forward_or_override_enum(
        &self,
        variant: &syn::Ident,
        which_fn: WhichFn,
        mut f: impl FnMut(&DiagnosticConcreteArgs) -> Option<TokenStream>,
    ) -> Option<TokenStream> {
        match self {
            Self::Transparent(forward) => Some(forward.gen_enum_match_arm(variant, which_fn)),
            Self::Concrete(concrete) => f(concrete).or_else(|| {
                concrete
                    .forward
                    .as_ref()
                    .map(|forward| forward.gen_enum_match_arm(variant, which_fn))
            }),
        }
    }
}

#[derive(Default)]
pub struct DiagnosticConcreteArgs {
    pub code: Option<Code>,
    pub severity: Option<Severity>,
    pub help: Option<Help>,
    pub labels: Option<Labels>,
    pub url: Option<Url>,
    pub forward: Option<Forward>,
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
        let mut forward = None;
        for arg in args {
            match arg {
                DiagnosticArg::Transparent => {
                    return Err(syn::Error::new_spanned(attr, "transparent not allowed"));
                }
                DiagnosticArg::Forward(to_field) => {
                    forward = Some(to_field);
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
        let labels = Labels::from_fields(fields)?;
        let concrete = DiagnosticConcreteArgs {
            code,
            help,
            severity,
            labels,
            url,
            forward,
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
    ) -> syn::Result<Self> {
        let args =
            attr.parse_args_with(Punctuated::<DiagnosticArg, Token![,]>::parse_terminated)?;
        if allow_transparent
            && args.len() == 1
            && matches!(args.first(), Some(DiagnosticArg::Transparent))
        {
            let forward = Forward::for_transparent_field(fields)?;
            return Ok(Self::Transparent(forward));
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
        Ok(DiagnosticDefArgs::Concrete(Box::new(concrete)))
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
                    DiagnosticDefArgs::Transparent(forward) => {
                        let code_method = forward.gen_struct_method(WhichFn::Code);
                        let help_method = forward.gen_struct_method(WhichFn::Help);
                        let url_method = forward.gen_struct_method(WhichFn::Url);
                        let labels_method = forward.gen_struct_method(WhichFn::Labels);
                        let severity_method = forward.gen_struct_method(WhichFn::Severity);

                        quote! {
                            impl #impl_generics miette::Diagnostic for #ident #ty_generics #where_clause {
                                #code_method
                                #help_method
                                #url_method
                                #labels_method
                                #severity_method
                            }
                        }
                    }
                    DiagnosticDefArgs::Concrete(concrete) => {
                        let forward = |which| {
                            concrete
                                .forward
                                .as_ref()
                                .map(|fwd| fwd.gen_struct_method(which))
                        };
                        let code_body = concrete
                            .code
                            .as_ref()
                            .and_then(|x| x.gen_struct())
                            .or_else(|| forward(WhichFn::Code));
                        let help_body = concrete
                            .help
                            .as_ref()
                            .and_then(|x| x.gen_struct(fields))
                            .or_else(|| forward(WhichFn::Help));
                        let sev_body = concrete
                            .severity
                            .as_ref()
                            .and_then(|x| x.gen_struct())
                            .or_else(|| forward(WhichFn::Severity));
                        let url_body = concrete
                            .url
                            .as_ref()
                            .and_then(|x| x.gen_struct(ident, fields))
                            .or_else(|| forward(WhichFn::Url));
                        let labels_body = concrete
                            .labels
                            .as_ref()
                            .and_then(|x| x.gen_struct(fields))
                            .or_else(|| forward(WhichFn::Url));

                        quote! {
                            impl #impl_generics miette::Diagnostic for #ident #ty_generics #where_clause {
                                #code_body
                                #help_body
                                #sev_body
                                #url_body
                                #labels_body
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
                let labels_body = Labels::gen_enum(variants);
                let url_body = Url::gen_enum(ident, variants);
                quote! {
                    impl #impl_generics miette::Diagnostic for #ident #ty_generics #where_clause {
                        #code_body
                        #help_body
                        #sev_body
                        #labels_body
                        #url_body
                    }
                }
            }
        }
    }
}
