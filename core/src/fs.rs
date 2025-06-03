// Lionbootloader Core - Filesystem Module
// File: core/src/fs.rs

#[cfg(feature = "with_alloc")]
use alloc::{string::String, vec::Vec};

use crate::hal::HalServices; // To access storage devices
use crate::logger;

// Filesystem driver implementations
#[cfg(feature = "fs_fat32")]
pub mod fat32;
#[cfg(feature = "fs_ext4")]
pub mod ext4; // Placeholder
#[cfg(feature = "fs_ntfs")]
pub mod ntfs; // Placeholder
#[cfg(feature = "fs_btrfs")]
pub mod btrfs; // Placeholder

// Filesystem plugin manager and generic interface
pub mod interface;
pub mod manager;

/// Represents a mounted filesystem volume.
#[cfg(feature = "with_alloc")]
#[derive(Debug, Clone)]
pub struct Volume {
    pub id: String,                           // Unique identifier (e.g., UUID, label, or LBL-generated)
    pub fs_type: String,                      // Filesystem type (e.g., "FAT32", "ext4")
    pub label: Option<String>,                // Volume label, if any
    pub device_path: String,                  // Path/ID of the underlying block device
    pub mount_point: Option<String>,          // If LBL supports a VFS-like structure (optional)
    pub capacity_bytes: u64,
    pub free_space_bytes: u64,                // Might be hard to get for all read-only FS drivers
    pub is_read_only: bool,
    // Internal handle for the filesystem driver to interact with this volume
    // pub(crate) driver_handle: Box<dyn interface::FileSystemInstance>, // This makes Volume not Clone easily.
    // Instead, manager can hold driver_handle and Volume just refers to it by id.
}

// Simplified Volume for no_alloc
#[cfg(not(feature = "with_alloc"))]
#[derive(Debug, Clone, Copy)]
pub struct Volume {
    pub id: u32, // Simple numeric ID
    pub fs_type_id: u8, // Enum representing FS type
    // block_device_id: u64, // ID of the underlying HAL device
    // ... other simple fields
    pub is_read_only: bool,
}


/// Initializes the filesystem subsystem.
/// This primarily means setting up the FilesystemDriverManager.
pub fn initialize_manager(
    hal_services: &HalServices,
    // plugin_paths: &[&str] // Paths to .lblfs plugin files, to be loaded by manager
) -> manager::FilesystemManager {
    logger::info!("[FS] Initializing Filesystem Manager...");
    let mut fs_manager = manager::FilesystemManager::new(hal_services);

    // Register built-in filesystem drivers based on features
    #[cfg(all(feature = "fs_fat32", feature = "with_alloc"))]
    {
        logger::info!("[FS] Registering built-in FAT32 driver...");
        // The FAT32 driver would implement the `FileSystemDriver` trait.
        // fs_manager.register_driver(Box::new(fat32::Fat32Driver::new()));
        // For now, this is conceptual until `fat32.rs` provides `Fat32Driver`.
    }
    #[cfg(all(feature = "fs_ext4", feature = "with_alloc"))]
    {
        logger::info!("[FS] Registering built-in ext4 driver (stub)...");
        // fs_manager.register_driver(Box::new(ext4::Ext4Driver::new()));
    }
    // ... register other built-in drivers (NTFS, Btrfs) similarly.

    // TODO: Load external .lblfs plugins if specified in config and supported.
    // for path in plugin_paths {
    //     match fs_manager.load_plugin(path) {
    //         Ok(()) => logger::info!("[FS] Loaded plugin: {}", path),
    //         Err(e) => logger::error!("[FS] Failed to load plugin {}: {:?}", path, e),
    //     }
    // }

    logger::info!("[FS] Filesystem Manager initialized.");
    fs_manager
}

/// Attempts to mount all suitable storage devices found by HAL.
/// This would iterate over storage devices, try to detect filesystems, and mount them.
#[cfg(feature = "with_alloc")]
pub fn mount_all_volumes(
    fs_manager: &mut manager::FilesystemManager,
    hal_services: &HalServices,
) {
    logger::info!("[FS] Attempting to mount all available volumes...");
    let storage_devices = hal_services
        .device_manager
        .get_devices_by_type(crate::hal::DeviceType::Storage);

    for device in storage_devices {
        logger::debug!("[FS] Probing device for filesystems: {} ({})", device.name, device.id);
        match fs_manager.mount_volume_from_device(device) {
            Ok(volume) => {
                logger::info!(
                    "[FS] Successfully mounted volume: ID='{}', Type='{}', Label='{:?}' on device '{}'",
                    volume.id,
                    volume.fs_type,
                    volume.label,
                    device.name
                );
            }
            Err(e) => {
                logger::warn!(
                    "[FS] Failed to mount/recognize filesystem on device '{}': {:?}",
                    device.name,
                    e
                );
            }
        }
    }
}

/// Reads a file from a mounted volume into a Vec<u8>.
/// Path is relative to the root of the volume.
#[cfg(feature = "with_alloc")]
pub fn read_file_to_vec(
    fs_manager: &manager::FilesystemManager,
    volume_id: &str, // ID of the volume as returned by mount_volume_from_device
    path: &str,
) -> Result<Vec<u8>, interface::FilesystemError> {
    logger::debug!("[FS] Reading file: volume='{}', path='{}'", volume_id, path);
    fs_manager.read_file(volume_id, path)
}

// A simplified file read for no_alloc might read into a pre-allocated buffer.
#[cfg(not(feature = "with_alloc"))]
pub fn read_file_to_buffer(
    _fs_manager: &manager::FilesystemManager,
    _volume_id: u32,
    _path: &str, // Paths might be simple numeric IDs or fixed length strings
    _buffer: &mut [u8],
) -> Result<usize, interface::FilesystemError> {
    logger::debug!("[FS] Reading file (no_alloc), volume_id={}, path='{}'", _volume_id, _path);
    // ... implementation for no_alloc file reading ...
    Err(interface::FilesystemError::NotImplemented)
}


/// Lists directory contents.
#[cfg(feature = "with_alloc")]
pub fn list_directory(
    fs_manager: &manager::FilesystemManager,
    volume_id: &str,
    path: &str,
) -> Result<Vec<interface::DirectoryEntry>, interface::FilesystemError> {
    logger::debug!("[FS] Listing directory: volume='{}', path='{}'", volume_id, path);
    fs_manager.list_directory(volume_id, path)
}

// Define common error types for FS operations, possibly in interface.rs
// pub use interface::FilesystemError;
// pub use interface::DirectoryEntry;
// pub use interface::FileMetadata;