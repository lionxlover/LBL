// Lionbootloader Core - Plugin System
// File: core/src/plugins.rs

#[cfg(feature = "with_alloc")]
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};

use crate::logger;
// To load filesystem plugins, we'd need access to FilesystemManager
// use crate::fs::manager::FilesystemManager;
// use crate::fs::interface::FileSystemDriver;

/// Represents a loaded plugin.
#[cfg(feature = "with_alloc")]
#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    // path: String, // Path from where it was loaded
    // handle: Option<libloading::Library>, // For dynamic libs, requires 'std' typically
    // Other metadata
}

#[cfg(not(feature = "with_alloc"))]
#[derive(Debug, Clone, Copy)]
pub struct Plugin { // Simplified for no_alloc
    pub id: u32,
    pub plugin_type: PluginType,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    Filesystem, // .lblfs
    Driver,     // e.g., for specific hardware not in core HAL
    Theme,      // Unlikely a code plugin, more like data
    Security,   // e.g., custom signature verifier
    Other,
}

#[derive(Debug)]
pub enum PluginError {
    NotFound(String),
    LoadFailed(String),
    InvalidFormat(String),
    InitFailed(String),
    UnsupportedType(String),
    DependencyMissing(String),
}

/// Manages the loading and registration of plugins.
#[cfg(feature = "with_alloc")]
pub struct PluginManager {
    loaded_plugins: BTreeMap<String, Plugin>, // Keyed by plugin name or path
                                             // fs_manager_ref: Option<&'a mut FilesystemManager<'a>> // If plugins need to register with managers
}

#[cfg(not(feature = "with_alloc"))]
pub struct PluginManager {
    // Simpler storage for no_alloc if plugins are statically known or very limited
    _placeholder: u8,
}


#[cfg(feature = "with_alloc")]
impl PluginManager {
    pub fn new(/* fs_manager_ref: Option<&'a mut FilesystemManager<'a>> */) -> Self {
        PluginManager {
            loaded_plugins: BTreeMap::new(),
            // fs_manager_ref,
        }
    }

    /// Loads a list of plugins specified in the configuration.
    ///
    /// `plugin_names_or_paths` would be the list from LblConfig.plugins.
    /// `fs_manager` is needed if loading .lblfs plugins to read them.
    /// `target_fs_manager` is where loaded FS drivers would register.
    pub fn load_configured_plugins(
        &mut self,
        plugin_names_or_paths: &[String],
        _fs_reader: &crate::fs::manager::FilesystemManager, // Used to read plugin files
        _target_fs_manager: &mut crate::fs::manager::FilesystemManager, // Used by FS plugins to register
    ) {
        logger::info!("[PluginManager] Loading configured plugins...");
        for plugin_path_str in plugin_names_or_paths {
            logger::debug!("[PluginManager] Attempting to load plugin: {}", plugin_path_str);
            match self.load_plugin_from_path(plugin_path_str, _fs_reader, _target_fs_manager) {
                Ok(plugin_name) => {
                    logger::info!("[PluginManager] Successfully loaded plugin: {} (from {})", plugin_name, plugin_path_str);
                }
                Err(e) => {
                    logger::error!("[PluginManager] Failed to load plugin '{}': {:?}", plugin_path_str, e);
                }
            }
        }
    }

    /// Loads a single plugin from a given path.
    /// The path is relative to a known plugin directory or an absolute path on a mounted volume.
    fn load_plugin_from_path(
        &mut self,
        plugin_path_str: &str,
        _fs_reader: &crate::fs::manager::FilesystemManager, // To read the plugin file
        _target_fs_manager: &mut crate::fs::manager::FilesystemManager, // To register FS plugins
    ) -> Result<String, PluginError> {
        // 1. Determine the type of plugin from its extension or metadata.
        //    For LBL, `.lblfs` indicates a filesystem plugin.
        if plugin_path_str.ends_with(".lblfs") {
            self.load_filesystem_plugin(plugin_path_str, _fs_reader, _target_fs_manager)
        } else {
            logger::warn!("[PluginManager] Unknown plugin type for: {}", plugin_path_str);
            Err(PluginError::UnsupportedType(plugin_path_str.to_string()))
        }
    }

    fn load_filesystem_plugin(
        &mut self,
        plugin_path_str: &str,
        _fs_reader: &crate::fs::manager::FilesystemManager,
        _target_fs_manager: &mut crate::fs::manager::FilesystemManager,
    ) -> Result<String, PluginError> {
        logger::info!("[PluginManager] Loading filesystem plugin: {} (STUBBED)", plugin_path_str);

        // THIS IS A MAJOR STUB. Real dynamic code loading is very complex.
        //
        // Option A: Statically Linked "Plugins" (Compile-time)
        // If .lblfs are not dynamically loaded code but rather data files that configure statically
        // linked driver modules, or if they are known at compile time and linked in.
        // In this case, "loading" means activating/configuring a pre-existing module.
        // Example: if plugin_path_str == "lblfs_ext4_static.lblfs", then:
        // if _target_fs_manager.has_static_driver("ext4_static") {
        //     _target_fs_manager.enable_static_driver("ext4_static");
        //     let plugin_name = "ext4_static_filesystem_driver".to_string();
        //     self.loaded_plugins.insert(plugin_name.clone(), Plugin {
        //         name: plugin_name.clone(), version: "0.1-static".to_string(), plugin_type: PluginType::Filesystem,
        //     });
        //     return Ok(plugin_name);
        // }

        // Option B: True Dynamic Code Loading (Very Advanced for no_std)
        // This would require:
        //  1. Reading the plugin file (e.g., an ELF relocatable object or custom format)
        //     `let plugin_data = _fs_reader.read_file(volume_id_of_plugins_dir, plugin_path_str)?;`
        //  2. A custom loader for this plugin format:
        //     - Allocate executable memory.
        //     - Perform relocations against the core LBL binary (if needed).
        //     - Resolve symbols for a predefined plugin API (e.g., `lbl_plugin_init()`).
        //  3. Calling the plugin's init function, which would return a `Box<dyn FileSystemDriver>`.
        //     `let driver_instance = call_plugin_init_function(loaded_code_ptr)?;`
        //  4. Registering this driver with `_target_fs_manager`.
        //     `_target_fs_manager.register_driver(driver_instance);`
        //
        //  This is akin to `libloading` crate functionality but in `no_std`. Extremely complex.
        //  It also has major security implications.

        // For now, as a pure stub, we just log and pretend it worked if the name is known.
        let plugin_name_part = plugin_path_str.trim_end_matches(".lblfs");
        let plugin_display_name = format!("{}_driver", plugin_name_part);

        // Simulate registration of a known, built-in (but feature-flagged) driver
        // This part bridges the "plugin" concept with compile-time features for now.
        match plugin_name_part {
            #[cfg(feature = "fs_ext4")]
            "lblfs_ext4" => {
                // Conceptual: if ext4 was a "plugin" that's actually compiled in via feature:
                // _target_fs_manager.register_driver(Box::new(crate::fs::ext4::Ext4Driver::new()));
                logger::info!("[PluginManager] Conceptual registration of ext4 (if it were a plugin).");
            }
            #[cfg(feature = "fs_btrfs")]
            "lblfs_btrfs" => {
                logger::info!("[PluginManager] Conceptual registration of btrfs (if it were a plugin).");
            }
            _ => {
                logger::warn!("[PluginManager] Filesystem plugin '{}' is not a recognized built-in staged as plugin.", plugin_display_name);
                return Err(PluginError::NotFound(plugin_display_name));
            }
        }

        self.loaded_plugins.insert(
            plugin_display_name.clone(),
            Plugin {
                name: plugin_display_name.clone(),
                version: "0.1.0-stub".to_string(),
                plugin_type: PluginType::Filesystem,
            },
        );
        Ok(plugin_display_name)
    }

    pub fn list_loaded_plugins(&self) -> Vec<&Plugin> {
        self.loaded_plugins.values().collect()
    }
}

#[cfg(not(feature = "with_alloc"))]
impl PluginManager {
    pub fn new() -> Self {
        PluginManager { _placeholder: 0 }
    }

    pub fn load_configured_plugins(
        &mut self,
        _plugin_names_or_paths: &[String], // String usage here is problematic for no_alloc config
        // ...
    ) {
        logger::info!("[PluginManager] load_configured_plugins (no_alloc - STUBBED, does nothing).");
        // Plugin loading in no_alloc is typically very limited, often to statically linked modules
        // whose activation might be controlled by a simple config flag.
    }
}