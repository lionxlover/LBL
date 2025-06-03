#![no_std]
#![no_main] // We will define our own _start entry point

// Link to the library part of our crate where lbl_core_entry is defined
extern crate lionbootloader_core_lib as lion_core; // Use the name from core/Cargo.toml [lib].name

use core::arch::asm;
use core::panic::PanicInfo;

// If boot_info_ptr is to be passed from assembly _start setup
// It needs to be globally accessible or passed through registers.
// For simplicity, if stage1 prepares it and _start can access it:
// static mut BOOT_INFO_PTR_GLOBAL: *const u8 = core::ptr::null();

/// Entry point for the Lionbootloader Core Engine when compiled as a binary.
///
/// The Stage 1 loader (BIOS or UEFI) is responsible for:
/// 1. Loading this core engine binary into memory.
/// 2. Setting up a valid stack.
/// 3. Switching to the correct CPU mode (e.g., 32-bit or 64-bit protected/long mode).
/// 4. Optionally, placing a pointer to a boot information structure in a known
///    register (e.g., `rdi` on x86_64, `ebx` or passed on stack for BIOS) or memory location.
/// 5. Jumping to this `_start` symbol.
///
/// This `_start` function should not return. It will eventually call `lbl_core_entry`
/// which also does not return (it either boots an OS or halts/reboots).
#[no_mangle]
pub unsafe extern "C" fn _start(boot_info_arg1: usize, boot_info_arg2: usize) -> ! {
    // --- Minimal Stack Setup (if not already perfectly set by Stage 1) ---
    // On some platforms/scenarios, you might need to ensure the stack pointer is valid
    // or switch to a new stack allocated by the core engine itself from available memory.
    // This is highly architecture-specific. For now, assume Stage 1 provides a good stack.

    // --- CPU Specific Initializations (Minimal) ---
    // Example: Clear direction flag on x86 (good practice)
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        // Clear direction flag (CLD)
        // For C code `asm volatile("cld");`
        // In Rust:
        // unsafe { asm!("cld", options(nomem, nostack, preserves_flags)); }
    }

    // --- Retrieve Boot Information ---
    // The way `boot_info_ptr` is passed from Stage 1 to `_start` and then to
    // `lbl_core_entry` depends on the agreed ABI between Stage 1 and Core.
    //
    // Option 1: Stage 1 passes it via registers conventions.
    //   - On x86_64 System V ABI, the first argument is in RDI.
    //   - On i386 cdecl, arguments are on the stack.
    // Let's assume for x86_64, boot_info_ptr is in the first argument register.
    // The arguments `boot_info_arg1` and `boot_info_arg2` are placeholders.
    // Typically, one pointer-sized argument is enough.
    let boot_info_ptr = boot_info_arg1 as *const u8; // Assuming it's passed as the first arg.

    // --- Call the main library function ---
    // The `lion_core::lbl_core_entry` function handles all further initialization and logic.
    lion_core::lbl_core_entry(boot_info_ptr);

    // `lbl_core_entry` should not return. If it does, it's an error similar to panic.
    // We can call the panic handler or a specific halt function.
    // For now, if it ever returns (which it shouldn't), loop indefinitely.
    hcf(); // Halt and Catch Fire
}

/// This function is called on panic.
/// This is a duplicate of the one in lib.rs if this main.rs is linked with lib.rs
/// and both define it. If compiling as a single binary, only one is needed.
/// If lib.rs is a true lib, then this panic_handler in main.rs takes precedence for the bin.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Call the panic handler from the library, if desired, or implement directly.
    // For now, a simple halt.
    // Ideally, print `info` to console/serial if possible.
    // log::error!("CORE BIN PANIC: {}", info); // if logger is set up
    hcf();
}

/// A simple halt function.
fn hcf() -> ! {
    loop {
        // On x86, `hlt` instruction can be used.
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
        // For other architectures, a simple loop might be the only option
        // or a platform-specific shutdown/reset mechanism.
    }
}

// If this crate were also to support being run in a hosted environment (e.g., for testing
// parts of the logic that don't depend on `no_std` features), you might have a conditional
// `main` function like this:
/*
#[cfg(feature = "std")]
fn main() {
    // This main function would only be compiled if 'std' feature is enabled.
    // It would run in a normal OS environment.
    // Not typically used for the core bootloader binary itself.
    println!("Lionbootloader Core (std mode - for testing/dev builds only)");
    // You could call test functions here.
}
*/