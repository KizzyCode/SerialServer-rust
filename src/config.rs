//! Implements a config object

use crate::error::Error;
use serde::Deserialize;
use std::{env, fs, path::Path};

/// The serial config
#[derive(Debug, Clone, Deserialize)]
pub struct Serial {
    /// The path to the serial device
    pub device: String,
    /// The baudrate to use with the serial port
    #[serde(default = "Serial::baudrate_default")]
    pub baudrate: u64,
}
impl Serial {
    /// The default baudrate
    const fn baudrate_default() -> u64 {
        115200
    }
}

/// The UDP configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Udp {
    /// The UDP address to listen on
    pub listen: String,
    /// The UDP address to send to
    #[serde(default)]
    pub send: Option<String>,
    /// The TTL for outgoing UDP packets
    #[serde(default)]
    pub ttl: u32,
}

/// The logger configuration
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Log {
    /// Whether to enable logging or not
    #[serde(default)]
    pub enabled: bool,
}

/// The config
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The serial device config
    pub serial: Serial,
    /// The UDP config
    pub udp: Udp,
    /// The logger configuration
    #[serde(default)]
    pub log: Log,
}
impl Config {
    /// The default config path
    const PATH: &'static str = "config.toml";

    /// Loads the config
    pub fn load() -> Result<Self, Error> {
        // Load the config file defined by the environment
        if let Ok(path) = env::var("SERIALSERVER_CONFIG") {
            return Self::load_file(&path);
        }

        // Load the config file from first argv
        if let Some(path) = env::args().nth(1) {
            return Self::load_file(&path);
        }

        // Load the local config
        if Self::file_exists(Self::PATH)? {
            return Self::load_file("config.toml");
        }

        // Raise an error if no config could be found
        Err(eio!("Config file not found"))
    }

    /// Checks if a file exists
    fn file_exists(path: &str) -> Result<bool, Error> {
        Ok(Path::new(path).is_file())
    }
    /// Loads the config from a file
    fn load_file(path: &str) -> Result<Self, Error> {
        let config_bin = fs::read(path)?;
        let config: Self = toml::from_slice(&config_bin)?;
        Ok(config)
    }
}
