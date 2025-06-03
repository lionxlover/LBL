// Lionbootloader Core - Loader Architecture Adapter
// File: core/src/loader/arch_adapter.rs

use crate::hal::HalServices; // To get memory map, ACPI ptr, etc.
use crate::loader::kernel_formats::{KernelInfo, LoadedImageInfo};
use crate::logger;

/// Prepares the architecture-specific environment and jumps to the kernel.
/// This function DOES NOT RETURN.
///
/// # Arguments
/// * `hal`: HAL services providing system information (memory map, etc.).
/// * `kernel_info`: Information about the loaded kernel (entry point, load address).
/// * `initrd_info`: Optional information about the loaded initrd.
/// * `cmdline`: The kernel command line string.
/// * Other parameters might include memory map, ACPI table pointers, framebuffer info, etc.
pub fn prepare_and_jump_to_kernel(
    hal: &HalServices,
    kernel_info: KernelInfo,
    initrd_info: Option<LoadedImageInfo>,
    cmdline: &str,
    // framebuffer_info: Option<FramebufferInfo>, // Example
    // acpi_rsdp_ptr: Option<u64>,             // Example
) -> ! {
    logger::info!(
        "[ArchAdapter] Preparing to jump to kernel at {:#x} with cmdline: '{}'",
        kernel_info.entry_point,
        cmdline
    );
    if let Some(ref initrd) = initrd_info {
        logger::info!(
            "[ArchAdapter] Initrd at {:#x}, size {} bytes",
            initrd.load_address,
            initrd.size
        );
    }

    // Dispatch to architecture-specific function
    #[cfg(target_arch = "x86_64")]
    {
        x86_64::jump_to_kernel_x86_64(hal, kernel_info, initrd_info, cmdline);
    }
    #[cfg(target_arch = "x86")] // i686, etc.
    {
        x86::jump_to_kernel_x86(hal, kernel_info, initrd_info, cmdline);
    }
    #[cfg(target_arch = "aarch64")]
    {
        aarch64::jump_to_kernel_aarch64(hal, kernel_info, initrd_info, cmdline);
    }
    #[cfg(target_arch = "arm")]
    {
        arm::jump_to_kernel_arm(hal, kernel_info, initrd_info, cmdline);
    }
    #[cfg(target_arch = "riscv64")]
    {
        riscv::jump_to_kernel_riscv(hal, kernel_info, initrd_info, cmdline, true);
    }
    #[cfg(target_arch = "riscv32")]
    {
        riscv::jump_to_kernel_riscv(hal, kernel_info, initrd_info, cmdline, false);
    }
    // Add other #[cfg(target_arch = "...")] blocks for PowerPC, MIPS, etc.

    // Fallback if no specific architecture is matched (should ideally not happen with correct cfgs)
    logger::error!(
        "[ArchAdapter] Target architecture '{}' not supported for kernel jump or no cfg matched.",
        core::env::consts::ARCH
    );
    panic!("Unsupported target architecture for kernel jump: {}", core::env::consts::ARCH);
}

// --- x86_64 specific implementation ---
#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use super::*; // Import parent types
    use core::arch::asm;
    // Potentially use crates like `x86_64` for structure definitions or CPU operations.

    // Define a boot information structure compatible with common x86_64 kernel expectations
    // (e.g., similar to Multiboot2, Linux boot protocol, or a custom LBL protocol).
    // This is highly dependent on the kernels LBL aims to boot.
    #[repr(C)]
    struct LblBootParamsX86_64 {
        magic: u64,
        version: u32,
        kernel_entry: u64,
        kernel_base: u64,
        kernel_size: u64,
        initrd_base: u64,
        initrd_size: u64,
        cmdline_ptr: u64, // Pointer to null-terminated command line string
        memory_map_ptr: u64,
        memory_map_entries: u64,
        framebuffer_addr: u64,
        framebuffer_width: u32,
        framebuffer_height: u32,
        framebuffer_pitch: u32,
        framebuffer_bpp: u8,
        acpi_rsdp_ptr: u64,
        // ... other fields
    }
    const LBL_BOOT_MAGIC_X86_64: u64 = 0xLBLx86_64BT;


    pub(super) fn jump_to_kernel_x86_64(
        _hal: &HalServices, // Use HAL to get memory map, ACPI, framebuffer etc.
        kernel_info: KernelInfo,
        initrd_info: Option<LoadedImageInfo>,
        cmdline: &str,
    ) -> ! {
        logger::info!("[x86_64] Preparing for kernel jump.");

        // 1. Finalize Memory Management:
        //    - Ensure kernel, initrd, boot params, and stack are in safe memory regions.
        //    - If LBL set up its own paging, it might need to map the kernel appropriately
        //      or tear down its own paging before jumping if kernel expects identity mapping.
        //    - This is a complex step. For now, assume memory is correctly laid out.
        //    - UEFI: Call ExitBootServices() if not already done by Stage1 or earlier core.
        //      This is critical. After this, only runtime services (if any) are available,
        //      and LBL cannot use boot services (like memory allocation, console output via BS).


        // #[cfg(target_os = "uefi")]
        // {
        //    crate::platform::uefi_utils::exit_boot_services();
        //    logger::info!("[x86_64] UEFI ExitBootServices called.");
        //    // After ExitBootServices, logging might stop working if it relied on UEFI console.
        //    // A simple serial or framebuffer logger might be needed from this point on.
        // }


        // 2. Prepare Boot Parameters Structure:
        //    - Allocate memory for LblBootParamsX86_64 and the command line string.
        //    - Populate it with information from kernel_info, initrd_info, HAL, cmdline.
        //    - This structure will be passed to the kernel, typically via a register (e.g., RSI or RDI).
        // For simplicity, let's assume these are prepared on the stack or a known static location.
        // Proper allocation from HAL memory services is needed.
        let boot_params = LblBootParamsX86_64 {
            magic: LBL_BOOT_MAGIC_X86_64,
            version: 1,
            kernel_entry: kernel_info.entry_point,
            kernel_base: kernel_info.load_address,
            kernel_size: kernel_info.size,
            initrd_base: initrd_info.as_ref().map_or(0, |i| i.load_address),
            initrd_size: initrd_info.as_ref().map_or(0, |i| i.size),
            cmdline_ptr: cmdline.as_ptr() as u64, // Kernel needs to copy this
            // Fill these from HAL:
            memory_map_ptr: 0, // Pointer to an array of memory map entries
            memory_map_entries: 0,
            framebuffer_addr: _hal.boot_info.framebuffer_addr,
            framebuffer_width: _hal.boot_info.framebuffer_width,
            framebuffer_height: _hal.boot_info.framebuffer_height,
            framebuffer_pitch: _hal.boot_info.framebuffer_pitch,
            framebuffer_bpp: _hal.boot_info.framebuffer_bpp,
            acpi_rsdp_ptr: 0, // Get from HAL if ACPI was parsed
        };
        let boot_params_ptr = &boot_params as *const _ as u64;

        // 3. Set up CPU State for Kernel:
        //    - Stack: Ensure a valid stack pointer for the kernel.
        //    - GDT/IDT: Kernel will set up its own, but a minimal one might be needed for the jump.
        //    - Paging: If kernel expects to start with paging enabled (e.g., higher-half),
        //      LBL needs to set up preliminary page tables. If kernel sets up its own from scratch,
        //      LBL might jump with identity mapping or paging disabled (if possible and expected).
        //    - Interrupts: Disable interrupts before jump (`cli`).

        // 4. Jump to Kernel Entry Point:
        //    The kernel entry point is `kernel_info.entry_point`.
        //    The boot parameters structure pointer is often passed in RSI (for Linux 64-bit) or RDI.
        //    Other registers might need to be zeroed or set according to kernel ABI.
        //    Example for a Linux-like kernel expecting params in RSI:
        unsafe {
            asm!(
                "cli",                          // Disable interrupts
                "mov rsp, {stack_ptr}",         // Set up a new stack if needed
                // Zero out general purpose registers as per some boot protocols, or set specific ones.
                // "xor rax, rax",
                // "xor rbx, rbx",
                // "xor rcx, rcx",
                // "xor rdx, rdx",
                // "xor rbp, rbp",
                // "xor r8, r8", /* ... through r15 */
                "mov rsi, {boot_params_ptr}",   // Pass boot parameters pointer in RSI
                "mov rdi, {magic_number}",      // Pass magic (e.g. Multiboot magic in EAX/RAX for some protocols)
                                                // Or pass another argument, Linux uses RDI for 32-bit pointer to boot_params on EFI stub
                "jmp {kernel_entry}",           // Jump to kernel
                kernel_entry = in(reg) kernel_info.entry_point,
                boot_params_ptr = in(reg) boot_params_ptr,
                magic_number = in(reg) 0u64, // Placeholder specific magic if kernel expects it
                stack_ptr = in(reg) kernel_info.stack_ptr.unwrap_or(0x0000_0000_FFFF_FFF0), // Example, kernel provides stack or LBL allocates
                options(noreturn, nostack, preserves_flags)
            );
        }
        // The `options(noreturn)` tells Rust this asm block will not return.
    }
}

// --- x86 (32-bit) specific implementation ---
#[cfg(target_arch = "x86")]
mod x86 {
    use super::*;
    // ... 32-bit specific structures and jump logic ...
    // Typically involves setting up GDT, protected mode, maybe basic paging.
    // Boot parameters might be passed via registers (e.g. EBX for Multiboot) or stack.
    pub(super) fn jump_to_kernel_x86(
        _hal: &HalServices,
        _kernel_info: KernelInfo,
        _initrd_info: Option<LoadedImageInfo>,
        _cmdline: &str,
    ) -> ! {
        logger::error!("[x86] 32-bit kernel jump not fully implemented.");
        // UEFI ExitBootServices if applicable (though 32-bit UEFI is rare, IA32 exists)
        // Setup GDT, stack.
        // Setup boot parameters (e.g. Multiboot info structure if booting Multiboot kernel).
        // Point EBX to Multiboot info struct.
        // EAX to Multiboot magic.
        // Jump to kernel entry.
        panic!("x86 32-bit jump stub");
    }
}

// --- AArch64 specific implementation ---
#[cfg(target_arch = "aarch64")]
mod aarch64 {
    use super::*;
    use core::arch::asm;
    // AArch64 boot often expects a Device Tree Blob (DTB) pointer in x0,
    // and to be started at a specific exception level (EL1 or EL2).

    pub(super) fn jump_to_kernel_aarch64(
        _hal: &HalServices, // HAL needs to provide DTB location or way to find/generate it.
        kernel_info: KernelInfo,
        _initrd_info: Option<LoadedImageInfo>, // Initrd location passed in DTB or bootargs
        _cmdline: &str, // Cmdline often part of DTB's /chosen/bootargs
    ) -> ! {
        logger::info!("[AArch64] Preparing for kernel jump.");
        // UEFI ExitBootServices if applicable.

        // 1. Get DTB pointer (from HAL, or passed by firmware). This is critical.
        let dtb_ptr: u64 = 0; // Placeholder, get from HAL based on firmware (e.g. UEFI GetSystemTable -> ConfigurationTable)

        // 2. Ensure CPU is in correct EL (usually EL1 for OS, or EL2 if hypervisor).
        //    LBL might already be in EL1/EL2 if it's a UEFI app.
        //    Switching EL is privileged.

        // 3. MMU/Caches: Kernel often expects MMU off or identity mapped. Or LBL sets up paging.
        //    Invalidate caches if necessary.

        // 4. Jump:
        //    - x0: physical address of DTB.
        //    - x1-x3: usually 0.
        //    - PC: to kernel entry point (`kernel_info.entry_point`).
        //    - Stack: ensure valid stack.
        unsafe {
            asm!(
                // "msr daifset, #0xf", // Disable interrupts (IRQ, FIQ, SError, Debug)
                "mov x0, {dtb_ptr}",
                "mov x1, #0",
                "mov x2, #0",
                "mov x3, #0",
                // "mov sp, {stack_ptr}", // Set new stack if needed
                "br {kernel_entry}",    // Branch to kernel
                kernel_entry = in(reg) kernel_info.entry_point,
                dtb_ptr = in(reg) dtb_ptr,
                // stack_ptr = in(reg) kernel_info.stack_ptr.unwrap_or(...),
                options(noreturn, nostack)
            );
        }
    }
}

// --- ARM (32-bit) specific implementation ---
#[cfg(target_arch = "arm")]
mod arm {
     use super::*;
    // ARM32 boot is diverse (U-Boot, UEFI, custom). DTB passing in r0/r1/r2, ATAGs.
    pub(super) fn jump_to_kernel_arm(
        _hal: &HalServices,
        _kernel_info: KernelInfo,
        _initrd_info: Option<LoadedImageInfo>,
        _cmdline: &str,
    ) -> ! {
        logger::error!("[ARM] 32-bit ARM kernel jump not fully implemented.");
        // Similar to AArch64: setup boot params (ATAGs or DTB pointer in r0-r2).
        // Ensure correct CPU mode (SVC).
        // Jump.
        panic!("ARM 32-bit jump stub");
    }
}
// --- RISC-V specific implementation ---
#[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
mod riscv {
    use super::*;
    use core::arch::asm;
    // RISC-V boot typically expects DTB pointer in a0, hartid in a1 (if from U-Boot/SBI).
    // Kernel runs in S-mode.

    pub(super) fn jump_to_kernel_riscv(
        _hal: &HalServices, // HAL provides DTB info
        kernel_info: KernelInfo,
        _initrd_info: Option<LoadedImageInfo>,
        _cmdline: &str, // Often in DTB
        is_rv64: bool,
    ) -> ! {
        logger::info!("[RISC-V] Preparing for kernel jump ({}).", if is_rv64 {"RV64"} else {"RV32"});
        // UEFI ExitBootServices if applicable. SBI environment should be set up.

        // 1. Get DTB pointer (from HAL, or firmware).
        let dtb_ptr: usize = 0; // Placeholder.
        // 2. Hart ID (current processor core ID). Usually 0 if single core or firmware provides it.
        let hart_id: usize = 0; // Placeholder.

        // 3. Ensure S-mode. MMU setup (e.g. sv39/sv48 for RV64) might be done by LBL or kernel.
        //    Usually, kernel expects identity mapping or MMU off.

        // 4. Jump:
        //    - a0: hartid (convention varies, sometimes DTB ptr)
        //    - a1: DTB physical address
        //    - PC: to kernel_info.entry_point
        //    - Stack: ensure valid stack.
        unsafe {
             // Disable interrupts (clear SIE bit in sstatus)
             // `csrw sie, zero`
            asm!(
                "mv a0, {hart_id}",
                "mv a1, {dtb_ptr}",
                // "mv sp, {stack_ptr}", // Set new stack if needed
                "jr {kernel_entry}",    // Jump to kernel via jump-register
                kernel_entry = in(reg) kernel_info.entry_point,
                hart_id = in(reg) hart_id,
                dtb_ptr = in(reg) dtb_ptr,
                // stack_ptr = in(reg) kernel_info.stack_ptr.unwrap_or(...),
                options(noreturn, nostack)
            );
        }
    }
}