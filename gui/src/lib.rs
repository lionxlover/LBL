#![no_std] // GUI runs in no_std environment
#![cfg_attr(feature = "alloc", feature(alloc_error_handler))] // If GUI manages its own alloc, not just via core

// Conditionally enable the `alloc` crate if the "alloc" feature is active for this crate
// AND if it's not already provided by depending on `lionbootloader_core_lib` which also enables it.
// Typically, if core enables alloc, this crate doesn't need to re-declare `extern crate alloc`.
// However, if GUI could be built standalone with alloc, this is needed.
#[cfg(all(feature = "alloc", not(feature="core_provides_alloc")))] // `core_provides_alloc` is a conceptual feature flag
extern crate alloc;

#[cfg(all(feature = "alloc", not(feature="core_provides_alloc")))]
mod dummy_gui_allocator { // Only if GUI had its own allocator distinct from core
    use alloc::alloc::{GlobalAlloc, Layout};
    use core::ptr;
    pub struct DummyGuiAlloc;
    unsafe impl GlobalAlloc for DummyGuiAlloc {
        unsafe fn alloc(&self, _layout: Layout) -> *mut u8 { ptr::null_mut() }
        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
    }
    #[global_allocator]
    static GUI_ALLOCATOR: DummyGuiAlloc = DummyGuiAlloc;
    #[alloc_error_handler]
    fn alloc_error_handler_gui(layout: Layout) -> ! {
        panic!("GUI Allocation error: layout = {:?}", layout);
    }
}


// --- GUI Module Declarations ---
pub mod animations;     // Animation engine
pub mod input;          // Input event handling and mapping
pub mod renderer;       // Rendering backend (framebuffer, NanoVG, etc.)
pub mod theme;          // Theme parsing and style management
pub mod ui;             // Main UI logic, layout, screen management
pub mod widgets;        // Reusable UI components (buttons, lists, etc.)

// Re-export key types or structs if needed by the core engine directly
// For example, the result of user interaction
#[cfg(feature = "with_alloc")] // Assuming BootEntry uses alloc features from core
use lionbootloader_core_lib::config::schema_types::BootEntry; // For GuiSelectionResult

#[cfg(feature = "with_alloc")]
use alloc::string::String;


/// Represents the user's selection or action from the GUI.
#[cfg(feature = "with_alloc")]
#[derive(Debug, Clone)]
pub enum GuiSelectionResult {
    BootEntrySelected(BootEntry), // The chosen boot entry
    EnterSettings,                // User wants to go to settings/config editor
    EnterDebugShell,              // User wants to access the debug shell
    Shutdown,                     // User selected shutdown
    Reboot,                       // User selected reboot
    Timeout,                      // Boot countdown timed out, default entry should be booted
    Error(String),                // An error occurred within the GUI, core should handle it
}

#[cfg(not(feature = "with_alloc"))]
#[derive(Debug, Clone, Copy)]
pub enum GuiSelectionResult {
    BootEntrySelectedById(u32), // ID of the chosen boot entry
    EnterSettings,
    EnterDebugShell,
    Shutdown,
    Reboot,
    Timeout,
    Error, // Simplified error
}


/// Context required by the GUI from the core engine.
/// This struct bundles all necessary services and data.
pub struct GuiContext<'a> {
    // Using a reference to LblConfig from the core crate.
    // Ensure lifetimes are managed correctly if core reloads config.
    pub config: &'a lionbootloader_core_lib::config::LblConfig,
    pub hal_services: &'a lionbootloader_core_lib::hal::HalServices,
    pub fs_manager: &'a lionbootloader_core_lib::fs::manager::FilesystemManager<'a>,
    // Add other services GUI might need, e.g., a way to trigger reboot/shutdown via core.
    // pub action_Liaison: &'a dyn GuiActionLiaison, // For callbacks to core
}

// pub trait GuiActionLiaison {
//    fn request_reboot(&self);
//    fn request_shutdown(&self);
// }


/// Initializes the GUI system.
///
/// This function should be called by the core engine after HAL, FS, and Config
/// are sufficiently initialized. It sets up the rendering backend, loads themes,
/// fonts, and prepares the initial UI state.
///
/// # Arguments
/// * `context`: A `GuiContext` struct containing references to core services.
///
/// # Returns
/// A `Result` indicating success or an error type specific to GUI initialization.
pub fn init_gui(context: &GuiContext) -> Result<(), GuiInitError> {
    lionbootloader_core_lib::logger::info!("[GUI] Initializing GUI system...");

    // 1. Initialize Renderer
    //    This involves getting framebuffer info from `context.hal_services.boot_info`
    //    and setting up the chosen rendering backend (e.g., mapping framebuffer, initializing NanoVG).
    match renderer::init_renderer(&context.hal_services.boot_info) {
        Ok(_) => lionbootloader_core_lib::logger::info!("[GUI] Renderer initialized successfully."),
        Err(e) => {
            lionbootloader_core_lib::logger::error!("[GUI] Renderer initialization failed: {:?}", e);
            return Err(GuiInitError::RendererInitFailed(e));
        }
    }

    // 2. Load Theme
    //    Parses `context.config.theme` and prepares theme resources (colors, styles).
    //    Also loads fonts specified in the theme using `context.fs_manager`.
    match theme::load_theme(&context.config.theme, context.fs_manager) {
        Ok(_) => lionbootloader_core_lib::logger::info!("[GUI] Theme loaded successfully."),
        Err(e) => {
            lionbootloader_core_lib::logger::error!("[GUI] Theme loading failed: {:?}", e);
            return Err(GuiInitError::ThemeLoadFailed(e));
        }
    }
    
    // 3. Initialize Input System
    //    Sets up handlers for keyboard, mouse, touch (if enabled and supported by HAL).
    input::init_input_system(context.config.advanced.enable_mouse, context.config.advanced.enable_touch);
    lionbootloader_core_lib::logger::info!("[GUI] Input system initialized.");

    // 4. Initialize UI Manager / Main Screen
    //    Creates the main UI structure, loads boot entries into a list, etc.
    ui::init_ui_manager(&context.config.entries, &context.config.advanced);
    lionbootloader_core_lib::logger::info!("[GUI] UI manager initialized.");


    lionbootloader_core_lib::logger::info!("[GUI] GUI system initialization complete.");
    Ok(())
}

/// Runs the main GUI event loop.
///
/// This function takes control and handles user input, updates animations,
/// redraws the UI, and waits for user selection or timeout.
///
/// # Arguments
/// * `context`: A `GuiContext` struct.
///
/// # Returns
/// A `GuiSelectionResult` indicating the user's choice or an event like timeout.
/// This function will block until a selection is made or a terminating event occurs.
pub fn run_main_loop(context: &GuiContext) -> GuiSelectionResult {
    lionbootloader_core_lib::logger::info!("[GUI] Starting GUI main loop...");

    // The ui::run_loop function would contain the actual event processing logic.
    // It would:
    //  - Poll for input events from HAL (via GuiContext or a dedicated input queue).
    //  - Update UI state based on input (hover, selection, navigation).
    //  - Update animations.
    //  - Trigger redraws.
    //  - Handle boot countdown timer.
    //  - Return when a boot entry is selected, settings are chosen, timeout occurs, etc.
    let result = ui::run_loop(context);

    lionbootloader_core_lib::logger::info!("[GUI] Main loop exited with result: {:?}", result);
    result
}

/// Cleans up GUI resources before LBL hands off to an OS or reboots.
/// (Optional, as memory will be reclaimed anyway on OS boot, but good practice).
pub fn shutdown_gui() {
    lionbootloader_core_lib::logger::info!("[GUI] Shutting down GUI system...");
    renderer::shutdown_renderer();
    // Other cleanup if necessary
}


#[derive(Debug)]
pub enum GuiInitError {
    RendererInitFailed(renderer::RendererError),
    ThemeLoadFailed(theme::ThemeError),
    // ... other specific init errors
}

// Panic handler for the GUI crate (if it were a separate binary or needs its own)
// Since it's a library linked by core, core's panic handler will be used.
/*
use core::panic::PanicInfo;
#[panic_handler]
fn panic_gui(info: &PanicInfo) -> ! {
    // Try to log, then halt.
    // lionbootloader_core_lib::logger::emergency_log_gui(format_args!("GUI PANIC: {}", info));
    loop {}
}
*/