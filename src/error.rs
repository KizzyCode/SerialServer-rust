//! Implements the crate's error type

use std::{
    error,
    ffi::NulError,
    fmt::{self, Display, Formatter},
    io,
};

use backtrace::Backtrace;

/// Creates a new I/O error
#[macro_export]
macro_rules! eio {
    ($($arg:tt)*) => {{
        let error = format!($($arg)*);
        $crate::error::Error::new(error)
    }};
}

/// The crates error type
#[derive(Debug)]
pub struct Error {
    /// The error description
    error: String,
    /// The underlying error
    source: Option<Box<dyn std::error::Error + Send>>,
    /// The backtrace
    backtrace: Backtrace,
}
impl Error {
    /// Creates a new error
    pub fn new<T>(error: T) -> Self
    where
        T: ToString,
    {
        let backtrace = Backtrace::new();
        Self { error: error.to_string(), source: None, backtrace }
    }
    /// Creates a new error
    pub fn with_error<T>(error: T) -> Self
    where
        T: std::error::Error + Send + 'static,
    {
        let error = Box::new(error);
        let backtrace = Backtrace::new();
        Self { error: error.to_string(), source: Some(error), backtrace }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{}", self.error)?;
        writeln!(f, "{:?}", self.backtrace)?;
        Ok(())
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        let source = self.source.as_ref()?;
        Some(source.as_ref())
    }
}
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::with_error(error)
    }
}
impl From<NulError> for Error {
    fn from(error: NulError) -> Self {
        Self::with_error(error)
    }
}
impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Self::with_error(error)
    }
}
