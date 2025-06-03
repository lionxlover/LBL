// Lionbootloader Core - Hardware Abstraction Layer (HAL)
// File: core/src/hal.rs

#[cfg(feature = "with_alloc")]
use alloc::vec::Vec;

use crate::logger; // Assuming logger is available for HAL messages

// Sub-modules for different HAL functionalities
pub mod async_probe;
pub mod device_manager;
// pub mod memory; // For memory map parsing and management
// pub mod cpu;    // For CPU specific features, mode switching (if not done by Stage1)
// pub mod pci;    // For PCI device enumeration
// pub mod acpi;   // For ACPI table parsing
// pub mod console; // For basic text output (framebuffer, serial)
// pub mod timer;   // For timers and delays
// pub mod interrupts; // For interrupt handling setup (if core manages them)

/// Structure to hold information passed from Stage 1 to the Core engine.
/// This needs to be defined consistently with what Stage 1 provides.
/// This is a placeholder and should be detailed based on Stage1 capabilities.
#[derive(Debug, Clone, Copy)]
#[repr(C)] // Ensure C-compatible layout if Stage1 is C/Asm
pub struct BootInfo {
    pub magic: u64, // To verify this struct is valid
    pub version: u32,
    pub memory_map_addr: u64,
    pub memory_map_entries: u64,
    pub framebuffer_addr: u64,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_pitch: u32,
    pub framebuffer_bpp: u8,
    // pub acpi_rsdp_ptr: u64, // Pointer to ACPI RSDP table
    // pub kernel_load_address: u64, // Suggested address to load the OS kernel
    // pub core_elf_segments_addr: u64, // Info about where the core itself is loaded
    // pub core_elf_segments_len: u64,
    // ... other fields like serial port config, etc.
}

impl BootInfo {
    pub const LBL_BOOT_INFO_MAGIC: u64 = 0 LBL_BI_MGC; // LionBootLoader_BootInfo_MaGiC
}

/// Represents a discovered hardware device.
/// This is a simplified representation.
#[cfg(feature = "with_alloc")]
#[derive(Debug, Clone)]
pub struct Device {
    pub id: u64, // Unique device ID
    pub name: alloc::string::String,
    pub device_type: DeviceType,
    // pub resources: Vec<Resource>, // e.g., memory regions, I/O ports, IRQs
}

#[cfg(not(feature = "with_alloc"))]
#[derive(Debug, Clone, Copy)] // No String if no_alloc
pub struct Device {
    // Simplified for no_alloc, might use fixed-size strings or just IDs
    pub id: u64,
    pub device_type: DeviceType,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Storage,      // HDD, SSD, NVMe, USB Drive
    Network,      // Ethernet, Wi-Fi
    Gpu,          // Graphics Processing Unit
    InputKeyboard,
    InputMouse,
    InputTouch,
    InputGamepad,
    SerialPort,
    UsbController,
    PciBridge,
    Other,
}

/// The main HAL services structure.
/// This structure will provide access to various hardware functions.
/// Its fields will be populated during HAL initialization.
#[cfg(feature = "with_alloc")]
pub struct HalServices {
    boot_info: BootInfo,
    pub device_manager: device_manager::DeviceManager,
    // pub memory_services: memory::MemoryServices,
    // pub acpi_services: acpi::AcpiServices,
    // pub console: console::Console, // For printing to screen/serial
    // pub timer: timer::Timer,
}

#[cfg(not(feature = "with_alloc"))]
pub struct HalServices { // Simplified for no_alloc
    boot_info: BootInfo,
    // device_manager might be simpler or omitted if no dynamic device list
}


/// Initializes the Hardware Abstraction Layer.
///
/// # Safety
/// `boot_info_ptr` must be a valid pointer to a `BootInfo` structure provided by Stage 1.
/// The memory regions described by `BootInfo` (e.g., memory map, framebuffer) must be valid
/// and accessible.
pub unsafe fn initialize(boot_info_ptr: *const u8) -> Result<HalServices, HalError> {
    if boot_info_ptr.is_null() {
        // Cannot use logger here yet as it might depend on HAL for console.
        // This is a critical failure.
        return Err(HalError::NullBootInfo);
    }

    let boot_info = unsafe { &*(boot_info_ptr as *const BootInfo) };

    // Verify magic number
    if boot_info.magic != BootInfo::LBL_BOOT_INFO_MAGIC {
        return Err(HalError::InvalidBootInfoMagic);
    }

    // TODO: Initialize early console for logging, if possible, using framebuffer_info
    // logger::init_video_console(boot_info.framebuffer_addr, ...);
    // Or logger::init_serial_console(...); if info is available.
    // For now, assume logger is functional or will be soon.
    logger::info!("[HAL] Initializing HAL...");
    logger::info!("[HAL] Boot Info: {:?}", boot_info); // Might be too verbose

    // Initialize memory services (parsing memory map, setting up page tables if core does it)
    // let memory_services = memory::initialize(boot_info.memory_map_addr, boot_info.memory_map_entries)?;
    // logger::info!("[HAL] Memory services initialized.");

    // Initialize ACPI services (find and parse ACPI tables)
    // let acpi_services = acpi::initialize(boot_info.acpi_rsdp_ptr)?;
    // logger::info!("[HAL] ACPI services initialized.");

    // Initialize device manager
    #[cfg(feature = "with_alloc")]
    let device_manager = device_manager::DeviceManager::new();
    // logger::info!("[HAL] Device manager initialized.");

    // TODO: Initialize other HAL components: PCI, timers, console, etc.
    // let console = console::initialize(boot_info);
    // let timer = timer::initialize();

    logger::info!("[HAL] HAL initialization complete.");

    #[cfg(feature = "with_alloc")]
    {
        Ok(HalServices {
            boot_info: *boot_info, // Store a copy
            device_manager,
            // memory_services,
            // acpi_services,
            // console,
            // timer,
        })
    }
    #[cfg(not(feature = "with_alloc"))]
    {
        Ok(HalServices {
            boot_info: *boot_info,
        })
    }
}

/// Starts the asynchronous device probing process.
/// This will spawn tasks to detect storage, network, GPU, input, etc.
#[cfg(feature = "with_alloc")] // Async probing typically needs allocation for tasks/futures
pub fn start_async_device_probes(hal: &mut HalServices) -> async_probe::ProbeHandle {
    logger::info!("[HAL] Starting asynchronous device probing...");
    // The async_probe module will use other HAL services (PCI, USB, ACPI) to find devices.
    async_probe::start_probes(&mut hal.device_manager /*, &hal.pci_services, &hal.usb_services etc */)
}


#[derive(Debug)]
pub enum HalError {
    NullBootInfo,
    InvalidBootInfoMagic,
    MemoryMapParsingFailed,
    AcpiInitializationFailed,
    PciInitializationFailed,
    DeviceNotFound,
    DriverError,
    Other(#[cfg(feature = "with_alloc")] alloc::string::String),
    #[cfg(not(feature = "with_alloc"))]
    OtherStatic(&'static str),
}


// General HAL utility functions might go here, or within specific submodules.

// Example:
// pub fn delay_ms(ms: u32, hal_services: &HalServices) {
//     hal_services.timer.delay_ms(ms);
// }

// pub fn print_to_console(s: &str, hal_services: &HalServices) {
//     hal_services.console.print_str(s);
// }

// pub fn read_pci_config(...) -> u32 { ... }
// pub fn map_physical_memory(...) -> *mut u8 { ... }
// In Rust (core/src/hal.rs)
#[repr(C)]
#[derive(Debug, Clone, Copy)] // If needed
pub struct LblBootInfoRaw {
    pub magic: u64,
    pub version: u32,
    pub header_size: u32,
    pub total_size: u32,

    pub core_load_addr: u64,
    pub core_size: u64,
    pub core_entry_offset: u64,

    pub memory_map_buffer: *const EfiMemoryDescriptorRaw, // Pointer to raw EFI descriptors
    pub memory_map_size: usize, // UEFI UINTN maps to Rust usize on same-arch
    pub memory_map_key: usize,
    pub memory_descriptor_size: usize,
    pub memory_descriptor_version: u32,

    pub framebuffer_addr: u64,
    pub framebuffer_size: u64,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_pitch: u32, // Assuming bytes per scan line
    pub framebuffer_bpp: u8,
    pub framebuffer_pixel_format_info: u8,
    pub reserved_graphics: u16,

    pub acpi_rsdp_ptr: u64,
    pub efi_system_table_ptr: u64,
    
    pub reserved1: u64,
    pub reserved2: u64,
}

// Rust equivalent of EFI_MEMORY_DESCRIPTOR would also be needed with #[repr(C)]
#[repr(C)]
pub struct EfiMemoryDescriptorRaw {
    pub r#type: u32, // `type` is a keyword in Rust
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}