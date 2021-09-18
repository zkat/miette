use core::fmt::{self, Debug, Display};

use std::error::Error as StdError;

use crate::Diagnostic;

use crate as miette;

#[repr(transparent)]
pub(crate) struct DisplayError<M>(pub(crate) M);

#[repr(transparent)]
pub(crate) struct MessageError<M>(pub(crate) M);

pub(crate) struct NoneError;

impl<M> Debug for DisplayError<M>
where
    M: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<M> Display for DisplayError<M>
where
    M: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<M> StdError for DisplayError<M> where M: Display + 'static {}
impl<M> Diagnostic for DisplayError<M> where M: Display + 'static {}

impl<M> Debug for MessageError<M>
where
    M: Display + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<M> Display for MessageError<M>
where
    M: Display + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<M> StdError for MessageError<M> where M: Display + Debug + 'static {}
impl<M> Diagnostic for MessageError<M> where M: Display + Debug + 'static {}

impl Debug for NoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt("Option was None", f)
    }
}

impl Display for NoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Option was None", f)
    }
}

impl StdError for NoneError {}
impl Diagnostic for NoneError {}

#[repr(transparent)]
pub(crate) struct BoxedError(pub(crate) Box<dyn Diagnostic + Send + Sync>);

impl Diagnostic for BoxedError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.0.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.0.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.0.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.0.url()
    }

    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::SourceSpan> + 'a>> {
        self.0.labels()
    }
}

impl Debug for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl StdError for BoxedError {}
