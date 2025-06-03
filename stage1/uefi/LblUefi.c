// Lionbootloader - Stage 1 - UEFI Loader Application (LblUefi.c)

#include <efi.h>
#include <efilib.h> // For gnu-efi helper functions like Print, AllocatePool, etc.
                    // Some might be used directly via BootServices (BS) too.

#include "LblUefi.h" // Own header for this file (if any specific declarations)
#include "../common/stage1_loader_utils.h" // Shared utilities

// Define global variables for EFI services, initialized in efi_main
EFI_SYSTEM_TABLE         *ST = NULL;
EFI_BOOT_SERVICES        *BS = NULL;
EFI_RUNTIME_SERVICES     *RS = NULL;
EFI_HANDLE               IH = NULL; // This image's handle


// Configuration for finding LBL Core
// These should match paths that the LBL installation process would create.
// Paths are relative to the root of a discovered FAT filesystem (typically ESP).
#define LBL_CORE_BIN_PATH       L"\\LBL\\CORE\\lbl_core.bin"
// If core could also be an EFI app:
// #define LBL_CORE_EFI_PATH    L"\\EFI\\LBL\\lbl_core.efi"


// Forward declaration (if needed, for functions defined later in this file)
EFI_STATUS FindAndLoadLBLCore(VOID** CoreBuffer, UINTN* CoreSize);
EFI_STATUS PrepareBootInfoForCore(LBL_BOOT_INFO* BootInfoStructure, VOID* CoreBuffer, UINTN CoreSize);


/**
 * efi_main - The entry point for the LBL UEFI application.
 * @ImageHandle: The firmware allocated handle for the EFI image.
 * @SystemTable: A pointer to the EFI System Table.
 */
EFI_STATUS
EFIAPI
efi_main (EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable)
{
    EFI_STATUS Status;
    VOID* LblCoreBuffer = NULL;
    UINTN LblCoreSize = 0;
    LBL_BOOT_INFO BootInfoForCore; // Defined in LblUefi.h or common header

    // Initialize global pointers. This pattern is common with gnu-efi.
    ST = SystemTable;
    BS = SystemTable->BootServices;
    RS = SystemTable->RuntimeServices;
    IH = ImageHandle;

    // Initialize libefi (gnu-efi helpers)
    // This sets up convenience functions like Print(), AllocatePool(), etc.
    // that wrap BootServices calls using the global ST and BS.
    // If not using libefi helpers extensively, direct BS calls are fine.
    InitializeLib(ImageHandle, SystemTable);

    Print(L"Lionbootloader Stage1 UEFI Initializing...\n");
    lbl_uefi_print_ascii_string("LBL Stage1 UEFI (using utility print)...\r\n");

    // 1. Locate and Load LBL Core Engine
    //    This involves finding a suitable FAT partition (usually ESP),
    //    then loading LBL_CORE_BIN_PATH from it.
    Status = FindAndLoadLBLCore(&LblCoreBuffer, &LblCoreSize);
    if (EFI_ERROR(Status)) {
        Print(L"Error: Failed to load LBL Core Engine. Status: %r\n", Status);
        lbl_uefi_print_ascii_string("Halting due to LBL Core load failure.\r\n");
        BS->Stall(5 * 1000 * 1000); // Stall for 5 seconds before exit
        return Status;
    }
    Print(L"LBL Core Engine loaded into memory at 0x%lx (Size: %u bytes).\n", LblCoreBuffer, LblCoreSize);


    // 2. Prepare Boot Information for the Core Engine
    //    This structure will be passed to the Rust core.
    //    It needs memory map, graphics info, ACPI tables, etc.
    Status = PrepareBootInfoForCore(&BootInfoForCore, LblCoreBuffer, LblCoreSize);
    if (EFI_ERROR(Status)) {
        Print(L"Error: Failed to prepare BootInfo for Core. Status: %r\n", Status);
        BS->FreePool(LblCoreBuffer); // Free core buffer on error
        BS->Stall(5 * 1000 * 1000);
        return Status;
    }
    Print(L"BootInfo prepared for LBL Core.\n");
    Print(L"  Memory Map Key: 0x%lx\n", BootInfoForCore.memory_map_key);
    Print(L"  Framebuffer: %ux%u @ 0x%lx, Pitch %u, BPP %u\n",
        BootInfoForCore.framebuffer_width, BootInfoForCore.framebuffer_height,
        BootInfoForCore.framebuffer_addr, BootInfoForCore.framebuffer_pitch, BootInfoForCore.framebuffer_bpp);


    // 3. (Optional, but common) Exit Boot Services before jumping to core.
    //    LBL Core might want to do this itself if it needs Boot Services for a while.
    //    If Stage1 exits boot services, Core must be prepared to run without them.
    //    The MemoryMapKey from BootInfoForCore.memory_map_key is crucial here.
    Print(L"Attempting to exit boot services with MapKey: 0x%lx...\n", BootInfoForCore.memory_map_key);
    Status = BS->ExitBootServices(IH, BootInfoForCore.memory_map_key);
    if (EFI_ERROR(Status)) {
        // This is a critical error. The memory map might have changed.
        // A robust loader would try to GetMemoryMap again and retry ExitBootServices
        // a few times.
        Print(L"CRITICAL Error: ExitBootServices failed! Status: %r\n", Status);
        Print(L"The system may be unstable. Halting.\n");
        // It's hard to recover here. If core was loaded, it might not be at a valid location anymore.
        BS->FreePool(LblCoreBuffer); // Attempt to free, might fail if map changed too much.
                                     // Freeing BootInfo.memory_map_buffer should also happen.
        BS->FreePool(BootInfoForCore.memory_map_buffer); 
        while(1) { BS->Stall(100000); } // Infinite stall
    }
    Print(L"Successfully exited boot services.\n");
    // IMPORTANT: After ExitBootServices, ST, BS, Print(), AllocatePool(), etc. are INVALID.
    // Only RS (Runtime Services) are available. Logging must use direct framebuffer/serial if needed.
    // The Rust core will be running in this post-ExitBootServices environment.

    // 4. Jump to LBL Core Engine
    //    The LBL_CORE_ENTRY_OFFSET is relative to LblCoreBuffer.
    //    The Core engine expects a pointer to BootInfoForCore in a specific register
    //    (defined by LBL's internal ABI, e.g., RDI/X0).
    UINT64 CoreEntryPoint = (UINT64)LblCoreBuffer + LBL_CORE_ENTRY_OFFSET;
    
    // Define a function pointer type for the Rust core entry
    // `lbl_core_entry(boot_info_ptr: *const u8)`
    typedef VOID (EFIAPI *LBL_CORE_ENTRY_FN)(LBL_BOOT_INFO* BootInfo);
    LBL_CORE_ENTRY_FN LblCoreEntry = (LBL_CORE_ENTRY_FN)CoreEntryPoint;

    // Pre-jump message (will use raw framebuffer if possible or just be the last UEFI print)
    // Cannot use Print() here if ExitBootServices was called.
    // A very raw print:
    // if (BootInfoForCore.framebuffer_addr != 0) { /* raw_fb_print("Jumping to LBL Core...") */ }


    // Make the jump.
    // The ABI (how BootInfoForCore is passed) needs to match what the Rust _start expects.
    // For x86_64 System V, first arg is in RDI. For AArch64, X0.
    // The Rust core's `_start` or `lbl_core_entry` should be `extern "C"` or `extern "efiapi"`.
    // The cast to `(LBL_BOOT_INFO*)` is important.
    LblCoreEntry(&BootInfoForCore);


    // Should NOT return here. If it does, something went wrong in the Core.
    // Print(L"CRITICAL Error: LBL Core Engine returned control to UEFI Stage1!\n");
    // This Print() would fail if ExitBootServices was successful.
    // If we reach here, it's a catastrophic failure.
    // Attempt to reboot or halt.
    // RS->ResetSystem(EfiResetWarm, EFI_SUCCESS, 0, NULL);

    // Infinite loop if ResetSystem fails or is not called.
    while(1) { /* Loop forever */ }

    return EFI_SUCCESS; // Should be unreachable
}


/**
 * @brief Finds a suitable partition (usually ESP), and loads the LBL Core file.
 * This involves iterating through block devices, checking for FAT filesystems.
 */
EFI_STATUS FindAndLoadLBLCore(VOID** CoreBuffer, UINTN* CoreSize) {
    EFI_STATUS Status;
    UINTN NumHandles = 0;
    EFI_HANDLE* HandleBuffer = NULL;
    UINTN i;

    Print(L"Locating LBL Core: %s\n", LBL_CORE_BIN_PATH);

    // Get all handles that support Simple File System Protocol
    Status = BS->LocateHandleBuffer(ByProtocol, &gEfiSimpleFileSystemProtocolGuid, NULL, &NumHandles, &HandleBuffer);
    if (EFI_ERROR(Status) || NumHandles == 0) {
        Print(L"Error: No filesystems found (SimpleFileSystemProtocol). Status: %r\n", Status);
        return Status == EFI_SUCCESS ? EFI_NOT_FOUND : Status; // If success but no handles
    }

    Print(L"Found %u filesystem handle(s).\n", NumHandles);

    for (i = 0; i < NumHandles; i++) {
        Print(L"  Attempting to load core from FS handle [%u]...\n", i);
        Status = lbl_uefi_load_file_from_device(HandleBuffer[i], LBL_CORE_BIN_PATH, CoreBuffer, CoreSize);
        if (!EFI_ERROR(Status)) {
            Print(L"    LBL Core found and loaded from filesystem handle %u.\n", i);
            BS->FreePool(HandleBuffer);
            return EFI_SUCCESS; // Found and loaded
        } else {
            Print(L"    Failed to load from FS handle [%u]. Status: %r\n", i, Status);
            // If CoreBuffer was partially allocated by a failed load_file, it should be freed by load_file.
            if (*CoreBuffer != NULL) { // Defensive: ensure load_file cleans up on error
                BS->FreePool(*CoreBuffer);
                *CoreBuffer = NULL;
            }
        }
    }

    Print(L"Error: LBL Core file '%s' not found on any accessible filesystem.\n", LBL_CORE_BIN_PATH);
    BS->FreePool(HandleBuffer);
    return EFI_NOT_FOUND;
}

/**
 * @brief Gathers system information and prepares the LBL_BOOT_INFO struct.
 */
EFI_STATUS PrepareBootInfoForCore(LBL_BOOT_INFO* BootInfoStructure, VOID* CoreBuffer, UINTN CoreSize) {
    EFI_STATUS Status;
    EFI_GRAPHICS_OUTPUT_PROTOCOL *Gop = NULL;

    if (!BootInfoStructure) return EFI_INVALID_PARAMETER;

    // Zero out the structure first
    BS->SetMem(BootInfoStructure, sizeof(LBL_BOOT_INFO), 0);

    BootInfoStructure->magic = LBL_BOOT_INFO_MAGIC_VALUE; // From LblUefi.h
    BootInfoStructure->version = LBL_BOOT_INFO_VERSION;   // From LblUefi.h

    // Store Core Engine load info
    BootInfoStructure->core_load_addr = (UINT64)CoreBuffer;
    BootInfoStructure->core_size = CoreSize;
    BootInfoStructure->core_entry_offset = LBL_CORE_ENTRY_OFFSET; // From LblUefi.h

    // 1. Get Memory Map
    //    The actual memory map buffer will be pointed to by BootInfoStructure->memory_map_buffer
    Status = lbl_uefi_get_memory_map(
        &BootInfoStructure->memory_map_buffer,
        &BootInfoStructure->memory_map_size,
        &BootInfoStructure->memory_map_key,
        &BootInfoStructure->memory_descriptor_size,
        &BootInfoStructure->memory_descriptor_version
    );
    if (EFI_ERROR(Status)) {
        Print(L"Error: Failed to get UEFI Memory Map. Status: %r\n", Status);
        return Status;
    }

    // 2. Get Graphics/Framebuffer Information
    Status = BS->LocateProtocol(&gEfiGraphicsOutputProtocolGuid, NULL, (VOID **)&Gop);
    if (!EFI_ERROR(Status) && Gop != NULL && Gop->Mode != NULL && Gop->Mode->Info != NULL && Gop->Mode->FrameBufferBase != 0) {
        BootInfoStructure->framebuffer_addr = Gop->Mode->FrameBufferBase;
        BootInfoStructure->framebuffer_size = Gop->Mode->FrameBufferSize;
        BootInfoStructure->framebuffer_width = Gop->Mode->Info->HorizontalResolution;
        BootInfoStructure->framebuffer_height = Gop->Mode->Info->VerticalResolution;
        BootInfoStructure->framebuffer_pitch = Gop->Mode->Info->PixelsPerScanLine * 4; // Assuming 4 bytes/pixel (BGRA32)
                                                                                      // This pitch calculation needs to be accurate based on PixelFormat.
        BootInfoStructure->framebuffer_bpp = 32; // Assuming 32bpp BGRA8888, common for GOP.
                                                 // Real BPP should come from Gop->Mode->Info->PixelFormat

        // A more robust way to get BPP and correct pitch:
        switch (Gop->Mode->Info->PixelFormat) {
            case PixelRedGreenBlueReserved8BitPerColor: // RGBR
            case PixelBlueGreenRedReserved8BitPerColor: // BGRR
                BootInfoStructure->framebuffer_bpp = 32;
                BootInfoStructure->framebuffer_pitch = Gop->Mode->Info->PixelsPerScanLine * 4;
                break;
            // case PixelBitMask: // Requires parsing PixelInformation
            // case PixelBltOnly: // No direct framebuffer access
            default: // Minimal assumption
                BootInfoStructure->framebuffer_bpp = 32; // Fallback assumption
                BootInfoStructure->framebuffer_pitch = Gop->Mode->Info->PixelsPerScanLine * 4;
                break;
        }
    } else {
        Print(L"Warning: Graphics Output Protocol not found or invalid. Framebuffer info unavailable. Status: %r\n", Status);
        // Core will have to manage without direct framebuffer from Stage1 or use basic VGA if available
        BootInfoStructure->framebuffer_addr = 0; // Indicate no framebuffer
    }

    // 3. Get ACPI Table Pointer (RSDP)
    //    ACPI 2.0 table GUID: EFI_ACPI_20_TABLE_GUID
    //    ACPI 1.0 table GUID: ACPI_TABLE_GUID (older)
    BootInfoStructure->acpi_rsdp_ptr = 0; // Default to not found
    for (UINTN i = 0; i < ST->NumberOfTableEntries; i++) {
        EFI_CONFIGURATION_TABLE *ct = &ST->ConfigurationTable[i];
        if (CompareGuid(&ct->VendorGuid, &gEfiAcpi20TableGuid) ||  // Check for ACPI 2.0+ table
            CompareGuid(&ct->VendorGuid, &gAcpiTableGuid)) {       // Check for ACPI 1.0 table (legacy)
            BootInfoStructure->acpi_rsdp_ptr = (UINT64)ct->VendorTable;
            Print(L"ACPI RSDP found at 0x%lx\n", BootInfoStructure->acpi_rsdp_ptr);
            break;
        }
    }
    if (BootInfoStructure->acpi_rsdp_ptr == 0) {
        Print(L"Warning: ACPI RSDP pointer not found in EFI Configuration Tables.\n");
    }
    
    // 4. Other information (e.g., boot drive, command line if LBL EFI app took one) can be added.
    // BootInfoStructure->boot_drive_signature = ...; // If identifiable

    return EFI_SUCCESS;
}