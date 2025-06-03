// Lionbootloader Core - Boot Executor (Loader)
// File: core/src/loader.rs

#[cfg(feature = "with_alloc")]
use alloc::{string::String, vec::Vec};

use crate::config::schema_types::{BootEntry, BootEntryType};
use crate::fs::manager::FilesystemManager;
#[cfg(feature = "with_alloc")]
use crate::fs::interface::FilesystemError;
use crate::hal::HalServices;
use crate::logger;
use crate::security::SecurityManager; // For signature verification

// Sub-modules for kernel format parsing and architecture adaptation
pub mod arch_adapter;   // Architecture-specific actions (e.g., setting up page tables, CPU mode)
pub mod kernel_formats; // Parsers for ELF, PE, multiboot, etc.

/// Represents errors that can occur during the boot execution phase.
#[derive(Debug)]
pub enum BootExecuteError {
    KernelNotFound(String),
    InitrdNotFound(String),
    FilesystemError(#[cfg(feature = "with_alloc")] FilesystemError, #[cfg(not(feature = "with_alloc"))] &'static str),
    UnsupportedKernelFormat(String),
    KernelLoadFailed(String), // e.g., memory allocation, parsing error
    InitrdLoadFailed(String),
    SignatureVerificationFailed(String),
    ArchitectureAdaptationFailed(String),
    InvalidBootEntryType,
    InternalError(String),
    FirmwareError(String), // Errors from UEFI services etc.
}

#[cfg(feature = "with_alloc")]
impl From<FilesystemError> for BootExecuteError {
    fn from(e: FilesystemError) -> Self {
        BootExecuteError::FilesystemError(e)
    }
}


/// Main function to execute a boot entry.
///
/// This function will:
/// 1. Verify the boot entry's signature (if required and `security_manager` is provided).
/// 2. Locate and load the kernel image.
/// 3. Locate and load the initrd/initramfs (if specified).
/// 4. Prepare the environment for the kernel (via `arch_adapter`).
/// 5. Jump to the kernel's entry point.
///
/// This function should not return if booting is successful.
pub fn execute_boot(
    hal: &HalServices,
    fs_manager: &FilesystemManager,
    security_manager: &SecurityManager,
    entry: &BootEntry,
) -> ! {
    logger::info!("[Loader] Executing boot entry: '{}' (ID: {})", entry.title, entry.id);

    // 1. Security Verification (if applicable)
    if entry.secure {
        logger::info!("[Loader] Performing security verification for entry: {}", entry.id);
        match security_manager.verify_boot_entry(hal, fs_manager, entry) {
            Ok(()) => {
                logger::info!("[Loader] Security verification successful for entry: {}", entry.id);
            }
            Err(sec_err) => {
                logger::error!("[Loader] Security verification FAILED for entry '{}': {:?}", entry.id, sec_err);
                // TODO: Handle failure: UI notification, retry, or fallback.
                // For now, halt or panic. This needs a proper UI feedback loop.
                panic!("Boot security verification failed: {:?}", sec_err); // Placeholder
            }
        }
    } else {
        logger::info!("[Loader] Security verification skipped for entry (not required): {}", entry.id);
    }

    // Determine the volume to load from
    // This logic needs to be more robust: use entry.volume_id, scan if "auto", etc.
    // For simplicity, assume the first mounted volume or a specific one for now.
    #[cfg(feature = "with_alloc")]
    let volume_id_to_use = if let Some(vol_id) = &entry.volume_id {
        vol_id.clone()
    } else {
        // Fallback: try to find the kernel on any mounted volume if no volume_id is specified.
        // This is a simplified approach. A better way would be to intelligently pick or
        // require volume_id for clarity.
        logger::warn!("[Loader] No volume_id specified for entry '{}'. Searching all volumes.", entry.id);
        // This part is tricky: which volume to pick if kernel found on multiple?
        // For now, let's assume we'll find it on the first one where base path exists.
        // Or, this could be an error condition if volume_id is strictly needed.
        // A more robust system would iterate through fs_manager.list_mounted_volumes()
        // and check for the kernel path. We'll assume fs_manager handles this or a default.
        // For a simple stub, pick the first one if available.
        fs_manager.list_mounted_volumes().get(0).map_or_else(
            || {
                logger::error!("[Loader] No volumes mounted, cannot load kernel for '{}'", entry.id);
                panic!("No volumes available to load kernel."); // Placeholder
            },
            |vol| vol.id.clone()
        )
    };

    #[cfg(not(feature = "with_alloc"))]
    let volume_id_to_use = 0u32; // Placeholder for no_alloc

    // Handle different boot entry types
    match entry.entry_type {
        BootEntryType::KernelDirect => {
            load_and_execute_kernel(hal, fs_manager, entry, &volume_id_to_use)
        }
        BootEntryType::UefiChainload => {
            #[cfg(target_os = "uefi")] // Chainloading is primarily a UEFI concept implemented here
            {
                chainload_uefi_application(hal, fs_manager, entry, &volume_id_to_use)
            }
            #[cfg(not(target_os = "uefi"))]
            {
                logger::error!("[Loader] UEFI Chainload not supported on this platform (not UEFI).");
                panic!("UEFI Chainload not supported."); // Placeholder
            }
        }
        BootEntryType::UefiApplication => {
             #[cfg(target_os = "uefi")]
            {
                 // Similar to chainload, but might not be a full OS boot manager
                chainload_uefi_application(hal, fs_manager, entry, &volume_id_to_use)
            }
            #[cfg(not(target_os = "uefi"))]
            {
                logger::error!("[Loader] UEFI Application execution not supported on this platform (not UEFI).");
                panic!("UEFI Application execution not supported."); // Placeholder
            }
        }
        BootEntryType::InternalTool => {
            execute_internal_tool(hal, entry)
        }
    }
}

/// Loads a kernel and initrd (if specified) and executes.
fn load_and_execute_kernel(
    hal: &HalServices,
    fs_manager: &FilesystemManager,
    entry: &BootEntry,
    volume_id: &str,
) -> ! {
    // 2. Load Kernel Image
    logger::info!("[Loader] Loading kernel: {} from volume: {}", entry.kernel, volume_id);
    #[cfg(feature = "with_alloc")]
    let kernel_data = match fs_manager.read_file(volume_id, &entry.kernel) {
        Ok(data) => data,
        Err(e) => {
            logger::error!("[Loader] Failed to read kernel file '{}': {:?}", entry.kernel, e);
            panic!("Kernel file read failed: {:?}", e); // Placeholder
        }
    };
    #[cfg(not(feature = "with_alloc"))]
    let mut kernel_data_buf = [0u8; 1024 * 1024 * 8]; // Example 8MB buffer for kernel
    #[cfg(not(feature = "with_alloc"))]
    let kernel_data_len = match crate::fs::read_file_to_buffer(fs_manager, volume_id_to_use, &entry.kernel, &mut kernel_data_buf) {
        Ok(len) => len,
        Err(e) => {
            logger::error!("[Loader] Failed to read kernel file '{}': {:?}", entry.kernel, e);
            panic!("Kernel file read failed: {:?}", e); // Placeholder
        }
    };
    #[cfg(not(feature = "with_alloc"))]
    let kernel_data A= &kernel_data_buf[..kernel_data_len];


    // Detect kernel type and parse/load it into memory
    // `kernel_info` would contain entry point, memory layout requirements, etc.
    let kernel_info = match kernel_formats::load_kernel(hal, &kernel_data) {
        Ok(info) => info,
        Err(e) => {
            logger::error!("[Loader] Failed to load/parse kernel image '{}': {:?}", entry.kernel, e);
            panic!("Kernel image processing failed: {:?}", e); // Placeholder
        }
    };
    logger::info!("[Loader] Kernel '{}' loaded successfully. Entry point: {:#x}", entry.kernel, kernel_info.entry_point);

    // 3. Load Initrd (if specified)
    let mut initrd_info: Option<kernel_formats::LoadedImageInfo> = None;
    if let Some(initrd_path) = &entry.initrd {
        if !initrd_path.is_empty() {
            logger::info!("[Loader] Loading initrd: {} from volume: {}", initrd_path, volume_id);
            #[cfg(feature = "with_alloc")]
            let initrd_data_bytes = match fs_manager.read_file(volume_id, initrd_path) {
                Ok(data) => data,
                Err(e) => {
                    logger::error!("[Loader] Failed to read initrd file '{}': {:?}", initrd_path, e);
                    panic!("Initrd file read failed: {:?}", e); // Placeholder
                }
            };
            #[cfg(not(feature = "with_alloc"))]
            let mut initrd_data_buf = [0u8; 1024 * 1024 * 32]; // Example 32MB buffer
            #[cfg(not(feature = "with_alloc"))]
            let initrd_data_len = match crate::fs::read_file_to_buffer(fs_manager, volume_id_to_use, initrd_path, &mut initrd_data_buf) {
                Ok(len) => len,
                Err(e) => {
                    logger::error!("[Loader] Failed to read initrd file '{}': {:?}", initrd_path, e);
                    panic!("Initrd file read failed: {:?}", e); // Placeholder
                }
            };
            #[cfg(not(feature = "with_alloc"))]
            let initrd_data_bytes = &initrd_data_buf[..initrd_data_len];


            // Initrd is typically just a blob, loaded to a contiguous memory region.
            // The `load_initrd` function would allocate memory and copy data.
            match kernel_formats::load_initrd(hal, &initrd_data_bytes) {
                Ok(info) => {
                    logger::info!("[Loader] Initrd '{}' loaded successfully at addr={:#x}, size={}",
                        initrd_path, info.load_address, info.size);
                    initrd_info = Some(info);
                }
                Err(e) => {
                    logger::error!("[Loader] Failed to load initrd image '{}': {:?}", initrd_path, e);
                    panic!("Initrd image loading failed: {:?}", e); // Placeholder
                }
            }
        }
    }

    // 4. Prepare architecture-specific environment and jump to kernel
    // This involves setting up page tables, passing boot parameters (cmdline, memory map, initrd location),
    // and ensuring the CPU is in the correct state.
    logger::info!("[Loader] Preparing architecture and jumping to kernel...");
    arch_adapter::prepare_and_jump_to_kernel(
        hal,
        kernel_info,
        initrd_info,
        &entry.cmdline,
        // Other boot parameters (e.g., memory map from HAL, ACPI tables)
    );
    // prepare_and_jump_to_kernel should not return.
}


/// Chains to another UEFI application (e.g., Windows Boot Manager, GRUB, OpenCore).
#[cfg(target_os = "uefi")]
fn chainload_uefi_application(
    _hal: &HalServices, // May need HAL for UEFI services access
    fs_manager: &FilesystemManager,
    entry: &BootEntry,
    volume_id: &str,
) -> ! {
    use crate::platform::uefi_utils; // Assuming a module for UEFI specific utilities

    logger::info!("[Loader] Chainloading UEFI application: {} from volume: {}", entry.kernel, volume_id);

    // 1. Get the UEFI file path. UEFI paths usually involve device paths + file path.
    //    For simplicity, we assume entry.kernel is a path relative to the volume's root.
    //    A robust solution needs to convert (volume_id, entry.kernel) to a UEFI DevicePath + FilePath.

    // 2. Load the .efi file into memory.
    //    UEFI Boot Services' LoadImage() typically handles this.
    //    It needs a UEFI DevicePath to the .efi file.
    //    Alternatively, read it manually and use LoadImage with a memory buffer.
    #[cfg(feature = "with_alloc")]
    let efi_app_data = match fs_manager.read_file(volume_id, &entry.kernel) {
        Ok(data) => data,
        Err(e) => {
            logger::error!("[Loader] Failed to read UEFI app file '{}': {:?}", entry.kernel, e);
            panic!("UEFI app file read failed: {:?}", e); // Placeholder
        }
    };
     #[cfg(not(feature = "with_alloc"))]
     unimplemented!("UEFI chainload without alloc for file reading not fully sketched");


    // 3. Get a UEFI handle for the loaded image.
    // This is a simplified flow. Direct use of UEFI services is complex.
    // `uefi_utils::load_efi_image_from_path` or `..._from_buffer` would encapsulate BS->LoadImage().
    let image_handle = match uefi_utils::load_efi_image_from_buffer(&efi_app_data, &entry.kernel) {
        Ok(handle) => handle,
        Err(e) => {
            logger::error!("[Loader] UEFI LoadImage failed for {}: {:?}", entry.kernel, e);
            panic!("UEFI LoadImage failed: {:?}", e); // Placeholder
        }
    };

    // 4. (Optional) Set load options (command line for the .efi application).
    if !entry.cmdline.is_empty() {
        if let Err(e) = uefi_utils::set_load_options(image_handle, &entry.cmdline) {
            logger::warn!("[Loader] Failed to set UEFI load options for {}: {:?}", entry.kernel, e);
        }
    }

    // 5. Start the loaded image using UEFI Boot Services' StartImage().
    logger::info!("[Loader] Starting UEFI image: {}", entry.kernel);
    if let Err(e) = uefi_utils::start_efi_image(image_handle) {
        logger::error!("[Loader] UEFI StartImage failed for {}: {:?}", entry.kernel, e);
        // TODO: UI feedback. Attempt to unload image, return to LBL menu?
        panic!("UEFI StartImage failed: {:?}", e); // Placeholder
    }

    // If StartImage returns (it shouldn't if the app took over, but might on error or if it's a utility that exits),
    // it's an issue. LBL should probably attempt to return to its menu.
    logger::error!("[Loader] UEFI application {} exited or StartImage returned unexpectedly.", entry.kernel);
    // TODO: Attempt to return to LBL main menu. This requires careful state management.
    // For now, panic.
    panic!("UEFI application exited unexpectedly.");
}


/// Executes a built-in LBL tool.
fn execute_internal_tool(
    _hal: &HalServices, // Tools might need HAL access
    entry: &BootEntry,
) -> ! {
    logger::info!("[Loader] Executing internal tool: {}", entry.kernel);
    match entry.kernel.as_str() {
        "internal://lbl_shell" => {
            // TODO: Implement and launch the LBL debug shell/recovery console.
            // shell::run_shell(hal); // Conceptual
            logger::info!("[Loader] LBL Shell requested (not implemented). Halting.");
        }
        // Add other internal tools here
        _ => {
            logger::error!("[Loader] Unknown internal tool: {}", entry.kernel);
        }
    }
    // After tool finishes (if it returns control), what to do?
    // For a shell, it might have an 'exit' or 'reboot' command.
    // If it just returns, LBL might loop back to GUI or halt.
    // For now, halt.
    panic!("Internal tool finished or not found. Halting."); // Placeholder
}