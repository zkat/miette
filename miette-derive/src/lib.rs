use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use diagnostic::Diagnostic;

mod code;
mod diagnostic;
mod diagnostic_arg;
mod help;
mod severity;
mod snippets;

#[proc_macro_derive(Diagnostic, attributes(diagnostic, snippet, highlight))]
pub fn derive_diagnostic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let cmd = match Diagnostic::from_derive_input(input) {
        Ok(cmd) => cmd.gen(),
        Err(err) => return err.to_compile_error().into(),
    };
    // panic!("{:#}", cmd.to_token_stream());
    quote!(#cmd).into()
}
