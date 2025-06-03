// Lionbootloader GUI - ListView Widget
// File: gui/src/widgets/list_view.rs

#[cfg(feature = "with_alloc")]
use alloc::{string::String, vec::Vec};

use crate::widgets::{Rect, WidgetId};
use crate::input::{InputEvent, KeyCode, MouseButton};
use crate::renderer;
use crate::theme::AppliedTheme;
use crate::animations::{self, PROP_OPACITY}; // For fade-in of items perhaps
use lionbootloader_core_lib::logger;

/// Represents a single item in the ListView.
#[cfg(feature = "with_alloc")]
#[derive(Clone, Debug)]
pub struct ListViewItem {
    pub id: String, // Unique ID for the item (e.g., boot entry ID)
    pub text: String, // Text to display for the item
    // pub icon: Option<IconId>, // Optional icon
    // pub sub_text: Option<String>, // Optional secondary text line
    // pub data: UserDataType, // Generic user data associated with item
}

#[cfg(feature = "with_alloc")]
pub struct ListView {
    pub id: WidgetId,
    pub bounds: Rect,
    items: Vec<ListViewItem>,
    pub selected_index: Option<usize>,
    scroll_offset_y: f32, // For smooth scrolling, pixel offset
    item_height: i32,     // Height of a single item in pixels
    visible: bool,
    enabled: bool,

    // State for mouse interaction
    hovered_index: Option<usize>,
    // is_mouse_dragging_scroll: bool,
    // drag_start_y: i32,
    // drag_start_scroll_offset: f32,
}

#[cfg(feature = "with_alloc")]
impl ListView {
    pub fn new(id: WidgetId, bounds: Rect, item_height: i32) -> Self {
        ListView {
            id,
            bounds,
            items: Vec::new(),
            selected_index: None,
            scroll_offset_y: 0.0,
            item_height,
            visible: true,
            enabled: true,
            hovered_index: None,
            // is_mouse_dragging_scroll: false,
        }
    }

    pub fn set_items(&mut self, new_items: Vec<ListViewItem>) {
        self.items = new_items;
        self.selected_index = if self.items.is_empty() { None } else { Some(0) };
        self.scroll_offset_y = 0.0; // Reset scroll on new items
        // TODO: Trigger animations for items appearing if desired
    }

    pub fn select_item_by_id(&mut self, item_id: &str) -> bool {
        if let Some(idx) = self.items.iter().position(|item| item.id == item_id) {
            self.selected_index = Some(idx);
            self.ensure_selected_item_visible();
            true
        } else {
            false
        }
    }
    
    pub fn get_selected_item_id(&self) -> Option<String> {
        self.selected_index.and_then(|idx| self.items.get(idx).map(|item| item.id.clone()))
    }


    /// Handles input events. Returns an action if an item is activated.
    pub fn handle_input(&mut self, event: &InputEvent, mouse_pos: (i32, i32)) -> Option<ListViewAction> {
        if !self.visible || !self.enabled || self.items.is_empty() {
            self.hovered_index = None;
            return None;
        }

        let mut action = None;
        let prev_hovered_index = self.hovered_index;
        self.hovered_index = None; // Reset hover unless mouse is over an item

        match event {
            InputEvent::KeyPress { key_code, .. } => {
                match key_code {
                    KeyCode::UpArrow => {
                        if let Some(mut idx) = self.selected_index {
                            if idx > 0 {
                                idx -= 1;
                                self.selected_index = Some(idx);
                                self.ensure_selected_item_visible();
                            }
                        } else if !self.items.is_empty() {
                            self.selected_index = Some(self.items.len() - 1); // Wrap to bottom
                            self.ensure_selected_item_visible();
                        }
                        action = Some(ListViewAction::SelectionChanged(self.id, self.get_selected_item_id()));
                    }
                    KeyCode::DownArrow => {
                        if let Some(mut idx) = self.selected_index {
                            if idx < self.items.len() - 1 {
                                idx += 1;
                                self.selected_index = Some(idx);
                                self.ensure_selected_item_visible();
                            }
                        } else if !self.items.is_empty() {
                            self.selected_index = Some(0); // Wrap to top
                            self.ensure_selected_item_visible();
                        }
                        action = Some(ListViewAction::SelectionChanged(self.id, self.get_selected_item_id()));
                    }
                    KeyCode::Enter => {
                        if let Some(idx) = self.selected_index {
                            if let Some(item) = self.items.get(idx) {
                                logger::debug!("[ListView] Item '{}' activated by Enter.", item.text);
                                action = Some(ListViewAction::ItemActivated(self.id, item.id.clone()));
                            }
                        }
                    }
                    KeyCode::PageUp => {
                        self.scroll_relative(-self.bounds.height);
                        // Optionally, also move selection
                    }
                    KeyCode::PageDown => {
                        self.scroll_relative(self.bounds.height);
                    }
                    _ => {}
                }
            }
            InputEvent::MouseMove { x, y } => {
                if self.bounds.contains_point(*x, *y) {
                    let relative_y = *y - self.bounds.y + self.scroll_offset_y as i32;
                    let current_hover_idx = (relative_y / self.item_height) as usize;
                    if current_hover_idx < self.items.len() {
                        self.hovered_index = Some(current_hover_idx);
                    }
                }
            }
            InputEvent::MouseClick { button, x, y, .. } => {
                if *button == MouseButton::Left && self.bounds.contains_point(*x, *y) {
                    let relative_y = *y - self.bounds.y + self.scroll_offset_y as i32;
                    let clicked_idx = (relative_y / self.item_height) as usize;
                    if clicked_idx < self.items.len() {
                        self.selected_index = Some(clicked_idx);
                        self.ensure_selected_item_visible();
                        if let Some(item) = self.items.get(clicked_idx) {
                             logger::debug!("[ListView] Item '{}' activated by Mouse Click.", item.text);
                            action = Some(ListViewAction::ItemActivated(self.id, item.id.clone()));
                        }
                    }
                }
            }
            InputEvent::MouseScroll { delta_y, .. } => {
                if self.bounds.contains_point(mouse_pos.0, mouse_pos.1) { // Scroll if mouse is over list
                    // delta_y is often in lines or fractional lines. Convert to pixels.
                    // Positive delta_y for scroll down, negative for scroll up.
                    let scroll_amount_pixels = -(*delta_y * self.item_height as f32 * 1.5) as i32; // Adjust multiplier
                    self.scroll_relative(scroll_amount_pixels);
                }
            }
            _ => {}
        }
        
        if prev_hovered_index != self.hovered_index {
            // Could trigger redraw or specific item hover animations if needed (not done here)
        }
        
        action
    }
    
    fn scroll_relative(&mut self, delta_pixels: i32) {
        let total_content_height = (self.items.len() * self.item_height as usize) as f32;
        let max_scroll_offset = (total_content_height - self.bounds.height as f32).max(0.0);

        self.scroll_offset_y += delta_pixels as f32;
        self.scroll_offset_y = self.scroll_offset_y.clamp(0.0, max_scroll_offset);
    }

    fn ensure_selected_item_visible(&mut self) {
        if let Some(idx) = self.selected_index {
            let item_top_y = (idx * self.item_height as usize) as f32;
            let item_bottom_y = item_top_y + self.item_height as f32;

            let view_top_y = self.scroll_offset_y;
            let view_bottom_y = self.scroll_offset_y + self.bounds.height as f32;

            if item_top_y < view_top_y { // Item is above visible area
                self.scroll_offset_y = item_top_y;
            } else if item_bottom_y > view_bottom_y { // Item is below visible area
                self.scroll_offset_y = item_bottom_y - self.bounds.height as f32;
            }
            // Clamp scroll_offset_y again after adjustment
            let total_content_height = (self.items.len() * self.item_height as usize) as f32;
            let max_scroll_offset = (total_content_height - self.bounds.height as f32).max(0.0);
            self.scroll_offset_y = self.scroll_offset_y.clamp(0.0, max_scroll_offset);
        }
    }


    pub fn update(&mut self, _delta_time_ms: u64) {
        if !self.visible { return; }
        // Update animations for items or scroll effects if any
    }

    pub fn draw(&self, theme: &AppliedTheme) {
        if !self.visible {
            return;
        }

        // Draw background for the list view (optional, could be transparent)
        // renderer::draw_rect(self.bounds.x, self.bounds.y, self.bounds.width, self.bounds.height, theme.panel_bg_color());
        // For "glassmorphism", the panel_bg_color should have alpha.

        // Determine which items are visible
        let first_visible_index = (self.scroll_offset_y / self.item_height as f32).floor() as usize;
        let items_that_fit_in_view = (self.bounds.height as f32 / self.item_height as f32).ceil() as usize;
        let last_visible_index = (first_visible_index + items_that_fit_in_view).min(self.items.len());

        for i in first_visible_index..last_visible_index {
            if let Some(item) = self.items.get(i) {
                let item_y_abs = self.bounds.y + (i * self.item_height as usize) as i32 - self.scroll_offset_y as i32;

                // Clipping: only draw if item is at least partially within bounds.y
                if item_y_abs + self.item_height < self.bounds.y || item_y_abs > self.bounds.y + self.bounds.height {
                    continue;
                }
                
                let item_bounds = Rect {
                    x: self.bounds.x,
                    y: item_y_abs,
                    width: self.bounds.width,
                    height: self.item_height,
                };

                // Determine item background and text color based on state
                let mut item_bg_color = 0x00000000; // Transparent by default
                let mut item_text_color = theme.text_color;

                if Some(i) == self.selected_index {
                    item_bg_color = theme.accent_color;
                    item_text_color = theme.selected_text_color;
                } else if Some(i) == self.hovered_index {
                    item_bg_color = theme.hover_bg_color;
                }
                
                // Apply opacity (e.g., for fade-in of list itself)
                let list_opacity = animations::get_animated_value(self.id, PROP_OPACITY, 1.0);
                item_bg_color = apply_opacity(item_bg_color, list_opacity);
                item_text_color = apply_opacity(item_text_color, list_opacity);


                if (item_bg_color >> 24) & 0xFF > 0 { // Draw background only if not fully transparent
                    renderer::draw_rect(item_bounds.x, item_bounds.y, item_bounds.width, item_bounds.height, item_bg_color);
                }
                
                // Draw item text (simple left-aligned, vertically centered anicon_spacer_widthd padding)
                let text_padding_x = 10;
                let text_padding_y = (self.item_height - 16) / 2; // Assuming ~16px text height
                renderer::draw_text(
                    &item.text,
                    item_bounds.x + text_padding_x, // + icon_spacer_width if icons are present
                    item_bounds.y + text_padding_y + 12, // Approx baseline adjust for 16px font
                    item_text_color,
                );
                
                // TODO: Draw icons, subtext, health indicators etc.
                // For macOS style, items usually have icons.
                // let icon_name = item.icon.as_ref().map_or("default_icon", |id| &id.name_for_theme);
                // renderer::draw_icon(icon_name, item_bounds.x + icon_padding_x, item_bounds.y + ...);

            }
        }

        // TODO: Draw scrollbar if content height > view height
        // let total_content_height = (self.items.len() * self.item_height as usize) as f32;
        // if total_content_height > self.bounds.height as f32 {
        //    draw_scrollbar(self.bounds, self.scroll_offset_y, total_content_height, theme);
        // }
    }
}

#[cfg(feature = "with_alloc")]
#[derive(Debug, Clone)]
pub enum ListViewAction {
    ItemActivated(WidgetId, String),        // list_id, item_id
    SelectionChanged(WidgetId, Option<String>), // list_id, new_selected_item_id
}

// Helper from button.rs, could be moved to a common utils module
fn apply_opacity(color: u32, opacity: f32) -> u32 {
    let alpha = ((color >> 24) & 0xFF) as f32;
    let new_alpha = (alpha * opacity.clamp(0.0, 1.0)) as u32;
    (color & 0x00FFFFFF) | (new_alpha << 24)
}

// --- No Alloc Stubs ---
#[cfg(not(feature = "with_alloc"))]
pub struct ListViewItem { /* Potentially fixed-size fields */ }
#[cfg(not(feature = "with_alloc"))]
pub struct ListView { /* Simpler state, fixed-size item array */ }
#[cfg(not(feature = "with_alloc"))]
impl ListView { /* Simplified methods */ }
#[cfg(not(feature = "with_alloc"))]
pub enum ListViewAction { /* Simplified actions */ }