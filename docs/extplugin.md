# Lionbootloader (LBL) - External Plugins

This directory is intended for managing Lionbootloader plugins that are not part of the core LBL distribution or are developed by third parties.

## Plugin Types

Lionbootloader is designed to support various types of plugins, with the primary initial focus on:

*   **Filesystem Plugins (`.lblfs`)**: These allow LBL to read and understand different filesystem formats (e.g., XFS, JFS, HFS+, or custom filesystems).
*   **Future Plugin Types**: May include specialized hardware drivers, security modules, or UI extensions.

## Important Note on Dynamic Code Plugins

**True dynamic loading of compiled code plugins (e.g., shared libraries) in Lionbootloader's `no_std` (bare-metal) environment is an advanced feature with significant technical and security challenges. The initial versions of LBL might primarily support "plugins" as statically linked modules that are included at compile-time based on features and then enabled via the `config.json` "plugins" array.**

For example, an `ext4.lblfs` entry in `config.json` might activate a built-in, feature-flagged ext4 driver within LBL, rather than loading an external `ext4.lblfs` binary file from disk at runtime.

As LBL matures, a more robust and secure mechanism for loading truly external, pre-compiled binary plugins might be developed. Please refer to the [LBL Plugin API Specification](../docs/PLUGIN_API.md) for details on plugin development.

## Installing External Plugins (Conceptual)

Assuming a future version of LBL supports loading truly external binary plugins:

1.  **Obtain the Plugin**: Download the pre-compiled plugin file (e.g., `my_custom_fs.lblfs`) from a trusted source. Always ensure plugins come from reputable developers to avoid security risks.
2.  **Plugin Directory**:
    *   LBL will search for plugins in one or more predefined directories on a bootable LBL partition (typically the ESP or a dedicated LBL partition).
    *   A common location might be `/LBL/plugins/`.
    *   Copy the `.lblfs` file (or other plugin type) into this directory.
        ```
        # Example (on a mounted LBL boot partition)
        # Ensure the directories exist:
        # mkdir -p /mnt/lbl_boot/LBL/plugins/fs
        #
        # Copy the filesystem plugin:
        # cp path/to/my_custom_fs.lblfs /mnt/lbl_boot/LBL/plugins/fs/
        ```
3.  **Configure LBL**:
    *   Edit your `config.json` file.
    *   Add the filename of the plugin to the `plugins` array. The path should be relative to LBL's plugin search paths, or LBL might just use the filename and expect it to be in the standard plugin directory.
        ```json
        {
          // ... other LBL config ...
          "plugins": [
            "lblfs_fat32.lblfs", // Built-in or common plugin
            "lblfs_ext4.lblfs",  // Built-in or common plugin
            "fs/my_custom_fs.lblfs" // Path if LBL supports subdirs, or just "my_custom_fs.lblfs"
          ],
          // ...
        }
        ```
4.  **Reboot**: Upon the next boot, LBL's Plugin Manager will attempt to find, load, and initialize the configured plugins.

## Security Considerations

*   **Source**: Only install plugins from sources you trust implicitly. Dynamically loaded code runs with the same privileges as LBL itself, meaning a malicious plugin could compromise the entire boot process and system.
*   **Signatures**: Future versions of LBL may require plugins themselves to be signed by a trusted authority or the LBL developers to enhance security.
*   **Permissions**: LBL aims to provide a well-defined API to plugins, limiting their access to only necessary system resources (see `PLUGIN_API.md`).

## Developing External Plugins

If you are interested in developing plugins for Lionbootloader, please consult the [LBL Plugin API Specification](../docs/PLUGIN_API.md). This document details the required interfaces, data structures, and build considerations.

Given the complexities of `no_std` dynamic loading, early plugin development will likely involve contributing code directly to the LBL project for static linking or working closely with the LBL team to define a stable ABI for external binaries.

## Compatibility

Plugins will be tied to specific versions or version ranges of the LBL Plugin API. Ensure any external plugin you install is compatible with your version of Lionbootloader.