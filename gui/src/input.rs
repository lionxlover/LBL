// Lionbootloader GUI - Input Handling
// File: gui/src/input.rs

#[cfg(feature = "with_alloc")]
use alloc::collections::VecDeque; // For an event queue

use lionbootloader_core_lib::logger;

// --- Input Event Definitions ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    // Function Keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Special Keys
    Enter, Escape, Backspace, Tab, Space,
    Insert, Delete, Home, End, PageUp, PageDown,
    UpArrow, DownArrow, LeftArrow, RightArrow,
    // Modifiers (usually tracked separately but can be key codes too)
    LeftShift, RightShift, LeftCtrl, RightCtrl, LeftAlt, RightAlt, LeftMeta, RightMeta, // Meta = Windows/Command key
    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadEnter, NumpadPlus, NumpadMinus, NumpadMultiply, NumpadDivide, NumpadDecimal,
    // Other
    CapsLock, ScrollLock, NumLock, PrintScreen, Pause,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModifiersState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Windows/Command key
}

impl ModifiersState {
    pub fn none() -> Self {
        ModifiersState { shift: false, ctrl: false, alt: false, meta: false }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    KeyPress {
        key_code: KeyCode,
        modifiers: ModifiersState,
        // character: Option<char>, // Decoded character, if applicable
    },
    KeyRelease {
        key_code: KeyCode,
        modifiers: ModifiersState,
    },
    MouseMove {
        x: i32,
        y: i32,
        // delta_x: i32,
        // delta_y: i32,
    },
    MousePress {
        button: MouseButton,
        x: i32,
        y: i32,
        modifiers: ModifiersState,
    },
    MouseRelease {
        button: MouseButton,
        x: i32,
        y: i32,
        modifiers: ModifiersState,
    },
    MouseScroll {
        delta_x: f32, // Horizontal scroll
        delta_y: f32, // Vertical scroll (positive for scroll down, negative for scroll up typically)
        x: i32,       // Mouse X at time of scroll
        y: i32,       // Mouse Y at time of scroll
    },
    TouchBegin {
        id: u64, // Touch point ID for multi-touch
        x: i32,
        y: i32,
    },
    TouchMove {
        id: u64,
        x: i32,
        y: i32,
    },
    TouchEnd {
        id: u64,
        x: i32,
        y: i32,
    },
    // Gamepad events (simplified)
    // GamepadButtonPress { pad_id: u8, button: GamepadButton },
    // GamepadAxisMove { pad_id: u8, axis: GamepadAxis, value: f32 },
    Quit, // Event to signal GUI should close (e.g. from window manager, not relevant for LBL)
}


// --- Input System State ---
// Event queue for decoupling input producers (HAL via core) from consumers (GUI loop).
#[cfg(feature = "with_alloc")]
static mut INPUT_EVENT_QUEUE: Option<VecDeque<InputEvent>> = None;
// For no_alloc, a simpler fixed-size ring buffer might be used, or direct polling.

// static mut CURRENT_MODIFIERS: ModifiersState = ModifiersState::none();
// static mut MOUSE_POSITION: (i32, i32) = (0,0);
// static mut IS_MOUSE_ENABLED: bool = true;
// static mut IS_TOUCH_ENABLED: bool = false;

// Simulate a global flag for enabling mouse, normally this would be part of a larger state struct
static MOUSE_ENABLED_FLAG: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);
static TOUCH_ENABLED_FLAG: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);


/// Initializes the input system. Called by `gui::lib::init_gui`.
pub fn init_input_system(mouse_enabled_from_config: bool, touch_enabled_from_config: bool) {
    logger::info!("[Input] Initializing input system...");
    #[cfg(feature = "with_alloc")]
    unsafe {
        INPUT_EVENT_QUEUE = Some(VecDeque::with_capacity(32)); // Pre-allocate for a few events
    }
    
    MOUSE_ENABLED_FLAG.store(mouse_enabled_from_config, core::sync::atomic::Ordering::Relaxed);
    TOUCH_ENABLED_FLAG.store(touch_enabled_from_config, core::sync::atomic::Ordering::Relaxed);

    logger::info!("[Input] Mouse enabled: {}, Touch enabled: {}", is_mouse_enabled(), is_touch_enabled());
    // TODO:
    // - If HAL provides ways to enable/disable specific input devices (e.g., mouse pointer visibility),
    //   do that here based on config.
    // - Register callbacks with HAL for input events, if HAL uses a callback mechanism.
    //   Alternatively, core's main loop might poll HAL for input and then call `push_input_event`.
}

/// Pushes a raw input event from HAL/core into the GUI's event queue.
/// This function would be called by the `core` engine when it receives input from HAL.
#[cfg(feature = "with_alloc")]
pub fn push_input_event(event: InputEvent) {
    unsafe {
        if let Some(queue) = INPUT_EVENT_QUEUE.as_mut() {
            if queue.len() < queue.capacity() { // Avoid unbounded growth if consumer is slow
                queue.push_back(event);
            } else {
                logger::warn!("[Input] Input event queue full, dropping event: {:?}", event);
            }
        } else {
            logger::error!("[Input] push_input_event called before init_input_system or on uninitialized queue.");
        }
    }
}

/// Polls for the next pending input event from the queue.
/// Called by the GUI's main loop (`gui::ui::run_loop`).
#[cfg(feature = "with_alloc")]
pub fn poll_pending_event() -> Option<InputEvent> {
    unsafe {
        if let Some(queue) = INPUT_EVENT_QUEUE.as_mut() {
            queue.pop_front()
        } else {
            None
        }
    }
}

// For no_alloc, event handling might be direct polling from HAL within the GUI loop,
// or HAL directly calls a GUI event handler.
#[cfg(not(feature = "with_alloc"))]
pub fn poll_pending_event() -> Option<InputEvent> {
    // In no_alloc, this might directly call a HAL function:
    // returnlionbootloader_core_lib::hal::input_service::poll_next_gui_event();
    // For now, stub returns None.
    None
}


// --- Utility functions ---
pub fn is_mouse_enabled() -> bool { MOUSE_ENABLED_FLAG.load(core::sync::atomic::Ordering::Relaxed) }
pub fn is_touch_enabled() -> bool { TOUCH_ENABLED_FLAG.load(core::sync::atomic::Ordering::Relaxed) }

// Functions to convert HAL-specific key codes/scan codes to LBL's `KeyCode`.
// These would live here or in HAL, called by core before pushing event.
// pub fn translate_hal_keycode(hal_keycode: u32) -> KeyCode { /* ... */ KeyCode::Unknown }
// pub fn translate_hal_mouse_button(hal_button: u8) -> MouseButton { /* ... */ MouseButton::Other(hal_button) }


// --- Example of how core might feed input to GUI ---
// This code would NOT live in the GUI crate, but in the core engine's main loop.
/*
fn core_level_input_polling_loop(hal: &HalServices, gui_input_queue: &mut InputEventQueue) {
    if let Some(raw_hal_event) = hal.poll_raw_input() { // Conceptual HAL function
        match raw_hal_event.kind {
            RawHalInputType::Keyboard => {
                let key_code = input::translate_hal_keycode(raw_hal_event.code);
                let modifiers = ModifiersState { ... }; // From HAL state
                if raw_hal_event.is_press {
                    gui::input::push_input_event(InputEvent::KeyPress { key_code, modifiers });
                } else {
                    gui::input::push_input_event(InputEvent::KeyRelease { key_code, modifiers });
                }
            }
            RawHalInputType::Mouse => {
                // ... translate and push mouse events ...
            }
            // ...
        }
    }
}
*/