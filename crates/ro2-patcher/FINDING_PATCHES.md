# RO2 Client Patcher - Finding Patch Locations

## Status
✅ Patcher crate created and compiles  
✅ Original client backed up (`Rag2.exe.bak`)  
✅ Original HSHIELD restored  
⏳ **NEED: Actual patch locations from Ghidra**

## Client Information
- **File**: `Rag2.exe`
- **Size**: 18,884,864 bytes (19 MB)
- **SHA-256**: `5f6e211535d4b541b8c26c921a5fc8a968db151d9bef4a9df1f9982cf9e2e099`
- **Backup**: `Rag2.exe.bak` (created)

## What We Need to Find in Ghidra

### Critical Function: SendReqLogin
- **Address**: 0x00E52FE0
- **Purpose**: Builds and sends the 0x2EE2 login packet
- **Problem**: This function is NEVER CALLED when HackShield blocks

### Strategy Options

#### Option 1: Find the HackShield Check (Recommended)
We need to find where the client checks if HackShield is initialized before allowing login.

**What to look for:**
1. Open Ghidra with `Rag2.exe`
2. Search for calls to HackShield DLLs (`EHSvc.dll`)
3. Find conditional jumps that skip `SendReqLogin` 
4. Look for functions that check HackShield status

**Example pattern:**
```asm
call    CheckHackShieldInit    ; Returns 0 if failed, 1 if ok
test    eax, eax
jz      skip_login             ; Jump if zero (HackShield failed)
call    SendReqLogin           ; Only called if HackShield ok
skip_login:
```

**Patch:** Change `jz` (74 XX) to `jmp` (EB XX) to always call SendReqLogin

#### Option 2: Find LoadLibrary Calls
Find where client loads `EHSvc.dll` and patch it to skip:

```asm
push    offset aEhsvcDll       ; "EHSvc.dll"
call    LoadLibraryA
test    eax, eax
jz      error_handler
```

**Patch:** NOP out the LoadLibrary call or force success

#### Option 3: Patch SendReqLogin Caller
Find the UI button handler that calls SendReqLogin and remove the HackShield check.

## Ghidra Search Commands

### Search for String References
```
Search → For Strings → "EHSvc" or "HackShield" or "HSHIELD"
```

### Search for Function Calls
```
Search → For Scalars → 0x00E52FE0 (SendReqLogin address)
```

### Find Conditional Jumps
Look for these opcodes near SendReqLogin callers:
- `74 XX` - `jz` (jump if zero)
- `75 XX` - `jnz` (jump if not zero)
- `84 XX XX XX XX` - `jz` (far jump)

## How to Extract Patch Data

Once you find the location:

1. **Note the file offset** (not virtual address!)
   - In Ghidra: Right-click address → "Copy Special" → "File Offset"
   
2. **Copy the original bytes** (before patching)
   - Select the instruction → Right-click → "Copy Special" → "Byte String"
   
3. **Determine the patch**
   - Usually change `jz` (74) to `jmp` (EB)
   - Or NOP out instructions (90 90 90...)

4. **Update the patcher**
```rust
Patch {
    name: "bypass_hackshield_check",
    description: "Bypasses HackShield initialization check before login",
    offset: 0xABCDEF,  // File offset from Ghidra
    original: &[0x74, 0x10],  // jz +0x10
    patched: &[0xEB, 0x10],   // jmp +0x10
},
```

## Alternative: Simple Approach

If we can't find the exact check, we can try a **brute force search** for common patterns:

### Search for: LoadLibraryA("EHSvc.dll")
```bash
# Search for the string "EHSvc.dll" in the binary
strings -t x Rag2.exe | grep -i "ehsvc"
```

### Search for: Test patterns before SendReqLogin (0x00E52FE0)
```bash
# Find references to SendReqLogin address
xxd Rag2.exe | grep "e0 2f e5 00"  # Little-endian: 00E52FE0
```

## Next Steps

1. **Open Ghidra** and load Rag2.exe
2. **Navigate to SendReqLogin** (0x00E52FE0)
3. **Find cross-references** (CTRL+SHIFT+F → Find references to)
4. **Analyze the caller** - look for HackShield checks
5. **Extract patch data** using the method above
6. **Update `ro2-patcher/src/main.rs`** with actual patch offsets
7. **Test the patch**!

## Testing the Patcher

Once we have real patch data:

```bash
# List patches
cargo run -p ro2-patcher -- list

# Apply patches
cargo run -p ro2-patcher -- patch "/path/to/Rag2.exe"

# Verify
cargo run -p ro2-patcher -- verify "/path/to/Rag2.exe"

# Restore if needed
cargo run -p ro2-patcher -- restore "/path/to/Rag2.exe"
```

## Safety Notes

- ✅ Original exe backed up to `Rag2.exe.bak`
- ✅ Patcher verifies checksums
- ✅ Can restore backup anytime
- ✅ Patcher checks original bytes before patching

---

**Current blocker**: We need to analyze Rag2.exe in Ghidra to find the actual patch locations.

The patcher framework is ready - we just need the addresses!
