use cookie::Key;
use hex::FromHexError;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::fs;
use std::path::Path;

const KEY_MIN_LEN: usize = 64;

#[derive(Deserialize, Serialize)]
pub struct Config {
    key: Option<String>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        // Read the entire file
        let data = fs::read(path)?;
        // Parse the file as toml
        let config = toml::from_slice(&data)?;

        Ok(config)
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        // Serialize to bytes
        let ser = toml::to_vec(self)?;
        // Save to file
        fs::write(path, &ser)?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { key: None }
    }
}

impl From<&SecureConfig> for Config {
    fn from(secure_config: &SecureConfig) -> Config {
        let key = hex::encode(&secure_config.key);
        Config { key: Some(key) }
    }
}

pub struct SecureConfig {
    key: Vec<u8>,
}

impl SecureConfig {
    pub fn get_cookie_key(&self) -> Key {
        Key::from(&self.key)
    }
}

impl TryFrom<Config> for SecureConfig {
    type Error = SecureConfigError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        // Check if there is an existing key
        if let Some(key) = config.key {
            // Decode key from hex
            let key = hex::decode(key).map_err(SecureConfigError::HexDecodeFailure)?;
            // Ensure the length of the key is greater than or equal to 64 bytes
            if key.len() >= KEY_MIN_LEN {
                Ok(Self { key })
            } else {
                Err(SecureConfigError::KeyTooSmall {
                    given: key.len(),
                    minimum: KEY_MIN_LEN,
                })
            }
        } else {
            // Generate a key
            let mut key = vec![0; KEY_MIN_LEN];
            OsRng.fill_bytes(&mut key);
            Ok(Self { key })
        }
    }
}

#[derive(Debug)]
pub enum SecureConfigError {
    HexDecodeFailure(FromHexError),
    KeyTooSmall { given: usize, minimum: usize },
}

impl fmt::Display for SecureConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use SecureConfigError::*;
        write!(
            f,
            "{}",
            match self {
                HexDecodeFailure(err) => format!("Error decoding given key as hex: {}", err),
                KeyTooSmall { given, minimum } => format!(
                    "Given key is too small. Given: {}, minimum: {}",
                    given, minimum
                ),
            }
        )
    }
}

impl std::error::Error for SecureConfigError {}
