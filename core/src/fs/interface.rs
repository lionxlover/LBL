// Lionbootloader Core - Filesystem Interface
// File: core/src/fs/interface.rs

#[cfg(feature = "with_alloc")]
use alloc::{boxed::Box, string::String, vec::Vec};

use crate::hal::Device as HalDevice; // Represents a block device from HAL

/// Common error type for all filesystem operations.
#[derive(Debug)]
pub enum FilesystemError {
    NotFound,
    PermissionDenied,
    IoError,
    NotADirectory,
    NotAFile,
    AlreadyExists,
    InvalidInput,
    Unsupported,
    CorruptedFs,
    NoSpace,
    DriverNotRegistered(String),
    VolumeNotMounted(String),
    PluginLoadError(String),
    NotImplemented,
    Other(String),
}

#[cfg(not(feature = "with_alloc"))]
#[derive(Debug, Clone, Copy)]
pub enum FilesystemErrorNoAlloc { // Simpler error for no_alloc
    NotFound,
    PermissionDenied,
    IoError,
    NotADirectory,
    NotAFile,
    Unsupported,
    CorruptedFs,
    NotImplemented,
    Other,
}


/// Metadata for a file or directory.
#[cfg(feature = "with_alloc")]
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub name: String,
    pub entry_type: EntryType,
    pub size: u64,         // Size in bytes (0 for directories conventionally)
    pub created_time: Option<u64>, // Timestamp (e.g., seconds since epoch)
    pub modified_time: Option<u64>,
    pub accessed_time: Option<u64>,
    // pub permissions: u16, // e.g., POSIX style permissions
}

#[cfg(not(feature = "with_alloc"))]
#[derive(Debug, Clone, Copy)]
pub struct FileMetadata { // name would be fixed-size or just not part of this struct
    pub entry_type: EntryType,
    pub size: u64,
    // Timestamps might be omitted or simplified
}


/// Type of a directory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Other,
}

/// Represents an entry within a directory.
#[cfg(feature = "with_alloc")]
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub entry_type: EntryType,
    // pub metadata: Option<FileMetadata>, // Optionally include full metadata
}

#[cfg(not(feature = "with_alloc"))]
#[derive(Debug, Clone, Copy)]
pub struct DirectoryEntry { // Again, name might be tricky
    // name_id: u32, // e.g. an ID into a string table or fixed array
    pub entry_type: EntryType,
}


/// Trait for a filesystem driver.
/// Each filesystem type (FAT32, ext4, etc.) will have an implementation of this trait.
/// The driver is responsible for recognizing and mounting a filesystem on a block device.
#[cfg(feature = "with_alloc")]
pub trait FileSystemDriver: Send + Sync {
    /// Returns the name of the filesystem type this driver supports (e.g., "FAT32").
    fn name(&self) -> String;

    /// Probes the given block device to see if it contains a filesystem
    /// recognizable by this driver.
    /// Should be relatively quick and non-destructive.
    /// `device` is the HAL device representing the storage.
    fn detect(&self, device: &HalDevice /*, block_io: &dyn BlockIo */) -> bool;

    /// Mounts the filesystem on the given block device.
    /// If successful, returns a `FileSystemInstance` that can be used for file operations.
    /// `device` is the HAL device.
    /// `volume_id` is a unique ID assigned by the FS manager for this mount.
    fn mount(
        &self,
        device: &HalDevice,
        // block_io: Box<dyn BlockIo>, // Abstract block I/O for the device
        volume_id: &str,
        read_only: bool,
    ) -> Result<Box<dyn FileSystemInstance>, FilesystemError>;
}

// TODO: Define a BlockIo trait for reading/writing blocks from a HalDevice.
// pub trait BlockIo {
//     fn read_blocks(&self, start_lba: u64, num_blocks: u32, buffer: &mut [u8]) -> Result<(), FilesystemError>;
//     // fn write_blocks(...); // If write support is needed
//     fn block_size(&self) -> u32; // e.g. 512 or 4096
//     fn total_blocks(&self) -> u64;
// }


/// Trait for an instance of a mounted filesystem.
/// This is what the `FilesystemManager` uses to perform operations on a mounted volume.
#[cfg(feature = "with_alloc")]
pub trait FileSystemInstance: Send + Sync {
    /// Returns the unique ID of this mounted volume.
    fn volume_id(&self) -> &str;

    /// Reads the entire content of a file into a byte vector.
    /// `path` is relative to the root of this filesystem instance.
    fn read_file(&self, path: &str) -> Result<Vec<u8>, FilesystemError>;

    /// Writes data to a file. Creates the file if it doesn't exist, overwrites otherwise.
    /// (Optional, many bootloader FS drivers are read-only).
    // fn write_file(&self, path: &str, data: &[u8]) -> Result<(), FilesystemError>;

    /// Lists the contents of a directory.
    /// `path` is relative to the root of this filesystem instance.
    fn list_directory(&self, path: &str) -> Result<Vec<DirectoryEntry>, FilesystemError>;

    /// Retrieves metadata for a file or directory.
    fn metadata(&self, path: &str) -> Result<FileMetadata, FilesystemError>;

    /// Checks if a path exists.
    fn exists(&self, path: &str) -> Result<bool, FilesystemError> {
        match self.metadata(path) {
            Ok(_) => Ok(true),
            Err(FilesystemError::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Returns true if the filesystem instance is mounted read-only.
    fn is_read_only(&self) -> bool;

    /// Gets filesystem-specific stats (capacity, free space).
    /// (Optional, as this can be complex for some FS types or read-only drivers).
    // fn fs_stats(&self) -> Result<FsStats, FilesystemError>;

    // Unmounts the filesystem. (Optional, might not be needed if LBL doesn't explicitly unmount)
    // fn unmount(self: Box<Self>) -> Result<(), FilesystemError>;
}

// Simplified traits for no_alloc would avoid Box<dyn Trait> and Vec/String.
// They might take pre-allocated buffers and return data via iterators or fill slices.
// This is a significant design change for no_alloc.
// For now, the above traits primarily target `alloc` environments.
// If strict_no_alloc FS support is needed, these traits would need conditional compilation
// or separate no_alloc versions. E.g.:
/*
#[cfg(not(feature = "with_alloc"))]
pub trait FileSystemDriverNoAlloc {
    // ...
    fn mount_no_alloc(&self, device: &HalDevice, read_only: bool) -> Result<impl FileSystemInstanceNoAlloc, FilesystemErrorNoAlloc>;
}

#[cfg(not(feature = "with_alloc"))]
pub trait FileSystemInstanceNoAlloc {
    fn read_file_no_alloc(&self, path_id: u32, buffer: &mut [u8]) -> Result<usize, FilesystemErrorNoAlloc>;
    // ...
}
*/


// pub struct FsStats {
//     pub total_bytes: u64,
//     pub free_bytes: u64,
//     pub block_size: u32,
// }