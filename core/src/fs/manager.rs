// Lionbootloader Core - Filesystem Manager
// File: core/src/fs/manager.rs

#[cfg(feature = "with_alloc")]
use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use crate::fs::interface::{
    DirectoryEntry, FileMetadata, FileSystemDriver, FileSystemInstance, FilesystemError,
};
use crate::hal::{Device as HalDevice, HalServices};
use crate::logger;

// Placeholder for BlockIo when we define it.
// For now, drivers will have to assume direct access or a HAL-provided mechanism.
// use crate::fs::interface::BlockIo;

/// Manages registered filesystem drivers and mounted filesystem instances (volumes).
#[cfg(feature = "with_alloc")]
pub struct FilesystemManager<'a> {
    hal_services: &'a HalServices, // For accessing block devices
    drivers: Vec<Box<dyn FileSystemDriver>>,
    mounted_volumes: BTreeMap<String, MountedVolumeInfo>,
    next_volume_numeric_id: u32,
}

#[cfg(feature = "with_alloc")]
struct MountedVolumeInfo {
    instance: Box<dyn FileSystemInstance>,
    lbl_volume_info: crate::fs::Volume, // The public Volume struct
}


#[cfg(feature = "with_alloc")]
impl<'a> FilesystemManager<'a> {
    /// Creates a new FilesystemManager.
    pub fn new(hal_services: &'a HalServices) -> Self {
        FilesystemManager {
            hal_services,
            drivers: Vec::new(),
            mounted_volumes: BTreeMap::new(),
            next_volume_numeric_id: 1,
        }
    }

    /// Registers a new filesystem driver.
    pub fn register_driver(&mut self, driver: Box<dyn FileSystemDriver>) {
        logger::info!("[FS Manager] Registering driver: {}", driver.name());
        self.drivers.push(driver);
    }

    /// Generates a unique string ID for a new volume.
    fn generate_volume_id(&mut self, device_name: &str, fs_type_name: &str) -> String {
        let id_num = self.next_volume_numeric_id;
        self.next_volume_numeric_id += 1;
        // Example: "vol-1-sda1-fat32"
        format!("vol-{}-{}-{}", id_num, device_name.replace("/", "_"), fs_type_name.to_lowercase())
    }

    /// Attempts to mount a filesystem on a given HAL device.
    /// It tries all registered drivers until one successfully detects and mounts.
    pub fn mount_volume_from_device(
        &mut self,
        device: &HalDevice,
    ) -> Result<crate::fs::Volume, FilesystemError> {
        logger::debug!("[FS Manager] Attempting to mount device ID: {}, Name: '{}'", device.id, device.name);

        // TODO: Create a BlockIo implementation for the HalDevice.
        // This part is crucial and currently missing.
        // let block_io = self.hal_services.get_block_io_for_device(device.id)?;
        // For now, drivers will have to assume they can get this from HalDevice or a global service.

        for driver in &self.drivers {
            // logger::trace!("[FS Manager] Trying driver: {}", driver.name());
            if driver.detect(device /*, &block_io */) {
                logger::info!(
                    "[FS Manager] Driver '{}' detected filesystem on device '{}'",
                    driver.name(),
                    device.name
                );

                // For now, assume read-only mounting
                let read_only = true;
                let volume_id_str = self.generate_volume_id(&device.name, &driver.name());

                match driver.mount(device /*, block_io.clone_for_driver() */, &volume_id_str, read_only) {
                    Ok(instance) => {
                        logger::info!(
                            "[FS Manager] Successfully mounted volume '{}' (type: {}) on device '{}'",
                            volume_id_str,
                            driver.name(),
                            device.name
                        );
                        
                        let lbl_volume_info = crate::fs::Volume {
                            id: volume_id_str.clone(),
                            fs_type: driver.name(),
                            label: None, // TODO: Get label from instance if possible
                            device_path: device.name.clone(),
                            mount_point: None, // LBL might not use traditional mount points
                            capacity_bytes: 0, // TODO: Get from instance
                            free_space_bytes: 0, // TODO: Get from instance
                            is_read_only: instance.is_read_only(),
                        };

                        self.mounted_volumes.insert(
                            volume_id_str.clone(),
                            MountedVolumeInfo { instance, lbl_volume_info: lbl_volume_info.clone() },
                        );
                        return Ok(lbl_volume_info);
                    }
                    Err(e) => {
                        logger.warn!(
                            "[FS Manager] Driver '{}' failed to mount detected FS on device '{}': {:?}",
                            driver.name(),
                            device.name,
                            e
                        );
                        // Continue to try other drivers if mount fails
                    }
                }
            }
        }
        logger.warn!("[FS Manager] No suitable driver found for device '{}'", device.name);
        Err(FilesystemError::Unsupported)
    }

    /// Retrieves a specific mounted volume instance by its ID.
    fn get_instance(&self, volume_id: &str) -> Result<&dyn FileSystemInstance, FilesystemError> {
        self.mounted_volumes
            .get(volume_id)
            .map(|info| info.instance.as_ref())
            .ok_or_else(|| FilesystemError::VolumeNotMounted(volume_id.to_string()))
    }

    /// Reads the entire content of a file from a specified volume.
    pub fn read_file(
        &self,
        volume_id: &str,
        path: &str,
    ) -> Result<Vec<u8>, FilesystemError> {
        self.get_instance(volume_id)?.read_file(path)
    }

    /// Lists the contents of a directory on a specified volume.
    pub fn list_directory(
        &self,
        volume_id: &str,
        path: &str,
    ) -> Result<Vec<DirectoryEntry>, FilesystemError> {
        self.get_instance(volume_id)?.list_directory(path)
    }

    /// Retrieves metadata for a file or directory on a specified volume.
    pub fn metadata(
        &self,
        volume_id: &str,
        path: &str,
    ) -> Result<FileMetadata, FilesystemError> {
        self.get_instance(volume_id)?.metadata(path)
    }

    /// Checks if a path exists on a specified volume.
    pub fn exists(
        &self,
        volume_id: &str,
        path: &str,
    ) -> Result<bool, FilesystemError> {
        self.get_instance(volume_id)?.exists(path)
    }

    /// Returns a list of all mounted volumes' public info.
    pub fn list_mounted_volumes(&self) -> Vec<crate::fs::Volume> {
        self.mounted_volumes.values().map(|info| info.lbl_volume_info.clone()).collect()
    }

    // TODO: Implement plugin loading for .lblfs files
    // pub fn load_plugin(&mut self, _plugin_path: &str) -> Result<(), FilesystemError> {
    //     logger::info!("[FS Manager] Loading FS plugin (not implemented): {}", _plugin_path);
    //     // This would involve:
    //     // 1. Reading the plugin file (which is itself a filesystem operation, careful with dependencies)
    //     // 2. If it's a dynamic library (.so, .dll):
    //     //    - Using `libloading` crate or similar (requires std or OS services).
    //     //    - Resolving a known symbol that provides a `FileSystemDriver` instance.
    //     // 3. If it's a statically linked plugin format LBL defines:
    //     //    - Parsing the format and instantiating the driver.
    //     // This is complex in a no_std environment.
    //     Err(FilesystemError::NotImplemented)
    // }
}


// If no_alloc is active, FilesystemManager would be very different.
// It might manage a fixed array of statically known FS drivers and mounted instances.
#[cfg(not(feature = "with_alloc"))]
pub struct FilesystemManager<'a> {
    // Placeholder for no_alloc version
    _hal_services: &'a HalServices,
}

#[cfg(not(feature = "with_alloc"))]
impl<'a> FilesystemManager<'a> {
    pub fn new(_hal_services: &'a HalServices) -> Self {
        FilesystemManager { _hal_services }
    }

    // no_alloc versions of methods would be very different, avoiding String, Vec, Box.
    // For example, read_file would take a mutable slice as a buffer.
    // Mounting might return a simple ID for the volume.
}