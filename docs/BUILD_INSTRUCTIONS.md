# Lionbootloader (LBL) - Build Instructions

This document provides instructions for setting up the development environment and building Lionbootloader (LBL) and its components.
**Note: LBL is currently in a conceptual and early development phase. These instructions outline the *intended* build process once the codebase is functional.**

## 1. Prerequisites

Before you can build LBL, you will need to install several tools and dependencies.

### 1.1. Common Tools (All Platforms)

*   **Git**: For cloning the repository.
*   **Make**: GNU Make, for orchestrating the build process (especially Stage 1).
*   **Rust Toolchain**:
    *   Install Rust via [rustup](https://rustup.rs/).
    *   Ensure you have the latest stable Rust compiler, Cargo, and `rustfmt`, `clippy`.
    *   You will need to add targets for cross-compilation, e.g.:
        ```bash
        rustup target add x86_64-unknown-none  # For bare-metal x86_64 Rust core
        rustup target add thumbv7em-none-eabihf # Example for ARM Cortex-M
        # Add other targets as needed (e.g., riscv64gc-unknown-none-elf)
        ```
*   **NASM (Netwide Assembler)**: For assembling Stage 1 BIOS MBR code. Version 2.15+ recommended.
*   **QEMU (Optional, Recommended for Testing)**: For running LBL in virtual machines.
*   **dd, mtools, fdisk/gparted, mkfs.vfat (Optional, for Image Creation)**: Standard Unix utilities for creating and manipulating disk images.

### 1.2. For BIOS Stage 1 Development

*   **GCC (C Compiler for 32-bit)**: A GCC toolchain capable of producing 32-bit `elf_i386` code, often available as part of a standard Linux distribution's development tools or via MinGW on Windows.
    *   On Linux: `gcc` (ensure 32-bit multilib support is installed if on a 64-bit system, e.g., `gcc-multilib`).
    *   For cross-compiling from macOS/Windows: An `i686-elf-gcc` cross-compiler is ideal.

### 1.3. For UEFI Stage 1 Development

*   **Cross-Compiler for UEFI Targets**:
    *   **x86_64 UEFI**:
        *   Typically `x86_64-w64-mingw32-gcc` (from MinGW-w64 toolchain) for Windows PE/COFF format.
        *   Or, Clang with `--target=x86_64-pc-win32-coff` or `--target=x86_64-unknown-uefi`.
    *   **IA32 UEFI**:
        *   Typically `i686-w64-mingw32-gcc`.
        *   Or, Clang with `--target=i686-pc-win32-coff`.
    *   **AArch64 UEFI**:
        *   `aarch64-linux-gnu-gcc` with appropriate flags for PE/COFF (more complex).
        *   Or, Clang with `--target=aarch64-pc-win32-coff` or `--target=aarch64-unknown-uefi`.
*   **GNU-EFI Development Kit**:
    *   LBL's UEFI Stage 1 C code uses `gnu-efi`. You need to obtain and build `gnu-efi` for your target UEFI architectures.
    *   Clone `gnu-efi`: `git clone https://git.code.sf.net/p/gnu-efi/code gnu-efi`
    *   Follow `gnu-efi`'s instructions to build it (e.g., `make TARGET=x86_64-w64-mingw32` for x86_64 UEFI apps using MinGW).
    *   The LBL Stage 1 Makefile (`stage1/Makefile`) assumes `gnu-efi` headers and libraries are accessible, potentially by placing the built `gnu-efi` distribution inside `stage1/uefi/gnu-efi` or adjusting include/library paths in the Makefile.

### 1.4. (Optional) For specific Rust Core/GUI features:

*   If NanoVG is used for GUI and built from source: Cmake, and necessary C development tools.
*   If specific crypto libraries are used, they might have their own system dependencies.

## 2. Cloning the Repository

```bash
git clone https://github.com/your-org/lionbootloader.git # Replace with actual LBL repository URL
cd lionbootloader
```

## 3. Building Lionbootloader

LBL consists of multiple components that can be built individually or via a master script.

### 3.1. Using the Master Build Script (Recommended)

A master build script `tools/build.sh` will be provided to simplify the build process.

```bash
# Ensure the script is executable
chmod +x tools/build.sh

# Build all components (Stage 1 for common targets, Rust Core & GUI)
./tools/build.sh all

# Clean all build artifacts
./tools/build.sh clean
```
(The `tools/build.sh` script will internally call `make` and `cargo` with appropriate configurations.)

### 3.2. Building Components Manually

#### 3.2.1. Stage 1 (BIOS MBR & UEFI Application)

Navigate to the `stage1` directory and use its Makefile:

```bash
cd stage1

# Build all Stage 1 components (BIOS and UEFI for default targets like x86_64)
make all

# Build only BIOS components
make bios

# Build only UEFI x86_64 component
make uefi_x64 # (May require setting UEFI_X64_CC, etc. environment variables)

# Build only UEFI IA32 component
make uefi_ia32 # (May require setting UEFI_IA32_CC, etc.)

# Clean Stage 1 artifacts
make clean

cd .. # Return to project root
```
**Note on Stage 1 Makefile**: You might need to set environment variables for cross-compilers (e.g., `UEFI_X64_CC=x86_64-w64-mingw32-gcc`) before running `make` if they are not found automatically or if your system defaults are incorrect. Refer to `stage1/Makefile` for toolchain variables.

#### 3.2.2. LBL Core Engine (Rust)

The Core Engine is a Rust crate.

```bash
# Build LBL Core in release mode for a specific bare-metal target (example)
# The target triple (e.g., x86_64-unknown-none) tells Rust how to compile for no_std.
cargo build --release --manifest-path core/Cargo.toml --target x86_64-unknown-none

# Build with specific features (see core/Cargo.toml for available features)
# cargo build --release --manifest-path core/Cargo.toml --target x86_64-unknown-none --features "fs_ext4,security_tpm"

# Clean Core build artifacts
cargo clean --manifest-path core/Cargo.toml
```
The output binary (e.g., `core/target/x86_64-unknown-none/release/lionbootloader_core_lib.lib` or an ELF if built as a bin) will then need to be processed (e.g., by `objcopy -O binary ... lbl_core.bin`) to get the flat binary Stage 1 expects, or packaged as an EFI if `lbl_core` is itself an EFI app. The main Makefile or `build.sh` should handle this post-processing.

#### 3.2.3. LBL GUI Layer (Rust)

The GUI Layer is also a Rust crate, typically linked by the Core Engine.

```bash
# Build LBL GUI library (usually built as a dependency of Core)
cargo build --release --manifest-path gui/Cargo.toml
# If it needs a specific target different from core, specify --target.

# Clean GUI build artifacts
cargo clean --manifest-path gui/Cargo.toml
```

### 3.3. Post-Processing (Creating `lbl_core.bin`)

If the Rust Core Engine is built as an ELF file (common for `*-unknown-none` targets), it needs to be converted to a flat binary if Stage 1 BIOS or a simple UEFI loader expects that.

```bash
# Example for an ELF output from Rust Core targeting x86_64-unknown-none
# Adjust CORE_ELF_PATH and LBL_CORE_BIN_PATH as per actual output from `cargo build`
# and where Stage 1 expects it.
CORE_ELF_PATH="core/target/x86_64-unknown-none/release/lionbootloader_core_bin" # (if main.rs defines a bin)
# or if core/src/lib.rs is the main artifact for a static lib, this step might be different.
# If the core product is lionbootloader_core_lib.a, then stage1 links it.
# If stage1 loads core as a raw binary:
LBL_CORE_BIN_DEST="build/core/lbl_core.bin" # Destination for the flat binary

objcopy -O binary ${CORE_ELF_PATH} ${LBL_CORE_BIN_DEST}
```
The main project Makefile (in the root) or `tools/build.sh` should automate this.

## 4. Creating a Bootable Disk Image (Example)

After building all components, you'll need to assemble them onto a bootable medium. This process is highly dependent on the target (BIOS/UEFI) and desired disk layout.

A script `tools/mkimage.sh` will be provided to automate common scenarios, e.g., creating a bootable FAT32 floppy/USB image.

**Conceptual steps for a BIOS FAT32 USB/Floppy:**

1.  **Partition and Format**: Create a FAT32 partition on the USB drive.
2.  **Install MBR**: `dd if=build/stage1/mbr.bin of=/dev/sdX bs=446 count=1` (Careful! `sdX` is your target device. Writes only boot code part, leaves partition table). Or, if `mbr.bin` includes a partition table (less common for a generic MBR), `dd if=build/stage1/mbr.bin of=/dev/sdX bs=512 count=1`.
    *   For a floppy image, `mbr.bin` might be the VBR: `dd if=build/stage1/mbr.bin of=floppy.img conv=notrunc`.
3.  **Copy Files**:
    *   Copy `build/stage1/boot_32.bin` to where MBR expects to load it (e.g., specific sectors or a file like `LBLBT32.SYS` on the FAT root if MBR is also a FAT VBR).
    *   Create directories `/LBL/CORE/` on the FAT partition.
    *   Copy `build/core/lbl_core.bin` to `/LBL/CORE/lbl_core.bin`.
    *   Copy `config/default.json` to `/LBL/config.json` (or other configured path).
    *   Copy any theme assets (fonts, images) to `/LBL/fonts/`, `/LBL/themes/`.

**Conceptual steps for a UEFI USB:**

1.  **Partition**: Create a GPT partitioned disk with an EFI System Partition (ESP), formatted as FAT32.
2.  **Copy Files to ESP**:
    *   Create directory `EFI/BOOT/` on the ESP.
    *   Copy `build/stage1/BOOTX64.EFI` to `EFI/BOOT/BOOTX64.EFI` (for x86_64, this is the fallback boot path).
    *   Alternatively, copy to `EFI/LBL/LBL.EFI` and create a UEFI boot entry pointing to it.
    *   Create `/LBL/CORE/` on ESP.
    *   Copy `build/core/lbl_core.bin` (or `lbl_core.efi`) to `/LBL/CORE/`.
    *   Copy `config/default.json` to `/LBL/config.json`.
    *   Copy theme assets.

## 5. Running and Testing

*   **QEMU**:
    *   For BIOS: `qemu-system-x86_64 -fda build/images/lionbootloader_bios_floppy.img`
    *   For UEFI (x86_64): Requires OVMF (UEFI firmware for QEMU).
        `qemu-system-x86_64 -bios OVMF.fd -hda fat:rw:path_to_esp_directory_or_image`
*   **Real Hardware**: Boot from the prepared USB drive. Ensure your machine's firmware is set to boot from USB in the correct mode (Legacy BIOS or UEFI).

## 6. Troubleshooting

*   Ensure all prerequisite toolchains are installed and in your `PATH`.
*   Verify cross-compiler prefixes and capabilities.
*   Check `gnu-efi` build and paths for UEFI Stage 1.
*   For Rust `no_std` builds, ensure target triples are correct and a global allocator/panic handler are properly set up in the Rust code if `alloc` is used.
*   Pay close attention to paths and output locations in Makefiles and scripts.

(This section will be expanded with common issues and solutions as development progresses.)
