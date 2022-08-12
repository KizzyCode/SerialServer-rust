//! The logging facility

use std::{io, io::Write};

/// Logs messages
#[derive(Debug, Clone, Copy)]
pub struct Logger {
    _private: (),
}
impl Logger {
    /// Creates a new logger
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Logs some data
    pub fn log<T>(&self, data: T)
    where
        T: AsRef<[u8]>,
    {
        // Write the bytes to stdout
        let mut stdout = io::stdout();
        for &byte in data.as_ref() {
            // Check if the char can be printed
            let mut is_valid = byte.is_ascii_alphanumeric();
            is_valid |= byte.is_ascii_punctuation();
            is_valid |= byte.is_ascii_whitespace();

            // Print the char
            match is_valid {
                true => _ = write!(&mut stdout, "{}", byte as char),
                false => _ = write!(&mut stdout, "\\x{byte:02x}"),
            };
        }
    }
}
