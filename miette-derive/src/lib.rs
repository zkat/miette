use darling::{
    ast::{self, Fields},
    FromDeriveInput, FromField, FromVariant, ToTokens,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use code::Code;
use help::Help;
use severity::Severity;

mod code;
mod help;
mod severity;

#[proc_macro_derive(Diagnostic, attributes(diagnostic))]
pub fn derive_diagnostic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let cmd = match Diagnostic::from_derive_input(&input) {
        Ok(cmd) => cmd,
        Err(err) => return err.write_errors().into(),
    };
    // panic!("{:#}", cmd.to_token_stream());
    quote!(#cmd).into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(any), attributes(diagnostic))]
struct Diagnostic {
    ident: syn::Ident,
    data: ast::Data<DiagnosticVariant, DiagnosticField>,
    generics: syn::Generics,
    #[darling(default)]
    code: Option<Code>,
    #[darling(default)]
    severity: Option<Severity>,
    #[darling(default)]
    help: Option<Help>,
}

#[derive(Debug, FromField)]
struct DiagnosticField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(diagnostic))]
struct DiagnosticVariant {
    ident: syn::Ident,
    code: Code,
    #[darling(default)]
    severity: Option<Severity>,
    #[darling(default)]
    help: Option<Help>,
}

impl ToTokens for Diagnostic {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = match self.data.as_ref() {
            ast::Data::Enum(variants) => self.gen_enum(variants),
            ast::Data::Struct(fields) => self.gen_struct(fields),
        };
        tokens.extend(ts);
    }
}

impl Diagnostic {
    fn gen_enum(&self, variants: Vec<&DiagnosticVariant>) -> TokenStream {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = &self.generics.split_for_impl();
        let code_body = Code::gen_enum(self, &variants);
        let help_body = Help::gen_enum(self, &variants);
        let sev_body = Severity::gen_enum(self, &variants);

        quote! {
            impl #impl_generics miette::Diagnostic for #ident #ty_generics #where_clause {
                #code_body
                #help_body
                #sev_body
            }
        }
    }

    fn gen_struct(&self, fields: Fields<&DiagnosticField>) -> TokenStream {
        let ident= &self.ident;
        let (impl_generics, ty_generics, where_clause) = &self.generics.split_for_impl();
        let code_body = Code::gen_struct(self, &fields);
        let help_body = Help::gen_struct(self, &fields);
        let sev_body = Severity::gen_struct(self, &fields);

        quote! {
            impl #impl_generics miette::Diagnostic for #ident #ty_generics #where_clause {
                #code_body
                #help_body
                #sev_body
            }
        }
    }
}
