// Lionbootloader GUI - Theme Manager
// File: gui/src/theme.rs

#[cfg(feature = "with_alloc")]
use alloc::{string::String, sync::Arc, collections::BTreeMap}; // Arc for shared font data

use lionbootloader_core_lib::config::schema_types::Theme as ConfigTheme;
use lionbootloader_core_lib::fs::manager::FilesystemManager;
#[cfg(feature = "with_alloc")]
use lionbootloader_core_lib::fs::interface::FilesystemError;
use lionbootloader_core_lib::logger;

// If using fontdue for font rendering
#[cfg(all(feature = "font_rendering", feature = "with_alloc"))]
use fontdue::{Font, FontSettings};

#[derive(Debug)]
pub enum ThemeError {
    FontLoadFailed(String),
    FontParseFailed(String),
    ColorParseFailed(String),
    ResourceNotFound(String, #[cfg(feature = "with_alloc")] FilesystemError),
    InvalidThemeConfiguration(String),
}


/// Holds processed theme data, like parsed colors and loaded font(s).
#[cfg(feature = "with_alloc")]
#[derive(Clone)] // Cloneable if Arc is used for shared resources like fonts
pub struct AppliedTheme {
    pub name: String, // e.g., "Default Dark", "NordLBL" from config (if we add names to themes)
    
    // Parsed colors (e.g., u32 for ARGB8888)
    pub background_color: u32,
    pub accent_color: u32,
    pub text_color: u32,
    pub secondary_text_color: u32,
    pub panel_bg_color: u32, // For "glass" panels
    pub hover_bg_color: u32,
    pub selected_text_color: u32,
    
    // Custom properties from JSON
    pub custom_props: BTreeMap<String, String>,

    // Handle to the primary UI font
    #[cfg(feature = "font_rendering")]
    pub primary_font: Option<Arc<Font>>,
    // pub icon_font: Option<Arc<Font>>, // If using an icon font
    
    // Other theme properties (sizes, paddings, etc.) could go here
    // pub default_font_size: f32,
}

// Static storage for the currently active theme.
// This avoids passing AppliedTheme everywhere, but access needs care (e.g., init once).
// Using Option and unsafe for simplicity; a Mutex/OnceCell would be safer.
#[cfg(feature = "with_alloc")]
static mut CURRENT_THEME: Option<AppliedTheme> = None;


/// Loads the theme specified in the configuration.
/// This involves parsing colors and loading font files via the FsManager.
#[cfg(all(feature = "with_alloc"))] // Theme processing relies heavily on alloc
pub fn load_theme(
    config_theme: &ConfigTheme,
    fs_manager: &FilesystemManager,
) -> Result<(), ThemeError> {
    logger::info!("[Theme] Loading theme configuration...");

    // --- Parse Colors ---
    // Helper function to parse hex string (e.g., #RRGGBB) to u32 ARGB (0xAARRGGBB or 0xFFRRGGBB)
    let parse_hex_color = |hex_str: &str, default_alpha: u8| -> Result<u32, ThemeError> {
        if !hex_str.starts_with('#') || (hex_str.len() != 7 && hex_str.len() != 9) {
            return Err(ThemeError::ColorParseFailed(format!("Invalid hex color format: {}", hex_str)));
        }
        let hex_value = u32::from_str_radix(&hex_str[1..], 16)
            .map_err(|_| ThemeError::ColorParseFailed(format!("Could not parse hex: {}", hex_str)))?;
        
        if hex_str.len() == 7 { // #RRGGBB -> 0xFFRRGGBB
            Ok((default_alpha as u32) << 24 | hex_value)
        } else { // #AARRGGBB -> use provided alpha
            Ok(hex_value)
        }
    };
    
    // Default colors if custom_properties are not defined or keys are missing
    let default_text_color = 0xFF_D8DEE9; // Light grey/white (Nord Polar Night text)
    let default_secondary_text_color = 0xFF_4C566A; // Darker grey (Nord Frost text)
    let default_panel_bg_color = 0x80_3B4252; // Semi-transparent dark grey (Nord Polar Night panel)
    let default_hover_bg_color = 0x20_88C0D0; // Semi-transparent accent (Nord Frost hover)
    let default_selected_text_color = 0xFF_2E3440; // Usually dark for light accent, or light for dark accent


    let background_color = parse_hex_color(&config_theme.background, 0xFF)?;
    let accent_color = parse_hex_color(&config_theme.accent, 0xFF)?;
    
    let mut custom_props_map = config_theme.custom_properties.clone().unwrap_or_default();

    let text_color = custom_props_map.get("text_dark") // Assuming dark theme base for now
        .map_or(Ok(default_text_color), |s| parse_hex_color(s, 0xFF))?;
    let secondary_text_color = custom_props_map.get("secondary_text_dark")
        .map_or(Ok(default_secondary_text_color), |s| parse_hex_color(s, 0xFF))?;
    let panel_bg_color = custom_props_map.get("panel_background_dark")
        .map_or(Ok(default_panel_bg_color), |s| parse_hex_color(s, 0x80))?; // Default some alpha
    let hover_bg_color = custom_props_map.get("hover_bg_dark")
        .map_or(Ok(default_hover_bg_color), |s| parse_hex_color(s, 0x20))?;
    let selected_text_color = custom_props_map.get("selected_text_dark")
        .map_or(Ok(default_selected_text_color), |s| parse_hex_color(s, 0xFF))?;


    // --- Load Fonts ---
    #[cfg(feature = "font_rendering")]
    let primary_font: Option<Arc<Font>> = {
        if let Some(font_path_str) = &config_theme.font {
            if !font_path_str.is_empty() {
                logger::info!("[Theme] Loading primary font from: {}", font_path_str);
                // Font path could be:
                // 1. Relative to a known /LBL/fonts/ directory on a volume.
                // 2. An absolute path on a specific volume (e.g., "ESP:/EFI/LBL/fonts/Inter.ttf").
                // This requires robust path resolution by FsManager or here.
                // For simplicity, assume FsManager searches for it.
                
                // Try to find font on any mounted volume
                let mut loaded_font_data: Option<Vec<u8>> = None;
                for volume in fs_manager.list_mounted_volumes() {
                    // Try common font paths or the direct path if it's absolute-like
                    let paths_to_try = [
                        font_path_str.clone(), // Direct path
                        format!("/LBL/fonts/{}", font_path_str),
                        format!("/boot/lbl/fonts/{}", font_path_str),
                    ];
                    for path_attempt in paths_to_try {
                        match fs_manager.read_file(&volume.id, &path_attempt) {
                            Ok(font_data) => {
                                logger::info!("[Theme] Found font '{}' on volume '{}'", path_attempt, volume.id);
                                loaded_font_data = Some(font_data);
                                break;
                            }
                            Err(FilesystemError::NotFound) => continue,
                            Err(e) => return Err(ThemeError::ResourceNotFound(font_path_str.clone(), e)),
                        }
                    }
                    if loaded_font_data.is_some() { break; }
                }

                if let Some(font_data) = loaded_font_data {
                    match Font::from_bytes(font_data, FontSettings::default()) {
                        Ok(font) => Some(Arc::new(font)),
                        Err(e_str) => {
                            // fontdue returns &'static str, convert to String for ThemeError
                            let e_string = String::from(e_str);
                            logger::error!("[Theme] Failed to parse font '{}': {}", font_path_str, e_string);
                            return Err(ThemeError::FontParseFailed(e_string));
                        }
                    }
                } else {
                    logger::error!("[Theme] Primary font file not found: {}", font_path_str);
                    return Err(ThemeError::FontLoadFailed(font_path_str.clone()));
                }
            } else { None }
        } else { None }
    };
    #[cfg(not(feature = "font_rendering"))]
    let primary_font: Option<Arc<Font>> = None; // No font if feature is off


    let applied_theme = AppliedTheme {
        name: "Default".to_string(), // TODO: Allow naming themes in config
        background_color,
        accent_color,
        text_color,
        secondary_text_color,
        panel_bg_color,
        hover_bg_color,
        selected_text_color,
        custom_props: custom_props_map,
        #[cfg(feature = "font_rendering")]
        primary_font,
    };

    unsafe {
        CURRENT_THEME = Some(applied_theme);
    }

    logger::info!("[Theme] Theme loaded and applied.");
    Ok(())
}

/// Retrieves a reference to the currently active theme.
/// Panics if `load_theme` has not been called successfully.
#[cfg(feature = "with_alloc")]
pub fn current_theme() -> &'static AppliedTheme {
    unsafe { CURRENT_THEME.as_ref().expect("Theme not loaded or load_theme failed.") }
}

// -- no_alloc stubs --
#[cfg(not(feature = "with_alloc"))]
pub struct AppliedTheme; // Dummy struct

#[cfg(not(feature = "with_alloc"))]
pub fn load_theme(
    _config_theme: &ConfigTheme, // ConfigTheme would also be simpler for no_alloc
    _fs_manager: &FilesystemManager,
) -> Result<(), ThemeError> {
    logger::info!("[Theme] load_theme (no_alloc - STUBBED).");
    // A no_alloc theme would likely use fixed color palettes and a pre-rendered bitmap font.
    // unsafe { CURRENT_THEME = Some(AppliedTheme); } // If CURRENT_THEME adapted for no_alloc
    Ok(())
}

#[cfg(not(feature = "with_alloc"))]
pub fn current_theme() -> &'static AppliedTheme {
    // unsafe { CURRENT_THEME.as_ref().unwrap() }
    // For now, just return a static dummy instance
    static DUMMY_NO_ALLOC_THEME: AppliedTheme = AppliedTheme;
    &DUMMY_NO_ALLOC_THEME
}