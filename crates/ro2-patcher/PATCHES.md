# RO2 Client Patches

This document describes the binary patches applied to `Rag2.exe` to bypass HackShield anti-cheat protection.

## Target Executable

- **File**: Rag2.exe (RO2 Jawaii SHIPPING build)
- **Size**: 18,884,864 bytes
- **SHA-256**: `5f6e211535d4b541b8c26c921a5fc8a968db151d9bef4a9df1f9982cf9e2e099`

## Patches Applied

### Patch 1: bypass_game_protection_check

**Purpose**: Force `CheckGameProtectionEnabled()` to always return TRUE

**Location**:
- Virtual Address: `0x00A4FFA0`
- File Offset: `0x0064F3A0`

**Original Bytes** (16 bytes):
```
55 8B EC 6A FF 68 B8 2D 2D 01 64 A1 00 00 00 00
```

**Disassembly**:
```asm
PUSH    EBP                     ; 55
MOV     EBP, ESP                ; 8B EC
PUSH    -1                      ; 6A FF
PUSH    0x012D2DB8              ; 68 B8 2D 2D 01
MOV     EAX, dword ptr fs:[0]   ; 64 A1 00 00 00 00
```

**Patched Bytes** (16 bytes):
```
B0 01 C3 90 90 90 90 90 90 90 90 90 90 90 90 90
```

**Disassembly**:
```asm
MOV     AL, 1      ; B0 01  (set return value to TRUE)
RET                ; C3     (return immediately)
NOP                ; 90     (×13 - fill remaining bytes)
```

**Effect**: The function immediately returns TRUE (1 in AL register), making the client believe HackShield is enabled and initialized.

---

### Patch 2: bypass_protection_system_check

**Purpose**: Force `CheckProtectionSystemEnabled()` to always return TRUE

**Location**:
- Virtual Address: `0x00A4CEF0`
- File Offset: `0x0064C2F0`

**Original Bytes** (16 bytes):
```
55 8B EC 6A FF 68 58 25 2D 01 64 A1 00 00 00 00
```

**Disassembly**:
```asm
PUSH    EBP                     ; 55
MOV     EBP, ESP                ; 8B EC
PUSH    -1                      ; 6A FF
PUSH    0x012D2558              ; 68 58 25 2D 01
MOV     EAX, dword ptr fs:[0]   ; 64 A1 00 00 00 00
```

**Patched Bytes** (16 bytes):
```
B0 01 C3 90 90 90 90 90 90 90 90 90 90 90 90 90
```

**Disassembly**:
```asm
MOV     AL, 1      ; B0 01  (set return value to TRUE)
RET                ; C3     (return immediately)
NOP                ; 90     (×13 - fill remaining bytes)
```

**Effect**: The function immediately returns TRUE (1 in AL register), making the client believe the protection system is active.

---

## How It Works

### Background

The RO2 client uses **HackShield by AhnLab** for anti-cheat protection. When connecting to a server, the client performs these checks:

1. Verifies HackShield is loaded (`CheckGameProtectionEnabled`)
2. Verifies protection system is active (`CheckProtectionSystemEnabled`)
3. If either check fails, the Login button is disabled and login packets are never sent

### The Problem

Without HackShield DLLs installed and initialized, both functions return FALSE, which blocks the login process. The client enters an error handler that prevents `SendReqLogin` (packet 0x2EE2) from being called.

### The Solution

Instead of trying to implement HackShield stubs (which have complex dependencies), we patch the check functions themselves to always return TRUE. This tricks the client into thinking HackShield is properly initialized.

**Why this works**:
- Both functions return a boolean value in the AL register (x86 calling convention)
- `MOV AL, 1` sets the return value to TRUE
- `RET` immediately returns from the function
- The rest of the original function code is never executed
- The caller sees a successful check and allows login to proceed

### Code Flow (Patched)

```
User clicks Login button
    ↓
Login handler checks protection status
    ↓
Calls CheckGameProtectionEnabled() → Returns TRUE (patched)
    ↓
Calls CheckProtectionSystemEnabled() → Returns TRUE (patched)
    ↓
All checks pass! ✓
    ↓
SendReqLogin() is called
    ↓
Packet 0x2EE2 (ReqLogin) is sent to server
```

## Technical Notes

### PE File Offset Calculation

Virtual addresses in Ghidra don't directly translate to file offsets in PE files. The correct calculation requires:

1. Calculate RVA: `RVA = VirtualAddress - ImageBase`
2. Find which section contains the RVA
3. Calculate file offset: `FileOffset = (RVA - Section.VirtualAddress) + Section.PointerToRawData`

For this executable:
- Image Base: `0x00400000`
- Both functions are in the `.text` section (VA: `0x00001000`, Raw: `0x00000400`)

Example for first function:
```
VA: 0x00A4FFA0
RVA: 0x00A4FFA0 - 0x00400000 = 0x0064FFA0
Offset in .text: 0x0064FFA0 - 0x00001000 = 0x0064EFA0
File Offset: 0x0064EFA0 + 0x00000400 = 0x0064F3A0
```

### Why NOPs?

The patched code is only 3 bytes, but we replace 16 bytes of the original function prologue. The remaining 13 bytes are filled with `NOP` (0x90) instructions to:

1. Ensure the patch size matches the original size (for the patcher validation)
2. Prevent any potential issues if code somehow falls through
3. Make the patch obvious in a hex editor/disassembler

NOPs are harmless instructions that do nothing and take 1 CPU cycle.

### Safety

- Original executable is backed up automatically (`.exe.bak`)
- Patches can be reverted using `ro2-patcher restore`
- Checksum validation ensures correct executable version
- Only the function prologues are modified, no other code is touched

## References

### Ghidra Analysis

The target functions were identified in Ghidra through analysis of the login error handler:

- **Function**: `FUN_00636830` (login error handler, case 5)
- **Calls**: Both protection check functions
- **Behavior**: If checks fail, error state is set and login is blocked

### Related Code

- `SendReqLogin` @ `0x00E52FE0` - Builds and sends packet 0x2EE2
- `FUN_00636830` @ `0x00636830` - Login error handler with protection checks

## Testing

To verify the patches work:

1. Patch the client: `ro2-patcher patch Rag2.exe`
2. Launch the patched client
3. Click the Login button
4. Watch server logs for packet 0x2EE2 (ReqLogin)

**Success indicator**: Server receives `0x2EE2` packet after clicking Login button.

## Future Improvements

Potential additional patches that may be needed:

1. **Runtime HackShield checks** - The client may perform periodic checks during gameplay
2. **Memory integrity checks** - HackShield normally monitors memory for tampering
3. **Update checks** - Client may verify HackShield version/signature

These will only be needed if the client crashes or disconnects after login succeeds.
