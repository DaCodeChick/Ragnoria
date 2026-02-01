# Session Summary - January 31, 2026

## Overview

Deep dive into RO2 client login flow through Ghidra analysis. Discovered that the client has two distinct login modes (STARTER and NORMAL) and properly identified and renamed critical functions following the DICK protocol.

## Key Discoveries

### 1. Two Login Modes

The RO2 client supports two different login flows:

**STARTER MODE** (`/STARTER=2` parameter):
- Simplified login flow for development/testing
- Hides username/password UI fields
- Uses auto-login with `/CODE=` parameter
- Sends login via `GameUI_AutoLoginStarter()` or `GameUI_SendStarterModeLogin()`

**NORMAL MODE** (default):
- Full production login flow
- Shows username/password fields
- User manually enters credentials and clicks Login
- Sends login via `GameUI_SendNormalModeLogin()`

### 2. Function Naming (DICK Protocol Applied)

Successfully renamed 7 critical functions in Ghidra:

| Original | New Name | Address | Purpose |
|----------|----------|---------|---------|
| `FUN_00a4ff60` | `IsStarterModeEnabled` | 0x00A4FF60 | Checks if `/STARTER=2` parameter exists |
| `FUN_00a4fe00` | `GetStarterParameterValue` | 0x00A4FE00 | Gets STARTER value from launch params |
| `FUN_00636830` | `StageLogin_HandleUIEvent` | 0x00636830 | Main UI event handler (switch with 70+ cases) |
| `FUN_00a6e200` | `GameUI_SendStarterModeLogin` | 0x00A6E200 | Sends login packet in STARTER mode |
| `FUN_00a6de40` | `GameUI_SendNormalModeLogin` | 0x00A6DE40 | Sends login packet in NORMAL mode |
| `FUN_00a6dbe0` | `GameUI_ShowHackShieldError` | 0x00A6DBE0 | Shows error when HackShield check fails |
| `FUN_00a6d2c0` | `GameUI_AutoLoginStarter` | 0x00A6D2C0 | Auto-login handler for STARTER mode |

### 3. Login Flow Analysis

**Case 0: STARTER Mode Auto-Login**
```c
case 0:
    if (IsStarterModeEnabled()) {
        // Get account code from parameters
        // Convert to wide string
        // Call GameUI_AutoLoginStarter()
    }
```

**Case 5: Manual Login Button Click**
```c
case 5:
    if (IsStarterModeEnabled()) {
        // STARTER path
        if (CheckGameProtectionEnabled() == FALSE) {
            if (CheckProtectionSystemEnabled() == TRUE) {
                GameUI_SendStarterModeLogin();
            }
        } else {
            GameUI_ShowHackShieldError();
        }
    } else {
        // NORMAL path
        // Get username/password from UI
        // Hash password
        // Call GameUI_SendNormalModeLogin()
    }
```

### 4. Protection System Architecture

**Two-Layer Protection in WinMain:**
1. `CheckGameProtectionEnabled()` → `InitializeGameProtection()`
2. `CheckProtectionSystemEnabled()` → `InitializeProtectionSystem()`

**Protection Checks in Login Flow:**
- Only active in STARTER mode
- NORMAL mode bypasses HackShield checks
- Client can connect without HackShield running

### 5. Testing Results

**Unpatched Client (No STARTER mode):**
- ✅ Connects to server successfully
- ✅ Completes ProudNet handshake
- ✅ Sends keep-alive and heartbeat packets
- ✅ Does NOT crash from missing HackShield
- ❌ Login packet not sent (need to test UI interaction)

**Patched Client with STARTER mode:**
- ✅ Connects to server
- ✅ Completes handshake
- ❌ Hides login UI (by design)
- ❌ Login packet not sent automatically

## Recommended Approach

### For Development/Testing
Use **NORMAL MODE** (no patches required):
1. Launch client: `wine Rag2.exe /FROM=-FromLauncher /IP=127.0.0.1`
2. Client shows login UI with username/password fields
3. User enters credentials and clicks Login
4. Case 5 triggers → `GameUI_SendNormalModeLogin()` called
5. Server receives 0x2EE2 packet with credentials

### Launch Parameters
```bash
# Correct (NORMAL mode)
wine SHIPPING/Rag2.exe /FROM=-FromLauncher /IP=127.0.0.1

# STARTER mode (for reference)
wine SHIPPING/Rag2.exe /FROM=-FromLauncher /IP=127.0.0.1 /STARTER=2 /CODE=testuser
```

## Patches Status

### Current Patches (Modified for NORMAL mode support)
```rust
// Patch 1: CheckGameProtectionEnabled → Returns FALSE
// File Offset: 0x0064F3A0
// Patched: MOV AL, 0; RET

// Patch 2: CheckProtectionSystemEnabled → Returns TRUE  
// File Offset: 0x0064C2F0
// Patched: MOV AL, 1; RET
```

**Note:** These patches were originally designed for STARTER mode. For NORMAL mode, **no patches are required** - the client works fine without HackShield.

## Files Modified

### Code
- `crates/launcher/src/main.rs` - Removed `/STARTER=2` and `/CODE=` parameters
- `crates/ro2-patcher/src/main.rs` - Updated patch comments and logic

### Documentation
- `docs/ghidra-findings/LOGIN-FLOW-ANALYSIS.md` - Complete login flow analysis
- `.opencode/AGENTS.md` - DICK protocol enforcement for function naming

## Next Steps

1. **Test NORMAL mode login UI**
   - Launch unpatched client
   - Verify login screen appears
   - Enter test credentials
   - Monitor for 0x2EE2 packet

2. **Implement server-side login handler**
   - Parse 0x2EE2 packet structure
   - Extract username/password
   - Validate credentials
   - Send appropriate response

3. **Handle post-login flow**
   - Character list request
   - Character selection
   - World server handoff

## Lessons Learned

### DICK Protocol Importance
- Unnamed functions create confusion and wasted effort
- Proper naming reveals purpose and relationships
- Systematic naming enables better analysis
- Created `.opencode/AGENTS.md` to enforce this practice

### Over-Engineering Pitfall
- Initially assumed patches were necessary
- STARTER mode was a red herring that hid the UI
- Simplest solution: use unpatched client in NORMAL mode
- Always test the simplest approach first

### Wine/HackShield Interaction
- Wine environment doesn't fully emulate Windows security APIs
- HackShield checks fail gracefully without crashing
- Client can run without anti-cheat in Wine
- This is convenient for development but not a production solution

## Technical Deep Dive

### StageLogin_HandleUIEvent Structure
- 70+ case statements handling different UI events
- Cases 0-0x4F mapped to various error conditions and states
- Case 0: Auto-login (STARTER mode)
- Case 5: Manual login button click
- Each case shows localized error messages or triggers actions

### Parameter Parsing
- Launch parameters stored in `g_LaunchParametersMap` (0x015B64B8)
- Functions query this map by key (e.g., "STARTER", "CODE", "IP")
- `GetStarterParameterValue()` looks up "STARTER" and returns string value
- `IsStarterModeEnabled()` checks if value equals "2"

### UI Element Management
```c
// Hide UI elements in STARTER mode
if (!IsStarterModeEnabled()) {
    FUN_00434c50(DAT_015aed30, 2, 1);  // Show UI element 2
    FUN_00434c50(DAT_015aed30, 4, 1);  // Show UI element 4
}
```

## References

- **Ghidra Project:** `/home/admin/Documents/null.gpr`
- **Client Binary:** `/run/media/admin/FE6407F46407AE89/Gravity/Ragnarok Online 2 - Jawaii/SHIPPING/Rag2.exe`
- **SHA-256:** `5f6e211535d4b541b8c26c921a5fc8a968db151d9bef4a9df1f9982cf9e2e099`
- **Server Code:** `crates/ro2-login/`
- **Patcher Code:** `crates/ro2-patcher/`

## Conclusion

The RO2 client's dual login modes were the source of confusion. STARTER mode is designed for automated testing and hides the login UI, while NORMAL mode provides the standard user experience. For custom server development, NORMAL mode is the correct choice as it doesn't require HackShield patches and provides full control over the authentication flow.

The key takeaway: **Always DICK your functions** - proper naming through the Document, Identify, Categorize, Know protocol saves significant debugging time and reveals the actual architecture of the system.
