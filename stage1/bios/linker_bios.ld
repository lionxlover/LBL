/*
 * Lionbootloader - Stage 1 - BIOS Linker Script (linker_bios.ld)
 *
 * This script would be used if linking multiple object files (e.g., from ASM and C)
 * for the second stage BIOS loader (boot_32).
 *
 * If boot_32.asm is assembled directly to a flat binary ('nasm -f bin ...'),
 * then this linker script is NOT directly used for that process.
 * ORG in assembly handles the addressing for flat binaries.
 *
 * This example assumes boot_32 is loaded by MBR to a specific address,
 * e.g., 0x8000 (linear). The MBR would jump to this address.
 * LBL_STAGE2_LOAD_ADDRESS would be 0x8000.
 */

ENTRY(stage2_start) /* Entry point symbol, must be defined in one of the .o files */

/* LBL_STAGE2_LOAD_ADDRESS should be defined by the build system, e.g., 0x8000 */
/* For this example, hardcode a common address if not passed by CFLAGS/LDFLAGS */
LBL_STAGE2_LOAD_ADDRESS = DEFINED(LBL_STAGE2_LOAD_ADDRESS) ? LBL_STAGE2_LOAD_ADDRESS : 0x8000;

SECTIONS
{
    . = LBL_STAGE2_LOAD_ADDRESS; /* Physical load address of the combined binary */

    .text : AT(ADDR(.text)) /* "AT(ADDR(.text))" is for LMA = VMA */
    {
        *(.entry)           /* Custom section for the very first code */
        *(.text*)           /* All .text sections from input files */
        *(.rodata*)         /* Read-only data */
         /* GDT data can be placed here explicitly if desired */
        . = ALIGN(4);
    }

    .data : AT(ADDR(.data))
    {
        *(.data*)           /* All .data sections */
        . = ALIGN(4);
    }

    .bss : AT(ADDR(.bss))
    {
        *(.bss*)            /* All .bss sections */
        *(COMMON)           /* Common uninitialized data */
        . = ALIGN(4);
    }

    /*
     * The GDT itself, if not directly embedded in .asm, could be defined here
     * or placed by the .asm code into a .gdt section.
     * Example:
     * .gdt ALIGN(8) : AT(ADDR(.gdt))
     * {
     *    KEEP(*(.gdt_null))
     *    KEEP(*(.gdt_code))
     *    KEEP(*(.gdt_data))
     * }
     * The assembly would define symbols like `gdt_start` inside these sections.
     * The GDTR structure would then reference these symbols.
     */


    /DISCARD/ :
    {
        *(.comment)
        *(.note*)
        *(.eh_frame*) /* Exception handling, not typically used in bootloader stage */
    }

    /*
     * Ensure the total size is known or padded if MBR needs to load a fixed number of sectors.
     * However, for a linked ELF then converted to binary, the final binary size is just what it is.
     * The MBR would need to know this size.
     */
    /* Example: Fill to 4KB boundary if needed, for sector alignment */
    /* . = ALIGN(4096); */
}

/*
 * To use this linker script with `ld`:
 * ld -m elf_i386 -T stage1/bios/linker_bios.ld \
 *    -defsym LBL_STAGE2_LOAD_ADDRESS=0x8000 \
 *    stage1_build/boot_32_asm.o stage1_build/stage1_loader_utils_bios.o \
 *    -o stage1_build/boot_32.elf
 *
 * Then convert ELF to binary:
 * objcopy -O binary stage1_build/boot_32.elf stage1_build/boot_32.bin
 */