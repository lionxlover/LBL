// Lionbootloader Core - FAT32 Filesystem Driver (Stub)
// File: core/src/fs/fat32.rs

#![cfg(all(feature = "fs_fat32", feature = "with_alloc"))] // Ensure this file is compiled only when features are enabled

use alloc::{boxed::Box, string::String, vec::Vec};

use crate::fs::interface::{
    DirectoryEntry, EntryType, FileMetadata, FileSystemDriver, FileSystemInstance, FilesystemError,
};
use crate::hal::Device as HalDevice;
use crate::logger;

// Placeholder for BlockIo from HalDevice
// In a real driver, this would be an abstraction to read sectors from the device.
// pub trait BlockAccess {
//     fn read_sector(&self, sector: u64, buffer: &mut [u8]) -> Result<(), FilesystemError>;
//     fn sector_size(&self) -> u32;
// }

pub struct Fat32Driver;

impl Fat32Driver {
    pub fn new() -> Self {
        Fat32Driver
    }
}

impl FileSystemDriver for Fat32Driver {
    fn name(&self) -> String {
        "FAT32".to_string()
    }

    /// Detects if the device might contain a FAT32 filesystem.
    /// This usually involves reading the first sector (Boot Sector) and checking signatures.
    fn detect(&self, device: &HalDevice /*, block_io: &dyn BlockIo */) -> bool {
        logger::debug!("[FAT32 Driver] Detecting on device: {}", device.name);
        // TODO: Implement actual FAT32 detection logic
        // 1. Get a BlockIo interface for the device.
        // 2. Read the first sector (LBA 0).
        // 3. Parse the Boot Parameter Block (BPB).
        // 4. Check for FAT32 specific signatures/values:
        //    - Bytes 510-511 should be 0x55 0xAA.
        //    - FileSystemType string at offset 0x52 (for FAT32) should be "FAT32   ".
        //    - Check other BPB sanity.
        // For this stub, let's assume it's not FAT32 by default.
        // In a test environment, you might make it always return true for a specific device name.
        false // Stub: always returns false
    }

    /// Mounts a FAT32 filesystem from the given device.
    fn mount(
        &self,
        device: &HalDevice,
        // block_io: Box<dyn BlockIo>,
        volume_id: &str,
        read_only: bool,
    ) -> Result<Box<dyn FileSystemInstance>, FilesystemError> {
        logger::info!(
            "[FAT32 Driver] Attempting to mount device: {} as FAT32 (Volume ID: {})",
            device.name,
            volume_id
        );

        if !read_only {
            logger::warn!("[FAT32 Driver] Read-write mount requested but stub is read-only.");
            // return Err(FilesystemError::Unsupported); // Or proceed as read-only
        }

        // TODO: Implement actual FAT32 mount logic:
        // 1. Thoroughly parse BPB and FSInfo sector.
        // 2. Calculate FAT start, data region start, root directory cluster.
        // 3. Store necessary metadata in Fat32Instance.
        // 4. Perform sanity checks.

        // For this stub, we'll return a dummy instance if preconditions were met.
        // Assuming detection passed and mount logic is successful for the stub:
        Ok(Box::new(Fat32Instance {
            volume_id: volume_id.to_string(),
            device_name: device.name.clone(),
            // block_io, // Store the block I/O interface
            // fat_type: FatType::Fat32,
            // sectors_per_cluster: bpb.sectors_per_cluster,
            // root_dir_first_cluster: bpb.root_cluster,
            // fat_lba_start: ...,
            // data_lba_start: ...,
            read_only: true, // Stub is always read-only
        }))
    }
}

struct Fat32Instance {
    volume_id: String,
    #[allow(dead_code)] // Will be used with BlockIo
    device_name: String,
    // block_io: Box<dyn BlockIo>,
    // fat_type: FatType,
    // sectors_per_cluster: u8,
    // root_dir_first_cluster: u32,
    // fat_lba_start: u64,
    // data_lba_start: u64,
    read_only: bool,
}

impl FileSystemInstance for Fat32Instance {
    fn volume_id(&self) -> &str {
        &self.volume_id
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>, FilesystemError> {
        logger::debug!("[FAT32 Instance] Reading file: {} on volume {}", path, self.volume_id);
        // TODO: Implement file reading:
        // 1. Parse path, traverse directories to find the file's directory entry.
        //    - Start from root_dir_first_cluster.
        //    - For each path component, search the current directory's cluster chain.
        // 2. Get the file's starting cluster and size from its directory entry.
        // 3. Read the file's cluster chain from the FAT.
        // 4. Read data from corresponding data region clusters.
        // 5. Concatenate into Vec<u8>.
        Err(FilesystemError::NotImplemented) // Stub
    }

    fn list_directory(&self, path: &str) -> Result<Vec<DirectoryEntry>, FilesystemError> {
        logger::debug!("[FAT32 Instance] Listing directory: {} on volume {}", path, self.volume_id);
        // TODO: Implement directory listing:
        // 1. Parse path, traverse to the target directory cluster.
        // 2. Read all directory entries from the target directory's cluster chain.
        // 3. Handle Long File Names (LFNs) if supported.
        // 4. Convert to Vec<DirectoryEntry>.
        let mut entries = Vec::new();
        if path == "/" || path == "" { // Stub: fake root directory listing
            entries.push(DirectoryEntry { name: "BOOT".to_string(), entry_type: EntryType::Directory });
            entries.push(DirectoryEntry { name: "LBL".to_string(), entry_type: EntryType::Directory });
            entries.push(DirectoryEntry { name: "KERNEL.ELF".to_string(), entry_type: EntryType::File });
            entries.push(DirectoryEntry { name: "CONFIG.JSON".to_string(), entry_type: EntryType::File });
            return Ok(entries);
        }
        Err(FilesystemError::NotFound) // Stub
    }

    fn metadata(&self, path: &str) -> Result<FileMetadata, FilesystemError> {
        logger::debug!("[FAT32 Instance] Getting metadata for: {} on volume {}", path, self.volume_id);
        // TODO: Implement metadata retrieval:
        // 1. Traverse to the parent directory of the path.
        // 2. Find the directory entry for the target file/directory.
        // 3. Extract metadata (name, type, size, timestamps from short 8.3 entry or LFN).
        if (path == "/KERNEL.ELF" || path == "KERNEL.ELF") { // Stub for a file in root
            return Ok(FileMetadata {
                name: "KERNEL.ELF".to_string(),
                entry_type: EntryType::File,
                size: 1024 * 1024 * 2, // 2MB dummy size
                created_time: None,
                modified_time: None,
                accessed_time: None,
            });
        }
        Err(FilesystemError::NotFound) // Stub
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }
}

// Helper enums and structs for FAT parsing, e.g.:
// enum FatType { Fat12, Fat16, Fat32 }
// struct BiosParameterBlock { ... }