// Lionbootloader - Stage 1 - Common Loader Utilities
// File: stage1/common/stage1_loader_utils.c
//
// Contains utility functions potentially usable by both BIOS and UEFI Stage 1 loaders,
// though their implementations would differ significantly based on the environment.
// For BIOS, these would need to be callable from 16-bit or 32-bit assembly
// and might involve inline assembly for BIOS calls if not already in PM.
// For UEFI, they would use EFI Boot Services.

#include "stage1_loader_utils.h" // Corresponding header file

// Conditional compilation for BIOS vs UEFI context could be used here,
// or separate files for truly distinct implementations.
// For this example, we'll use comments to denote where platform differences are huge.

#if defined(LBL_BIOS_ENV) // This macro would be defined by the Makefile for BIOS compilation
// --- BIOS Environment Specific Code ---

/**
 * @brief Prints a string to the console using BIOS services (INT 10h).
 * !! Only usable in 16-bit Real Mode before Protected Mode transition. !!
 * Assembly helpers are usually preferred for this in MBR/Stage2_16bit.
 * @param str Null-terminated string to print.
 */
void lbl_bios_print_string(const char* str) {
    // This is a C-callable wrapper for what print_string_bios in ASM does.
    // It's more for if Stage2 itself was partly C in 16-bit mode.
    // For direct VGA buffer access in PM, see print_string_32_pm in boot_32.asm.
    if (!str) return;

    // This requires being in 16-bit real mode.
    // For a C function called from 32-bit PM after boot_32.asm switches,
    // this would not work. Direct VGA buffer write is needed then.
    // Example of inline assembly for demonstration if this C code was 16-bit.
    /*
    while (*str) {
        __asm__ __volatile__ (
            "mov ah, 0x0E\n"    // Teletype output
            "mov bh, 0\n"       // Page number
            "mov bl, 0x07\n"    // White on black attribute
            "int 0x10"
            : // No outputs
            : "a" (*str)        // Input: AL = character
            : "bh", "bl"        // Clobbers
        );
        str++;
    }
    */
    // In reality, if print_string_16 is in boot_32.asm, C code wouldn't reimplement.
    // This is more conceptual.
    (void)str; // Mark as used to avoid compiler warning on stub
}


/**
 * @brief Loads sectors from disk using BIOS INT 13h (LBA or CHS).
 * !! Only usable in 16-bit Real Mode. !!
 * Assembly is typically used for this in MBR/Stage2 for direct control.
 * @param drive Drive number (e.g., 0x80 for first HDD).
 * @param lba Starting Logical Block Address.
 * @param num_sectors Number of sectors to read.
 * @param target_segment Segment part of the destination memory address.
 * @param target_offset Offset part of the destination memory address.
 * @return 0 on success, non-zero on error.
 */
int lbl_bios_read_sectors(unsigned char drive, unsigned long long lba, unsigned short num_sectors,
                           unsigned short target_segment, unsigned short target_offset) {
    // This function would encapsulate the LBA DAP logic shown in mbr.asm/boot_32.asm
    // but in C, likely using inline assembly for the INT 13h calls.
    // This is highly complex to do robustly in C for 16-bit mode linked with 32-bit.
    // Primarily for demonstration that C *could* do this.
    (void)drive; (void)lba; (void)num_sectors; (void)target_segment; (void)target_offset; // Mark as used
    // Return error for stub
    return -1; // Error, not implemented
}


#elif defined(LBL_UEFI_ENV) // This macro would be defined by Makefile for UEFI compilation
// --- UEFI Environment Specific Code ---

#include <efi.h>          // Standard EFI types
#include <efilib.h>       // For Print, etc. (from gnu-efi) - LblPrint might use BS->ConOut

// Global EFI System Table pointer, initialized by efi_main in LblUefi.c
extern EFI_SYSTEM_TABLE         *ST;
extern EFI_BOOT_SERVICES        *BS;
extern EFI_RUNTIME_SERVICES     *RS;
extern EFI_HANDLE               IH; // Image Handle for this EFI app

/**
 * @brief Prints a string to the UEFI console.
 * @param str Null-terminated CHAR16 (wide character) string.
 */
void lbl_uefi_print_string(const CHAR16* str) {
    if (!ST || !ST->ConOut || !str) return;
    ST->ConOut->OutputString(ST->ConOut, (CHAR16*)str); // Cast needed as str is const
}

/**
 * @brief Converts an ASCII string to CHAR16 and prints it.
 */
void lbl_uefi_print_ascii_string(const char* ascii_str) {
    if (!ascii_str || !ST || !ST->ConOut) return;
    
    // Max buffer size for temporary conversion
    CHAR16 wide_buffer[256];
    UINTN i = 0;
    while (ascii_str[i] != '\0' && i < 255) {
        wide_buffer[i] = (CHAR16)ascii_str[i];
        i++;
    }
    wide_buffer[i] = L'\0';
    ST->ConOut->OutputString(ST->ConOut, wide_buffer);
}


/**
 * @brief Locates the LBL Core Engine file on a given device handle and file path.
 * @param device_handle Handle to the block I/O device (partition).
 * @param file_path Path to the LBL Core Engine EFI file (e.g., L"\\LBL\\CORE\\lbl_core.efi")
 *                  or raw binary L"\\LBL\\CORE\\lbl_core.bin".
 * @param file_buffer Pointer to receive allocated buffer with file contents.
 * @param file_size Pointer to receive file size.
 * @return EFI_STATUS code.
 */
EFI_STATUS lbl_uefi_load_file_from_device(
    EFI_HANDLE device_handle,
    CHAR16* file_path,
    VOID** file_buffer,
    UINTN* file_size
) {
    EFI_STATUS status;
    EFI_LOADED_IMAGE_PROTOCOL* loaded_image = NULL;
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL* fs_protocol = NULL;
    EFI_FILE_PROTOCOL* root_fs = NULL;
    EFI_FILE_PROTOCOL* file_handle = NULL;
    EFI_FILE_INFO* file_info = NULL;
    UINTN buffer_size;

    if (!BS || !device_handle || !file_path || !file_buffer || !file_size) {
        return EFI_INVALID_PARAMETER;
    }
    *file_buffer = NULL;
    *file_size = 0;

    // Get the LoadedImageProtocol for the current image to find its device handle (if file_path is relative to LBL itself)
    // Or, device_handle can be a specific partition handle found by scanning.
    // For simplicity, assume device_handle is the correct one for the filesystem.

    // Open the filesystem protocol on the device handle
    status = BS->HandleProtocol(device_handle, &gEfiSimpleFileSystemProtocolGuid, (VOID**)&fs_protocol);
    if (EFI_ERROR(status)) {
        lbl_uefi_print_ascii_string("Error: Could not open FS protocol.\r\n");
        return status;
    }

    // Open the root directory of the filesystem
    status = fs_protocol->OpenVolume(fs_protocol, &root_fs);
    if (EFI_ERROR(status)) {
        lbl_uefi_print_ascii_string("Error: Could not open FS volume root.\r\n");
        return status;
    }

    // Open the target file
    status = root_fs->Open(root_fs, &file_handle, file_path, EFI_FILE_MODE_READ, 0);
    if (EFI_ERROR(status)) {
        // Optionally print file_path, but it's CHAR16
        lbl_uefi_print_ascii_string("Error: Could not open file: ");
        lbl_uefi_print_string(file_path); // Print the wide string path
        lbl_uefi_print_ascii_string("\r\n");
        root_fs->Close(root_fs); // Close root before returning
        return status;
    }

    // Get file info to determine its size
    buffer_size = 0; // Must pass 0 initially to get required size for file_info
    status = file_handle->GetInfo(file_handle, &gEfiFileInfoGuid, &buffer_size, NULL);
    if (status != EFI_BUFFER_TOO_SMALL) {
        lbl_uefi_print_ascii_string("Error: Could not get file info size.\r\n");
        file_handle->Close(file_handle);
        root_fs->Close(root_fs);
        return status == EFI_SUCCESS ? EFI_DEVICE_ERROR : status; // if success, it's weird
    }

    status = BS->AllocatePool(EfiLoaderData, buffer_size, (VOID**)&file_info);
    if (EFI_ERROR(status)) {
        lbl_uefi_print_ascii_string("Error: Could not allocate buffer for file info.\r\n");
        file_handle->Close(file_handle);
        root_fs->Close(root_fs);
        return status;
    }

    status = file_handle->GetInfo(file_handle, &gEfiFileInfoGuid, &buffer_size, file_info);
    if (EFI_ERROR(status)) {
        lbl_uefi_print_ascii_string("Error: Could not get file info.\r\n");
        BS->FreePool(file_info);
        file_handle->Close(file_handle);
        root_fs->Close(root_fs);
        return status;
    }
    
    *file_size = file_info->FileSize;
    BS->FreePool(file_info); // Free the file_info buffer

    // Allocate buffer for the file contents
    status = BS->AllocatePool(EfiLoaderData, *file_size, file_buffer);
    if (EFI_ERROR(status)) {
        lbl_uefi_print_ascii_string("Error: Could not allocate buffer for file contents.\r\n");
        file_handle->Close(file_handle);
        root_fs->Close(root_fs);
        return status;
    }

    // Read the file
    buffer_size = *file_size; // Set to actual size for read
    status = file_handle->Read(file_handle, &buffer_size, *file_buffer);
    if (EFI_ERROR(status) || buffer_size != *file_size) {
        lbl_uefi_print_ascii_string("Error: File read failed or wrong size read.\r\n");
        BS->FreePool(*file_buffer);
        *file_buffer = NULL;
        file_handle->Close(file_handle);
        root_fs->Close(root_fs);
        return status == EFI_SUCCESS ? EFI_DEVICE_ERROR : status; // Handle partial read as error
    }

    // Cleanup
    file_handle->Close(file_handle);
    root_fs->Close(root_fs);

    lbl_uefi_print_ascii_string("Success: File loaded into memory.\r\n");
    return EFI_SUCCESS;
}

/**
 * @brief Gets the UEFI Memory Map.
 * @param memory_map_ptr Pointer to receive allocated buffer with memory map.
 * @param map_size Pointer to receive total size of the memory map.
 * @param map_key Pointer to receive the key for the current memory map.
 * @param descriptor_size Pointer to receive size of a single EFI_MEMORY_DESCRIPTOR.
 * @param descriptor_version Pointer to receive descriptor version.
 * @return EFI_STATUS code.
 */
EFI_STATUS lbl_uefi_get_memory_map(
    EFI_MEMORY_DESCRIPTOR** memory_map_ptr, 
    UINTN* map_size, 
    UINTN* map_key,
    UINTN* descriptor_size,
    UINT32* descriptor_version
) {
    EFI_STATUS status;
    *map_size = 0; // Important: must be 0 for first call to GetMemoryMap to get size
    *memory_map_ptr = NULL;

    // First call to get the size of the memory map
    status = BS->GetMemoryMap(map_size, *memory_map_ptr, map_key, descriptor_size, descriptor_version);
    if (status != EFI_BUFFER_TOO_SMALL) {
         lbl_uefi_print_ascii_string("Error: GetMemoryMap did not return EFI_BUFFER_TOO_SMALL on first call.\r\n");
        // This could mean map_size was not 0, or another error.
        return (status == EFI_SUCCESS) ? EFI_DEVICE_ERROR : status;
    }

    // Add some padding to map_size in case the map grows between calls
    *map_size += (*descriptor_size * 5); 

    // Allocate pool for the memory map
    status = BS->AllocatePool(EfiLoaderData, *map_size, (VOID**)memory_map_ptr);
    if (EFI_ERROR(status)) {
        lbl_uefi_print_ascii_string("Error: Could not allocate pool for memory map.\r\n");
        *memory_map_ptr = NULL;
        return status;
    }

    // Second call to actually get the memory map
    status = BS->GetMemoryMap(map_size, *memory_map_ptr, map_key, descriptor_size, descriptor_version);
    if (EFI_ERROR(status)) {
        lbl_uefi_print_ascii_string("Error: GetMemoryMap failed on second call.\r\n");
        BS->FreePool(*memory_map_ptr);
        *memory_map_ptr = NULL;
    }
    return status;
}


#else
// --- Stub for environments not BIOS or UEFI, or if no env macro is defined ---
void lbl_generic_print_string(const char* str) {
    // Placeholder for other environments or default stub
    (void)str;
}

#endif // LBL_BIOS_ENV / LBL_UEFI_ENV