// Lionbootloader Core - Configuration Schema Types
// File: core/src/config/schema_types.rs

#![cfg(feature = "with_alloc")] // These types use String and Vec, requiring alloc

use alloc::{string::String, vec::Vec, collections::BTreeMap};
use serde::Deserialize; // Make sure `serde` with `derive` feature is in Cargo.toml

/// Root LBL Configuration Structure.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Handles JSON keys like timeout_ms
pub struct LblConfig {
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u32,
    pub theme: Theme,
    pub entries: Vec<BootEntry>,
    #[serde(default)] // Defaults to empty Vec if missing
    pub plugins: Vec<String>,
    #[serde(default)] // Defaults to AdvancedSettings::default() if missing or partially specified
    pub advanced: AdvancedSettings,
}

fn default_timeout_ms() -> u32 {
    5000 // Default value from schema
}

/// Theme configuration.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub background: String, // Hex color string
    pub accent: String,     // Hex color string
    #[serde(default)]
    pub font: Option<String>,
    #[serde(default, alias = "customProperties")] // Allow "customProperties" or "custom_properties"
    pub custom_properties: Option<BTreeMap<String, String>>,
}

/// A single boot entry.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BootEntry {
    pub id: String,
    pub title: String,
    pub kernel: String, // Path to kernel or chainload target
    #[serde(default)]
    pub initrd: Option<String>,
    #[serde(default)] // Empty string if missing, but kernel cmdline can be truly empty
    pub cmdline: String,
    #[serde(default)]
    pub order: Option<i32>, // Use Option for optional fields not having schema default
    #[serde(default = "default_secure_false")]
    pub secure: bool,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(rename = "type", default = "default_boot_entry_type")] // "type" is a reserved keyword in Rust
    pub entry_type: BootEntryType,
    #[serde(default, alias = "volumeId")]
    pub volume_id: Option<String>,
    #[serde(default = "default_architecture")]
    pub architecture: Option<ArchitectureType>,
}

fn default_secure_false() -> bool {
    false
}

fn default_boot_entry_type() -> BootEntryType {
    BootEntryType::KernelDirect // Default from schema
}

fn default_architecture() -> Option<ArchitectureType> {
    Some(ArchitectureType::Any) // Default from schema if field is present but null, or make it None
}


/// Type of boot entry.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")] // Matches "kernel_direct", "uefi_chainload"
pub enum BootEntryType {
    KernelDirect,
    UefiChainload,
    UefiApplication,
    InternalTool,
}

/// CPU Architecture type.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")] // e.g., "x86_64", "aarch64"
pub enum ArchitectureType {
    X86,
    #[serde(alias = "x64", alias = "amd64")]
    X86_64,
    Arm,
    #[serde(alias = "arm64")]
    Aarch64,
    Riscv32,
    Riscv64,
    Powerpc,
    Mips,
    Any,
}


/// Advanced settings.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedSettings {
    #[serde(default = "default_debug_shell_false")]
    pub debug_shell: bool,
    #[serde(default = "default_log_level_info")]
    pub log_level: LogLevel,
    #[serde(default = "default_false")]
    pub enable_network_boot: bool,
    #[serde(default, alias = "defaultBootEntryId")]
    pub default_boot_entry_id: Option<String>,
    #[serde(default = "default_resolution_auto")]
    pub resolution: String, // Could be an enum: Auto, Specific(u32,u32)
    #[serde(default = "default_true")]
    pub show_countdown: bool,
    #[serde(default = "default_progress_bar_style_modern")]
    pub progress_bar_style: ProgressBarVisualStyle, // Enum for styles
    #[serde(default = "default_true")]
    pub enable_mouse: bool,
    #[serde(default = "default_false")]
    pub enable_touch: bool,
}

// Default impl for AdvancedSettings if it's entirely missing from JSON
impl Default for AdvancedSettings {
    fn default() -> Self {
        AdvancedSettings {
            debug_shell: default_debug_shell_false(),
            log_level: default_log_level_info(),
            enable_network_boot: default_false(),
            default_boot_entry_id: None,
            resolution: default_resolution_auto(),
            show_countdown: default_true(),
            progress_bar_style: default_progress_bar_style_modern(),
            enable_mouse: default_true(),
            enable_touch: default_false(),
        }
    }
}

fn default_false() -> bool { false }
fn default_true() -> bool { true }
fn default_debug_shell_false() -> bool { false } // Schema default
fn default_log_level_info() -> LogLevel { LogLevel::Info } // Schema default
fn default_resolution_auto() -> String { "auto".to_string() } // Schema default
fn default_progress_bar_style_modern() -> ProgressBarVisualStyle { ProgressBarVisualStyle::Modern } // Schema default


/// Log level enum.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")] // "error", "warn", "info", "debug", "trace"
pub enum LogLevel {
    None, // Added for more control
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Progress bar visual style.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProgressBarVisualStyle {
    Classic,
    Modern,
    Minimal,
    Dots,
}