// Lionbootloader Core - HAL Device Manager
// File: core/src/hal/device_manager.rs

#[cfg(feature = "with_alloc")]
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};

use crate::hal::{Device, DeviceType}; // Use Device from the parent hal module
use crate::logger;

// A callback type that probe tasks can use to report a discovered device.
// Using a Box<dyn Fn(...)> allows for flexibility but requires `alloc`.
#[cfg(feature = "with_alloc")]
pub type DiscoveredDeviceCallback = Box<dyn Fn(Device) + Send + Sync>;

// For no_alloc, the callback might not be feasible in this dynamic way,
// or it would be a simple function pointer `fn(Device)`.
// For simplicity, let's assume direct calls to DeviceManager in no_alloc scenarios.
#[cfg(not(feature = "with_alloc"))]
pub type DiscoveredDeviceCallback = fn(Device); // Simplistic function pointer

/// Manages the list of discovered hardware devices.
pub struct DeviceManager {
    #[cfg(feature = "with_alloc")]
    devices: BTreeMap<u64, Device>, // Store devices by a unique ID for quick lookup
    #[cfg(feature = "with_alloc")]
    device_ids_by_type: BTreeMap<DeviceType, Vec<u64>>,

    //计数器，用于生成唯一的设备ID
    next_device_id: u64,

    // For no_alloc, we might use a fixed-size array or a simpler scheme if dynamic listing is too hard.
    // For this example, let's assume DeviceManager is less featureful or not used in no_alloc probe handle.
    // Or, it could use a pre-allocated static array.
    #[cfg(not(feature = "with_alloc"))]
    devices_storage: [Option<Device>; MAX_DEVICES_NO_ALLOC], // Example fixed-size storage
    #[cfg(not(feature = "with_alloc"))]
    device_count: usize,
}

#[cfg(not(feature = "with_alloc"))]
const MAX_DEVICES_NO_ALLOC: usize = 32; // Max devices if not using dynamic allocation

impl DeviceManager {
    /// Creates a new, empty DeviceManager.
    pub fn new() -> Self {
        logger::info!("[DeviceManager] Initializing...");
        DeviceManager {
            #[cfg(feature = "with_alloc")]
            devices: BTreeMap::new(),
            #[cfg(feature = "with_alloc")]
            device_ids_by_type: BTreeMap::new(),
            next_device_id: 1, // Start IDs from 1
            #[cfg(not(feature = "with_alloc"))]
            devices_storage: [None; MAX_DEVICES_NO_ALLOC],
            #[cfg(not(feature = "with_alloc"))]
            device_count: 0,
        }
    }

    /// Generates a unique ID for a new device.
    fn generate_id(&mut self) -> u64 {
        let id = self.next_device_id;
        self.next_device_id += 1;
        id
    }

    /// Adds a newly discovered device to the manager.
    /// This would typically be called by a probe task.
    #[cfg(feature = "with_alloc")]
    pub fn add_device(&mut self, mut device_info: Device) {
        if device_info.id == 0 { // If ID is not pre-assigned by probe
            device_info.id = self.generate_id();
        } else {
            // If ID was pre-assigned, ensure next_id is greater to avoid collision
            if device_info.id >= self.next_device_id {
                self.next_device_id = device_info.id + 1;
            }
        }

        logger::info!(
            "[DeviceManager] Adding device: ID={}, Name='{}', Type={:?}",
            device_info.id,
            device_info.name,
            device_info.device_type
        );

        self.devices.insert(device_info.id, device_info.clone());
        self.device_ids_by_type
            .entry(device_info.device_type)
            .or_insert_with(Vec::new)
            .push(device_info.id);
    }

    #[cfg(not(feature = "with_alloc"))]
    pub fn add_device(&mut self, mut device_info: Device) {
        if self.device_count >= MAX_DEVICES_NO_ALLOC {
            logger::warn!("[DeviceManager] Max device limit reached in no_alloc mode. Cannot add more devices.");
            return;
        }

        if device_info.id == 0 {
            device_info.id = self.generate_id();
        } else {
            if device_info.id >= self.next_device_id {
                self.next_device_id = device_info.id + 1;
            }
        }
        
        logger::info!(
            "[DeviceManager] Adding device: ID={}, Type={:?} (no_alloc)",
            device_info.id,
            device_info.device_type
        );

        self.devices_storage[self.device_count] = Some(device_info);
        self.device_count += 1;
        // Note: device_ids_by_type is not implemented for no_alloc simplicity here
    }

    /// Retrieves a device by its ID.
    #[cfg(feature = "with_alloc")]
    pub fn get_device_by_id(&self, id: u64) -> Option<&Device> {
        self.devices.get(&id)
    }

    #[cfg(not(feature = "with_alloc"))]
    pub fn get_device_by_id(&self, id: u64) -> Option<&Device> {
        for i in 0..self.device_count {
            if let Some(ref device) = self.devices_storage[i] {
                if device.id == id {
                    return Some(device);
                }
            }
        }
        None
    }

    /// Retrieves all devices of a specific type.
    #[cfg(feature = "with_alloc")]
    pub fn get_devices_by_type(&self, device_type: DeviceType) -> Vec<&Device> {
        let mut result_devices = Vec::new();
        if let Some(ids) = self.device_ids_by_type.get(&device_type) {
            for id in ids {
                if let Some(device) = self.devices.get(id) {
                    result_devices.push(device);
                }
            }
        }
        result_devices
    }

    #[cfg(not(feature = "with_alloc"))]
    pub fn get_devices_by_type(&self, device_type: DeviceType) -> Vec<&Device> {
        // This is inefficient for no_alloc without the BTreeMap index.
        // For a real no_alloc, you might build fixed-size lists per type, or just iterate all.
        let mut result_devices = Vec::new(); // This `Vec` usage is problematic for strict no_alloc.
                                             // A real no_alloc version would return an iterator or fill a pre-allocated slice.
                                             // For demonstration with current Device structure:
        for i in 0..self.device_count {
            if let Some(ref device) = self.devices_storage[i] {
                if device.device_type == device_type {
                    // This `push` would fail if `Vec` is not available.
                    // result_devices.push(device); // Placeholder
                }
            }
        }
        result_devices // This return type needs to change for true no_alloc.
    }


    /// Returns an iterator over all discovered devices.
    #[cfg(feature = "with_alloc")]
    pub fn iter_devices(&self) -> impl Iterator<Item = &Device> {
        self.devices.values()
    }

    // For no_alloc, iter_devices would iterate over self.devices_storage
    #[cfg(not(feature = "with_alloc"))]
    pub fn iter_devices(&self) -> impl Iterator<Item = &Device> {
        self.devices_storage[0..self.device_count].iter().filter_map(Option::as_ref)
    }


    /// Creates a callback function that can be passed to probe tasks.
    /// This is a simplified way to give probes access to `add_device`.
    /// It's tricky with lifetimes and ownership; a real system might use `Arc<Mutex<DeviceManager>>`
    /// or a channel/queue for messages if probes are on different threads (not applicable for simple cooperative tasks).
    /// For single-threaded polling, a mutable borrow passed around or a RefCell could work.
    ///
    /// This current approach with `Box<dyn Fn>` is fine for `alloc` enabled single-threaded async.
    #[cfg(feature = "with_alloc")]
    pub fn get_add_device_callback(_manager_instance_placeholder: &mut Self) -> DiscoveredDeviceCallback {
        // THIS IS A VERY SIMPLIFIED AND LIKELY PROBLEMATIC APPROACH FOR REAL CONCURRENCY.
        // It assumes the callback will be called in a context where it can safely access
        // a mutable DeviceManager or that the DeviceManager uses internal mutability (e.g. Mutex).
        // For now, this method implies probes get a way to call `add_device`.
        // A proper implementation would involve `Arc<Mutex<DeviceManager>>` and cloning the Arc
        // for each task, or a message-passing system.
        //
        // Since `async_probe::start_probes` currently gets `DiscoveredDeviceCallback` without
        // `self`, the callback cannot directly mutate `self` unless `self` is `&'static Mutex<...>`
        // or similar.
        //
        // For now, let's assume the callback is for a conceptual, single-threaded poller
        // where the `device_manager` instance is globally accessible or passed around carefully.
        // This aspect needs robust design in a full implementation.

        // A more realistic `get_add_device_callback` would be on an `Arc<Mutex<DeviceManager>>`.
        // For example:
        // fn get_add_device_callback(manager: Arc<Mutex<DeviceManager>>) -> DiscoveredDeviceCallback {
        //     Box::new(move |device| {
        //         manager.lock().add_device(device);
        //     })
        // }

        // Placeholder, this needs to be properly implemented with a shared DeviceManager state.
        // The current signature of `async_probe::start_probes` takes `DiscoveredDeviceCallback`
        // directly, which implies this callback needs to be self-contained or access statics.
        // Let's log a warning.
        logger::warn!("[DeviceManager] get_add_device_callback is a placeholder. Proper shared state management needed.");
        Box::new(|device: Device| {
            logger::debug!("[DeviceManager CB] Device reported: ID={}, Name='{}', Type={:?}",
                device.id, device.name, device.device_type);
            // Here, it would call `device_manager.add_device(device)` on the actual instance.
            // THIS WILL NOT WORK as is because the closure has no `DeviceManager` instance.
        })
    }

    #[cfg(not(feature = "with_alloc"))]
    pub fn get_add_device_callback() -> DiscoveredDeviceCallback {
        // In no_alloc, the callback might be simpler, or probes call a static function that
        // accesses a global static DeviceManager instance (if one exists and is safe).
        fn static_add_device_cb(device: Device) {
            logger::debug!("[DeviceManager CB no_alloc] Device reported: ID={}, Type={:?}", device.id, device.device_type);
            // Access a global static mut DeviceManager here (requires unsafe and careful synchronization)
            // GLOBAL_DEVICE_MANAGER.lock().add_device(device);
        }
        static_add_device_cb
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}