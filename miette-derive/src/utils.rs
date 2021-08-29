use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};

pub(crate) enum MemberOrString {
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

use crate::{
    diagnostic::{DiagnosticConcreteArgs, DiagnosticDef},
    forward::WhichFn,
};

pub(crate) fn gen_all_variants_with(
    variants: &[DiagnosticDef],
    which_fn: WhichFn,
    mut f: impl FnMut(&syn::Ident, &syn::Fields, &DiagnosticConcreteArgs) -> Option<TokenStream>,
) -> Option<TokenStream> {
    let pairs = variants
        .iter()
        .map(|def| {
            def.args
                .forward_or_override_enum(&def.ident, which_fn, |concrete| {
                    f(&def.ident, &def.fields, concrete)
                })
        })
        .flatten()
        .collect::<Vec<_>>();
    if pairs.is_empty() {
        return None;
    }
    let signature = which_fn.signature();
    let catchall = which_fn.catchall_arm();
    Some(quote! {
        #signature {
            #[allow(unused_variables, deprecated)]
            match self {
                #(#pairs)*
                #catchall
            }
        }
    })
}
