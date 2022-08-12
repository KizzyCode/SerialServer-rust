#![doc = include_str!("../README.md")]

#[macro_use]
pub mod error;
pub mod config;
pub mod logger;
pub mod serial;
pub mod server;

use crate::{config::Config, error::Error, server::Server};
use std::process;

pub fn main() {
    /// The real main function
    fn _main() -> Result<(), Error> {
        // Parse the args and start the server
        let config = Config::load()?;
        let server = Server::new(config)?;
        server.runloop()
    }

    // Call the real main function
    if let Err(e) = _main() {
        eprintln!("{e}");
        process::exit(1);
    }
}
