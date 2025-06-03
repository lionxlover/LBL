# Lionbootloader Stage 1 Makefile
# Builds MBR for BIOS and EFI application for UEFI.

# --- Toolchain Configuration ---
# Override these with environment variables if cross-compiling or using different tools.
NASM = nasm
LD = ld
OBJCOPY = objcopy

# For BIOS (typically 32-bit, building on a 64-bit host might need -m32)
BIOS_CC = gcc
BIOS_CFLAGS = -m32 -O2 -ffreestanding -nostdlib -Wall -Wextra -c
BIOS_LDFLAGS = -m elf_i386 -T stage1/bios/linker_bios.ld --oformat binary

# For UEFI x86_64 (using a MinGW cross-compiler is common, or a dedicated EFI toolchain)
# Example using x86_64-w64-mingw32-gcc:
UEFI_X64_CC ?= x86_64-w64-mingw32-gcc
UEFI_X64_LD ?= x86_64-w64-mingw32-ld
UEFI_X64_OBJCOPY ?= x86_64-w64-mingw32-objcopy
# Common flags for UEFI applications. GNU-EFI headers/libs are needed.
# Assumes gnu-efi is checked out or installed in `stage1/uefi/gnu-efi`
# Or installed system-wide and paths are adjusted.
UEFI_X64_CFLAGS = -Wall -Wextra -std=c11 -pedantic \
                  -target x86_64-pc-win32-coff \
                  -mno-red-zone -nostdlib -nostdinc -ffreestanding \
                  -fno-stack-protector -fshort-wchar \
                  -I stage1/uefi/gnu-efi/inc -I stage1/uefi/gnu-efi/inc/x86_64 \
                  -I stage1/common
UEFI_X64_LDFLAGS = -nostdlib -Wl,-dll,-subsystem,10,-entry:efi_main \
                   stage1/uefi/gnu-efi/crt0-efi-x86_64.o \
                   stage1/uefi/gnu-efi/lib/libgnuefi.a stage1/uefi/gnu-efi/lib/libefi.a
# Note: The exact crt0 and lib paths depend on your gnu-efi setup.
# The `-target x86_64-pc-win32-coff` is for Clang. For GCC, it might infer from prefix.

# For UEFI IA32 (32-bit)
UEFI_IA32_CC ?= i686-w64-mingw32-gcc
UEFI_IA32_LD ?= i686-w64-mingw32-ld
UEFI_IA32_OBJCOPY ?= i686-w64-mingw32-objcopy
UEFI_IA32_CFLAGS = -Wall -Wextra -std=c11 -pedantic \
                   -target i686-pc-win32-coff \
                   -mno-red-zone -nostdlib -nostdinc -ffreestanding \
                   -fno-stack-protector -fshort-wchar \
                   -I stage1/uefi/gnu-efi/inc -I stage1/uefi/gnu-efi/inc/ia32 \
                   -I stage1/common
UEFI_IA32_LDFLAGS = -nostdlib -Wl,-dll,-subsystem,10,-entry:efi_main \
                    stage1/uefi/gnu-efi/crt0-efi-ia32.o \
                    stage1/uefi/gnu-efi/lib/libgnuefi.a stage1/uefi/gnu-efi/lib/libefi.a


# --- Output Variables ---
STAGE1_OUT_DIR = ../build/stage1 # Output relative to Lionbootloader root's build dir
DISK_IMG_DIR = ../build/images

# BIOS outputs
MBR_SRC = stage1/bios/mbr.asm
MBR_BIN = $(STAGE1_OUT_DIR)/mbr.bin
MBR_LST = $(STAGE1_OUT_DIR)/mbr.lst

BOOT32_SRC = stage1/bios/boot_32.asm # Second stage BIOS loader
BOOT32_BIN = $(STAGE1_OUT_DIR)/boot_32.bin
BOOT32_LST = $(STAGE1_OUT_DIR)/boot_32.lst

# Common C utility for Stage1
STAGE1_COMMON_SRC_C = stage1/common/stage1_loader_utils.c
STAGE1_COMMON_HDR_C = stage1/common/stage1_loader_utils.h
STAGE1_COMMON_OBJ_BIOS = $(STAGE1_OUT_DIR)/stage1_loader_utils_bios.o
STAGE1_COMMON_OBJ_UEFI_X64 = $(STAGE1_OUT_DIR)/stage1_loader_utils_uefi_x64.o
STAGE1_COMMON_OBJ_UEFI_IA32 = $(STAGE1_OUT_DIR)/stage1_loader_utils_uefi_ia32.o


# UEFI outputs
UEFI_LOADER_SRC_C = stage1/uefi/LblUefi.c
UEFI_LOADER_HDR_C = stage1/uefi/LblUefi.h

LBL_UEFI_X64_ELF = $(STAGE1_OUT_DIR)/LblUefi_x64.elf
LBL_UEFI_X64_EFI = $(STAGE1_OUT_DIR)/BOOTX64.EFI # Standard name for fallback boot
LBL_UEFI_IA32_ELF = $(STAGE1_OUT_DIR)/LblUefi_ia32.elf
LBL_UEFI_IA32_EFI = $(STAGE1_OUT_DIR)/BOOTIA32.EFI


# LBL Core binary - Stage1 needs to know where to find this.
# This is a dependency, assumed to be built by core's build system.
LBL_CORE_BIN ?= ../build/core/lbl_core.bin # Adjust if path or name differs

# Ensure output directories exist
$(shell mkdir -p $(STAGE1_OUT_DIR) $(DISK_IMG_DIR))

# --- Default Target ---
all: bios uefi

# --- BIOS Targets ---
bios: $(MBR_BIN) $(BOOT32_BIN)

$(MBR_BIN): $(MBR_SRC) stage1/bios/lbl_config_bios.inc
	$(NASM) -f bin $< -o $@ -l $(MBR_LST) -Pstage1/bios/lbl_config_bios.inc

# STAGE1_COMMON_OBJ_BIOS: C utilities for BIOS (if boot_32.asm links C)
# This is complex if boot_32.asm is pure assembly loading a flat core binary.
# If boot_32.asm needs to call C functions from stage1_loader_utils.c:
$(STAGE1_COMMON_OBJ_BIOS): $(STAGE1_COMMON_SRC_C) $(STAGE1_COMMON_HDR_C)
	$(BIOS_CC) $(BIOS_CFLAGS) $< -o $@

$(BOOT32_BIN): $(BOOT32_SRC) stage1/bios/lbl_config_bios.inc # $(STAGE1_COMMON_OBJ_BIOS)
	# If boot_32.asm is pure assembly:
	$(NASM) -f bin $< -o $@ -l $(BOOT32_LST) -Pstage1/bios/lbl_config_bios.inc
	# If boot_32.asm needs to be linked with C objects like STAGE1_COMMON_OBJ_BIOS:
	# $(NASM) -f elf32 $(BOOT32_SRC) -o $(STAGE1_OUT_DIR)/boot_32.o -l $(BOOT32_LST) -Pstage1/bios/lbl_config_bios.inc
	# $(LD) $(BIOS_LDFLAGS) $(STAGE1_OUT_DIR)/boot_32.o $(STAGE1_COMMON_OBJ_BIOS) -o $@

# --- UEFI Targets ---
uefi: uefi_x64 uefi_ia32

# Common C utility, compiled for UEFI x64 context
$(STAGE1_COMMON_OBJ_UEFI_X64): $(STAGE1_COMMON_SRC_C) $(STAGE1_COMMON_HDR_C)
	$(UEFI_X64_CC) $(UEFI_X64_CFLAGS) -c $< -o $@

$(LBL_UEFI_X64_EFI): $(UEFI_LOADER_SRC_C) $(UEFI_LOADER_HDR_C) $(STAGE1_COMMON_OBJ_UEFI_X64) \
                     stage1/uefi/gnu-efi/crt0-efi-x86_64.o \
                     stage1/uefi/gnu-efi/lib/libgnuefi.a \
                     stage1/uefi/gnu-efi/lib/libefi.a
	$(UEFI_X64_CC) $(UEFI_X64_CFLAGS) \
		$(UEFI_LOADER_SRC_C) $(STAGE1_COMMON_OBJ_UEFI_X64) \
		-o $(LBL_UEFI_X64_ELF) $(UEFI_X64_LDFLAGS)
	$(UEFI_X64_OBJCOPY) -j .text -j .sdata -j .data -j .dynamic -j .dynsym -j .rel -j .rela -j .reloc \
		--target=efi-app-x86_64 $(LBL_UEFI_X64_ELF) $@

uefi_x64: $(LBL_UEFI_X64_EFI)

# Common C utility, compiled for UEFI IA32 context
$(STAGE1_COMMON_OBJ_UEFI_IA32): $(STAGE1_COMMON_SRC_C) $(STAGE1_COMMON_HDR_C)
	$(UEFI_IA32_CC) $(UEFI_IA32_CFLAGS) -c $< -o $@

$(LBL_UEFI_IA32_EFI): $(UEFI_LOADER_SRC_C) $(UEFI_LOADER_HDR_C) $(STAGE1_COMMON_OBJ_UEFI_IA32) \
                      stage1/uefi/gnu-efi/crt0-efi-ia32.o \
                      stage1/uefi/gnu-efi/lib/libgnuefi.a \
                      stage1/uefi/gnu-efi/lib/libefi.a
	$(UEFI_IA32_CC) $(UEFI_IA32_CFLAGS) \
		$(UEFI_LOADER_SRC_C) $(STAGE1_COMMON_OBJ_UEFI_IA32) \
		-o $(LBL_UEFI_IA32_ELF) $(UEFI_IA32_LDFLAGS)
	$(UEFI_IA32_OBJCOPY) -j .text -j .sdata -j .data -j .dynamic -j .dynsym -j .rel -j .rela -j .reloc \
		--target=efi-app-ia32 $(LBL_UEFI_IA32_ELF) $@

uefi_ia32: $(LBL_UEFI_IA32_EFI)


# --- Build Disk Image (Example for BIOS) ---
# This requires that LBL_CORE_BIN, MBR_BIN, BOOT32_BIN, and config files are available.
# A proper image creation script in `tools/` is better.
LBL_BIOS_FLOPPY_IMG = $(DISK_IMG_DIR)/lionbootloader_bios_floppy.img
LBL_DEFAULT_CONFIG_JSON = ../config/default.json # Path to default JSON config

image_bios_floppy: $(MBR_BIN) $(BOOT32_BIN) $(LBL_CORE_BIN) $(LBL_DEFAULT_CONFIG_JSON)
	@echo "Creating BIOS floppy disk image (example)..."
	# Create a 1.44MB floppy image file
	dd if=/dev/zero of=$(LBL_BIOS_FLOPPY_IMG) bs=1024 count=1440
	# Format as FAT12 (mformat is part of mtools)
	# Ensure the partition for mformat doesn't include the MBR if MBR handles FS itself.
	# For simplicity, let's assume MBR is written first, then a FS is made on the *rest* of the disk
	# then files are copied. A more robust way uses a partition table.
	# This example assumes a raw FAT filesystem starting at sector 0 after MBR.
	# This is non-standard for hard disks but common for floppies where MBR *is* the VBR.
	
	# Write MBR (which also acts as VBR for FAT on a floppy)
	dd if=$(MBR_BIN) of=$(LBL_BIOS_FLOPPY_IMG) conv=notrunc

	# Use mtools to copy files to the FAT filesystem on the image.
	# The MBR/VBR should have set up FAT parameters so mtools can understand it.
	# This assumes MBR_BIN IS a FAT12/16 boot sector.
	# If MBR_BIN is just a loader stub, you need to format first:
	# mkfs.fat -F 12 $(LBL_BIOS_FLOPPY_IMG) # This would overwrite MBR if not careful.
	#
	# A common approach for floppies: MBR *is* the VBR.
	# MBR loads BOOT32_BIN from subsequent sectors. BOOT32_BIN understands FAT to load LBL_CORE_BIN.
	# Copy BOOT32_BIN to a known location (e.g. sector 1, if MBR is VBR and expects it there implicitly)
	# Or, if MBR itself is simple and loads BOOT32_BIN from file "LBLBT32.SYS":
	# mcopy -i $(LBL_BIOS_FLOPPY_IMG) $(BOOT32_BIN) ::/LBLBT32.SYS

	# For this conceptual Make, let's assume `tools/mkimage.sh` handles the complexity.
	../tools/mkimage.sh bios_floppy $(LBL_BIOS_FLOPPY_IMG) $(MBR_BIN) $(BOOT32_BIN) $(LBL_CORE_BIN) $(LBL_DEFAULT_CONFIG_JSON)
	@echo "BIOS floppy image $(LBL_BIOS_FLOPPY_IMG) creation process invoked."


# --- Clean ---
clean:
	@echo "Cleaning Stage1 build artifacts..."
	rm -f $(STAGE1_OUT_DIR)/*.bin $(STAGE1_OUT_DIR)/*.lst $(STAGE1_OUT_DIR)/*.o
	rm -f $(STAGE1_OUT_DIR)/*.elf $(STAGE1_OUT_DIR)/*.EFI
	rm -f $(DISK_IMG_DIR)/*.img
	@echo "Stage1 clean complete."

.PHONY: all bios uefi uefi_x64 uefi_ia32 clean image_bios_floppy