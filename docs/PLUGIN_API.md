# Lionbootloader (LBL) - Plugin API Specification (v0.1 - Conceptual)

This document outlines the Application Programming Interface (API) for developing plugins for Lionbootloader (LBL). The primary focus of this initial specification is on **Filesystem Plugins** (`.lblfs`).

**Note: This is a conceptual API for a `no_std` environment. True dynamic loading of compiled code plugins in such an environment is extremely complex and has significant security and stability implications. The initial implementation of "plugins" in LBL might rely on statically linked modules activated by configuration, rather than runtime loading of external binaries.**

## 1. Plugin Philosophy

LBL plugins are intended to extend the bootloader's capabilities, primarily for:

*   **Filesystem Support**: Allowing LBL to read kernel images, initrds, and configuration files from a variety of filesystems.
*   **Hardware Drivers (Future)**: Potentially supporting specific hardware not covered by the core HAL.
*   **Security Modules (Future)**: e.g., custom signature verification schemes.

Plugins should be:

*   **Lightweight**: Minimal impact on LBL's footprint and boot time.
*   **Robust**: Well-tested and error-resilient.
*   **Secure**: If handling sensitive operations, developed with security best practices.
*   **`no_std` Compatible**: Must operate within LBL's `#[no_std]` (with `alloc`) environment.

## 2. Filesystem Plugin (`.lblfs`) API

Filesystem plugins enable LBL to recognize, mount (read-only), and read files from different filesystem types.

### 2.1. Plugin Lifecycle and Structure (Conceptual for Dynamic Loading)

If true dynamic loading were implemented (e.g., for ELF-like plugin objects):

1.  **Discovery**: LBL's Plugin Manager finds `.lblfs` files listed in `config.json` or in a predefined plugin directory.
2.  **Loading**: The Plugin Manager reads the `.lblfs` file into memory.
3.  **Linking/Relocation**: If the plugin is a relocatable object, LBL performs necessary relocations against the LBL core's symbols (for accessing HAL, logger, etc.) and resolves symbols expected by LBL from the plugin.
4.  **Initialization**: LBL calls a well-known entry point in the plugin, e.g., `lbl_plugin_init()`.
5.  **Registration**: The `lbl_plugin_init()` function returns an object that implements the `FileSystemDriver` trait (see below). LBL's Filesystem Manager registers this driver.
6.  **Usage**: The Filesystem Manager uses the registered driver to `detect()` and `mount()` filesystems.
7.  **Unloading (Optional)**: LBL might call `lbl_plugin_uninit()` if plugins can be hot-swapped or disabled (advanced feature).

**Simplified Model (Static Linking or Pre-known Plugins):**
For initial LBL versions, "plugins" listed in `config.json` might simply enable corresponding, statically-linked modules within LBL. The `.lblfs` name would map to a known internal driver. No dynamic code loading occurs. The API below would still serve as the internal interface for these modules.

### 2.2. Required Plugin Entry Point (Conceptual for Dynamic Loading)

If dynamically loaded, each `.lblfs` plugin (compiled as a compatible object file) must export a C-ABI function:

```c
// C/Rust FFI Definition
typedef struct LblPluginApiTable LblPluginApiTable; // Provided by LBL Core
typedef void* LblFileSystemDriverHandle; // Opaque handle to the plugin's driver object

/**
 * @brief Initializes the filesystem plugin.
 * This function is called by LBL after loading the plugin.
 * It should return an instance of this plugin's FileSystemDriver implementation.
 *
 * @param api_table A pointer to LBL's API table, providing access to core services
 *                  (e.g., memory allocation, logging, HAL block I/O).
 * @param plugin_allocator (Optional) A specific allocator a plugin might need to use.
 * @return An opaque handle to the FileSystemDriver object, or NULL on error.
 *         The type behind this handle must implement the LBL FileSystemDriver trait.
 */
LblFileSystemDriverHandle lbl_plugin_init(const LblPluginApiTable* api_table /*, Allocator* plugin_allocator */);

/**
 * @brief Uninitializes the filesystem plugin (optional).
 * Called by LBL before unloading the plugin. The plugin should free its resources.
 *
 * @param driver_handle The handle returned by lbl_plugin_init.
 */
// void lbl_plugin_uninit(LblFileSystemDriverHandle driver_handle);
```

### 2.3. `LblPluginApiTable` (Services Provided by LBL Core)

Plugins need access to certain LBL core functionalities. This is provided via an API table passed to `lbl_plugin_init()`.

```c
// Conceptual C definition, Rust equivalent uses traits/structs
struct LblPluginApiTable {
    uint32_t version; // Version of this API table structure

    // Logging
    void (*log_info)(const char* message);
    void (*log_warn)(const char* message);
    void (*log_error)(const char* message);
    void (*log_debug)(const char* message);

    // Memory Allocation (if plugins need to allocate memory via LBL's allocator)
    // void* (*allocate_memory)(size_t size, size_t alignment);
    // void (*free_memory)(void* ptr);

    // Block Device I/O (Crucial for FS plugins)
    // This would allow plugins to read from the underlying block device.
    // `device_handle` is an opaque handle from LBL's HAL.
    // `lba` is the logical block address.
    // `count` is the number of blocks.
    // `buffer` is the destination for read data.
    // Returns 0 on success.
    int (*read_blocks)(void* device_handle, uint64_t lba, uint32_t count, void* buffer);
    uint32_t (*get_block_size)(void* device_handle); // e.g., 512, 4096
    uint64_t (*get_total_blocks)(void* device_handle);
};
```
In Rust, this would likely be passed as references to services implementing specific traits.

### 2.4. `FileSystemDriver` Trait (Rust)

The core of a filesystem plugin is the implementation of the `FileSystemDriver` trait. This trait is defined in `core/src/fs/interface.rs`. Plugins effectively provide an object that fulfills this contract.

```rust
// From core/src/fs/interface.rs (Simplified)
pub trait FileSystemDriver {
    fn name(&self) -> String; // e.g., "FAT32_LBL_Plugin"
    
    // `block_io_device` is an opaque handle/struct from which BlockIo can be obtained using LblPluginApiTable
    fn detect(&self, block_io_device: LblHalDeviceHandle, api: &LblPluginApiTable) -> bool;
    
    fn mount(
        &self,
        block_io_device: LblHalDeviceHandle,
        api: &LblPluginApiTable,
        volume_id: &str, // Assigned by LBL FS Manager
        read_only: bool,
    ) -> Result<Box<dyn FileSystemInstance>, FilesystemError>;
}
```
The `LblHalDeviceHandle` would be an opaque type passed by LBL, which the plugin then uses with `api->read_blocks` etc. `Box<dyn FileSystemInstance>` requires `alloc`.

### 2.5. `FileSystemInstance` Trait (Rust)

Once mounted, the driver returns an object implementing `FileSystemInstance`, also defined in `core/src/fs/interface.rs`.

```rust
// From core/src/fs/interface.rs (Simplified)
pub trait FileSystemInstance {
    fn volume_id(&self) -> &str;
    fn read_file(&self, path: &str) -> Result<Vec<u8>, FilesystemError>;
    fn list_directory(&self, path: &str) -> Result<Vec<DirectoryEntry>, FilesystemError>;
    fn metadata(&self, path: &str) -> Result<FileMetadata, FilesystemError>;
    fn is_read_only(&self) -> bool;
}
```

### 2.6. Building a Filesystem Plugin

1.  **Language**: Rust is preferred, leveraging `#[no_std]` and `alloc`. C/C++ could also be used if adhering to the C-ABI entry points and `LblPluginApiTable`.
2.  **Crate Type (Rust)**: If dynamically loaded, compiled as a `cdylib` or a static library that LBL knows how to process into a loadable module. For statically linked "plugins", it's just another Rust module.
3.  **FFI**: If the plugin is Rust and LBL core is Rust, direct trait object passing is possible if statically linked. If dynamically loaded as a C-ABI library, the Rust plugin would expose `lbl_plugin_init` as `extern "C"`, and internal Rust `FileSystemDriver` instance would be boxed and returned as an opaque `LblFileSystemDriverHandle`. LBL Core would then have FFI wrappers to call trait methods on this handle.
4.  **Dependencies**:
    *   Minimal. Should only rely on `core`, `alloc`, and the `LblPluginApiTable`.
    *   Avoid `std` or platform-specific libraries not provided by LBL.
5.  **Output**: A `.lblfs` file (which could be a `.so`/`.dll`/`.dylib` renamed, or a custom object format if LBL defines one for `no_std` loading).

## 3. Data Plugins (e.g., Themes, Fonts)

These are not code plugins but data files.
*   **Themes**: JSON files parsed by LBL's theme engine. See `theme.rs` and `schema.json`.
*   **Fonts**: Standard font files (e.g., `.ttf`, `.otf`) loaded by the theme engine via the FS module and rasterized by a font library.

No special API is needed for these beyond adhering to expected file formats and being discoverable by LBL's FS/Theme systems.

## 4. API Stability and Versioning

*   The `LblPluginApiTable` will be versioned. Plugins should check this version for compatibility.
*   The `FileSystemDriver` and `FileSystemInstance` traits will also be versioned implicitly by LBL releases.
*   LBL will strive for API stability but breaking changes might occur in early development (pre-1.0).

## 5. Security Considerations for Plugins

*   **Trust**: Dynamically loaded code plugins execute with LBL's privileges. They must be sourced from trusted authors or verified.
*   **Sandboxing (Future)**: True sandboxing of plugins in a `no_std` bootloader is extremely challenging.
*   **Resource Limits**: Plugins should be mindful of memory and CPU usage.
*   **Input Validation**: Plugins (especially FS drivers parsing on-disk structures) must rigorously validate all external data to prevent vulnerabilities.

## Conclusion

This initial Plugin API focuses on making filesystem support extensible. The primary challenge for future LBL versions will be defining a safe and practical way to achieve true dynamic code plugin loading in its constrained `no_std` environment. For now, developers should consider that "plugins" may refer to compile-time configurable modules rather than runtime-loaded binaries.
