#![no_std] // Don't link the Rust standard library
#![feature(alloc_error_handler)] // Required for implementing the global allocator
#![feature(abi_efiapi)] // For UEFI types if we interact with them directly

// Conditionally enable the `alloc` crate when the `with_alloc` feature is active.
#[cfg(feature = "with_alloc")]
extern crate alloc;

// Global Allocator
// You need to choose and implement a global allocator if `with_alloc` feature is used.
// For example, using `wee_alloc` (add `wee_alloc` to Cargo.toml dependencies).
/*
#[cfg(feature = "with_alloc")]
#[global_allocator]
static ALLOCATOR: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
*/

// Or a dummy allocator for now if you haven't chosen one:
#[cfg(feature = "with_alloc")]
mod dummy_allocator {
    use alloc::alloc::{GlobalAlloc, Layout};
    use core::ptr;

    pub struct Dummy;

    unsafe impl GlobalAlloc for Dummy {
        unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
            ptr::null_mut() // Always fail to allocate
        }
        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
            // Do nothing
        }
    }

    #[global_allocator]
    static DUMMY_ALLOCATOR: Dummy = Dummy;

    #[alloc_error_handler]
    fn alloc_error_handler(layout: Layout) -> ! {
        // For now, just loop indefinitely. A real bootloader should handle this gracefully.
        // Perhaps print an error message to the screen if possible.
        // log::error!("Allocation error: layout = {:?}", layout);
        panic!("Allocation error: layout = {:?}", layout);
    }
}


// --- Module Declarations ---

// Hardware Abstraction Layer
pub mod hal;

// Filesystem Module Manager and drivers
pub mod fs;

// Configuration Loading and Management
pub mod config;

// Boot Executor (Kernel Loader)
pub mod loader;

// Plugin System
pub mod plugins;

// Security Manager (TPM, Secure Boot)
pub mod security;

// Logging facilities
pub mod logger;

// (Optional) GUI interface if tightly coupled with core logic early on
// pub mod gui_iface;


// --- Panic Handler ---
// This function is called on panic.
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Try to use the logger if available and initialized
    // logger::emergency_log(format_args!("CORE PANIC: {}", info));

    // In a real bootloader, you might:
    // 1. Print the error to the screen (if video is initialized).
    // 2. Attempt to reboot the system.
    // 3. Halt the system.
    // For now, we'll just loop indefinitely to halt execution.
    // Ensure this is visible (e.g. by Stage1 setting up some basic text output)
    // or by having a dedicated panic output function.
    
    // If logger is available and we can print:
    // println_via_logger_or_serial!("PANIC: {}", info);

    // Simplest form: loop forever
    loop {}
}


// --- Entry Point ---
// This is the main entry point for the Lionbootloader Core Engine.
// It will be called by Stage 1 (BIOS or UEFI loader).
// The exact signature and calling convention might depend on how Stage 1 is implemented
// and what information it passes.
//
// For simplicity, let's assume Stage 1 passes a pointer to a boot services structure
// or some initial hardware information.
//
// If Stage 1 just jumps to an address, this function needs `pub extern "C"` and `#[no_mangle]`.

/// # Safety
///
/// This function must be called by the Stage 1 bootloader after basic hardware
/// initialization (e.g., setting up a stack, CPU mode, memory map).
/// `boot_info_ptr` is a raw pointer to a structure provided by Stage 1,
/// containing essential information for the core engine to start.
#[no_mangle]
pub unsafe extern "C" fn lbl_core_entry(boot_info_ptr: *const u8) -> ! {
    // 1. Initialize critical subsystems (e.g., logging, basic error handling output)
    //    If a serial port or framebuffer is available from Stage1, initialize it for logging.
    //    The `logger` module should provide a basic init function.
    //    Example: logger::init(boot_info_ptr_for_logger);
    //    For now, we assume some form of console output might be available later.

    // Replace with actual logger init
    // logger::init_early_logging(); // Example for very early logs

    // log::info!("Lionbootloader Core Engine started.");
    // log::info!("Boot info pointer: {:?}", boot_info_ptr);

    // 2. Initialize Hardware Abstraction Layer (HAL)
    //    This will involve parsing `boot_info_ptr` to understand memory map,
    //    available devices passed by Stage1, etc.
    //    let hal_services = hal::initialize(boot_info_ptr);
    //    log::info!("HAL initialized.");

    // 3. Initialize Allocator (if `with_alloc` is enabled and not using a static dummy)
    //    This step might be part of hal::initialize or done here if HAL provides memory regions.
    //    #[feature = "with_alloc"]
    //    {
    //        // Initialize your chosen allocator with memory regions from HAL.
    //        // e.g., my_allocator::init(hal_services.memory_map());
    //        log::info!("Global allocator initialized.");
    //    }


    // 4. Initialize Filesystem Manager and load FS plugins
    //    let fs_manager = fs::initialize_manager(&hal_services);
    //    log::info!("Filesystem manager initialized.");

    // 5. Load Configuration
    //    - Find config file (e.g., on a FAT partition located by fs_manager)
    //    - Parse and validate JSON config
    //    let config_data = config::load_configuration(&fs_manager, "/lbl/config.json"); // Path might be configurable or searched
    //    log::info!("Configuration loaded and validated.");
    //    logger::set_log_level(config_data.advanced_settings.log_level); // Adjust log level from config

    // 6. Initialize Security Manager
    //    let security_manager = security::initialize(&hal_services, &config_data.security_settings);
    //    log::info!("Security manager initialized.");

    // 7. Initialize GUI & UX Layer
    //    This is where the `gui` crate would be initialized.
    //    The Core engine would pass necessary info (HAL, Config, etc.) to the GUI.
    //    log::info!("Initializing GUI...");
    //    let gui_facade = lionbootloader_gui::init_gui(hal_services, config_data, fs_manager, security_manager);
    //    log::info!("GUI Initialized. Running main loop.");
    //    let selected_boot_entry = gui_facade.run_main_loop(); // This function would block until user makes a selection or timeout

    // 8. Perform Boot Execution based on GUI selection
    //    log::info!("Boot entry selected: {:?}", selected_boot_entry.id);
    //    loader::execute_boot(
    //        &hal_services,
    //        &fs_manager,
    //        &security_manager,
    //        selected_boot_entry,
    //    );

    // If execute_boot returns, it means booting failed.
    // log::error!("Failed to boot selected entry. Returning to GUI or recovery.");
    // gui_facade.show_boot_failure_screen(selected_boot_entry);
    // loop {} // Or re-enter GUI main loop or recovery mode

    // Placeholder until real logic is implemented
    // Print directly to a known framebuffer or serial if available from boot_info_ptr
    // For example, if Stage 1 provides a simple print function:
    // unsafe {
    //     let print_fn: unsafe extern "C" fn(*const u8) = core::mem::transmute(SOME_PRINT_FN_ADDRESS_FROM_STAGE1);
    //     print_fn("LBL Core Entry Reached!\n\0".as_ptr());
    // }

    // Minimal action: halt
    loop {} // Should not be reached if boot is successful
}

// Example function to be called from main.rs if this is a lib.
// Or main.rs could directly call lbl_core_entry.
pub fn core_init_and_run(boot_info_ptr: *const u8) -> ! {
    unsafe { lbl_core_entry(boot_info_ptr) }
}