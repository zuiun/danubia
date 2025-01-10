use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive (Debug)]
pub enum DanubiaErrorKind {
    FactionError,
    ReaderError,
}

impl Display for DanubiaErrorKind {
    fn fmt (&self, f: &mut Formatter) -> Result {
        todo!()
    }
}

impl Error for DanubiaErrorKind {}
