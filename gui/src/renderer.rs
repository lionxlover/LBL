// Lionbootloader GUI - Renderer
// File: gui/src/renderer.rs

use lionbootloader_core_lib::hal::BootInfo; // To get framebuffer details
use lionbootloader_core_lib::logger;

// If using embedded-graphics:
// use embedded_graphics::prelude::*;
// use embedded_graphics::pixelcolor::{Rgb888, Bgr888, Rgb565}; // Or whatever the framebuffer format is
// use embedded_graphics::draw_target::DrawTarget;
// use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};

// If using NanoVG:
// use nanovg_rs as nanovg; // Alias for nanovg crate

// --- Renderer State ---
static mut FRAMEBUFFER_INFO: Option<FramebufferDetails> = None;
// For direct framebuffer access, the pointer needs to be stored.
// This needs to be managed carefully regarding safety.
// A spin::Mutex could protect access if needed, though rendering is often single-threaded context.

#[derive(Debug, Clone, Copy)]
struct FramebufferDetails {
    address: u64,
    width: u32,
    height: u32,
    pitch: u32,  // Bytes per line
    bpp: u8,     // Bits per pixel
    // format: PixelFormat, // e.g., RGB, BGR, order of color channels
}

// Enum for pixel formats LBL might support or detect
// #[derive(Debug, Clone, Copy)]
// enum PixelFormat {
//     Rgb888, Bgr888, Rgba8888, Bgra8888, Unknown
// }


#[derive(Debug)]
pub enum RendererError {
    NotInitialized,
    FramebufferUnavailable,
    InvalidParameters(String),
    UnsupportedPixelFormat(String),
    DrawingError(String),
    // NanoVGErr(String), // If using NanoVG
}


/// Initializes the rendering system.
///
/// `boot_info` from HAL provides the necessary framebuffer details.
pub fn init_renderer(boot_info: &BootInfo) -> Result<(), RendererError> {
    logger::info!("[Renderer] Initializing renderer...");

    if boot_info.framebuffer_addr == 0 || boot_info.width == 0 || boot_info.height == 0 || boot_info.pitch == 0 || boot_info.bpp == 0 {
        logger::error!("[Renderer] Framebuffer information from BootInfo is invalid or unavailable.");
        return Err(RendererError::FramebufferUnavailable);
    }

    let details = FramebufferDetails {
        address: boot_info.framebuffer_addr,
        width: boot_info.width,
        height: boot_info.height,
        pitch: boot_info.pitch,
        bpp: boot_info.bpp,
        // TODO: Determine PixelFormat based on bpp and potentially UEFI GOP mode info
        // format: determine_pixel_format(boot_info),
    };

    logger::info!(
        "[Renderer] Framebuffer: Addr={:#x}, {}x{}, Pitch={}, BPP={}",
        details.address, details.width, details.height, details.pitch, details.bpp
    );

    // Validate BPP supported by this simple renderer (e.g., 32-bit or 24-bit)
    if details.bpp != 32 && details.bpp != 24 {
        // Basic renderer might only support common BPPs.
        // A more advanced one (like one backing NanoVG) might have wider support.
        let err_msg = format!("Unsupported bits per pixel: {}. Only 24 or 32 bpp supported by basic renderer.", details.bpp);
        logger::error!("[Renderer] {}", err_msg);
        // return Err(RendererError::UnsupportedPixelFormat(err_msg)); // Be strict
    }


    unsafe {
        FRAMEBUFFER_INFO = Some(details);
    }

    // If using NanoVG or embedded-graphics, initialize them here,
    // providing them with a way to draw to our framebuffer.
    // Example for NanoVG:
    // let mut context_flags = nanovg::CreateFlags::empty();
    // if antialias { context_flags |= nanovg::CreateFlags::ANTIALIAS; }
    // unsafe { NANOVG_CONTEXT = Some(nanovg::Context::create_framebuffer(details.address as *mut _, details.width, details.height, details.pitch, nanovg::FramebufferPixelFormat::BGRA8, context_flags).map_err(|_| RendererError::NanoVGErr("NanoVG FB init failed".into()))?); }


    logger::info!("[Renderer] Renderer initialized for direct framebuffer access.");
    Ok(())
}

/// Called at the beginning of each frame's rendering.
pub fn begin_frame() -> Result<(), RendererError> {
    // If using a retained mode graphics library (like NanoVG), this is where you'd call its begin_frame.
    // nanovg_context.begin_frame(width, height, device_pixel_ratio);
    // For direct framebuffer, this might be a no-op or clear screen.
    Ok(())
}

/// Called at the end of each frame's rendering.
pub fn end_frame() -> Result<(), RendererError> {
    // If using NanoVG: nanovg_context.end_frame();
    // If double buffering (not common for bootloader direct FB), this is where you'd flip buffers.
    Ok(())
}

/// Clears the entire screen to a specified color.
/// Color is typically ARGB8888 (0xAARRGGBB) or RGB888.
pub fn clear_screen(color: u32) {
    let fb_info = unsafe { FRAMEBUFFER_INFO.as_ref() };
    if fb_info.is_none() {
        logger::warn!("[Renderer] clear_screen called but renderer not initialized.");
        return;
    }
    let details = fb_info.unwrap();

    // Direct framebuffer pixel manipulation
    let bytes_per_pixel = details.bpp as usize / 8;
    if bytes_per_pixel != 4 && bytes_per_pixel != 3 { // Assuming 32bpp or 24bpp
        logger::warn!("[Renderer] clear_screen: Unsupported bpp {} for direct clear.", details.bpp);
        return;
    }

    let fb_ptr = details.address as *mut u8;
    for y in 0..details.height {
        for x in 0..details.width {
            let offset = (y * details.pitch + x * bytes_per_pixel as u32) as usize;
            unsafe {
                // This assumes ARGB where A is most significant, or XRGB.
                // And that framebuffer expects BGR or RGB order. This needs to match PixelFormat.
                // Example for 32bpp BGRA (common in UEFI GOP):
                if bytes_per_pixel == 4 {
                    // Assuming color is 0xAARRGGBB
                    // Framebuffer might be BGRA, RGBA, ARGB, ABGR. This needs to be known.
                    // For BGRA:
                    fb_ptr.add(offset).write_volatile((color & 0xFF) as u8);         // B
                    fb_ptr.add(offset + 1).write_volatile(((color >> 8) & 0xFF) as u8); // G
                    fb_ptr.add(offset + 2).write_volatile(((color >> 16) & 0xFF) as u8); // R
                    fb_ptr.add(offset + 3).write_volatile(((color >> 24) & 0xFF) as u8); // A (or X)
                } else { // 24bpp BGR
                    fb_ptr.add(offset).write_volatile((color & 0xFF) as u8);         // B
                    fb_ptr.add(offset + 1).write_volatile(((color >> 8) & 0xFF) as u8); // G
                    fb_ptr.add(offset + 2).write_volatile(((color >> 16) & 0xFF) as u8); // R
                }
            }
        }
    }
}

/// Draws a filled rectangle. (Stub)
pub fn draw_rect(x: i32, y: i32, width: i32, height: i32, color: u32) {
    let fb_info = unsafe { FRAMEBUFFER_INFO.as_ref() };
    if fb_info.is_none() { return; }
    let details = fb_info.unwrap();

    // Clip rectangle to screen bounds
    let x0 = core::cmp::max(0, x) as u32;
    let y0 = core::cmp::max(0, y) as u32;
    let x1 = core::cmp::min(details.width, (x + width) as u32);
    let y1 = core::cmp::min(details.height, (y + height) as u32);

    if x1 <= x0 || y1 <= y0 { return; } // Clipped to empty or invalid

    let bytes_per_pixel = details.bpp as usize / 8;
    if bytes_per_pixel != 4 && bytes_per_pixel != 3 { return; }

    let fb_ptr = details.address as *mut u8;
    for r_y in y0..y1 {
        for r_x in x0..x1 {
            let offset = (r_y * details.pitch + r_x * bytes_per_pixel as u32) as usize;
            unsafe {
                if bytes_per_pixel == 4 { // BGRA example
                    fb_ptr.add(offset).write_volatile((color & 0xFF) as u8);
                    fb_ptr.add(offset + 1).write_volatile(((color >> 8) & 0xFF) as u8);
                    fb_ptr.add(offset + 2).write_volatile(((color >> 16) & 0xFF) as u8);
                    fb_ptr.add(offset + 3).write_volatile(((color >> 24) & 0xFF) as u8);
                } else { // BGR example
                    fb_ptr.add(offset).write_volatile((color & 0xFF) as u8);
                    fb_ptr.add(offset + 1).write_volatile(((color >> 8) & 0xFF) as u8);
                    fb_ptr.add(offset + 2).write_volatile(((color >> 16) & 0xFF) as u8);
                }
            }
        }
    }
}

/// Draws text using a pre-loaded font. (Major Stub)
/// This requires a font rasterizer (like `fontdue` integrated in `theme.rs` or here)
/// and a glyph cache.
pub fn draw_text(text: &str, x: i32, y: i32, color: u32 /*, font: &FontHandle */) {
    let fb_info = unsafe { FRAMEBUFFER_INFO.as_ref() };
    if fb_info.is_none() { return; }
    // let _details = fb_info.unwrap();

    // TODO: Implement text drawing:
    // 1. Get the active font (from theme module or a default).
    // 2. For each character in `text`:
    //    a. Get glyph bitmap from font rasterizer/cache.
    //    b. Blend/draw the glyph bitmap onto the framebuffer at (x, y + baseline_offset), advancing x.
    //       - Color modulation: apply `color` to the glyph pixels.
    //       - Alpha blending if font has anti-aliasing and framebuffer supports alpha.
    // This is a complex task. The `fontdue` crate can help with rasterization.
    
    // Super simple stub: "draws" by logging.
    logger::trace!("[Renderer] draw_text (STUB): '{}' at ({}, {}) with color {:#x}", text, x, y, color);

    // If we had a very basic bitmap font (e.g. 8x16 fixed width):
    // for (char_idx, char_code) in text.chars().enumerate() {
    //    draw_bitmap_char(char_code, x + (char_idx * 8) as i32, y, color);
    // }
}


/// Draws an image/icon. (Stub)
pub fn draw_image(/* image_data: &ImageData, */ x: i32, y: i32 /*, options... */) {
    logger::trace!("[Renderer] draw_image (STUB) at ({}, {})", x, y);
    // TODO: Implement image drawing:
    // 1. Decode image data if needed (e.g. PNG, QOI, BMP).
    // 2. Blit pixels to framebuffer, potentially with alpha blending.
}

/// Cleans up renderer resources.
pub fn shutdown_renderer() {
    logger::info!("[Renderer] Shutting down renderer...");
    unsafe {
        FRAMEBUFFER_INFO = None;
        // If using NanoVG: drop(NANOVG_CONTEXT.take());
    }
}

// Helper to determine pixel format (conceptual)
// fn determine_pixel_format(boot_info: &BootInfo) -> PixelFormat {
//     if boot_info.bpp == 32 {
//         // UEFI GOP often provides PixelFormat in its ModeInfo.
//         // If boot_info has `uefi_pixel_format_enum`:
//         // match boot_info.uefi_pixel_format_enum {
//         //     PIXEL_BGR: return PixelFormat::Bgra8888, // Common UEFI default
//         //     PIXEL_RGB: return PixelFormat::Rgba8888,
//         //     ...
//         // }
//         // Fallback guess if no explicit format info:
//         return PixelFormat::Bgra8888; // Or Rgba8888
//     } else if boot_info.bpp == 24 {
//         return PixelFormat::Bgr888; // Or Rgb888
//     }
//     PixelFormat::Unknown
// }