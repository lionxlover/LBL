// Lionbootloader Core - Configuration Parser
// File: core/src/config/parser.rs

#![cfg(feature = "with_alloc")] // Uses serde_json which typically requires alloc

use super::schema_types::LblConfig; // LblConfig from the same module
use super::ConfigError;            // ConfigError from the parent config module
use crate::logger;

// When a no_std compatible JSON schema validator is chosen, it would be imported here.
// Example: use some_no_std_jsonschema_validator::validate;

/// Parses a JSON string into an `LblConfig` struct.
/// Optionally validates against an embedded schema first if a validator is available.
pub fn parse_config_json(json_string: &str) -> Result<LblConfig, ConfigError> {
    logger::info!("[ConfigParser] Parsing configuration JSON...");

    // --- Optional Schema Validation Step ---
    // If a no_std JSON schema validator is available and integrated:
    /*
    let schema_str = super::get_embedded_schema_string();
    match serde_json::from_str(schema_str) {
        Ok(schema_json_value) => {
            match serde_json::from_str(json_string) {
                Ok(config_json_value) => {
                    // Perform validation using a library like 'jsonschema' (if a no_std version exists)
                    // or a custom lightweight validator.
                    // Example conceptual call:
                    // if let Err(validation_errors) = validate_value(&config_json_value, &schema_json_value) {
                    //     logger::error!("[ConfigParser] JSON schema validation failed.");
                    //     // Construct a meaningful error message from validation_errors
                    //     let error_details = validation_errors.iter().map(|e| format!("{:?}", e)).collect::<Vec<_>>().join("\n");
                    //     return Err(ConfigError::ValidationError(error_details));
                    // }
                    logger::info!("[ConfigParser] Schema validation placeholder: SKIPPED/PASSED.");
                }
                Err(e) => {
                    logger::error!("[ConfigParser] Failed to parse config string into JSON value for validation: {}", e);
                    return Err(ConfigError::InvalidFormat(format!("Config is not valid JSON: {}", e)));
                }
            }
        }
        Err(e) => {
            logger::error!("[ConfigParser] Embedded schema is not valid JSON: {}. This is an internal LBL error.", e);
            // This is a critical internal error, as the embedded schema should always be valid.
            return Err(ConfigError::LogicError("Embedded schema is malformed".to_string()));
        }
    }
    */
    // For now, schema validation is conceptual. Direct parsing will occur.
    // A simple check might be to ensure top-level required fields are somewhat present
    // before full parsing. But serde handles missing fields based on struct definitions.

    // --- Deserialization using Serde ---
    match serde_json::from_str(json_string) {
        Ok(config) => {
            logger::info!("[ConfigParser] JSON configuration parsed successfully.");
            // Perform any post-parsing logic checks if needed
            if let Err(e) = validate_parsed_config(&config) {
                return Err(e);
            }
            Ok(config)
        }
        Err(e) => {
            logger::error!("[ConfigParser] Failed to deserialize JSON into LblConfig: {}", e);
            Err(ConfigError::from(e)) // Converts serde_json::Error to ConfigError
        }
    }
}

/// Performs additional logical validation on the parsed LblConfig.
/// This is for checks that are hard to express in JSON Schema or for semantic integrity.
fn validate_parsed_config(config: &LblConfig) -> Result<(), ConfigError> {
    logger::debug!("[ConfigParser] Performing logical validation on parsed config...");

    // Example: Check if default_boot_entry_id (if set) actually exists in entries
    if let Some(default_id) = &config.advanced.default_boot_entry_id {
        if !config.entries.iter().any(|entry| entry.id == *default_id) {
            let err_msg = format!(
                "default_boot_entry_id '{}' does not match any entry ID.",
                default_id
            );
            logger::error!("[ConfigParser] Validation error: {}", err_msg);
            return Err(ConfigError::LogicError(err_msg));
        }
    }

    // Example: Ensure all entries have unique IDs
    let mut ids = alloc::collections::BTreeSet::new();
    for entry in &config.entries {
        if !ids.insert(&entry.id) {
            let err_msg = format!("Duplicate boot entry ID found: '{}'. Entry IDs must be unique.", entry.id);
            logger::error!("[ConfigParser] Validation error: {}", err_msg);
            return Err(ConfigError::LogicError(err_msg));
        }
    }
    
    if config.entries.is_empty() {
        let err_msg = "Configuration must contain at least one boot entry.".to_string();
        logger::error!("[ConfigParser] Validation error: {}", err_msg);
        return Err(ConfigError::LogicError(err_msg));
    }


    // TODO: Add more logical checks:
    // - Ensure kernel paths are not empty for relevant entry types.
    // - Validate color hex patterns in theme (though schema should do this, could be a fallback).
    // - Check for consistency between `secure` flag and TPM availability/Secure Boot status from HAL (later).
    // - If specific plugins are required for certain entry types, check their presence.

    logger::debug!("[ConfigParser] Logical validation passed.");
    Ok(())
}


// --- Schema Validation (Conceptual and Simplified) ---
// A full JSON schema validator in no_std is a large undertaking.
// `jsonschema` crate is the standard but depends on `std`.
// For a bootloader, you might:
// 1. Skip schema validation if `serde`'s typed parsing is deemed sufficient for robustness.
// 2. Implement a very lightweight, custom validator for critical parts of the schema.
// 3. Pre-compile the schema into Rust validation functions (e.g., using a build script).

// Example of a very simple custom validation (not a full schema validator)
/*
fn custom_structural_check(json_value: &serde_json::Value) -> Result<(), ConfigError> {
    if !json_value.is_object() {
        return Err(ConfigError::ValidationError("Root must be an object.".to_string()));
    }
    let obj = json_value.as_object().unwrap();

    if !obj.contains_key("entries") || !obj["entries"].is_array() {
        return Err(ConfigError::ValidationError("'entries' must be an array and is required.".to_string()));
    }
    // ... more such checks ...
    Ok(())
}
*/