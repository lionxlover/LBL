// Lionbootloader GUI - ProgressBar Widget
// File: gui/src/widgets/progress_bar.rs

#[cfg(feature = "with_alloc")]
use alloc::string::String; // For optional text display

use crate::widgets::{Rect, WidgetId};
// InputEvent not usually handled directly by a progress bar, it's mostly display-only
// use crate::input::InputEvent;
use crate::renderer;
use crate::theme::AppliedTheme;
use crate::animations::{self, PROP_OPACITY}; // For visibility animations
use lionbootloader_core_lib::logger;


#[cfg(feature = "with_alloc")]
pub struct ProgressBar {
    pub id: WidgetId,
    pub bounds: Rect,
    value: f32, // Progress value from 0.0 to 1.0
    visible: bool,
    // Optional text to display on or near the bar
    // text: Option<String>, 
    
    // Style properties (could also come entirely from theme)
    track_color: Option<u32>,
    bar_color: Option<u32>,
    // text_color: Option<u32>,

    current_opacity: f32,
}

#[cfg(feature = "with_alloc")]
impl ProgressBar {
    pub fn new(id: WidgetId, bounds: Rect) -> Self {
        ProgressBar {
            id,
            bounds,
            value: 0.0, // Start empty
            visible: true,
            // text: None,
            track_color: None, // Will use theme defaults if None
            bar_color: None,
            // text_color: None,
            current_opacity: 1.0,
        }
    }

    /// Sets the progress value (0.0 to 1.0).
    pub fn set_value(&mut self, new_value: f32) {
        self.value = new_value.clamp(0.0, 1.0);
        // Could trigger an animation for the bar filling up if desired,
        // rather than direct value setting for smoother visual change.
        // For now, direct update.
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    // pub fn set_text(&mut self, text: Option<String>) {
    //     self.text = text;
    // }

    pub fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            // Trigger fade in/out animation
            let target_opacity = if visible { 1.0 } else { 0.0 };
            animations::start_animation(
                self.id,
                PROP_OPACITY,
                self.current_opacity,
                target_opacity,
                300, // 300ms fade
                0,
                animations::EasingFunction::EaseInOutQuad,
            );
        }
    }
    
    pub fn is_visible(&self) -> bool {
        // Consider opacity for effective visibility
        self.visible && self.current_opacity > 0.01
    }


    /// Updates the progress bar's state (e.g., for animations).
    pub fn update(&mut self, _delta_time_ms: u64) {
        if !self.visible && self.current_opacity == 0.0 { return; } // Fully faded out and hidden

        self.current_opacity = animations::get_animated_value(self.id, PROP_OPACITY, if self.visible {1.0} else {0.0});

        // If the bar fill itself was animated, update that here.
    }

    /// Draws the progress bar.
    pub fn draw(&self, theme: &AppliedTheme) {
        if !self.is_visible() { // Checks effective visibility including opacity
            return;
        }

        // Determine colors
        let track_c = self.track_color.unwrap_or_else(|| {
            // Default track color from theme or hardcode
            // e.g., a slightly darker version of panel_bg or a specific progress_track_color
            let panel_alpha = (theme.panel_bg_color >> 24) & 0xFF;
            let base_panel_rgb = theme.panel_bg_color & 0x00FFFFFF;
            if panel_alpha > 0x40 { // If panel is not too transparent, make track slightly darker
                 apply_brightness(base_panel_rgb, 0.8) | (panel_alpha << 24)
            } else {
                0xFF_444444 // Fallback dark grey track
            }
        });

        let bar_c = self.bar_color.unwrap_or(theme.accent_color);
        // let text_c = self.text_color.unwrap_or(theme.text_color);

        // Apply opacity animation
        let actual_track_color = apply_opacity(track_c, self.current_opacity);
        let actual_bar_color = apply_opacity(bar_c, self.current_opacity);
        // let actual_text_color = apply_opacity(text_c, self.current_opacity);

        // Draw the track (background of the progress bar)
        // For "modern" style from spec, a rounded track would be nice.
        // renderer::draw_rounded_rect(self.bounds, track_corner_radius, actual_track_color);
        renderer::draw_rect(self.bounds.x, self.bounds.y, self.bounds.width, self.bounds.height, actual_track_color);


        // Calculate width of the filled portion
        let filled_width = (self.bounds.width as f32 * self.value) as i32;
        if filled_width > 0 {
            // Draw the filled portion (the actual progress bar)
            // Also ideally rounded if the track is.
            // renderer::draw_rounded_rect(Rect { x: self.bounds.x, y: self.bounds.y, width: filled_width, height: self.bounds.height }, bar_corner_radius, actual_bar_color);
            renderer::draw_rect(self.bounds.x, self.bounds.y, filled_width, self.bounds.height, actual_bar_color);
        }

        // Optionally, draw text (e.g., percentage or custom status)
        // if let Some(txt) = &self.text {
        //     // Text drawing with centering and styling
        //     let text_x = self.bounds.x + (self.bounds.width - (txt.len() * 8) as i32) / 2; // Approx centering
        //     let text_y = self.bounds.y + (self.bounds.height - 16) / 2 + 12; // Approx centering
        //     renderer::draw_text(txt, text_x, text_y, actual_text_color);
        // } else { // Default: draw percentage
             let percentage_text = format!("{}%", (self.value * 100.0) as u8);
             let text_width_approx = percentage_text.len() as i32 * 8;
             let text_height_approx = 16;
             let text_x = self.bounds.x + (self.bounds.width - text_width_approx) / 2;
             let text_y = self.bounds.y + (self.bounds.height - text_height_approx) / 2 + text_height_approx - 4; // Approx baseline
             let text_color_for_percent = if self.value > 0.55 { theme.selected_text_color } else { theme.text_color }; // Contrast
             renderer::draw_text(&percentage_text, text_x, text_y, apply_opacity(text_color_for_percent, self.current_opacity));
        // }
    }
}

// Helper for opacity (could be in a shared util module)
fn apply_opacity(color: u32, opacity: f32) -> u32 {
    let alpha = ((color >> 24) & 0xFF) as f32;
    let new_alpha = (alpha * opacity.clamp(0.0, 1.0)) as u32;
    (color & 0x00FFFFFF) | (new_alpha << 24)
}

// Helper for adjusting brightness (very basic)
fn apply_brightness(rgb: u32, factor: f32) -> u32 {
    let r = ((rgb >> 16) & 0xFF) as f32;
    let g = ((rgb >> 8) & 0xFF) as f32;
    let b = (rgb & 0xFF) as f32;

    let new_r = (r * factor).clamp(0.0, 255.0) as u32;
    let new_g = (g * factor).clamp(0.0, 255.0) as u32;
    let new_b = (b * factor).clamp(0.0, 255.0) as u32;

    (new_r << 16) | (new_g << 8) | new_b
}



// --- No Alloc Stubs ---
#[cfg(not(feature = "with_alloc"))]
pub struct ProgressBar { /* Simpler state, no String text */ }
#[cfg(not(feature = "with_alloc"))]
impl ProgressBar { /* Simplified or TBD methods */ }