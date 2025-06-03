@echo off
setlocal ENABLEEXTENSIONS

:: Root directory
set ROOT=Lionbootloader

:: Create directories
mkdir %ROOT%
mkdir %ROOT%\config
mkdir %ROOT%\core\src\config
mkdir %ROOT%\core\src\fs
mkdir %ROOT%\core\src\hal
mkdir %ROOT%\core\src\loader
mkdir %ROOT%\core\src\plugins
mkdir %ROOT%\core\src\security
mkdir %ROOT%\docs
mkdir %ROOT%\gui\src\widgets
mkdir %ROOT%\plugins_external
mkdir %ROOT%\stage1\bios
mkdir %ROOT%\stage1\common
mkdir %ROOT%\stage1\uefi
mkdir %ROOT%\tools

:: Create top-level files
echo.>%ROOT%\.gitignore
echo.>%ROOT%\Cargo.toml
echo.>%ROOT%\Makefile

:: Config files
echo.>%ROOT%\config\default.json
echo.>%ROOT%\config\schema.json

:: Core crate
echo.>%ROOT%\core\Cargo.toml
echo.>%ROOT%\core\src\lib.rs
echo.>%ROOT%\core\src\main.rs
echo.>%ROOT%\core\src\config.rs
echo.>%ROOT%\core\src\fs.rs
echo.>%ROOT%\core\src\hal.rs
echo.>%ROOT%\core\src\loader.rs
echo.>%ROOT%\core\src\logger.rs
echo.>%ROOT%\core\src\plugins.rs
echo.>%ROOT%\core\src\security.rs

:: Config module
echo.>%ROOT%\core\src\config\mod.rs
echo.>%ROOT%\core\src\config\parser.rs
echo.>%ROOT%\core\src\config\schema_types.rs

:: Filesystem module
echo.>%ROOT%\core\src\fs\mod.rs
echo.>%ROOT%\core\src\fs\btrfs.rs
echo.>%ROOT%\core\src\fs\ext4.rs
echo.>%ROOT%\core\src\fs\fat32.rs
echo.>%ROOT%\core\src\fs\interface.rs
echo.>%ROOT%\core\src\fs\manager.rs
echo.>%ROOT%\core\src\fs\ntfs.rs

:: HAL module
echo.>%ROOT%\core\src\hal\mod.rs
echo.>%ROOT%\core\src\hal\async_probe.rs
echo.>%ROOT%\core\src\hal\device_manager.rs

:: Loader module
echo.>%ROOT%\core\src\loader\mod.rs
echo.>%ROOT%\core\src\loader\arch_adapter.rs
echo.>%ROOT%\core\src\loader\kernel_formats.rs

:: Plugins module
echo.>%ROOT%\core\src\plugins\mod.rs
echo.>%ROOT%\core\src\plugins\manager.rs

:: Security module
echo.>%ROOT%\core\src\security\mod.rs
echo.>%ROOT%\core\src\security\signature.rs
echo.>%ROOT%\core\src\security\tpm.rs

:: Docs
echo.>%ROOT%\docs\ARCHITECTURE.md
echo.>%ROOT%\docs\BUILD_INSTRUCTIONS.md
echo.>%ROOT%\docs\LBL_SPECIFICATION.md
echo.>%ROOT%\docs\LICENSE_APACHE2.txt
echo.>%ROOT%\docs\LICENSE_MIT.txt
echo.>%ROOT%\docs\PLUGIN_API.md
echo.>%ROOT%\docs\README.md

:: GUI crate
echo.>%ROOT%\gui\Cargo.toml
echo.>%ROOT%\gui\src\lib.rs
echo.>%ROOT%\gui\src\animations.rs
echo.>%ROOT%\gui\src\input.rs
echo.>%ROOT%\gui\src\renderer.rs
echo.>%ROOT%\gui\src\theme.rs
echo.>%ROOT%\gui\src\ui.rs
echo.>%ROOT%\gui\src\widgets\mod.rs
echo.>%ROOT%\gui\src\widgets\button.rs
echo.>%ROOT%\gui\src\widgets\list_view.rs
echo.>%ROOT%\gui\src\widgets\progress_bar.rs

:: Plugins external
echo.>%ROOT%\plugins_external\README.md

:: Stage 1 BIOS
echo.>%ROOT%\stage1\Makefile
echo.>%ROOT%\stage1\bios\boot_32.asm
echo.>%ROOT%\stage1\bios\linker_bios.ld
echo.>%ROOT%\stage1\bios\mbr.asm

:: Stage 1 Common
echo.>%ROOT%\stage1\common\stage1_loader_utils.c
echo.>%ROOT%\stage1\common\stage1_loader_utils.h

:: Stage 1 UEFI
echo.>%ROOT%\stage1\uefi\LblUefi.c
echo.>%ROOT%\stage1\uefi\LblUefi.h

:: Tools
echo.>%ROOT%\tools\build_core_gui.sh
echo.>%ROOT%\tools\build_stage1.sh
echo.>%ROOT%\tools\build.sh
echo.>%ROOT%\tools\cross_compile_setup.sh
echo.>%ROOT%\tools\mkimage.sh

echo Project structure created successfully.
pause
