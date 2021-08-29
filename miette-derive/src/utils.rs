use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
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

// bool here is whether to use curly braces
pub fn single_field_name(fields: &syn::Fields) -> Option<(bool, &syn::Field)> {
    match fields {
        syn::Fields::Named(f) if f.named.len() == 1 => f.named.first().map(|x| (true, x)),
        syn::Fields::Unnamed(f) if f.unnamed.len() == 1 => f.unnamed.first().map(|x| (false, x)),
        _ => None,
    }
}

// Returns a match arm
pub fn forward_to_single_field_variant(
    ident: &syn::Ident,
    fields: &syn::Fields,
    method_call: TokenStream,
) -> TokenStream {
    if let Some((curly, single_field)) = single_field_name(fields) {
        let field_name = single_field
            .ident
            .clone()
            .unwrap_or_else(|| format_ident!("unnamed"));
        if curly {
            quote! { Self::#ident { #field_name } => #field_name.#method_call, }
        } else {
            quote! { Self::#ident(#field_name) => #field_name.#method_call, }
        }
    } else {
        quote! {
            _ => compile_error!("miette: used `#[diagnostic(transparent)]` on variant without one single field"),
        }
    }
}
