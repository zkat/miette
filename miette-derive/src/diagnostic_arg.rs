use syn::parse::{Parse, ParseStream};

use crate::code::Code;
use crate::help::Help;
use crate::severity::Severity;

pub enum DiagnosticArg {
    Code(Code),
    Severity(Severity),
    Help(Help),
}

impl Parse for DiagnosticArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.fork().parse::<syn::Ident>()?;
        if ident == "code" {
            Ok(DiagnosticArg::Code(input.parse()?))
        } else if ident == "severity" {
            Ok(DiagnosticArg::Severity(input.parse()?))
        } else if ident == "help" {
            Ok(DiagnosticArg::Help(input.parse()?))
        } else {
            Err(syn::Error::new(
                ident.span(),
                "Unrecognized diagnostic option",
            ))
        }
    }
}
