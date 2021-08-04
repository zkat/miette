/*!
Default trait implementations for [Source].
*/
use std::fs::File;
use std::io::{Cursor, Read, Result};
use std::path::{Path, PathBuf};

use crate::Source;

impl Source for String {
    fn open(&self) -> Result<Box<dyn Read>> {
        Ok(Box::new(Cursor::new(self.clone())))
    }
}

impl Source for Path {
    fn open(&self) -> Result<Box<dyn Read>> {
        Ok(Box::new(File::open(self)?))
    }
}

impl Source for PathBuf {
    fn open(&self) -> Result<Box<dyn Read>> {
        Ok(Box::new(File::open(self)?))
    }
}
