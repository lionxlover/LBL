// Lionbootloader GUI - Widgets Module
// File: gui/src/widgets/mod.rs

// --- Declare individual widget modules ---
pub mod button;
pub mod list_view;
pub mod progress_bar;
// pub mod label;
// pub mod text_input;
// pub mod checkbox;
// pub mod image_widget;
// pub mod panel; // For grouping or glassmorphism effect

// Re-export widget structs for easier access from ui.rs
// Example:
// pub use button::Button;
// pub use list_view::ListView;
// pub use progress_bar::ProgressBarWidget;


// --- Common Widget Traits or Data (Optional) ---

// #[cfg(feature = "with_alloc")]
// use alloc::string::String;
// use crate::input::InputEvent;
// use crate::renderer; // For drawing context/canvas
// use crate::theme::AppliedTheme;

/// A unique identifier for a widget.
pub type WidgetId = u64;

/// Basic geometric rectangle, commonly used for widget bounds.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rect { x, y, width, height }
    }

    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < (self.x + self.width) &&
        py >= self.y && py < (self.y + self.height)
    }
}

/// Common state for many widgets.
// pub struct WidgetState {
//     pub id: WidgetId,
//     pub bounds: Rect,
//     pub is_visible: bool,
//     pub is_enabled: bool,
//     pub is_hovered: bool,
//     pub is_focused: bool, // e.g., for keyboard navigation
//     // pub needs_redraw: bool,
// }

/// A generic trait that all UI widgets could implement.
/// This allows the UI manager to treat them polymorphically for updates and drawing.
// pub trait Widget {
//     fn id(&self) -> WidgetId;
//     fn get_bounds(&self) -> Rect;
//     fn set_bounds(&mut self, bounds: Rect);

//     // Handles an input event. Returns true if the event was consumed.
//     fn handle_input(&mut self, event: &InputEvent, theme: &AppliedTheme) -> bool;

//     // Updates widget state (e.g., for animations). Delta time in ms.
//     fn update(&mut self, delta_time_ms: u64, theme: &AppliedTheme);

//     // Draws the widget onto the provided rendering context/canvas.
//     fn draw(&self, renderer_ctx: &mut renderer::RenderingContext, theme: &AppliedTheme);
    
//     fn is_visible(&self) -> bool;
//     fn set_visible(&mut self, visible: bool);

//     // fn request_redraw(&mut self); // Method for widget to signal it needs to be redrawn
// }

// Note on Widget Trait:
// Using `Box<dyn Widget>` for a list of widgets in `ui.rs` would require `alloc`.
// Alternative approaches for `no_std` or to avoid dynamic dispatch:
// 1. Enums: An `enum UiElement { Button(ButtonWidget), List(ListWidget), ... }`
//    Then match on the enum type for `handle_input`, `update`, `draw`.
// 2. Generics and specific lists: `Vec<ButtonWidget>`, `Vec<ListWidget>`. The UI layout
//    engine would then know which list to iterate for which type. This is less flexible
//    for arbitrary widget compositions but often more performant and `no_std` friendly.
//
// Given the "macOS-style GUI" goal, a more dynamic approach (closer to trait objects or
// component-based systems) is often used, usually implying `alloc`.

// For now, individual widget modules will define their own structs and the `ui.rs`
// module will manage instances of these concrete types. A unifying trait might be
// introduced later if the hierarchy benefits from it.