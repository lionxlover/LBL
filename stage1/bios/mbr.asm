; Lionbootloader - Stage 1 - MBR (Master Boot Record)
; File: stage1/bios/mbr.asm
; Purpose: Loads the second stage bootloader (boot_32.bin) from disk.

BITS 16             ; We are in 16-bit real mode
ORG 0x7C00          ; BIOS loads MBR to this address

%include "stage1/bios/lbl_config_bios.inc" ; Build-time configurations

ENTRY_POINT:
    cli             ; Disable interrupts
    hlt             ; Halt briefly - some BIOSes need this after CLI before segment changes.
    
    ; Setup segments and stack
    xor ax, ax      ; AX = 0
    mov ds, ax      ; Data Segment = 0
    mov es, ax      //xtra Segment = 0
    mov ss, ax      ; Stack Segment = 0
    mov sp, 0x7C00  ; Stack pointer grows downwards from 0x7C00 (below MBR code)
                    ; Be careful not to overwrite MBR itself if stack usage is high.
                    ; A common practice is to set SP below the MBR, e.g., 0x7000 if MBR is small
                    ; Or setup SS:SP to a known safe area like 0000:9000h after some memory sizing.
                    ; For this minimal MBR, SP at 0x7C00 is fine if we don't PUSH much.

    sti             ; Re-enable interrupts

    ; --- Print a startup message (Optional) ---
    mov si, msg_loading
    call print_string_bios

    ; --- Load Stage 2 (boot_32.bin) ---
    ; LBL_STAGE2_LOAD_ADDRESS is defined in lbl_config_bios.inc (e.g., 0x0000:0x8000 or other safe address)
    ; LBL_STAGE2_START_SECTOR is the LBA sector number where boot_32.bin starts (e.g., sector 1 if MBR is sector 0)
    ; LBL_STAGE2_SECTOR_COUNT is how many sectors boot_32.bin occupies.

    mov ax, word [LBL_STAGE2_LOAD_SEGMENT] ; Target segment for loading Stage 2
    mov es, ax                          ; ES:BX will be the destination buffer address
    mov bx, word [LBL_STAGE2_LOAD_OFFSET] ; Target offset

    mov ah, 0x02        ; BIOS Read Sectors function
    mov al, byte [LBL_STAGE2_SECTOR_COUNT] ; Number of sectors to read
    mov ch, byte [LBL_STAGE2_TRACK_NUMBER]   ; Cylinder/Track number (for CHS addressing) - LBA is preferred.
    mov cl, byte [LBL_STAGE2_START_SECTOR_CHS] ; Sector number (for CHS, usually starts at 1)
    mov dh, byte [LBL_STAGE2_HEAD_NUMBER]    ; Head number
    mov dl, byte [boot_drive] ; Drive number (usually 0x80 for first HDD, passed by BIOS in DL)
                              ; We should save BIOS-provided DL

    ; Check if INT 13h extensions are available (for LBA addressing)
    push ds
    mov ah, 0x41        ; INT 13h, AH=41h - Check Extensions Present
    mov bx, 0x55AA      ; Required magic value for BX
    mov dl, byte [boot_drive] ; Drive number
    int 0x13            ; Call BIOS disk service
    jc .no_lba_extensions ; Jump if Carry Flag is set (error or extensions not present)
    cmp bx, 0xAA55      ; Check magic value in BX
    jne .no_lba_extensions ; If not AA55, extensions not present
    test cl, 0x01       ; Check bit 0 of CL (1 if fixed disk subset supported)
    jz .no_lba_extensions  ; If not set, LBA might not be fully supported as expected.
    pop ds
    jmp .use_lba        ; Extensions are present, use LBA addressing

.no_lba_extensions:
    pop ds
    ; Fallback to CHS addressing (already set up above) or error
    mov si, msg_chs_mode
    call print_string_bios
    int 0x13            ; Call BIOS Read Sectors (CHS)
    jmp .read_done

.use_lba:
    ; LBA Packet structure for INT 13h, AH=42h
    dap:
        db 0x10         ; Size of packet (16 bytes)
        db 0            ; Reserved, should be zero
    dap_num_sectors:
        dw word [LBL_STAGE2_SECTOR_COUNT_WORD] ; Number of sectors to read (word)
    dap_buffer_offset:
        dw word [LBL_STAGE2_LOAD_OFFSET] ; Transfer buffer offset
    dap_buffer_segment:
        dw word [LBL_STAGE2_LOAD_SEGMENT]; Transfer buffer segment
    dap_lba_low:
        dd dword [LBL_STAGE2_START_SECTOR_LBA_LOW] ; Lower 32 bits of LBA
    dap_lba_high:
        dd 0            ; Upper 32 bits of LBA (usually 0 for drives < 2TB)

    mov si, msg_lba_mode
    call print_string_bios

    mov ah, 0x42        ; INT 13h, AH=42h - Extended Read Sectors
    mov dl, byte [boot_drive] ; Drive number
    mov si, dap         ; DS:SI points to Disk Address Packet (DAP)
    int 0x13            ; Call BIOS disk service

.read_done:
    jc .disk_error      ; Jump if Carry Flag is set (disk error)

    ; --- Loading successful, jump to Stage 2 ---
    mov si, msg_stage2_loaded
    call print_string_bios

    ; The jump address is ES:BX if Stage 2 was loaded there by CHS,
    ; or dap_buffer_segment:dap_buffer_offset if loaded by LBA.
    ; These should be the same values defined by LBL_STAGE2_LOAD_SEGMENT/OFFSET.
    ; We can make a far jump to LBL_STAGE2_LOAD_SEGMENT:LBL_STAGE2_LOAD_OFFSET.
    ; Example: jmp 0x0800:0x0000 if LBL_STAGE2_LOAD_ADDRESS is 0x8000.
    ; The actual values come from lbl_config_bios.inc
    db 0xEA ; Far JMP opcode
    LOAD_ADDRESS_STAGE2_OFFSET: dw word [LBL_STAGE2_LOAD_OFFSET]
    LOAD_ADDRESS_STAGE2_SEGMENT: dw word [LBL_STAGE2_LOAD_SEGMENT]

.disk_error:
    mov si, msg_disk_error
    call print_string_bios
    ; Loop forever on error
    cli
    hlt
    jmp .disk_error


; --- Function to print a null-terminated string using BIOS Teletype output ---
; Input: SI = pointer to string
print_string_bios:
    mov ah, 0x0E        ; BIOS Teletype output function
    mov bh, 0           ; Page number
    mov bl, 0x07        ; White text on black background (for text mode)
.char_loop:
    lodsb               ; Load byte from [SI] into AL, increment SI
    or al, al           ; Check if AL is zero (null terminator)
    jz .done_printing
    int 0x10            ; Call BIOS video service
    jmp .char_loop
.done_printing:
    ret


; --- Data ---
boot_drive:
    db 0                ; Will be filled with BIOS boot drive number (DL) at runtime.
                        ; MBR.asm setup should save DL to [boot_drive] if it's not 0x7c00 based.
                        ; For ORG 0x7C00, if BIOS sets DL, it's available.
                        ; Better: `push dx` early, `pop dx` when needed for `mov [boot_drive], dl`.
                        ; Simplest: assume BIOS has DL set correctly when INT 13h is called.

msg_loading:
    db "LBL Stage1 (MBR) loading Stage2...", 0x0D, 0x0A, 0
msg_chs_mode:
    db "Using CHS mode.", 0x0D, 0x0A, 0
msg_lba_mode:
    db "Using LBA mode.", 0x0D, 0x0A, 0
msg_stage2_loaded:
    db "Stage2 loaded. Jumping.", 0x0D, 0x0A, 0
msg_disk_error:
    db "Disk read error!", 0x0D, 0x0A, 0


; --- Padding and Boot Signature ---
times 510 - ($-$$) db 0   ; Pad MBR to 510 bytes. $ is current address, $$ is start of section.
dw 0xAA55                 ; Boot signature (must be at bytes 510-511)