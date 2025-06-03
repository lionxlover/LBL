// Lionbootloader - Stage 1 - UEFI Loader Header (LblUefi.h)

#ifndef LBL_UEFI_H
#define LBL_UEFI_H

#include <efi.h> // For EFI types like UINT64, EFI_MEMORY_DESCRIPTOR

// --- LBL Boot Information Structure ---
// This structure is prepared by the UEFI Stage 1 loader (LblUefi.c)
// and passed to the LBL Core Engine (Rust).
// The Rust Core Engine must have a compatible #[repr(C)] struct to receive this.

#define LBL_BOOT_INFO_MAGIC_VALUE   0x4C424C42494E464F // "LBLBINFO" (LionBootLoaderBootINFO)
#define LBL_BOOT_INFO_VERSION       0x00010000       // Version 1.0

// Define the offset of the entry point within the loaded LBL Core binary.
// If lbl_core.bin is a flat binary loaded to run from its start, this is 0.
// If lbl_core.bin is an ELF and _start is not at offset 0, this would be non-zero.
// For a flat Rust binary (e.g., from x86_64-unknown-none target), _start is often at 0.
#define LBL_CORE_ENTRY_OFFSET       0x0

typedef struct {
    // --- Header ---
    UINT64 magic;                   // LBL_BOOT_INFO_MAGIC_VALUE
    UINT32 version;                 // LBL_BOOT_INFO_VERSION
    UINT32 header_size;             // Size of this LBL_BOOT_INFO header part
    UINT32 total_size;              // Total size of BootInfo + any appended data (e.g. memory map directly after)

    // --- LBL Core Engine Info ---
    UINT64 core_load_addr;          // Physical address where LBL Core binary was loaded
    UINT64 core_size;               // Size of the LBL Core binary in bytes
    UINT64 core_entry_offset;       // Offset of the entry point within the loaded core_binary (usually 0)

    // --- Memory Map (UEFI GetMemoryMap format) ---
    EFI_MEMORY_DESCRIPTOR* memory_map_buffer; // Pointer to the allocated buffer containing the memory map
    UINTN memory_map_size;          // Total size in bytes of the memory_map_buffer
    UINTN memory_map_key;           // Key for the current memory map (used for ExitBootServices)
    UINTN memory_descriptor_size;   // Size of a single EFI_MEMORY_DESCRIPTOR entry
    UINT32 memory_descriptor_version; // Version of the EFI_MEMORY_DESCRIPTOR structure

    // --- Graphics/Framebuffer Information (from UEFI GOP) ---
    UINT64 framebuffer_addr;        // Physical address of the linear framebuffer
    UINT64 framebuffer_size;        // Size of the framebuffer in bytes
    UINT32 framebuffer_width;       // Width in pixels
    UINT32 framebuffer_height;      // Height in pixels
    UINT32 framebuffer_pitch;       // Pixels per scan line (stride in bytes = pitch * bytes_per_pixel)
                                    // Note: UEFI GOP provides PixelsPerScanLine. Pitch is in bytes.
                                    // So, this field might better be named 'pixels_per_scan_line'
                                    // and pitch calculated, or store byte pitch directly.
                                    // Let's assume this means BYTES per scan line for consistency with BootInfo structure.
    UINT8  framebuffer_bpp;         // Bits per pixel (e.g., 32 for BGRA8888)
    UINT8  framebuffer_pixel_format_info; // Could store UEFI GOP PixelFormat enum or custom LBL mapping
    UINT16 reserved_graphics;       // Padding

    // --- ACPI Information ---
    UINT64 acpi_rsdp_ptr;           // Physical address of the ACPI RSDP (Root System Description Pointer)

    // --- Platform/Firmware Information ---
    UINT64 efi_system_table_ptr;    // Physical address of the EFI System Table (for Runtime Services access by Core if needed)
    // Could add boot drive signature, command line args passed to LBL.efi, etc.
    
    // --- Future Expansion ---
    UINT64 reserved1;
    UINT64 reserved2;

} LBL_BOOT_INFO;


// Globals defined in LblUefi.c that might be referenced by other C files
// in this stage1/uefi module (if any were added).
// extern EFI_SYSTEM_TABLE         *ST;
// extern EFI_BOOT_SERVICES        *BS;
// extern EFI_RUNTIME_SERVICES     *RS;
// extern EFI_HANDLE               IH;


#endif // LBL_UEFI_H