// Lionbootloader GUI - Animations
// File: gui/src/animations.rs

#[cfg(feature = "with_alloc")]
use alloc::{vec::Vec, boxed::Box};

use lionbootloader_core_lib::logger;
// Timer access would come via GuiContext -> HalServices
// use lionbootloader_core_lib::hal::HalTimerService; // Conceptual

// --- Animation Primitives ---

/// Easing functions determine the rate of change of a parameter over time.
#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseInQuad,    // t^2
    EaseOutQuad,   // 1 - (1-t)^2
    EaseInOutQuad, // if t < 0.5: 2*t^2, else: 1 - (-2*t+2)^2 / 2
    // Add more: Cubic, Sine, Expo, Bounce, etc.
}

impl EasingFunction {
    pub fnapply(&self, t: f32) -> f32 { // t is normalized time (0.0 to 1.0)
        let t_clamped = t.clamp(0.0, 1.0);
        match self {
            EasingFunction::Linear => t_clamped,
            EasingFunction::EaseInQuad => t_clamped * t_clamped,
            EasingFunction::EaseOutQuad => 1.0 - (1.0 - t_clamped) * (1.0 - t_clamped),
            EasingFunction::EaseInOutQuad => {
                if t_clamped < 0.5 {
                    2.0 * t_clamped * t_clamped
                } else {
                    1.0 - (-2.0 * t_clamped + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

/// Represents a single property being animated (e.g., opacity, position X).
#[cfg(feature = "with_alloc")]
struct AnimatedProperty {
    target_widget_id: u64, // ID of the widget this property belongs to
    property_id: u32,      // Enum or ID specifying which property (e.g., OPACITY, POS_X)
    start_value: f32,
    end_value: f32,
    current_value: f32, // For observers or direct application

    duration_ms: u64,
    elapsed_ms: u64,
    delay_ms: u64, // Delay before animation starts
    
    easing: EasingFunction,
    is_active: bool,
    // on_complete: Option<Box<dyn FnOnce(u64)>>, // Callback on completion
}

#[cfg(feature = "with_alloc")]
impl AnimatedProperty {
    fn update(&mut self, delta_time_ms: u64) -> bool { // Returns true if still active
        if !self.is_active { return false; }

        if self.delay_ms > 0 {
            if delta_time_ms >= self.delay_ms {
                self.delay_ms = 0;
                // Any overflow from delta_time_ms should ideally be used for elapsed_ms
            } else {
                self.delay_ms -= delta_time_ms;
                return true; // Still delayed
            }
        }
        
        self.elapsed_ms += delta_time_ms;
        if self.elapsed_ms >= self.duration_ms {
            self.current_value = self.end_value;
            self.is_active = false;
            // if let Some(cb) = self.on_complete.take() { cb(self.target_widget_id); }
            return false; // Animation finished
        }

        let t_normalized = self.elapsed_ms as f32 / self.duration_ms as f32;
        let eased_t = self.easing.apply(t_normalized);
        self.current_value = self.start_value + (self.end_value - self.start_value) * eased_t;
        
        true // Still active
    }
}


// --- Animation Manager ---
#[cfg(feature = "with_alloc")]
static mut ACTIVE_ANIMATIONS: Option<Vec<AnimatedProperty>> = None;
// In a more complex system, animations might be grouped or associated with specific UI elements.

/// Initializes the animation system.
#[cfg(feature = "with_alloc")]
pub fn init_animation_system() {
    logger::info!("[Animations] Initializing animation system...");
    unsafe {
        ACTIVE_ANIMATIONS = Some(Vec::new());
    }
}

/// Adds a new animation to the system.
#[cfg(feature = "with_alloc")]
pub fn start_animation(
    target_widget_id: u64,
    property_id: u32,
    start_value: f32,
    end_value: f32,
    duration_ms: u64,
    delay_ms: u64,
    easing: EasingFunction,
    // on_complete: Option<Box<dyn FnOnce(u64)>>,
) {
    logger::debug!(
        "[Animations] Starting animation for widget {}, prop {}, {:.2}->{:.2} in {}ms (delay {}ms)",
        target_widget_id, property_id, start_value, end_value, duration_ms, delay_ms
    );
    let anim = AnimatedProperty {
        target_widget_id,
        property_id,
        start_value,
        end_value,
        current_value: start_value,
        duration_ms,
        elapsed_ms: 0,
        delay_ms,
        easing,
        is_active: true,
        // on_complete,
    };

    unsafe {
        if let Some(animations) = ACTIVE_ANIMATIONS.as_mut() {
            // TODO: Check if an animation for this widget_id/property_id already exists.
            // If so, replace it or queue this one? For now, just add.
            animations.push(anim);
        } else {
            logger::warn!("[Animations] start_animation called before init_animation_system.");
        }
    }
}

/// Updates all active animations based on the elapsed time.
/// Called by the main UI loop (`gui::ui::run_loop`) each frame.
///
/// Returns `true` if any animation was updated and the UI might need a redraw.
#[cfg(feature = "with_alloc")]
pub fn update_animations(delta_time_ms: u64) -> bool {
    let mut needs_redraw = false;
    unsafe {
        if let Some(animations) = ACTIVE_ANIMATIONS.as_mut() {
            if animations.is_empty() { return false; }

            // Iterate and update. It's important to handle removal of completed animations.
            // A common way is to retain only active ones.
            animations.retain_mut(|anim| { // `retain_mut` is nightly, or use manual loop with swap_remove
                let was_active = anim.is_active;
                let still_active = anim.update(delta_time_ms);
                if was_active { // Only consider it "updated" if it was active before this tick
                    needs_redraw = true;
                }
                still_active
            });

            // if needs_redraw && !animations.is_empty() {
            //     logger::trace!("[Animations] {} animations active.", animations.len());
            // }

        }
    }
    needs_redraw
}

/// Gets the current animated value for a specific widget's property.
/// Widgets would call this during their draw phase to get animated values.
#[cfg(feature = "with_alloc")]
pub fn get_animated_value(target_widget_id: u64, property_id: u32, default_value: f32) -> f32 {
    unsafe {
        if let Some(animations) = ACTIVE_ANIMATIONS.as_ref() {
            for anim in animations {
                if anim.target_widget_id == target_widget_id && anim.property_id == property_id && anim.is_active {
                    return anim.current_value;
                }
            }
        }
    }
    default_value // Return default if no active animation for this property
}


// --- Property IDs (Example) ---
// These would be defined more globally, perhaps per widget type.
pub const PROP_OPACITY: u32 = 1;
pub const PROP_POSITION_X: u32 = 2;
pub const PROP_POSITION_Y: u32 = 3;
pub const PROP_SCALE: u32 = 4;
// ... and so on.


// --- No Alloc Stubs ---
#[cfg(not(feature = "with_alloc"))]
pub fn init_animation_system() {
    logger::info!("[Animations] Animation system (no_alloc - STUBBED, no-op).");
}

#[cfg(not(feature = "with_alloc"))]
pub fn update_animations(_delta_time_ms: u64) -> bool {
    // No animations in no_alloc version for now
    false
}

#[cfg(not(feature = "with_alloc"))]
pub fn get_animated_value(_target_widget_id: u64, _property_id: u32, default_value: f32) -> f32 {
    default_value
}

#[cfg(not(feature = "with_alloc"))]
pub fn start_animation(
    _target_widget_id: u64, _property_id: u32, _start_value: f32, _end_value: f32,
    _duration_ms: u64, _delay_ms: u64, _easing: EasingFunction,
) {
    // No-op
}