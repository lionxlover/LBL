# Lionbootloader (LBL) - System Architecture

This document details the architecture of Lionbootloader (LBL), a universal, modern bootloader. It expands upon the [LBL Specification](./LBL_SPECIFICATION.md).

## 1. Core Philosophy

LBL is designed with the following principles:

*   **Universality**: Support a wide range of hardware from legacy BIOS to modern UEFI, across diverse CPU architectures.
*   **Modularity**: A clean separation of concerns allows for easier development, testing, and extension. Key components like filesystem drivers, GUI widgets, and architecture adapters are designed as distinct modules.
*   **Performance**: Minimize boot time through asynchronous operations, optimized code, and a small footprint.
*   **Security**: Integrate modern security practices including signature verification, measured boot (TPM), and a design conducive to Secure Boot principles.
*   **Usability**: Provide a rich, intuitive GUI that is easy to navigate and configure.
*   **Configurability**: Utilize a human-readable JSON format for all user-facing configurations, validated against a strict schema.
*   **Future-Proofing**: An extensible plugin system, versioned schemas, and adaptable architecture support aim for long-term viability.
*   **Safety & Stability**: Leverage memory-safe languages like Rust where possible (Core, GUI) and rigorous testing.

## 2. High-Level Components

LBL's architecture is primarily divided into two main stages, followed by a sophisticated Core Engine and GUI layer.

![LBL High-Level Architecture Diagram (Conceptual - to be added)](./images/lbl_architecture_high_level.png)
*(Placeholder for a diagram showing Stage 1 -> Core Engine -> GUI -> Boot Executor -> Kernel)*

### 2.1. Stage 1: Firmware Interface & Initial Loader

The first stage is the most hardware-dependent part of LBL. Its sole purpose is to perform minimal hardware initialization and load the LBL Core Engine.

*   **Purpose**:
    *   Establish a known, consistent execution environment.
    *   Load the LBL Core Engine (Stage 2) from a boot device into memory.
    *   Transfer execution to the Core Engine.
*   **Implementations**:
    *   **BIOS MBR**:
        *   A 512-byte assembly stub (`mbr.asm`) located in the Master Boot Record.
        *   Performs very basic CPU setup (e.g., segment registers, stack).
        *   Uses BIOS INT 13h (with LBA extension preference) to load a second, slightly larger BIOS loader (`boot_32.asm`).
        *   The second BIOS loader (`boot_32.asm`):
            *   Enables A20 line.
            *   Transitions CPU to 32-bit Protected Mode.
            *   (Optionally) Detects basic memory map (E820) and VBE graphics modes.
            *   Loads the LBL Core Engine binary (`lbl_core.bin`) from disk (again, using INT 13h before PM, or a minimal FAT driver if complex enough) into a suitable memory location (e.g., above 1 MiB).
            *   Prepares a minimal `BootInfoS2` structure.
            *   Jumps to the Core Engine's entry point in 32-bit PM.
    *   **UEFI Application**:
        *   A standard UEFI application (`.efi` image, e.g., `LblUefi.c`).
        *   Leverages UEFI Boot Services for hardware abstraction.
        *   Locates and loads the LBL Core Engine binary (which could be a flat binary `lbl_core.bin` or another specially prepared PE/EFI image `lbl_core.efi`) using UEFI File System and LoadImage protocols/services.
        *   Gathers comprehensive system information:
            *   Memory Map (via `GetMemoryMap()`).
            *   Graphics Output Protocol (GOP) for framebuffer details.
            *   ACPI table pointers (RSDP).
            *   TPM 2.0 presence and event log (via TCG2 Protocol).
            *   Secure Boot state.
        *   Prepares a detailed `LBL_BOOT_INFO` structure.
        *   (Optionally) Calls `ExitBootServices()` or defers this to the Rust Core.
        *   Transfers execution to the Core Engine.
*   **Output**: A successfully loaded Core Engine in memory, and a pointer to a `BootInfo` structure containing vital platform information.
*   **Constraints**: Stage 1 must be extremely small and robust. BIOS MBR is < 512 bytes; the subsequent BIOS loader should remain < 16 KiB. UEFI loader size is less constrained but should still be efficient.

### 2.2. LBL Core Engine (Rust)

The Core Engine is the heart of LBL, written primarily in Rust. It runs after Stage 1 and provides the main functionalities of the bootloader before handing off to an OS kernel or the GUI.

*   **Key Modules**:
    *   **Hardware Abstraction Layer (HAL) (`hal.rs`)**:
        *   Abstracts hardware interactions. Parses `BootInfo` from Stage 1.
        *   Provides services for memory management (parsing memory map, allocating pages/frames for kernel/initrd), device discovery, CPU features, timers, and potentially low-level console output.
        *   `Async Device Probing`: Manages parallel detection of storage, network, GPU, and input devices. Feeds events to the GUI for progress updates.
    *   **Filesystem Module Manager (`fs.rs`)**:
        *   Manages filesystem drivers (plugins or built-in).
        *   `interface.rs`: Defines traits (`FileSystemDriver`, `FileSystemInstance`) for FS drivers.
        *   `manager.rs`: Handles mounting volumes from discovered storage devices and dispatches file I/O operations.
        *   Supports common filesystems (FAT32, ext4, NTFS, Btrfs) via loadable `.lblfs` plugins or statically linked modules. Read-only access is the primary goal.
    *   **Configuration Module (`config.rs`)**:
        *   `schema_types.rs`: Defines Rust structs mirroring the `config.json` schema.
        *   `parser.rs`: Parses the JSON configuration file found by the FS module. Performs JSON Schema validation against an embedded schema.
        *   Manages dynamic configuration reloading (planned).
    *   **Security Manager (`security.rs`)**:
        *   `signature.rs`: Handles cryptographic signature verification of boot entries (kernels, initrds).
        *   `tpm.rs`: Manages TPM 2.0 interactions: PCR measurements (Measured Boot), event logging, potentially sealing/unsealing data.
        *   Integrates with Secure Boot chain-of-trust (if LBL itself is signed and running in a Secure Boot environment).
    *   **Plugin Manager (`plugins.rs`)**:
        *   Generic system for loading and managing plugins beyond filesystems (though FS is the primary use case now).
        *   For a `no_std` bootloader, "plugins" are more likely statically linked modules activated by config, rather than dynamically loaded shared libraries.
    *   **Logger (`logger.rs`)**: Centralized logging facility, outputting to serial, framebuffer, or other targets defined by HAL. Uses the `log` crate facade.
    *   **Boot Executor (`loader.rs`)**:
        *   `kernel_formats.rs`: Detects and parses kernel image formats (ELF, PE, IMG, Multiboot). Loads kernel and initrd into memory allocated by HAL.
        *   `arch_adapter.rs`: Performs architecture-specific final setup (paging, CPU registers, boot parameters struct) before jumping to the OS kernel.
*   **Environment**: Typically `#[no_std]` with `alloc` support.
*   **Memory Footprint**: Target < 1 MiB for the core components.

### 2.3. GUI & UX Layer (Rust)

The GUI layer provides the user interface for LBL. It is also written in Rust and interacts closely with the Core Engine.

*   **Key Modules**:
    *   **Renderer (`renderer.rs`)**:
        *   Abstracts drawing operations.
        *   Backend options:
            *   Direct framebuffer access (pixel manipulation).
            *   Integration with a 2D graphics library like `embedded-graphics`.
            *   Integration with a vector graphics library like NanoVG (requires a custom framebuffer backend or software rasterizer for `no_std`).
            *   Custom GPU pipeline (highly advanced, for future protected mode rendering).
    *   **Theme Engine (`theme.rs`)**:
        *   Parses theme settings from `LblConfig.theme` (JSON-driven).
        *   Manages colors, fonts (loaded via FS module, rasterized using e.g., `fontdue`), and potentially other style attributes.
        *   Supports light/dark modes as defined in the theme.
    *   **Input Handling (`input.rs`)**:
        *   Processes raw input events (from HAL via Core) for keyboard, mouse, touch, gamepad into structured GUI events.
        *   Manages input focus and event dispatch to widgets.
    *   **Widget Toolkit (`widgets/`)**:
        *   A collection of reusable UI components: `Button`, `ListView`, `ProgressBar`, `Label`, `TextInput` (for config editor), etc.
        *   Widgets manage their own state and drawing logic.
    *   **Animation Engine (`animations.rs`)**:
        *   Manages time-based animations for UI elements (fades, slides, scaling).
        *   Uses easing functions for smooth transitions. Relies on HAL timer for `delta_time`.
    *   **UI Manager (`ui.rs`)**:
        *   Manages overall UI layout, screen transitions (e.g., Boot Menu <-> Settings).
        *   Contains the main GUI event loop, which processes input, updates animations, and triggers redraws.
        *   Orchestrates display of boot entries, theme previews, settings editor, etc.
*   **Interaction with Core**:
    *   Receives `LblConfig`, HAL services, and FS manager from Core during initialization.
    *   The main loop returns a `GuiSelectionResult` (e.g., chosen `BootEntry`) to the Core's Boot Executor.
*   **Design Goals**: macOS-style aesthetics (clean, animated, scalable), user-friendly, accessible (high contrast, keyboard navigation).

## 3. Logic Flow and Module Interactions

The primary boot flow is:

1.  **Power-On** -> **Firmware (BIOS/UEFI)**
2.  **Firmware** -> **LBL Stage 1** (MBR or EFI app loads).
    *   Minimal hardware init (CPU mode, memory visibility).
    *   Loads LBL Core Engine binary into memory.
    *   Prepares `BootInfo` struct.
    *   Jumps to Core Engine.
3.  **LBL Core Engine** (`lbl_core_entry`):
    *   Initializes Logger.
    *   Initializes HAL using `BootInfo`. (Parses memory map, framebuffer, ACPI, etc. Allocator might be set up here).
    *   Starts Asynchronous Device Probing (Storage, Network, GPU, Input) via HAL. Events feed GUI progress.
    *   Initializes Filesystem Manager. Registers FS drivers/plugins. Mounts volumes on discovered storage devices.
    *   Initializes Configuration Module. Loads `config.json` using FS Manager. Validates schema. Populates config structs.
    *   Initializes Security Manager (detects TPM, loads trusted keys).
    *   Initializes Plugin Manager (loads configured plugins like extra FS drivers).
4.  **Core Engine** -> **GUI Layer** (`gui::init_gui`, `gui::run_main_loop`):
    *   GUI initializes Renderer (using framebuffer info from HAL), Theme Engine (using config and FS for fonts), Input System, UI Manager.
    *   GUI Main Loop runs:
        *   Displays boot menu (entries from config), status bar (device probe progress, countdown).
        *   Handles user input (selection, navigation, search).
        *   Allows access to Settings (theme preview, JSON editor, module manager).
        *   Updates animations.
        *   Redraws UI.
    *   GUI returns `GuiSelectionResult` (e.g., selected `BootEntry` or action like settings/shutdown) to Core.
5.  **Core Engine** (Boot Executor `loader.rs`):
    *   If a `BootEntry` was selected:
        *   Security Manager verifies the entry (signature check, TPM measurement if configured).
        *   Kernel Format Parser loads kernel and initrd (if any) into memory (allocated by HAL).
        *   Architecture Adapter prepares CPU state, page tables (if LBL manages them), and boot parameters (cmdline, memory map, initrd location, ACPI pointer, etc.).
        *   Jumps to the OS kernel's entry point. **LBL execution ends here.**
    *   If another action (e.g., shutdown, reboot, debug shell) was selected, Core handles it.

## 4. Key Data Structures

*   **`LBL_BOOT_INFO` (C struct, Stage 1 -> Core)**: Passes critical low-level platform info from UEFI Stage 1 to Rust Core. BIOS Stage 1 passes a simpler version.
*   **`LblConfig` (Rust struct, from JSON)**: Holds all user configuration (boot entries, theme, plugins, advanced settings).
*   **`BootEntry` (Rust struct)**: Defines a single bootable OS or tool.
*   `HalServices` (Rust struct): Facade for hardware access.
*   `AppliedTheme` (Rust struct): Processed theme data (colors, fonts) for rendering.
*   `InputEvent` (Rust enum): Standardized GUI input events.
*   Filesystem traits (`FileSystemDriver`, `FileSystemInstance`).
*   `KernelInfo` (Rust struct): Information about a loaded kernel image.

## 5. Cross-Architecture Considerations

*   **Stage 1**: Necessarily architecture-specific (different assembly for MBR on x86, different EFI binaries for x86_64, IA32, AArch64).
*   **HAL**: Contains architecture-specific code for CPU features, memory management (paging setup), interrupt handling (if any by LBL).
*   **Architecture Adapter (`loader/arch_adapter.rs`)**: Dedicated module for final prep before kernel jump, tailored per architecture (register setup, boot protocol).
*   **Core Logic**: Most of the Core Engine (FS, Config, high-level Loader, high-level GUI logic) is intended to be architecture-agnostic Rust code.
*   **Build System**: Must support cross-compilation for both Stage 1 (C/ASM) and Core/GUI (Rust) for all target architectures.

## 6. Future Enhancements / Advanced Topics

*   **Network Boot (PXE)**: Integrating a network stack and PXE support.
*   **Full Disk Encryption (LUKS) Unlock**: Requiring a plugin and crypto for unlocking encrypted root filesystems before kernel boot.
*   **Hypervisor Booting**: Directly booting kernels designed to run under a hypervisor, potentially with LBL acting as a minimal management layer.
*   **Protected Mode GUI**: For even richer graphics or features, transitioning the GUI itself into a more capable environment (e.g., after full paging setup, potentially with basic multitasking within LBL). This is a significant step towards a micro-OS.
*   **Dynamic Plugin Hot-Loading/Unloading**: For true dynamic extensibility (very complex in `no_std`).

This architecture provides a flexible and robust foundation for Lionbootloader's ambitious goals.