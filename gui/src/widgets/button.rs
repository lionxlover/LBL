
// Lionbootloader GUI - Button Widget
// File: gui/src/widgets/button.rs

#[cfg(feature = "with_alloc")]
use alloc::string::String;

use crate::widgets::{Rect, WidgetId};
use crate::input::{InputEvent, KeyCode, MouseButton};
use crate::renderer; // For drawing
use crate::theme::AppliedTheme; // For styling
use crate::animations::{self, EasingFunction, PROP_OPACITY, PROP_SCALE}; // For hover/press effects
use lionbootloader_core_lib::logger;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

#[cfg(feature = "with_alloc")]
pub struct Button {
    pub id: WidgetId,
    pub text: String,
    pub bounds: Rect,
    pub state: ButtonState,
    pub visible: bool,
    pub enabled: bool,
    
    // For animations
    current_scale: f32,
    current_opacity: f32,
}

#[cfg(feature = "with_alloc")]
impl Button {
    pub fn new(id: WidgetId, text: String, bounds: Rect) -> Self {
        Button {
            id,
            text,
            bounds,
            state: ButtonState::Normal,
            visible: true,
            enabled: true,
            current_scale: 1.0,
            current_opacity: 1.0,
        }
    }

    /// Handles an input event. Returns `true` if the event was consumed by this button.
    /// This might trigger an action (e.g. click) which should be reported back.
    /// For now, returns true if clicked/pressed.
    pub fn handle_input(&mut self, event: &InputEvent, mouse_pos: (i32, i32)) -> Option<ButtonAction> {
        if !self.visible || !self.enabled {
            return None;
        }

        let mut action_taken = None;
        let previously_hovered = self.state == ButtonState::Hovered || self.state == ButtonState::Pressed;
        let currently_hovered = self.bounds.contains_point(mouse_pos.0, mouse_pos.1);

        match event {
            InputEvent::MouseMove { x, y } => {
                if self.bounds.contains_point(*x, *y) {
                    if self.state == ButtonState::Normal {
                        self.set_state(ButtonState::Hovered);
                    }
                } else {
                    if self.state == ButtonState::Hovered || self.state == ButtonState::Pressed {
                        self.set_state(ButtonState::Normal);
                    }
                }
            }
            InputEvent::MousePress { button, x, y, .. } => {
                if *button == MouseButton::Left && self.bounds.contains_point(*x, *y) {
                    self.set_state(ButtonState::Pressed);
                    // Action is usually triggered on release, but can also be on press.
                    // action_taken = Some(ButtonAction::Pressed(self.id));
                }
            }
            InputEvent::MouseRelease { button, x, y, .. } => {
                if *button == MouseButton::Left {
                    if self.state == ButtonState::Pressed { // Was pressed on this button
                        if self.bounds.contains_point(*x, *y) { // Released over the button
                            logger::debug!("[Button] Button {} clicked!", self.id);
                            action_taken = Some(ButtonAction::Clicked(self.id));
                            self.set_state(ButtonState::Hovered); // Return to hovered if mouse is still over
                        } else {
                            self.set_state(ButtonState::Normal); // Released outside
                        }
                    }
                }
            }
            // Keyboard interaction (e.g., if button is focused and Enter/Space is pressed)
            InputEvent::KeyPress { key_code, .. } => {
                if self.state == ButtonState::Hovered || self.state == ButtonState::Focused { // Assuming Focused state exists
                    if *key_code == KeyCode::Enter || *key_code == KeyCode::Space {
                        self.set_state(ButtonState::Pressed);
                        // Simulate quick press-release for keyboard
                        logger::debug!("[Button] Button {} activated by keyboard!", self.id);
                        action_taken = Some(ButtonAction::Clicked(self.id));
                        // Could add a short "pressed" animation for keyboard
                        self.set_state(ButtonState::Hovered); // Or Normal if not focused by mouse
                    }
                }
            }
            _ => {}
        }
        
        // Update hover state independent of specific event if mouse moved out without MouseMove event here
        // This logic is better handled in a general pre-event update or if MouseMove is guaranteed
        if !currently_hovered && previously_hovered && self.state != ButtonState::Pressed {
             self.set_state(ButtonState::Normal);
        }


        action_taken
    }

    /// Updates the button's internal state (e.g., for animations).
    pub fn update(&mut self, _delta_time_ms: u64) {
        if !self.visible { return; }
        // Update animated properties
        self.current_scale = animations::get_animated_value(self.id, PROP_SCALE, 1.0);
        self.current_opacity = animations::get_animated_value(self.id, PROP_OPACITY, 1.0);
    }

    /// Draws the button.
    pub fn draw(&self, theme: &AppliedTheme) {
        if !self.visible {
            return;
        }

        // --- Determine Colors based on State & Theme ---
        let mut bg_color = theme.panel_bg_color; // Or a specific button bg color from theme
        let mut text_color = theme.text_color;
        let mut border_color = theme.accent_color; // Example: use accent for border

        if !self.enabled || self.state == ButtonState::Disabled {
            // bg_color = theme.disabled_bg_color();
            // text_color = theme.disabled_text_color();
            // border_color = theme.disabled_border_color();
            bg_color = (bg_color & 0x00FFFFFF) | (0x80 << 24); // More transparent
            text_color = (text_color & 0x00FFFFFF) | (0x80 << 24);
        } else {
            match self.state {
                ButtonState::Normal => { /* Use default theme colors */ }
                ButtonState::Hovered => {
                    bg_color = theme.hover_bg_color;
                }
                ButtonState::Pressed => {
                    bg_color = theme.accent_color; // Use accent for pressed background
                    text_color = theme.selected_text_color; // Text color that contrasts with accent
                }
                _ => {} // Disabled handled above
            }
        }
        
        // Apply opacity animation
        let actual_bg_color = apply_opacity(bg_color, self.current_opacity);
        let actual_text_color = apply_opacity(text_color, self.current_opacity);
        let actual_border_color = apply_opacity(border_color, self.current_opacity);


        // --- Apply Scale Animation (conceptual: adjust bounds for drawing) ---
        let scaled_width = (self.bounds.width as f32 * self.current_scale) as i32;
        let scaled_height = (self.bounds.height as f32 * self.current_scale) as i32;
        let scaled_x = self.bounds.x + (self.bounds.width - scaled_width) / 2;
        let scaled_y = self.bounds.y + (self.bounds.height - scaled_height) / 2;
        let draw_bounds = Rect::new(scaled_x, scaled_y, scaled_width, scaled_height);


        // --- Draw ---
        // renderer::draw_rounded_rect(draw_bounds, corner_radius, actual_bg_color); // Ideal
        renderer::draw_rect(draw_bounds.x, draw_bounds.y, draw_bounds.width, draw_bounds.height, actual_bg_color); // Simpler rect for now

        // Draw border (optional)
        // renderer::draw_rounded_rect_outline(draw_bounds, corner_radius, 1.0, actual_border_color);

        // Draw text (centered)
        if let Some(font) = theme.primary_font.as_ref() {
            // Text centering logic:
            // 1. Get text dimensions using the font: `font.measure_text(&self.text, font_size)`
            //    This is a missing piece; `fontdue` can give glyph metrics.
            //    For a simple stub, approximate.
            let text_width_approx = self.text.len() as i32 * 8; // Assuming 8px per char avg width
            let text_height_approx = 16; // Assuming 16px char height
            
            let text_x = draw_bounds.x + (draw_bounds.width - text_width_approx) / 2;
            let text_y = draw_bounds.y + (draw_bounds.height - text_height_approx) / 2 + text_height_approx - 4; // Approx baseline

            renderer::draw_text(&self.text, text_x, text_y, actual_text_color /*, font_handle_from_theme */);
        } else {
            // Fallback if no font (e.g. draw simple placeholder)
            renderer::draw_text(&self.text, draw_bounds.x + 5, draw_bounds.y + draw_bounds.height / 2, actual_text_color);
        }
    }
    
    fn set_state(&mut self, new_state: ButtonState) {
        if self.state == new_state { return; }
        
        let old_state = self.state;
        self.state = new_state;

        // Trigger animations based on state changes
        // Example: Scale animation on hover/press
        match (old_state, new_state) {
            (_, ButtonState::Hovered) => {
                animations::start_animation(self.id, PROP_SCALE, self.current_scale, 1.05, 150, 0, EasingFunction::EaseOutQuad);
            }
            (ButtonState::Hovered, ButtonState::Normal) => {
                animations::start_animation(self.id, PROP_SCALE, self.current_scale, 1.0, 200, 0, EasingFunction::EaseOutQuad);
            }
            (_, ButtonState::Pressed) => {
                animations::start_animation(self.id, PROP_SCALE, self.current_scale, 0.95, 100, 0, EasingFunction::EaseOutQuad);
            }
            (ButtonState::Pressed, ButtonState::Hovered) | (ButtonState::Pressed, ButtonState::Normal) => {
                // Snap back or animate back from pressed state
                animations::start_animation(self.id, PROP_SCALE, self.current_scale, if new_state == ButtonState::Hovered {1.05} else {1.0}, 150, 0, EasingFunction::EaseOutQuad);
            }
            _ => {}
        }
    }
}

/// Represents an action performed on a button.
#[cfg(feature = "with_alloc")]
#[derive(Debug, Clone, Copy)]
pub enum ButtonAction {
    Clicked(WidgetId),
    // Pressed(WidgetId), // If distinguishing press from click is needed
}

fn apply_opacity(color: u32, opacity: f32) -> u32 {
    let alpha = ((color >> 24) & 0xFF) as f32;
    let new_alpha = (alpha * opacity.clamp(0.0, 1.0)) as u32;
    (color & 0x00FFFFFF) | (new_alpha << 24)
}


// Minimal stubs for no_alloc feature
#[cfg(not(feature = "with_alloc"))]
pub struct Button {
     pub id: WidgetId,
     // text: [char; 32], // Fixed size for no_alloc
     pub bounds: Rect,
     pub state: ButtonState,
     pub visible: bool,
     pub enabled: bool,
}
#[cfg(not(feature = "with_alloc"))]
impl Button {
    // pub fn new(...) -> Self { ... }
    // pub fn handle_input(...) -> Option<ButtonAction> { None }
    // pub fn update(...) {}
    // pub fn draw(...) {}
}
#[cfg(not(feature = "with_alloc"))]
#[derive(Debug, Clone, Copy)]
pub enum ButtonAction { Clicked(WidgetId) }