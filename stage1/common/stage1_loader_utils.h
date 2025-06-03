// Lionbootloader - Stage 1 - Common Loader Utilities Header
// File: stage1/common/stage1_loader_utils.h

#ifndef STAGE1_LOADER_UTILS_H
#define STAGE1_LOADER_UTILS_H

// --- Common Definitions ---
// Define NULL if not already defined (e.g. by efi.h)
#ifndef NULL
#define NULL ((void *)0)
#endif

// Define boolean types for C if not using <stdbool.h> or EFI types
#ifndef __cplusplus // In C++, bool is a keyword
    #if !defined(bool) && !defined(EFI_TYPES_H) // EFI_TYPES_H from efi.h defines BOOLEAN
        #ifndef bool
            typedef unsigned char bool;
            #define true 1
            #define false 0
        #endif
    #endif
#endif


// The Makefile should define LBL_BIOS_ENV or LBL_UEFI_ENV appropriately
// when compiling stage1_loader_utils.c for different targets.

#if defined(LBL_BIOS_ENV)
// --- Declarations for BIOS Environment ---

/**
 * @brief Prints a string to the console using BIOS services (INT 10h).
 * !!Only usable in 16-bit Real Mode. Assembly helpers preferred for MBR/Stage2_16bit.!!
 * @param str Null-terminated string to print.
 */
void lbl_bios_print_string(const char* str);

/**
 * @brief Loads sectors from disk using BIOS INT 13h (LBA or CHS).
 * !!Only usable in 16-bit Real Mode. Assembly typically used.!!
 * @param drive Drive number (e.g., 0x80 for first HDD).
 * @param lba Starting Logical Block Address.
 * @param num_sectors Number of sectors to read.
 * @param target_segment Segment part of the destination memory address.
 * @param target_offset Offset part of the destination memory address.
 * @return 0 on success, non-zero on error.
 */
int lbl_bios_read_sectors(unsigned char drive, unsigned long long lba, unsigned short num_sectors,
                           unsigned short target_segment, unsigned short target_offset);


#elif defined(LBL_UEFI_ENV)
// --- Declarations for UEFI Environment ---

#include <efi.h>    // For EFI_STATUS, CHAR16, EFI_HANDLE, EFI_MEMORY_DESCRIPTOR etc.
#include <efilib.h> // For Print related things if not using ConOut directly

/**
 * @brief Prints a CHAR16 (wide character) string to the UEFI console.
 * @param str Null-terminated CHAR16 string.
 */
void lbl_uefi_print_string(const CHAR16* str);

/**
 * @brief Converts an ASCII string to CHAR16 and prints it to the UEFI console.
 * @param ascii_str Null-terminated ASCII string.
 */
void lbl_uefi_print_ascii_string(const char* ascii_str);

/**
 * @brief Loads a file from a filesystem on a given UEFI device handle.
 * The caller is responsible for freeing `*file_buffer` using `BS->FreePool()` if successful.
 * @param device_handle EFI handle of the device/partition containing the filesystem.
 * @param file_path Null-terminated CHAR16 path to the file on the filesystem.
 * @param file_buffer Output: Pointer to receive a buffer allocated with the file's contents.
 * @param file_size Output: Pointer to receive the size of the file in bytes.
 * @return EFI_STATUS indicating success or failure.
 */
EFI_STATUS lbl_uefi_load_file_from_device(
    EFI_HANDLE device_handle,
    CHAR16* file_path,
    VOID** file_buffer,
    UINTN* file_size
);

/**
 * @brief Gets the current UEFI Memory Map.
 * The caller is responsible for freeing `*memory_map_ptr` using `BS->FreePool()` if successful.
 * @param memory_map_ptr Output: Pointer to receive an allocated buffer containing the memory map.
 * @param map_size In/Out: On input, typically 0 or current buffer size. On output, actual map size.
 * @param map_key Output: Pointer to receive the key for the current memory map.
 * @param descriptor_size Output: Pointer to receive the size of a single EFI_MEMORY_DESCRIPTOR.
 * @param descriptor_version Output: Pointer to receive the descriptor version.
 * @return EFI_STATUS indicating success or failure.
 */
EFI_STATUS lbl_uefi_get_memory_map(
    EFI_MEMORY_DESCRIPTOR** memory_map_ptr, 
    UINTN* map_size, 
    UINTN* map_key,
    UINTN* descriptor_size,
    UINT32* descriptor_version
);


#else
// --- Declarations for other/generic environments (stubs) ---

void lbl_generic_print_string(const char* str);

#endif // LBL_BIOS_ENV / LBL_UEFI_ENV


#endif // STAGE1_LOADER_UTILS_H