// Lionbootloader Core - Logger
// File: core/src/logger.rs

use core::fmt;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

#[cfg(feature = "with_alloc")]
use alloc::string::String;

// --- Static Global Logger Instance ---
// This allows the `log` macros (info!, error!, etc.) to work globally
// once the logger is initialized.
static LOGGER_INSTANCE: LblLogger = LblLogger {
    // Using core::sync::atomic for interior mutability of the level filter.
    // This avoids needing a Mutex if we only change the level filter.
    // For more complex state, a spin::Mutex might be needed.
    level_filter: core::sync::atomic::AtomicUsize::new(LEVEL_FILTER_INFO_USIZE), // Default to Info
    // early_init_done: core::sync::atomic::AtomicBool::new(false),
};

// Helper consts for AtomicUsize LevelFilter
const LEVEL_FILTER_OFF_USIZE: usize = 0;
const LEVEL_FILTER_ERROR_USIZE: usize = 1;
const LEVEL_FILTER_WARN_USIZE: usize = 2;
const LEVEL_FILTER_INFO_USIZE: usize = 3;
const LEVEL_FILTER_DEBUG_USIZE: usize = 4;
const LEVEL_FILTER_TRACE_USIZE: usize = 5;

fn level_filter_to_usize(filter: LevelFilter) -> usize {
    match filter {
        LevelFilter::Off => LEVEL_FILTER_OFF_USIZE,
        LevelFilter::Error => LEVEL_FILTER_ERROR_USIZE,
        LevelFilter::Warn => LEVEL_FILTER_WARN_USIZE,
        LevelFilter::Info => LEVEL_FILTER_INFO_USIZE,
        LevelFilter::Debug => LEVEL_FILTER_DEBUG_USIZE,
        LevelFilter::Trace => LEVEL_FILTER_TRACE_USIZE,
    }
}

fn usize_to_level_filter(val: usize) -> LevelFilter {
    match val {
        LEVEL_FILTER_OFF_USIZE => LevelFilter::Off,
        LEVEL_FILTER_ERROR_USIZE => LevelFilter::Error,
        LEVEL_FILTER_WARN_USIZE => LevelFilter::Warn,
        LEVEL_FILTER_INFO_USIZE => LevelFilter::Info,
        LEVEL_FILTER_DEBUG_USIZE => LevelFilter::Debug,
        LEVEL_FILTER_TRACE_USIZE => LevelFilter::Trace,
        _ => LevelFilter::Off, // Should not happen
    }
}


/// The LBL logger implementation.
struct LblLogger {
    level_filter: core::sync::atomic::AtomicUsize,
    // early_init_done: core::sync::atomic::AtomicBool,
}

impl Log for LblLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let current_filter_val = self.level_filter.load(core::sync::atomic::Ordering::Relaxed);
        let current_filter = usize_to_level_filter(current_filter_val);
        metadata.level() <= current_filter
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // In a no_std environment, `println!` is not available directly.
            // We need to implement or use a serial port / framebuffer print function.
            // For now, this is a placeholder.
            let level_str = match record.level() {
                Level::Error => "ERROR",
                Level::Warn => "WARN ",
                Level::Info => "INFO ",
                Level::Debug => "DEBUG",
                Level::Trace => "TRACE",
            };
            
            // Construct the log message string.
            // This part might require alloc::format! or a custom no_alloc formatter.
            #[cfg(feature = "with_alloc")]
            let message = alloc::format!("[LBL {}] {}: {}", level_str, record.target(), record.args());
            
            #[cfg(not(feature = "with_alloc"))]
            // Crude non-alloc version - will just print the level and target roughly.
            // `record.args()` is harder to handle without `alloc::format!`.
            // We can try a simple `core::fmt::Write` approach if a writer is available.
            let message_stub_part1 = alloc::format!("[LBL {}] {}: ", level_str, record.target());


            // --- Actual Output ---
            // This needs to be directed to a serial port or framebuffer.
            // The `CONSOLE_WRITER` would be a static Mutex-protected object
            // that implements `core::fmt::Write`.
            /*
            if let Some(writer) = CONSOLE_WRITER.try_lock() { // Or spin::Mutex for no_std critical sections
                if writer.is_some() {
                    let _ = writeln!(writer.as_mut().unwrap(), "{}", message);
                } else if self.early_init_done.load(core::sync::atomic::Ordering::Relaxed) {
                    // Logger was initialized but console writer is gone? (Should not happen)
                } else {
                    // Logger not fully initialized, try very early raw print (e.g. direct to known serial port)
                    // early_raw_print(&message);
                }
            }
            */
            // For now, just a placeholder print attempt.
            // On hosted OS, `eprintln` can simulate this.
            // For bare metal, this needs implementation based on HAL.
            #[cfg(feature = "with_alloc")]
            {
                // This will only work if running in a test/std environment or if a global
                // print hook is set up by the environment (like QEMU debug exits or semihosting).
                // For a real bootloader, replace with `serial_print` or `framebuffer_print`.
                // unsafe { host_eprintln(&message); } // Conceptual placeholder
            }
            #[cfg(not(feature = "with_alloc"))]
            {
                // unsafe { host_eprint(&message_stub_part1); }
                // unsafe { host_eprintln_args(record.args()); } // Conceptual
            }

            // TODO: Integrate with HAL's console (serial or framebuffer)
            // Example: crate::hal::console::print_log_record(record);
            // For now, if core::fmt::Write is implemented for a global writer:
            if let Some(mut writer) = try_get_global_writer() {
                 #[cfg(feature = "with_alloc")]
                 let _ = core::fmt::write(&mut writer, format_args!("{}\n", message));

                 #[cfg(not(feature = "with_alloc"))]
                 {
                    let _ = core::fmt::write(&mut writer, format_args!("{} ", message_stub_part1));
                    let _ = core::fmt::write(&mut writer, *record.args()); // Write the core arguments
                    let _ = core::fmt::write(&mut writer, format_args!("\n"));
                 }
            }


        }
    }

    fn flush(&self) {
        // If using a buffered writer for serial/framebuffer, flush it here.
        // crate::hal::console::flush_log_buffer();
        if let Some(mut writer) = try_get_global_writer() {
            let _ = writer.flush_log();
        }
    }
}

/// Initializes the LBL logging system.
/// This function should be called once, very early during boot.
/// It sets up the global logger and its maximum log level.
pub fn init_global_logger(max_level: LevelFilter) -> Result<(), SetLoggerError> {
    // LOGGER_INSTANCE.early_init_done.store(true, core::sync::atomic::Ordering::Relaxed);
    LOGGER_INSTANCE.level_filter.store(level_filter_to_usize(max_level), core::sync::atomic::Ordering::Relaxed);
    log::set_logger(&LOGGER_INSTANCE).map(|()| log::set_max_level(max_level))
}

/// Changes the active log level filter.
pub fn set_log_level(level: LevelFilter) {
    LOGGER_INSTANCE.level_filter.store(level_filter_to_usize(level), core::sync::atomic::Ordering::Relaxed);
    log::set_max_level(level); // Also inform the log facade
    log::info!("[Logger] Log level set to: {:?}", level);
}

/// Returns the current log level filter.
pub fn get_log_level() -> LevelFilter {
    usize_to_level_filter(LOGGER_INSTANCE.level_filter.load(core::sync::atomic::Ordering::Relaxed))
}


// --- Placeholder for Global Writer ---
// This needs to be implemented using HAL primitives (e.g., serial port, framebuffer).
// It should implement `core::fmt::Write`.
// For safety in `no_std` with potential concurrent access (even cooperative),
// it should be wrapped in a spinlock or a Mutex type suitable for the environment.

// pub static GLOBAL_CONSOLE_WRITER: spin::Mutex<Option<impl core::fmt::Write + Send>> = spin::Mutex::new(None);
// A simpler version for single-threaded early boot:
// static mut GLOBAL_WRITER_INSTANCE: Option<OutputWriter> = None;

pub trait LogWriter: core::fmt::Write {
    fn flush_log(&mut self) -> core::fmt::Result { Ok(()) }
}


// Example: A dummy writer that does nothing for now.
// In `hal::console`, you'd create a real `SerialWriter` or `FramebufferWriter`.
struct DummyWriter;
impl core::fmt::Write for DummyWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // On host for testing, could use std::eprint!("{}", s);
        // Here, it would write to serial/framebuffer.
        // For now, we can't do much without HAL.
        // To avoid unused 's', we can do a dummy operation.
        let _ = s.len(); 
        Ok(())
    }
}
impl LogWriter for DummyWriter {}

// This function needs to be called by HAL console initialization to set the actual writer.
// pub fn set_global_writer(writer: impl LogWriter + 'static + Send) {
//     unsafe {
//         // This is unsafe if called multiple times or from multiple threads without sync.
//         // GLOBAL_WRITER_INSTANCE = Some(Box::new(writer)); // Requires alloc for Box<dyn>
//         // For a static type known at compile time:
//         // GLOBAL_WRITER_INSTANCE = Some(writer_instance_of_known_type);
//     }
// }

// A function to get a mutable reference to the writer. Very simplified.
// Needs proper synchronization (e.g. spin::Mutex) in a real scenario.
fn try_get_global_writer() -> Option<impl LogWriter> {
    // In a real system, this would return a locked guard to the actual writer.
    // For now, let's return a dummy that does nothing or prints to host if testing.
    #[cfg(all(test, feature = "std"))]
    {
        struct TestWriter;
        impl core::fmt::Write for TestWriter { fn write_str(&mut self, s: &str) -> core::fmt::Result { std::eprint!("{}", s); Ok(()) } }
        impl LogWriter for TestWriter {}
        Some(TestWriter)
    }
    #[cfg(not(all(test, feature = "std")))]
    {
        Some(DummyWriter) // Does nothing on target by default
    }
}


// --- Global log macros (convenience, uses the `log` crate facade) ---
// These are already available via `use log::{info, warn, error, debug, trace};`
// This file just provides the backend for them.

// Example of direct use of log macros if you import them:
// use log::{info, error};
// pub fn example_logging() {
//     error!("This is an error message.");
//     info!("This is an info message with a parameter: {}", 42);
// }

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    #[test]
    fn test_logger_levels() {
        // Initialize with Info level
        let _ = init_global_logger(LevelFilter::Info);
        assert_eq!(get_log_level(), LevelFilter::Info);

        log::error!("Error visible (Info)"); // Should be visible
        log::warn!("Warn visible (Info)");   // Should be visible
        log::info!("Info visible (Info)");   // Should be visible
        log::debug!("Debug NOT visible (Info)"); // Should be hidden
        log::trace!("Trace NOT visible (Info)"); // Should be hidden

        set_log_level(LevelFilter::Debug);
        assert_eq!(get_log_level(), LevelFilter::Debug);
        log::debug!("Debug NOW visible (Debug)"); // Should be visible

        set_log_level(LevelFilter::Off);
        assert_eq!(get_log_level(), LevelFilter::Off);
        log::error!("Error NOT visible (Off)"); // Should be hidden
    }
}