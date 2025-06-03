# Lionbootloader Master Makefile
#
# This Makefile orchestrates the build of Stage1 (BIOS/UEFI)
# and the Rust-based Core Engine and GUI.

# Variables
NASM = nasm
GCC = gcc
LD = ld
OBJCOPY = objcopy
RUSTC = cargo
# For cross-compilation, these might need to be prefixed, e.g., x86_64-elf-gcc
# Override these with environment variables or by editing if needed.
# Example for cross-compiling UEFI for x86_64:
# TARGET_ARCH_UEFI = x86_64
# UEFI_GCC = x86_64-w64-mingw32-gcc # or x86_64-efi-gcc if you have a dedicated toolchain
# UEFI_OBJCOPY = x86_64-w64-mingw32-objcopy

# Default target architecture for Rust core (host default, or specify for cross-compilation)
# RUST_TARGET ?= # e.g., x86_64-unknown-none, thumbv7em-none-eabihf, riscv64gc-unknown-none-elf
# If RUST_TARGET is set, Rust builds will be for that target.

# Output directories
BUILD_DIR = build
STAGE1_BUILD_DIR = $(BUILD_DIR)/stage1
CORE_BUILD_DIR = $(BUILD_DIR)/core
DISK_IMG_DIR = $(BUILD_DIR)/images

# Stage 1 BIOS sources
MBR_SRC = stage1/bios/mbr.asm
BOOT32_SRC = stage1/bios/boot_32.asm
STAGE1_COMMON_SRC_C = stage1/common/stage1_loader_utils.c
STAGE1_COMMON_OBJ_C = $(STAGE1_BUILD_DIR)/stage1_loader_utils_bios.o

MBR_BIN = $(STAGE1_BUILD_DIR)/mbr.bin
BOOT32_BIN = $(STAGE1_BUILD_DIR)/boot_32.bin
LBL_BIOS_STAGE1_LOADER = $(STAGE1_BUILD_DIR)/lbl_bios_s1.bin # Combined MBR + subsequent stage

# Stage 1 UEFI sources
UEFI_LOADER_SRC_C = stage1/uefi/LblUefi.c
STAGE1_UEFI_COMMON_SRC_C = stage1/common/stage1_loader_utils.c
STAGE1_UEFI_COMMON_OBJ_C = $(STAGE1_BUILD_DIR)/stage1_loader_utils_uefi.o

LBL_UEFI_EFI_X64 = $(STAGE1_BUILD_DIR)/BOOTX64.EFI
LBL_UEFI_EFI_IA32 = $(STAGE1_BUILD_DIR)/BOOTIA32.EFI
LBL_UEFI_EFI_ARM = $(STAGE1_BUILD_DIR)/BOOTARM.EFI # AArch32
LBL_UEFI_EFI_AA64 = $(STAGE1_BUILD_DIR)/BOOTAA64.EFI # AArch64

# Core Engine (Rust)
CORE_CRATE_DIR = core
CORE_TARGET_DIR = $(CORE_CRATE_DIR)/target
# Assuming the core engine is built as a static library or a raw binary to be loaded by Stage 1
# The exact name depends on the crate type and name in core/Cargo.toml
# For a #[no_std] binary, it might be just the crate name.
# For this example, let's assume we produce a `lbl_core.elf` and then `lbl_core.bin`
LBL_CORE_ELF = $(CORE_TARGET_DIR)/$(RUST_TARGET)/release/lionbootloader_core # Adjust if RUST_TARGET is not set or crate name differs
LBL_CORE_BIN = $(CORE_BUILD_DIR)/lbl_core.bin

# GUI (Rust) - often built as part of the core or a library linked by core
# If GUI is a separate binary/lib, similar logic as core applies.

# Default target
all: bios uefi core_engine

# Create build directories
$(shell mkdir -p $(BUILD_DIR) $(STAGE1_BUILD_DIR) $(CORE_BUILD_DIR) $(DISK_IMG_DIR))

# --- Stage 1 BIOS ---
# Compile common C utility for BIOS context
$(STAGE1_COMMON_OBJ_C): $(STAGE1_COMMON_SRC_C) stage1/common/stage1_loader_utils.h
	@echo "Compiling Stage1 Common (BIOS): $<"
	$(GCC) -m32 -O2 -ffreestanding -nostdlib -c $< -o $@

# MBR (512 bytes)
$(MBR_BIN): $(MBR_SRC)
	@echo "Assembling MBR: $<"
	$(NASM) -f bin $< -o $@ -l $(STAGE1_BUILD_DIR)/mbr.lst

# Boot32 (loads core engine from disk, expects core engine at a known location/name)
$(BOOT32_BIN): $(BOOT32_SRC) $(STAGE1_COMMON_OBJ_C)
	@echo "Assembling Boot32: $<"
	# This is a simplified view. boot_32.asm might need to link stage1_loader_utils.o
	# or call functions from it. For now, assume it's mostly ASM.
	# If linking C:
	# $(NASM) -f elf32 $(BOOT32_SRC) -o $(STAGE1_BUILD_DIR)/boot_32.o
	# $(LD) -m elf_i386 -T stage1/bios/linker_bios.ld $(STAGE1_BUILD_DIR)/boot_32.o $(STAGE1_COMMON_OBJ_C) -o $(STAGE1_BUILD_DIR)/boot_32.elf --oformat binary -o $(BOOT32_BIN)
	# For pure NASM outputting bin:
	$(NASM) -f bin $< -o $@ -l $(STAGE1_BUILD_DIR)/boot_32.lst -Pstage1/bios/lbl_config.inc # Pass defines if needed

# Placeholder for combining MBR and BOOT32_BIN into a single stage1 binary or image
# This is highly dependent on how MBR loads the next stage.
# For example, MBR might load BOOT32_BIN from sectors immediately following it.
# $(LBL_BIOS_STAGE1_LOADER): $(MBR_BIN) $(BOOT32_BIN)
#   cat $(MBR_BIN) $(BOOT32_BIN) > $@ # Simplistic concatenation

bios_mbr: $(MBR_BIN)
bios_loader: $(BOOT32_BIN)

bios: bios_mbr bios_loader
	@echo "BIOS Stage 1 components built."

# --- Stage 1 UEFI ---
# UEFI applications are typically PE32+ format.
# We need a UEFI-compatible toolchain.
# The following are generic targets; specific arch builds (x64, ia32, arm, aa64) should be invoked.

# Compile common C utility for UEFI context
$(STAGE1_UEFI_COMMON_OBJ_C): $(STAGE1_UEFI_COMMON_SRC_C) stage1/common/stage1_loader_utils.h
	@echo "Compiling Stage1 Common (UEFI): $<"
	# This command needs to be adjusted for the target UEFI architecture (e.g., -target for clang)
	$(UEFI_GCC) $(UEFI_CFLAGS) -c $< -o $@

# UEFI x86_64
$(LBL_UEFI_EFI_X64): $(UEFI_LOADER_SRC_C) stage1/uefi/LblUefi.h $(STAGE1_UEFI_COMMON_OBJ_C)
	@echo "Building UEFI Application (x86_64): BOOTX64.EFI"
	$(X86_64_EFI_GCC) \
		-Wall -Wextra -std=c11 -pedantic \
		-target x86_64-unknown-windows -mno-red-zone \
		-nostdlib -nostdinc -ffreestanding \
		-fno-stack-protector -fshort-wchar \
		-I stage1/uefi/gnuefi/inc -I stage1/uefi/gnuefi/inc/x86_64 \
		-I stage1/common \
		$(UEFI_LOADER_SRC_C) $(STAGE1_UEFI_COMMON_OBJ_C) \
		-o $(STAGE1_BUILD_DIR)/LblUefi_x64.elf \
		-Wl,-dll,-subsystem,10,-entry:efi_main \
		stage1/uefi/gnuefi/crt0-efi-x86_64.o stage1/uefi/gnuefi/lib/libgnuefi.a stage1/uefi/gnuefi/lib/libefi.a
	$(X86_64_EFI_OBJCOPY) -j .text \
		-j .sdata -j .data -j .dynamic \
		-j .dynsym -j .rel -j .rela -j .reloc \
		--target=efi-app-x86_64 $(STAGE1_BUILD_DIR)/LblUefi_x64.elf $@

# Add similar targets for IA32, ARM, AARCH64 if needed, adjusting compiler and flags.
# Example for IA32:
# $(LBL_UEFI_EFI_IA32): $(UEFI_LOADER_SRC_C) stage1/uefi/LblUefi.h $(STAGE1_UEFI_COMMON_OBJ_C)
#   ... (use IA32_EFI_GCC, IA32_EFI_OBJCOPY and corresponding gnuefi paths/libs)
#   $(IA32_EFI_GCC) ... -target i686-unknown-windows ... -I stage1/uefi/gnuefi/inc/ia32 ...
#   $(IA32_EFI_OBJCOPY) ... --target=efi-app-ia32 ...

# Generic UEFI target - assumes x64 for now
uefi: $(LBL_UEFI_EFI_X64)
	@echo "UEFI Stage 1 Application built (default x86_64)."

# --- Core Engine & GUI (Rust) ---
core_engine:
	@echo "Building Lionbootloader Core Engine and GUI (Rust)..."
	@(cd $(CORE_CRATE_DIR) && $(RUSTC) build --release $(if $(RUST_TARGET),--target $(RUST_TARGET)))
	# If GUI is a separate crate:
	# @(cd gui && $(RUSTC) build --release $(if $(RUST_TARGET_GUI),--target $(RUST_TARGET_GUI)))
	@echo "Rust build finished."
	# Post-processing: Convert ELF/other format to raw binary if needed by Stage 1
	# This step is highly dependent on the RUST_TARGET and what Stage 1 expects.
	# For a `no_std` binary targeting `*-unknown-none`, an ELF is produced.
	# Stage 1 might load this ELF directly or require a flat binary.
	# Example:
	# $(OBJCOPY) -O binary $(LBL_CORE_ELF) $(LBL_CORE_BIN)
	# This needs LBL_CORE_ELF to be correctly defined based on Cargo's output path.
	# For now, this step is commented out as it needs precise target/crate configuration.
	# cp $(CORE_TARGET_DIR)/$(RUST_TARGET)/release/lionbootloader_core $(LBL_CORE_BIN) # if output is already binary, or use objcopy

# --- Disk Image Creation (Example for BIOS) ---
# This is a placeholder. Creating a bootable disk image is complex and tool-dependent.
# Tools like `dd`, `mformat`, `mkfs.vfat`, `grub-mkrescue` (for ISOs) might be used.
# Assumes LBL_CORE_BIN is the core engine binary to be placed on the disk.
# Assumes MBR_BIN is the boot sector.
# Assumes BOOT32_BIN is the second stage loader.
LBL_BIOS_IMG = $(DISK_IMG_DIR)/lionbootloader_bios.img
create_bios_image: $(MBR_BIN) $(BOOT32_BIN) # $(LBL_CORE_BIN) # Core engine must be built first
	@echo "Creating BIOS bootable disk image (example)..."
	# Create a ~10MB disk image file
	dd if=/dev/zero of=$(LBL_BIOS_IMG) bs=1M count=10
	# Write MBR
	dd if=$(MBR_BIN) of=$(LBL_BIOS_IMG) conv=notrunc
	# Write Stage 1.5 (boot_32.bin) - assuming MBR loads from sector 1
	# dd if=$(BOOT32_BIN) of=$(LBL_BIOS_IMG) seek=1 bs=512 conv=notrunc
	# Create a FAT filesystem or copy files using mtools (example)
	# This part is highly dependent on how BOOT32_BIN finds LBL_CORE_BIN and config files.
	# Option 1: Embed LBL_CORE_BIN directly after BOOT32_BIN
	# cat $(MBR_BIN) $(BOOT32_BIN) $(LBL_CORE_BIN) > $(LBL_BIOS_IMG_RAW_LOAD)
	# dd if=$(LBL_BIOS_IMG_RAW_LOAD) of=$(LBL_BIOS_IMG) conv=notrunc
	# Option 2: Use mtools to put files on a FAT formatted image (after MBR)
	# mkfs.vfat -F 32 -n "LBL_BOOT" $(LBL_BIOS_IMG) # This overwrites the MBR if not careful with offsets
	# Need to format a partition, not the whole disk image directly after writing MBR, or use offsets.
	# A more robust way is to create partitions, then format, then copy.
	# For simplicity, this example is incomplete.
	@echo "BIOS image $(LBL_BIOS_IMG) (partially) created. Core engine and config need to be added."
	@echo "Ensure LBL_CORE_BIN and config/default.json are placed where Stage 1 can find them."

# --- Clean ---
clean:
	@echo "Cleaning build artifacts..."
	rm -rf $(BUILD_DIR)
	# Clean Rust target directories
	(cd $(CORE_CRATE_DIR) && $(RUSTC) clean)
	# (cd gui && $(RUSTC) clean) # If GUI is a separate crate
	# Clean Stage1 specific intermediate files not in BUILD_DIR
	# (e.g., if .o files are generated alongside sources)
	find stage1 -name "*.o" -delete
	find stage1 -name "*.lst" -delete
	find stage1 -name "*.elf" -delete # if intermediate ELF files are created for stage1
	@echo "Clean complete."

# --- Help ---
help:
	@echo "Lionbootloader Makefile"
	@echo ""
	@echo "Common targets:"
	@echo "  all                - Build all components (BIOS Stage1, UEFI Stage1, Core Engine)."
	@echo "  bios               - Build BIOS Stage 1 components (MBR, loader)."
	@echo "  uefi               - Build UEFI Stage 1 application (default x86_64 BOOTX64.EFI)."
	@echo "  core_engine        - Build the Rust Core Engine and GUI."
	@echo "  create_bios_image  - Create a sample BIOS bootable disk image (experimental)."
	@echo "  clean              - Remove all build artifacts."
	@echo ""
	@echo "Toolchain variables (can be overridden):"
	@echo "  NASM, GCC, LD, OBJCOPY, RUSTC"
	@echo "  X86_64_EFI_GCC, X86_64_EFI_OBJCOPY (for UEFI x64 build)"
	@echo "  RUST_TARGET (for cross-compiling Rust code, e.g., x86_64-unknown-none)"
	@echo ""
	@echo "Example: make RUST_TARGET=x86_64-unknown-none core_engine"
	@echo "Example: make X86_64_EFI_GCC=x86_64-w64-mingw32-gcc X86_64_EFI_OBJCOPY=x86_64-w64-mingw32-objcopy uefi"

.PHONY: all bios uefi core_engine create_bios_image clean help bios_mbr bios_loader