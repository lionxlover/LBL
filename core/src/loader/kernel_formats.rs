// Lionbootloader Core - Loader Kernel Formats
// File: core/src/loader/kernel_formats.rs

#[cfg(feature = "with_alloc")]
use alloc::{boxed::Box, string::String, vec::Vec};

use crate::hal::HalServices; // For memory allocation, querying memory map
use crate::logger;

// --- Public Structs ---

/// Information about a loaded kernel or other image.
#[derive(Debug, Clone)]
pub struct KernelInfo {
    pub entry_point: u64,    // Virtual address of the kernel's entry point
    pub load_address: u64,   // Base physical address where kernel is loaded (or virtual if self-relocating)
    pub size: u64,           // Total size of the loaded kernel in memory
    pub stack_ptr: Option<u64>, // Suggested initial stack pointer for the kernel, if known
    // pub image_type: ImageType, // ELF, PE, Multiboot, etc.
    // pub higher_half_offset: Option<u64>, // For higher-half kernels
    // pub page_table_root: Option<u64>, // If loader sets up initial paging for kernel
}

/// Information about a loaded generic image (like initrd).
#[derive(Debug, Clone)]
pub struct LoadedImageInfo {
    pub load_address: u64, // Physical address where the image is loaded
    pub size: u64,         // Size of the image in memory
}

#[derive(Debug)]
pub enum LoadError {
    InvalidFormat(String),
    UnsupportedFormat(String),
    MemoryAllocationFailed(String),
    SegmentLoadFailed(String),
    IoError(String), // If reading directly from a device path
    Internal(String),
}


// --- Kernel Loading ---

/// Attempts to detect the kernel image type and load it.
/// Returns `KernelInfo` containing entry point and memory layout details.
pub fn load_kernel(hal: &HalServices, kernel_data: &[u8]) -> Result<KernelInfo, LoadError> {
    logger::info!("[KernelFormats] Loading kernel image ({} bytes)...", kernel_data.len());

    // Try to detect and load known formats
    if is_elf64(kernel_data) {
        logger::info!("[KernelFormats] Detected ELF64 format.");
        return load_elf64_kernel(hal, kernel_data);
    }
    // Add ELF32 detection and loader
    // if is_elf32(kernel_data) { ... }

    // Add PE32/PE32+ detection and loader (useful for UEFI context or Windows kernels if supported)
    // if is_pe(kernel_data) { ... }

    // Add Multiboot 1 / Multiboot 2 header detection if supporting those standards
    // if is_multiboot1(kernel_data) { ... }
    // if is_multiboot2(kernel_data) { ... }

    // Add other formats like Linux bzImage, raw binary, etc.

    logger::error!("[KernelFormats] Unknown or unsupported kernel image format.");
    Err(LoadError::UnsupportedFormat("Kernel image format not recognized.".into()))
}

// --- Initrd Loading ---

/// Loads an initrd image into a suitable memory location.
/// Initrds are typically treated as raw binary blobs.
pub fn load_initrd(hal: &HalServices, initrd_data: &[u8]) -> Result<LoadedImageInfo, LoadError> {
    logger::info!("[KernelFormats] Loading initrd image ({} bytes)...", initrd_data.len());

    // 1. Allocate memory for the initrd.
    //    The memory should be suitable for the OS (e.g., conventional memory,
    //    identity mapped, and recorded in the memory map passed to the OS).
    //    This requires HAL memory allocation services.
    let load_address = match allocate_contiguous_memory(hal, initrd_data.len() as u64, 16) { // 16-byte align
        Some(addr) => addr,
        None => {
            logger::error!("[KernelFormats] Failed to allocate memory for initrd.");
            return Err(LoadError::MemoryAllocationFailed("Initrd memory allocation failed".into()));
        }
    };

    // 2. Copy the initrd data to the allocated memory.
    unsafe {
        let dest_slice = core::slice::from_raw_parts_mut(load_address as *mut u8, initrd_data.len());
        dest_slice.copy_from_slice(initrd_data);
    }

    logger::info!(
        "[KernelFormats] Initrd loaded at physical address {:#x}, size {} bytes.",
        load_address,
        initrd_data.len()
    );

    Ok(LoadedImageInfo {
        load_address,
        size: initrd_data.len() as u64,
    })
}

// --- ELF64 Specific Loader ---
#[cfg(feature = "with_alloc")] // xmas-elf typically uses alloc for its data structures
fn load_elf64_kernel(hal: &HalServices, elf_data: &[u8]) -> Result<KernelInfo, LoadError> {
    use xmas_elf::{ElfFile, header, program::{Flags, Type as PhType}};

    let elf = ElfFile::new(elf_data)
        .map_err(|e_str| LoadError::InvalidFormat(format!("ELF parsing error: {}", e_str)))?;

    // Verify ELF header (64-bit, executable, target architecture matches LBL's target)
    if elf.header.pt1.class() != header::Class::SixtyFour {
        return Err(LoadError::InvalidFormat("Not a 64-bit ELF file.".into()));
    }
    if elf.header.pt1.type_().as_type() != header::Type::Executable {
        // Some kernels might be relocatable (DYN), needs different handling.
        // return Err(LoadError::InvalidFormat("ELF is not an executable.".into()));
        logger::warn!("[ELF64] ELF is not EXEC type, might be DYN (relocatable). Proceeding with caution.");
    }
    // Match elf.header.pt2.machine().as_machine() with current target_arch if needed.
    // Example: if elf.header.pt2.machine().as_machine() != header::Machine::X86_64 { ... }

    let entry_point = elf.header.pt2.entry_point();
    let mut min_load_addr = u64::MAX;
    let mut max_load_addr_plus_size = 0u64;

    logger::debug!("[ELF64] Kernel entry point from header: {:#x}", entry_point);

    // Iterate over program headers (segments)
    for ph in elf.program_iter() {
        if ph.get_type() == Ok(PhType::Load) {
            let flags = ph.flags();
            let segment_file_size = ph.file_size();
            let segment_mem_size = ph.mem_size();
            let segment_vaddr = ph.virtual_addr(); // Virtual address target
            let segment_offset = ph.offset();      // Offset in the ELF file

            if segment_mem_size == 0 {
                continue; // Skip segments with no memory footprint
            }

            logger::info!(
                "[ELF64] LOAD Segment: VAddr={:#010x}, FileSize={:#x}, MemSize={:#x}, Offset={:#x}, Flags={:?}",
                segment_vaddr, segment_file_size, segment_mem_size, segment_offset, flags
            );

            // For non-relocatable EXEs, segment_vaddr is the physical address.
            // For relocatable DYNs or position-independent EXEs, this might be a base offset.
            // Bootloaders often treat EXEs as directly loadable to their VAddr.
            // This assumes kernel is linked to run at these physical addresses or is PIE.
            let phys_load_addr = segment_vaddr; // Simplified: assume VAddr = PAddr for kernel segments for now.
                                                // A more complex loader might need to allocate memory and handle relocations.


            // "Allocate" or ensure memory is available at phys_load_addr for segment_mem_size.
            // For now, we assume HAL provides this or memory is clear.
            // A real bootloader would use HAL memory services to mark this memory as used.
            // Or, if kernel is higher-half, setup paging.

            if phys_load_addr < min_load_addr {
                min_load_addr = phys_load_addr;
            }
            if phys_load_addr + segment_mem_size > max_load_addr_plus_size {
                max_load_addr_plus_size = phys_load_addr + segment_mem_size;
            }

            // Ensure destination memory is writable (at least temporarily)
            // This is a simplification. Memory map interactions and permissions are complex.
            let dest_slice = unsafe {
                core::slice::from_raw_parts_mut(phys_load_addr as *mut u8, segment_mem_size as usize)
            };

            // Copy data from ELF file to memory
            if segment_file_size > 0 {
                let data_to_copy = elf_data
                    .get(segment_offset as usize .. (segment_offset + segment_file_size) as usize)
                    .ok_or_else(|| LoadError::SegmentLoadFailed("Segment data out of bounds in ELF file".into()))?;
                dest_slice[..segment_file_size as usize].copy_from_slice(data_to_copy);
            }

            // Zero out the .bss part of the segment (if mem_size > file_size)
            if segment_mem_size > segment_file_size {
                let bss_start = segment_file_size as usize;
                let bss_end = segment_mem_size as usize;
                dest_slice[bss_start..bss_end].fill(0);
            }

            // TODO: Set memory protections based on ph.flags() (Read, Write, Execute)
            // This would involve interacting with page tables if paging is active.
            // If no paging, this is mostly informational for the OS.
        }
    }

    if min_load_addr == u64::MAX { // No LOAD segments found
        return Err(LoadError::InvalidFormat("ELF file has no loadable segments.".into()));
    }

    let kernel_base_address = min_load_addr;
    let kernel_total_size = max_load_addr_plus_size - min_load_addr;

    logger::info!(
        "[ELF64] Kernel loaded: Base={:#x}, TotalSize={:#x} ({:.2} MiB)",
        kernel_base_address, kernel_total_size, kernel_total_size as f64 / (1024.0 * 1024.0)
    );

    // TODO: Determine initial stack pointer for the kernel.
    // Some ELF kernels might have a specific symbol (e.g., `_stack_top`) or expect
    // the bootloader to allocate and provide one.
    // This is often architecture and kernel specific.
    let initial_stack_ptr = None; // Placeholder

    Ok(KernelInfo {
        entry_point,
        load_address: kernel_base_address,
        size: kernel_total_size,
        stack_ptr: initial_stack_ptr,
    })
}


// --- Helper Functions ---

fn is_elf64(data: &[u8]) -> bool {
    data.len() > 4 && data[0..4] == [0x7f, b'E', b'L', b'F'] && data[4] == 2 // ELFCLASS64
}

// Placeholder for memory allocation (needs HAL integration)
fn allocate_contiguous_memory(_hal: &HalServices, size: u64, alignment: u64) -> Option<u64> {
    // TODO: Call HAL memory allocation service.
    // This service needs to find a free physical memory region of `size` bytes,
    // with the given `alignment`, and mark it as used (e.g., "Bootloader Reclaimable"
    // or "Kernel Data").
    // For now, a dummy implementation returning a fixed address (VERY UNSAFE).
    // This needs to be replaced with actual memory management from HAL.
    static mut NEXT_ALLOC_ADDR: u64 = 0x0200_0000; // Example: Start allocations at 32MB
    unsafe {
        let current_addr_unaligned = NEXT_ALLOC_ADDR;
        let current_addr = (current_addr_unaligned + alignment -1) & !(alignment - 1); // Align up
        
        if current_addr.checked_add(size).is_none() { return None; } // Overflow check

        NEXT_ALLOC_ADDR = current_addr + size;
        logger::debug!("[MemAllocStub] Allocated {:#x} bytes at {:#x} (align {})", size, current_addr, alignment);
        Some(current_addr)
    }
}