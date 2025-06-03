# Lionbootloader (LBL) Specification

## ; A modern, universal, and future-proof bootloader designed for any hardware—from legacy BIOS machines to next-gen UEFI and diverse CPU architectures. LBL emphasizes stability, performance, security, and a rich GUI, all configured via JSON.


---

## 1. High-Level Architecture

### 1. Stage 1 (Firmware Interface)

BIOS MBR (512 B stub) or UEFI .efi image.

Minimal hardware init: CPU mode switch, memory map, minimal device discovery.

Loads Stage 2 Core Engine.



2. Core Engine

Hardware Abstraction Layer (HAL): Async device I/O, unified driver interface.

Filesystem Module Manager: Loadable plugins (.lblfs), e.g. FAT32, ext4, Btrfs, NTFS.

Security Manager: TPM 2.0 integration, signature/secure-boot chain.

Config Module: JSON Schema validation, dynamic config reload.



3. GUI & UX Layer

Rendering: NanoVG/Custom GPU pipeline in protected mode.

Input Handling: Keyboard, mouse, touch, gamepad.

Theme Engine: JSON-driven styling, animations, custom fonts.



4. Boot Executor

Kernel image detection (ELF, PE, IMG).

Initrd/initramfs support.

Multiplatform launch via architecture adaptors (x86, ARM, RISC-V, POWER).





---

### 2. Logic Flow & Module Interactions

User Power-on->Firmware
Firmware->Stage1: Load Stub
Stage1->Core: Jump to Engine
Core->HAL: Start async probes
HAL->Core: Event(device_ready)
Core->Config: Parse JSON
Config->UI: Build Menu
UI->User: Display Menu
User->UI: Selection
UI->Executor: Boot(entry)
Executor->Kernel: Jump

1. Async Device Probing

Launch N parallel tasks: storage, network, GPU, input.

Ready events feed progress bar:
Progress = \frac{Completed}{N} \times 100%

Time to first menu:



2. Config Loading

Validate JSON against embedded Schema v1.1.

Populate structs: Entry[], Plugin[], Theme, Settings.



3. Menu & UX

Tree structure for snapshots, favorites, OS tools.

Search: real-time filter, fuzzy match.

Animations: 60 FPS by default, adaptive to hardware.



4. Security Verification

Signature check per entry:


Retry policy: up to 3 signature fetch attempts over network.



5. Boot Execution

Priority: order field, health score, last-boot time.

Load(entry): read blocks via async I/O, zero-copy buffer.

Jump to kernel entry point in correct mode.





---

### 3. Performance Metrics & Equations

Let:

T_{fw}: firmware handoff (50 μs typical).

T_{probe,i}: i-th device probe.

T_{parse}: JSON parse and validation.

T_{render}: initial GUI render.

T_{load}: kernel+initrd load.


3.1 Total Boot Time

T_{total} = T_{fw} + \max_{i=1..N}(T_{probe,i}) + T_{parse} + T_{render} + T_{load}

3.2 Probe Speedup vs Sequential

S_{probe} = \frac{\sum_{i=1}^N T_{seq,i}}{\max_i T_{probe,i}}

3.3 Memory Footprint

Base memory:

M_{core} = M_{base} + \sum_{j=1}^P M_{plugin,j} + M_{UI}

3.4 GUI Frame Time

T_{frame} = T_{draw} + T_{input} + T_{layout}


---

### 4. JSON Configuration Schema (v1.1)

{
  "$schema": "https://lionbootloader.org/schema/config-1.1.json#",
  "title": "LBL Configuration",
  "type": "object",
  "properties": {
    "timeout_ms": {"type": "integer", "minimum": 0, "default": 5000},
    "theme": {
      "type": "object",
      "properties": {
        "background": {"type": "string", "pattern": "^#([A-Fa-f0-9]{6})$"},
        "accent": {"type": "string", "pattern": "^#([A-Fa-f0-9]{6})$"},
        "font": {"type": "string"}
      },
      "required": ["background", "accent"]
    },
    "entries": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id","title","kernel","cmdline"],
        "properties": {
          "id": {"type":"string"},
          "title": {"type":"string"},
          "kernel": {"type":"string"},
          "initrd": {"type":"string"},
          "cmdline": {"type":"string"},
          "order": {"type":"integer"},
          "secure": {"type":"boolean"}
        }
      }
    },
    "plugins": {"type":"array","items":{"type":"string"}},
    "advanced": {
      "type":"object",
      "properties": {
        "debug_shell": {"type":"boolean"},
        "log_level": {"type":"string","enum":["error","warn","info","debug"]}
      }
    }
  },
  "required": ["entries"]
}


---

### 5. GUI Design & UX

1. Main Window

Sidebar: Config, Plugins, Logs.

Entry List: Icons, titles, health indicators.

Status Bar: Boot countdown, firmware info.



2. Settings Dialog

JSON Editor: Inline syntax highlighting, schema hints.

Theme Preview: Live color picker.

Module Manager: Enable/disable plugins.



3. Animations & Transitions

Smooth fades, slide-ins.

Progress bar synced with device events.



4. Accessibility

High-contrast mode, scalable fonts, keyboard navigation.





---

6. Compatibility & Future-Proofing

Hardware: All BIOS/UEFI, ARM/x86/RISC-V/PowerPC/MIPS.

Stability: 500+ hardware configs, CI across simulators & real devices.

Optimization: Core <1 MiB, Stage1 <16 KiB, async I/O.

Extensibility: Hot-swap plugins, versioned schema, telemetry hooks.



---

7. Industry Adoption & Governance

1. Open Foundation: Neutral governance, RFC process.


2. Vendor Engagement: Certify with Microsoft, Apple, major Linux distros, OEMs.


3. Security Audits: Quarterly audits, bug bounty program.


4. Licensing: Apache 2.0/MIT dual license.


5. Developer Resources: SDKs (Rust, C), reference implementations, dev tools.




---

8. Roadmap & Next Steps

1. Detailed Test Plan: Legacy & modern hardware, virtualization.


2. Module API Spec: C/Rust headers, ABI stability.


3. Schema Repository: Public JSON schemas & version history.


4. Prototype Stage1: NASM + C minimal loader.


5. Core Engine in Rust: Plugins, UI, security.


6. GUI Demo: NanoVG proof-of-concept.


7. Engage Vendors: Draft spec review, pilot builds.




---

Lionbootloader: the bootloader of tomorrow, available today.