// Lionbootloader Core - Configuration Module
// File: core/src/config.rs

#[cfg(feature = "with_alloc")]
use alloc::{string::String, vec::Vec};

use crate::logger;
use crate::fs::manager::FilesystemManager; // To read the config file
#[cfg(feature = "with_alloc")]
use crate::fs::interface::FilesystemError;


// Sub-modules
pub mod parser;         // For parsing JSON and validating against schema
pub mod schema_types;   // Rust structs mirroring the JSON schema

// Re-export the main configuration struct and other important types
pub use schema_types::{AdvancedSettings, BootEntry, LblConfig, Theme};


// Default path to the configuration file within a recognized filesystem.
// This might be searched for on all mounted volumes.
pub const DEFAULT_CONFIG_PATH_PRIMARY: &str = "/LBL/config.json"; // Primary location
pub const DEFAULT_CONFIG_PATH_SECONDARY: &str = "/boot/lbl/config.json"; // Secondary location
pub const DEFAULT_CONFIG_PATH_FALLBACK: &str = "/config.json"; // Fallback in root

/// Loads and parses the LBL configuration file.
/// It will search for the config file in predefined locations on available filesystems.
#[cfg(feature = "with_alloc")]
pub fn load_configuration(
    fs_manager: &FilesystemManager,
    // config_path_override: Option<&str>, // Allow overriding the search path
) -> Result<LblConfig, ConfigError> {
    logger::info!("[Config] Attempting to load configuration...");

    let search_paths = [
        DEFAULT_CONFIG_PATH_PRIMARY,
        DEFAULT_CONFIG_PATH_SECONDARY,
        DEFAULT_CONFIG_PATH_FALLBACK,
    ];

    let mounted_volumes = fs_manager.list_mounted_volumes();
    if mounted_volumes.is_empty() {
        logger.error!("[Config] No volumes mounted. Cannot load configuration.");
        return Err(ConfigError::NoVolumesMounted);
    }

    for volume in &mounted_volumes {
        logger::debug!("[Config] Searching on volume: {} (Type: {})", volume.id, volume.fs_type);
        for path_suffix in &search_paths {
            logger::trace!("[Config] Trying path: {} on volume {}", path_suffix, volume.id);
            match fs_manager.read_file(&volume.id, path_suffix) {
                Ok(json_data_bytes) => {
                    logger::info!(
                        "[Config] Found configuration file at: volume_id={}, path={}",
                        volume.id,
                        path_suffix
                    );
                    let json_string = match String::from_utf8(json_data_bytes) {
                        Ok(s) => s,
                        Err(e) => {
                            logger::error!("[Config] Config file is not valid UTF-8: {}", e);
                            return Err(ConfigError::InvalidFormat("UTF-8 decoding error".to_string()));
                        }
                    };

                    // TODO: Implement or use a schema validator if a robust one is available for no_std.
                    // For now, direct parsing. Later, call parser::validate_against_schema(&json_string, &get_embedded_schema())?;
                    
                    return parser::parse_config_json(&json_string);
                }
                Err(FilesystemError::NotFound) => {
                    // This is expected, continue searching
                    continue;
                }
                Err(e) => {
                    // Other FS error while trying to read a potential config file
                    logger.warn!(
                        "[Config] Filesystem error trying to read {} on {}: {:?}",
                        path_suffix,
                        volume.id,
                        e
                    );
                }
            }
        }
    }

    logger::error!("[Config] Configuration file not found in any search path on any mounted volume.");
    Err(ConfigError::FileNotFound)
}

#[cfg(not(feature = "with_alloc"))]
pub fn load_configuration(
    // fs_manager: &FilesystemManagerNoAlloc, // Simplified or different manager
) -> Result<LblConfig, ConfigError> { // LblConfig would need to be no_alloc compatible
    logger::info!("[Config] load_configuration (no_alloc version - stubbed)...");
    // In no_alloc, loading dynamic JSON is very hard.
    // Config might be statically compiled, or a very simple binary format.
    // This is a major simplification.
    Err(ConfigError::NotImplementedNoAlloc)
}


/// Retrieves the embedded JSON schema for validation.
/// In a real scenario, this schema string would be embedded into the binary at compile time.
#[cfg(feature = "with_alloc")] // Schema validation typically uses alloc for string processing
pub fn get_embedded_schema_string() -> &'static str {
    // This would be `include_str!("../../config/schema.json")` if accessible.
    // For now, a placeholder. The actual `schema.json` needs to be processed.
    // The content of `config/schema.json` should be pasted here or included via macro.
    // For brevity, using a very minimal placeholder.
    r#"{
        "type": "object",
        "properties": {
            "timeout_ms": {"type": "integer"}
        },
        "required": []
    }"#
    // A proper solution:
    // const SCHEMA_STR: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../config/schema.json"));
    // SCHEMA_STR
    // ^^ This requires that the build environment can access this path relative to the core crate.
    // May need to adjust path or use build scripts to place schema content into a .rs file.
}


#[derive(Debug)]
pub enum ConfigError {
    FileNotFound,
    NoVolumesMounted,
    ReadError(String),
    InvalidFormat(String), // E.g. JSON parsing error, UTF-8 error
    ValidationError(String), // Schema validation error
    LogicError(String), // E.g. inconsistent configuration
    NotImplementedNoAlloc,
    Filesystem(#[cfg(feature = "with_alloc")] FilesystemError),
}

#[cfg(feature = "with_alloc")]
impl From<FilesystemError> for ConfigError {
    fn from(fs_err: FilesystemError) -> Self {
        ConfigError::Filesystem(fs_err)
    }
}

#[cfg(feature = "with_alloc")]
impl From<serde_json::Error> for ConfigError {
    fn from(json_err: serde_json::Error) -> Self {
        ConfigError::InvalidFormat(format!("JSON parse error: {}", json_err))
    }
}

// If using a schema validation library that has its own error type:
// impl From<MySchemaValidationError> for ConfigError {
//     fn from(schema_err: MySchemaValidationError) -> Self {
//         ConfigError::ValidationError(schema_err.to_string())
//     }
// }