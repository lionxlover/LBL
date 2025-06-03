// Lionbootloader GUI - UI Manager and Main Loop
// File: gui/src/ui.rs

#[cfg(feature = "with_alloc")]
use alloc::{string::String, vec::Vec, rc::Rc, cell::RefCell}; // Rc/RefCell for shared UI state if needed

use lionbootloader_core_lib::config::schema_types::{BootEntry, AdvancedSettings};
use lionbootloader_core_lib::logger;

use crate::{GuiContext, GuiSelectionResult};
use crate::input::{self, InputEvent, KeyCode};
use crate::renderer; // For triggering redraws
use crate::theme::AppliedTheme; // Assuming theme module provides this
use crate::widgets; // For UI elements

// --- UI State ---

/// Represents the current active screen/view in the GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveScreen {
    BootMenu,
    Settings,
    // LogViewer,
    // PluginManagerView,
    // ShutdownConfirm,
    // RebootConfirm,
    // ErrorScreen,
}

/// Global UI state.
/// This might be wrapped in Rc<RefCell<UiState>> if shared mutably across UI components.
struct UiState<'a> {
    active_screen: ActiveScreen,
    boot_entries: &'a [BootEntry], // Reference to entries from LblConfig
    selected_entry_index: usize,
    // settings_state: SettingsScreenState, // State for the settings screen
    // theme_context: AppliedTheme<'a>,     // Current theme for drawing (from theme.rs)
    should_exit_loop: bool,
    exit_result: Option<GuiSelectionResult>,
    countdown_timer_ms: Option<u32>, // Remaining time for boot countdown
    last_tick_time_ms: u64,          // For animations and timers
    redraw_requested: bool,
    // Layouts for different screens
    // main_menu_layout: MainMenuLayout,
    // settings_layout: SettingsLayout,
}

// Placeholder for screen-specific layouts and states
// struct MainMenuLayout {
//     search_bar_rect: Rect,
//     entry_list_rect: Rect,
//     status_bar_rect: Rect,
//     // ... positions and sizes of elements
// }


// --- Main UI Loop ---

/// Runs the main GUI event loop.
/// This function is called by `gui::lib::run_main_loop`.
#[cfg(feature = "with_alloc")] // Most complex UI logic will use alloc
pub fn run_loop(context: &GuiContext) -> GuiSelectionResult {
    logger::info!("[UI] Entering main UI loop...");

    // Create initial UI state
    let mut ui_state = UiState {
        active_screen: ActiveScreen::BootMenu,
        boot_entries: &context.config.entries,
        selected_entry_index: context.config.advanced.default_boot_entry_id.as_ref()
            .and_then(|default_id| context.config.entries.iter().position(|e| &e.id == default_id))
            .unwrap_or(0), // Default to first entry if not found or not set
        should_exit_loop: false,
        exit_result: None,
        countdown_timer_ms: if context.config.advanced.show_countdown { Some(context.config.timeout_ms) } else { None },
        last_tick_time_ms: context.hal_services.get_current_time_ms(), // Needs HAL timer
        redraw_requested: true, // Initial draw
    };

    // Sort boot entries by 'order' field before displaying initially
    // This is complex here because ui_state.boot_entries is a slice.
    // Sorting should ideally happen when LblConfig is parsed, or entries are copied to a Vec here.
    // For now, we assume they are pre-sorted or we iterate based on a sorted index list.


    while !ui_state.should_exit_loop {
        // 1. Process Time and Timers
        let current_time_ms = context.hal_services.get_current_time_ms(); // Needs HAL timer
        let delta_time_ms = current_time_ms.saturating_sub(ui_state.last_tick_time_ms);
        ui_state.last_tick_time_ms = current_time_ms;

        if let Some(ref mut countdown) = ui_state.countdown_timer_ms {
            if *countdown > 0 {
                *countdown = countdown.saturating_sub(delta_time_ms as u32);
                if *countdown == 0 {
                    logger::info!("[UI] Boot countdown reached zero.");
                    // Select the default/currently selected entry for timeout
                    let selected_entry = ui_state.boot_entries.get(ui_state.selected_entry_index)
                        .expect("Selected entry index out of bounds at timeout"); // Should not happen
                    ui_state.exit_result = Some(GuiSelectionResult::BootEntrySelected(selected_entry.clone()));
                    ui_state.should_exit_loop = true;
                    continue; // Skip rest of loop iteration
                }
                ui_state.redraw_requested = true; // Countdown changed, request redraw
            }
        }

        // 2. Process Input
        //    `input::poll_events()` would get events from HAL (keyboard, mouse, touch).
        //    This is simplified; input might be pushed to a queue that `poll_events` reads.
        while let Some(event) = input::poll_pending_event() { // Needs input::poll_pending_event()
            handle_input_event(&mut ui_state, &event, context);
            if ui_state.should_exit_loop { break; }
        }
        if ui_state.should_exit_loop { continue; }


        // 3. Update UI Logic / State
        //    - Based on current screen, input events, and time.
        //    - Update animations (`crate::animations::update_animations(delta_time_ms)`).
        //    - Handle transitions between screens.
        //    - This is where screen-specific logic would reside.
        match ui_state.active_screen {
            ActiveScreen::BootMenu => {
                // Logic for boot menu screen (e.g., list navigation, search)
                // update_boot_menu_state(&mut ui_state, delta_time_ms);
            }
            ActiveScreen::Settings => {
                // Logic for settings screen
                // update_settings_state(&mut ui_state, delta_time_ms);
            }
            // ... other screens
        }

        // 4. Render UI (if needed)
        if ui_state.redraw_requested {
            // `renderer::begin_frame()` and `renderer::end_frame()` would manage drawing cycle.
            renderer::begin_frame().expect("Begin frame failed"); // Error handling needed

            match ui_state.active_screen {
                ActiveScreen::BootMenu => draw_boot_menu_screen(&ui_state, context),
                ActiveScreen::Settings => draw_settings_screen(&ui_state, context),
                // ... other screens
            }

            renderer::end_frame().expect("End frame failed"); // Error handling needed
            ui_state.redraw_requested = false;
        }

        // 5. Yield/Sleep (important in a real-time loop to not peg CPU)
        //    In a bootloader, this might be a short HAL delay or a HLT instruction if idle.
        //    `context.hal_services.yield_cpu_short_pause();`
        //    For 60 FPS target, loop should take ~16ms. If faster, insert small delay.
    }

    logger::info!("[UI] Exiting main UI loop.");
    ui_state.exit_result.unwrap_or_else(|| {
        logger::warn!("[UI] Loop exited without a specific result, defaulting to Timeout/Error.");
        GuiSelectionResult::Error("GUI loop exited unexpectedly.".into()) // Fallback
    })
}


/// Handles a single input event and updates UI state accordingly.
#[cfg(feature = "with_alloc")]
fn handle_input_event(ui_state: &mut UiState, event: &InputEvent, context: &GuiContext) {
    ui_state.redraw_requested = true; // Assume most inputs cause a redraw

    match ui_state.active_screen {
        ActiveScreen::BootMenu => {
            match event {
                InputEvent::KeyPress { key_code, .. } => match key_code {
                    KeyCode::UpArrow => {
                        if ui_state.selected_entry_index > 0 {
                            ui_state.selected_entry_index -= 1;
                        } else {
                            ui_state.selected_entry_index = ui_state.boot_entries.len().saturating_sub(1); // Wrap to bottom
                        }
                        ui_state.countdown_timer_ms = None; // Stop countdown on interaction
                    }
                    KeyCode::DownArrow => {
                        if ui_state.selected_entry_index < ui_state.boot_entries.len().saturating_sub(1) {
                            ui_state.selected_entry_index += 1;
                        } else {
                            ui_state.selected_entry_index = 0; // Wrap to top
                        }
                         ui_state.countdown_timer_ms = None; // Stop countdown
                    }
                    KeyCode::Enter => {
                        if let Some(entry) = ui_state.boot_entries.get(ui_state.selected_entry_index) {
                            ui_state.exit_result = Some(GuiSelectionResult::BootEntrySelected(entry.clone()));
                            ui_state.should_exit_loop = true;
                        }
                    }
                    KeyCode::Escape => {
                        // No action on Esc in main menu for now, or could be shutdown/reboot sequence.
                        // For simplicity, let's make it trigger shutdown/reboot confirmation/action.
                        // This needs a proper confirmation dialog in a real UI.
                        logger::info!("[UI] Escape pressed, initiating reboot (conceptual).");
                        ui_state.exit_result = Some(GuiSelectionResult::Reboot);
                        ui_state.should_exit_loop = true;
                    }
                    KeyCode::F1 => { /* Help? */ }
                    KeyCode::F2 | KeyCode::S => { // 'S' for settings
                        logger::info!("[UI] Transitioning to Settings screen.");
                        ui_state.active_screen = ActiveScreen::Settings;
                        ui_state.countdown_timer_ms = None;
                    }
                    KeyCode::F10 => { /* Debug Shell? */
                        ui_state.exit_result = Some(GuiSelectionResult::EnterDebugShell);
                        ui_state.should_exit_loop = true;
                    }
                    _ => { ui_state.redraw_requested = false; } // No relevant key, no redraw
                },
                InputEvent::MouseMove { .. } => { /* Handle mouse hover for selection */ }
                InputEvent::MouseClick { .. } => { /* Handle mouse click for selection or buttons */ }
                _ => { ui_state.redraw_requested = false; }
            }
        }
        ActiveScreen::Settings => {
            match event {
                InputEvent::KeyPress { key_code, .. } => match key_code {
                    KeyCode::Escape => {
                        logger::info!("[UI] Transitioning back to Boot Menu from Settings.");
                        ui_state.active_screen = ActiveScreen::BootMenu;
                        // Re-initialize countdown if it was active for boot menu
                        if context.config.advanced.show_countdown {
                            ui_state.countdown_timer_ms = Some(context.config.timeout_ms);
                        }
                    }
                    // ... other settings screen specific key handling (tabs, apply, cancel)
                    _ => { ui_state.redraw_requested = false; }
                },
                _ => { ui_state.redraw_requested = false; }
            }
        }
        // ... handle events for other screens
    }
}

/// Initializer for the UI manager (called once by `gui::lib::init_gui`).
#[cfg(feature = "with_alloc")]
pub fn init_ui_manager(_entries: &'a [BootEntry], _advanced_settings: &AdvancedSettings) {
    logger::info!("[UI] UI Manager initialized (stub).");
    // Pre-calculate layouts, load resources common to all screens, etc.
    // For example, sort entries based on `order` field here if not done by core.
}


// --- Drawing Functions (Stubs) ---
// These would use `crate::renderer` and `crate::theme` functions.

#[cfg(feature = "with_alloc")]
fn draw_boot_menu_screen(ui_state: &UiState, context: &GuiContext) {
    // Clear screen with theme background
    // renderer::clear_screen(theme_context.background_color());

    // Draw sidebar (Config, Plugins, Logs) - conceptual, per spec
    // renderer::draw_rect(sidebar_rect, theme_context.panel_color());

    // Draw boot entries list
    let mut y_offset = 50; // Example starting Y
    for (index, entry) in ui_state.boot_entries.iter().enumerate() {
        let display_text = format!("{}. {}", index + 1, entry.title);
        let text_color = if index == ui_state.selected_entry_index {
            // theme_context.accent_color()
            0xFF_FF_00_00 // Red for selected (example)
        } else {
            // theme_context.text_color()
            0xFF_EE_EE_EE // White (example)
        };
        renderer::draw_text(&display_text, 50, y_offset, text_color /* &font_handle */);
        y_offset += 30; // Example line height
    }

    // Draw status bar
    // let status_text = format!("LBL v0.1 | Countdown: {}s | Firmware: {}",
    //    ui_state.countdown_timer_ms.unwrap_or(0) / 1000,
    //    context.hal_services.get_firmware_type_str()); // Needs HAL function
    // renderer::draw_text(&status_text, status_bar_x, status_bar_y, theme_context.secondary_text());
    if let Some(countdown) = ui_state.countdown_timer_ms {
         renderer::draw_text(&format!("Booting in: {}s", countdown/1000), 20, 20, 0xFF_CC_CC_CC);
    }

    // Draw "macOS-style" elements as per your HTML mockup reference later.
    // This would involve drawing rounded rectangles, icons, applying glassmorphism-like effects
    // (semi-transparent panels), using the theme colors from context.config.theme.
}

#[cfg(feature = "with_alloc")]
fn draw_settings_screen(_ui_state: &UiState, _context: &GuiContext) {
    // renderer::clear_screen(theme_context.background_color());
    renderer::draw_text("Settings Screen (Not Implemented)", 50, 50, 0xFF_EE_EE_EE);
    renderer::draw_text("Press ESC to return", 50, 80, 0xFF_AA_AA_AA);
}

// --- No Alloc Stubs ---
#[cfg(not(feature = "with_alloc"))]
pub fn run_loop(_контекст: &GuiContext) -> GuiSelectionResult {
    logger::info!("[UI] Main UI loop (no_alloc - STUBBED).");
    // A no_alloc GUI would be much simpler, likely text-based or very basic graphics.
    // For now, we can imagine it just presents a numbered list and waits for number + Enter.
    // renderer::draw_text("Select an option (no_alloc mode):", 0,0,0);
    // ...
    // Return a default or error for the stub.
    GuiSelectionResult::Error // Placeholder
}
#[cfg(not(feature = "with_alloc"))]
pub fn init_ui_manager(_entries: &'a [BootEntry], _advanced_settings: &AdvancedSettings) {
    // No-op or minimal setup for no_alloc
}

// Helper method for HAL time (conceptual)
// This needs to be provided by HalServices actually
impl<'a> lionbootloader_core_lib::hal::HalServices {
    fn get_current_time_ms(&self) -> u64 {
        // TODO: Implement this in actual HAL using a hardware timer (PIT, HPET, RTC, or firmware service)
        // For now, a dummy incrementing value if we had mutable access or a global static.
        // This is a simplification for the UI loop structure.
        // A real impl would be: self.timer_service.current_time_ms()
        static mut FAKE_TIME: u64 = 0;
        unsafe {
            FAKE_TIME += 16; // Simulate ~16ms per "frame"
            FAKE_TIME
        }
    }
}