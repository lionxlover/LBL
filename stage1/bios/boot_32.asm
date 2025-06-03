; Lionbootloader - Stage 1 - Stage 2 BIOS Loader (boot_32.asm)
; File: stage1/bios/boot_32.asm
; Purpose: Switches to 32-bit protected mode, loads LBL Core Engine, and jumps to it.

BITS 16             ; Loaded by MBR in 16-bit mode
ORG 0x0000          ; Assembled to be loaded at LBL_STAGE2_LOAD_SEGMENT:0x0000
                    ; Actual load address (e.g. 0x8000) is determined by MBR via lbl_config_bios.inc

%include "stage1/bios/lbl_config_bios.inc" ; Common build-time configurations

jmp stage2_start    ; Jump over GDT data

; --- Global Descriptor Table (GDT) ---
; GDT structure: 8 bytes per descriptor
;   Limit (bits 0-15)
;   Base (bits 0-15)
;   Base (bits 16-23)
;   Access Byte (Type, S, DPL, P)
;   Flags (Limit 16-19, AVL, L, D/B, G)
;   Base (bits 24-31)

gdt_start:
    ; Null Descriptor (required)
    dq 0x0000000000000000

    ; Code Segment Descriptor (Ring 0, 32-bit, 4GB limit)
    ; Limit = 0xFFFFF (4GB when G=1), Base = 0x0
    ; Access = 10011010b (Present, Ring 0, Code, Exec/Read)
    ; Flags  = 11001111b (Granularity=4KB, 32-bit Default Operand Size)
gdt_code:
    dw 0xFFFF       ; Limit (low)
    dw 0x0000       ; Base (low)
    db 0x00         ; Base (mid)
    db 0x9A         ; Access Byte (P=1, DPL=0, S=1, Type=1010 -> Code, RX)
    db 0xCF         ; Flags (G=1, D=1, L=0, AVL=0) + Limit (high bits 1111)
    db 0x00         ; Base (high)

    ; Data Segment Descriptor (Ring 0, 32-bit, 4GB limit)
    ; Limit = 0xFFFFF, Base = 0x0
    ; Access = 10010010b (Present, Ring 0, Data, Read/Write)
    ; Flags  = 11001111b
gdt_data:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 0x92         ; Access Byte (P=1, DPL=0, S=1, Type=0010 -> Data, RW)
    db 0xCF
    db 0x00
gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1 ; GDT limit (size of table in bytes - 1)
    dd gdt_start               ; GDT base address (linear address)
                               ; This needs to be a physical address before paging.
                               ; Since ORG is 0, this will be LBL_STAGE2_LOAD_SEGMENT << 4 + gdt_start_offset

; Segment selectors (offsets into GDT)
CODE_SEG equ gdt_code - gdt_start
DATA_SEG equ gdt_data - gdt_start


boot_drive_s2   db 0    ; To store boot drive from MBR (if MBR passes it)
core_load_lba   dd LBL_CORE_START_SECTOR_LBA ; From lbl_config_bios.inc
core_num_sectors dw LBL_CORE_SECTOR_COUNT_WORD ; From lbl_config_bios.inc

; DAP for loading core engine
core_dap:
    db 0x10         ; Size of packet
    db 0
core_dap_num_sectors:
    dw LBL_CORE_SECTOR_COUNT_WORD ; Number of sectors (placeholder, fill at runtime)
core_dap_buffer_offset:
    dw 0x0000       ; Buffer offset (e.g. 0 for segment start)
core_dap_buffer_segment:
    dw 0x0000       ; Buffer segment (e.g. 0x10000 for 1MB / 0x01000 for 64KB segments if using 0x1000 for segment)
                    ; This will be LBL_CORE_LOAD_ADDRESS_SEG
core_dap_lba_low:
    dd LBL_CORE_START_SECTOR_LBA ; LBA start (placeholder, fill at runtime)
core_dap_lba_high:
    dd 0


stage2_start:
    ; Assume MBR has set up DS, ES, SS, SP correctly for this stage.
    ; Stage 2 code itself is loaded at LBL_STAGE2_LOAD_SEGMENT:LBL_STAGE2_LOAD_OFFSET
    ; The ORG 0 directive means all labels are relative to this load offset.
    ; So, gdt_start label value needs to be added to the load address (segment << 4 + offset)
    ; to get its physical address for the GDTR.

    mov [boot_drive_s2], dl ; Save boot drive passed by BIOS (MBR should preserve/pass this)

    mov si, msg_stage2_init
    call print_string_16

    ; --- Enable A20 Line ---
    call enable_a20_fastgate
    jc .a20_error            ; If CF set, A20 enable failed

    mov si, msg_a20_ok
    call print_string_16

    ; --- Detect Memory (Simplified using INT 15h, E820h) ---
    ; A full E820h loop is complex. Here we'll just note it.
    ; The memory map would be collected and stored, then passed to core engine.
    ; For now, assume core engine gets it from UEFI if available, or a simpler map.
    mov si, msg_mem_detect
    call print_string_16
    ; ... call get_e820_memory_map (complex function not implemented here) ...
    ; For this stub, we skip full memory map collection by Stage2 BIOS.
    ; Core engine will have to do it or rely on what Stage 1 can provide.

    ; --- Load LBL Core Engine ---
    mov si, msg_loading_core
    call print_string_16

    ; Set up DAP for core engine loading
    mov ax, word [LBL_CORE_LOAD_ADDRESS_SEG]  ; Target segment for core
    mov word [core_dap_buffer_segment], ax
    mov ax, word [LBL_CORE_LOAD_ADDRESS_OFF]  ; Target offset for core
    mov word [core_dap_buffer_offset], ax
    ; LBA and sector count are already defined or taken from core_load_lba/core_num_sectors above
    ; If needed, can also copy from core_load_lba and core_num_sectors into DAP here.

    mov ah, 0x42        ; Extended Read
    mov dl, byte [boot_drive_s2]
    mov si, core_dap    ; Pointer to DAP
    int 0x13
    jc .core_load_error

    mov si, msg_core_loaded
    call print_string_16

    ; --- Transition to 32-bit Protected Mode ---
    cli                 ; 1. Disable interrupts

    ; Load GDT Register (LGDT)
    ; GDTR needs a linear address. Calculate physical address of gdt_start.
    ; Assumes this Stage 2 code is loaded at LBL_STAGE2_LOAD_ADDRESS (segment:offset)
    ; The ORG 0 means gdt_descriptor is relative to where this code is loaded.
    mov eax, [LBL_STAGE2_LOAD_SEGMENT_LIN] ; Get linear base of Stage2 from config
                                           ; e.g., if 0x0800:0x0000, then linear is 0x8000
    add eax, gdt_descriptor_dd_offset_val ; Add offset of `dd gdt_start` within gdt_descriptor
                                          ; This is tricky; easier if gdt_descriptor structure is fixed
                                          ; Or, more robustly:
                                          ; mov ax, cs
                                          ; shl eax, 4
                                          ; add eax, gdt_start  (if gdt_start is symbol offset from current CS:0)
                                          ; mov [gdt_descriptor + 2], eax ; Store base in GDTR
    ; For simplicity, assume gdt_descriptor's base points correctly relative to load.
    ; We need to adjust the gdt_descriptor's base address part before lgdt
    ; if ORG 0x0000. A simple linker script might place GDT at absolute known address.
    push cs             ; Get current code segment
    pop ds              ; GDT itself is in current CS, which becomes DS after pop
                        ; Now, gdt_descriptor.base (dd gdt_start) refers to symbol gdt_start
                        ; that is an offset from start of this code.
                        ; We need physical address of gdt_start: (CS << 4) + gdt_start_offset symbol
    mov ebx, [cs]       ; Current CS value
    shl ebx, 4          ; EBX = CS linear base address
    add ebx, gdt_start  ; EBX = physical address of gdt_start label
    mov [gdt_descriptor_dd_offset_val_ptr], ebx ; Patch the GDTR base address

    lgdt [gdt_descriptor_dd_offset_val_ptr - 2] ; Load GDT, careful with label reference
                                    ; The label `gdt_descriptor` points to the limit word.
                                    ; The base dword is 2 bytes after that.
                                    ; So, [gdt_descriptor] has limit, [gdt_descriptor+2] has base.

    ; Enable Protected Mode by setting PE bit (bit 0) in CR0
    mov eax, cr0
    or eax, 0x1         ; Set PE bit
    mov cr0, eax

    ; Far jump to flush CPU pipeline and load CS with new descriptor
    ; This jump is to a 32-bit code segment.
    jmp CODE_SEG:.pm_entry  ; CODE_SEG is offset from GDT base (e.g., 0x08)

.a20_error:
    mov si, msg_a20_fail
    call print_string_16
    jmp hang

.core_load_error:
    mov si, msg_core_fail
    call print_string_16
    ; Fall through to hang

hang:
    cli
    hlt
    jmp hang


; --- 32-bit Protected Mode Code ---
BITS 32
.pm_entry:
    ; Now in 32-bit Protected Mode
    ; Set up data segment registers
    mov ax, DATA_SEG    ; DATA_SEG is offset from GDT base (e.g. 0x10)
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax          ; Stack segment
    mov esp, 0x0008FFFF ; Set up a new stack in a known-safe high memory area
                        ; (e.g., just under 1MB, ensure it doesn't clash)
                        ; LBL_STACK_TOP_ADDRESS from config

    ; At this point, we are in 32-bit PM. BIOS calls are generally unusable.
    ; We need to pass information to the Core Engine.
    ; This info could include: memory map, framebuffer details (if VBE used), boot drive.
    ; Store it in a known memory location or pass via registers if core ABI defines it.

    ; For LBL, we define a BootInfo struct. Stage2 must populate this.
    ; LBL_BOOT_INFO_ADDRESS is defined in lbl_config_bios.inc.
    mov edi, [LBL_BOOT_INFO_ADDRESS] ; EDI points to BootInfo struct

    ; Populate BootInfo (Example fields, actual struct must match core's expectation)
    mov dword [edi + 0], LBL_BOOT_INFO_MAGIC_VALUE ; magic
    mov dword [edi + 4], 1                    ; version
    ; [edi + 8] is memory_map_addr
    ; [edi + 12] is memory_map_entries
    ; ... (framebuffer info if VBE was used, etc.)
    ; For this stub, we fill minimal info. Full memory map and framebuffer
    ; detection are complex and omitted here for Stage2 BIOS, assuming Core will handle some.
    ; If MBR found framebuffer info or stage2 did, pass it.
    ; For now, core might need to rely on UEFI Stage1 for full info, or do its own detection.

    ; For BIOS, we pass very basic info, or core must do more detection.
    ; Let's assume Framebuffer info is detected via VBE (not shown here)
    ; and stored starting at known offsets in BootInfo struct in lbl_config_bios.inc
    ; mov eax, [detected_fb_addr]
    ; mov [edi + LBL_BOOTINFO_FB_ADDR_OFFSET], eax 
    ; ... etc for width, height, pitch, bpp

    mov byte [edi + LBL_BOOTINFO_BOOT_DRIVE_OFFSET], [boot_drive_s2_pm] ; Boot drive

    ; Jump to LBL Core Engine entry point
    ; LBL_CORE_ENTRY_POINT is the physical address from lbl_config_bios.inc
    ; This address is where lbl_core.bin was loaded and its _start symbol resides.
    mov esi, msg_jumping_to_core_32
    call print_string_32_pm ; Print using a PM-compatible method (e.g., direct framebuffer write)

    ; Pass BootInfo struct pointer to core engine via a register (e.g., EBX or EAX)
    mov ebx, edi ; EBX = pointer to BootInfo struct

    jmp dword [LBL_CORE_ENTRY_POINT_ABS] ; Absolute jump to core entry point (physical address)

    ; Should not return from core. If it does, hang.
    cli
    hlt
    jmp $


; --- Helper Functions (16-bit mode) ---
BITS 16
print_string_16:
    pusha
    mov ah, 0x0E
    mov bh, 0
    mov bl, 0x07
.loop_16:
    lodsb
    or al, al
    jz .done_16
    int 0x10
    jmp .loop_16
.done_16:
    popa
    ret

enable_a20_fastgate:
    ; Attempt to enable A20 via port 0x92 (fast A20 gate)
    pushf
    cli
    in al, 0x92         ; Read current value of port 0x92
    test al, 0x02       ; Check if A20 is already enabled (bit 1)
    jnz .a20_already_enabled ; If not zero, A20 is already enabled (or bit means something else)
    
    or al, 0x02         ; Set bit 1 to enable A20
    and al, 0xFE        ; Ensure bit 0 (fast reset) is not set accidentally
    out 0x92, al        ; Write new value to port 0x92

    ; Wait for a short time
    mov cx, 0xFF
.wait_loop1:
    loop .wait_loop1
    
    ; Verify A20 (simple check, not foolproof)
    in al, 0x92
    test al, 0x02
    jz .a20_failed      ; If still zero, A20 enabling failed via this method

.a20_already_enabled:
    popf
    clc                 ; Clear carry flag (success)
    ret

.a20_failed:
    ; Try keyboard controller method (more complex, omitted for this stub)
    ; For now, if fast gate fails, signal error.
    popf
    stc                 ; Set carry flag (failure)
    ret

; --- Helper Functions (32-bit Protected Mode) ---
BITS 32
; Simple PM string print (writes to VGA buffer 0xB8000 directly)
; Assumes standard text mode 80x25, white on black.
VGA_BUFFER equ 0xB8000
current_vga_pos dw 0 ; (row * 80 + col) * 2

print_string_32_pm:
    pushad
    mov edi, VGA_BUFFER
    add edi, [current_vga_pos_pm] ; Use a PM variable for position
.loop_32:
    mov al, [esi]       ; Get char from string (ESI points to string)
    inc esi
    or al, al           ; Null terminator?
    jz .done_32

    cmp al, 0x0D        ; Carriage return?
    je .handle_cr_32
    cmp al, 0x0A        ; Line feed?
    je .handle_lf_32

    mov ah, 0x07        ; Attribute: white on black
    mov [edi], ax       ; Write char + attr
    add edi, 2          ; Next screen character position
    jmp .loop_32

.handle_cr_32:          ; Move to beginning of current ESI line
    mov ebx, edi        ; Current linear address in VGA buffer
    sub ebx, VGA_BUFFER ; Offset from VGA_BUFFER start
    mov edx, 160        ; Bytes per line (80 chars * 2 bytes/char)
    xor eax, eax
    div edx             ; EAX = current line number (implicit), EDX = offset into line
    sub edi, edx        ; EDI points to start of current line
    jmp .loop_32

.handle_lf_32:          ; Move to next line, same column
    ; (Simplified: just move to start of next line for now)
    mov ebx, edi
    sub ebx, VGA_BUFFER
    mov edx, 160
    xor eax, eax
    div edx             ; EAX = current line number
    inc eax             ; Next line
    cmp eax, 25         ; Scrolled off screen?
    jl .no_scroll_32
    ; TODO: Basic scroll not implemented, just reset to top for simplicity
    mov edi, VGA_BUFFER
    mov eax, 0
    jmp .no_scroll_32
.no_scroll_32:
    mul edx             ; EAX = new line start offset in bytes
    mov edi, VGA_BUFFER
    add edi, eax        ; EDI points to start of next line
    jmp .loop_32

.done_32:
    mov ebx, edi
    sub ebx, VGA_BUFFER
    mov [current_vga_pos_pm], ebx ; Save new position

    popad
    ret


; --- Data (16-bit segment) ---
BITS 16
msg_stage2_init:    db "LBL Stage2 BIOS init...", 0x0D, 0x0A, 0
msg_a20_ok:         db "A20 line enabled.", 0x0D, 0x0A, 0
msg_a20_fail:       db "A20 enable FAILED!", 0x0D, 0x0A, 0
msg_mem_detect:     db "Memory detection (stubbed)...", 0x0D, 0x0A, 0
msg_loading_core:   db "Loading LBL Core Engine...", 0x0D, 0x0A, 0
msg_core_loaded:    db "LBL Core loaded.", 0x0D, 0x0A, 0
msg_core_fail:      db "LBL Core load FAILED!", 0x0D, 0x0A, 0

; Pointer to GDTR base for patching, assumes gdt_descriptor is at fixed offset early in file
gdt_descriptor_dd_offset_val: equ gdt_descriptor + 2
gdt_descriptor_dd_offset_val_ptr: dd gdt_start ; This value is patched at runtime

; --- Data (32-bit segment, ensure accessible after segments change) ---
; If DS is set to cover all 4GB, these can be anywhere.
; Otherwise, put them in a segment Stage2 knows how to access.
; For simplicity, assume they are accessible.
BITS 32 ; Switch context for label definitions if compiler needs it, or manage segments
current_vga_pos_pm: dd 0 ; Store as dword for 32-bit access
boot_drive_s2_pm:   db 0 ; Copy of boot_drive_s2 for PM access
                    times 3 db 0 ; Padding for alignment or future use
msg_jumping_to_core_32: db "Jumping to LBL Core (32-bit PM)...", 0x0D, 0x0A, 0

; End of boot_32.bin. Its size must be known for MBR to load it.
; The Makefile will typically determine size and provide to MBR config.