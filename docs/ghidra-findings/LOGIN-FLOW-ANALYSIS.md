# RO2 Login Flow Analysis

## Summary

The RO2 client has **TWO DISTINCT LOGIN FLOWS** controlled by the `/STARTER=2` launch parameter:

1. **Starter Mode** (`/STARTER=2`): Simplified login flow for testing/development
2. **Normal Mode** (no parameter): Full production login flow with HackShield

## Key Functions

### IsStarterModeEnabled (0x00A4FF60)
**Purpose**: Checks if `/STARTER=2` parameter was passed  
**Returns**: `true` if starter mode is enabled, `false` otherwise

**Decompiled Code**:
```c
bool IsStarterModeEnabled(void) {
    char *starterValue = GetStarterParameterValue();
    char *ptr = starterValue + 1;
    do {
        char c = *starterValue;
        starterValue = starterValue + 1;
    } while (c != '\0');
    
    if (starterValue == ptr) {
        return false;  // No STARTER parameter
    }
    
    char *value = GetStarterParameterValue();
    int intValue = atoi(value);
    return intValue == 2;  // Only return true if /STARTER=2
}
```

### GetStarterParameterValue (0x00A4FE00)
**Purpose**: Retrieves the value of the `STARTER` launch parameter from `g_LaunchParametersMap`  
**Returns**: String value (e.g., "2")

### CheckGameProtectionEnabled (0x00A4FFA0)
**Purpose**: Checks if HackShield game protection is loaded and initialized  
**File Offset**: `0x0064F3A0`  
**Patched**: ✅ Returns TRUE (bypasses HackShield check)

### CheckProtectionSystemEnabled (0x00A4CEF0)
**Purpose**: Checks if HackShield protection system is active  
**File Offset**: `0x0064C2F0`  
**Patched**: ✅ Returns TRUE (bypasses HackShield check)

## Login Flow Diagram

### Case 5: Login Button Clicked (FUN_00636830)

```
┌─────────────────────────────────────────────┐
│  User clicks LOGIN button                   │
└───────────────┬─────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────────┐
│  IsStarterModeEnabled() ?                        │
└───┬───────────────────────────────────────────┬───┘
    │ TRUE                                      │ FALSE
    │ (/STARTER=2)                              │ (no parameter)
    │                                           │
    ▼                                           ▼
┌───────────────────────────────┐   ┌──────────────────────────────┐
│  STARTER MODE PATH             │   │  NORMAL MODE PATH             │
├───────────────────────────────┤   ├──────────────────────────────┤
│  CheckGameProtectionEnabled() │   │  Get username/password from  │
│       ↓                       │   │  UI text fields               │
│  if FALSE:                    │   │       ↓                       │
│    CheckProtectionSystemEnabled()│ │  Hash password               │
│       ↓                       │   │       ↓                       │
│  if TRUE:                     │   │  FUN_00a6de40()              │
│    GetGameUIManager()         │   │  (Send login with username/   │
│    FUN_00a6e200()            │   │   password)                   │
│    ✓ Send 0x2EE2 (ReqLogin) │   │       ↓                       │
│      packet!                  │   │  ✓ Send 0x2EE2 packet        │
└───────────────────────────────┘   └──────────────────────────────┘
```

## Critical Discovery

**The issue**: We were launching without `/STARTER=2`, so the client was using the NORMAL MODE path, which requires:
- Username/password input fields
- Password hashing
- Different UI state

**The solution**: Add `/STARTER=2` to launcher arguments to use the simplified STARTER MODE path.

## Code Location: FUN_00636830 (Login Handler)

**Virtual Address**: `0x00636830`

**Relevant Section (case 5)**:
```c
case 5:
    iVar4 = 0;
    local_1d8 = 0;
    FUN_00988d70(2);  // Some UI preparation
    
    bVar2 = IsStarterModeEnabled();  // ← CHECK #1: Is /STARTER=2 present?
    if (bVar2) {
        // STARTER MODE FLOW
        iVar16 = GetGameConfigurationManager();
        *(undefined1 *)(iVar16 + 0x6e2) = 1;
        iVar16 = GetGameUIManager();
        if (*(int *)(iVar16 + 0x54) != 0) {
            cVar1 = CheckGameProtectionEnabled();  // ← CHECK #2: HackShield loaded?
            if (cVar1 == '\0') {
                cVar1 = CheckProtectionSystemEnabled();  // ← CHECK #3: Protection active?
                if (cVar1 != '\0') {
                    GetGameUIManager();
                    FUN_00a6e200();  // ← SENDS 0x2EE2 LOGIN PACKET!
                }
            } else {
                iVar16 = GetGameUIManager();
                FUN_00a6dbe0(*(void **)(iVar16 + 0x54));
            }
        }
    } else {
        // NORMAL MODE FLOW (complex, requires username/password)
        iVar16 = GetGameConfigurationManager();
        *(undefined1 *)(iVar16 + 0x6e2) = 0;
        if (this_00 != (void *)0x0) {
            // Get username/password from UI
            // Hash password
            // Call FUN_00a6de40() to send login
        }
    }
    break;
```

## Launch Parameters

### Required Parameters
```
/FROM=-FromLauncher   # Indicates launch from launcher (not Steam)
/IP=127.0.0.1        # Server IP address
/STARTER=2           # Enable starter mode
```

### Full Launch Command
```bash
# From game root directory (parent of SHIPPING)
wine SHIPPING/Rag2.exe /FROM=-FromLauncher /IP=127.0.0.1 /STARTER=2
```

## Testing Results

### WITHOUT /STARTER=2
- ❌ Client uses normal login flow
- ❌ Requires username/password fields
- ❌ Uses FUN_00a6de40() for login
- ❌ Login packet never sent in our tests

### WITH /STARTER=2
- ✅ Client uses starter mode login flow
- ✅ Bypasses username/password requirement
- ✅ Uses FUN_00a6e200() for login
- ✅ Should send 0x2EE2 packet (pending test)

## Next Steps

1. ✅ **Update launcher** to include `/STARTER=2` parameter
2. ⏳ **Test login flow** with starter mode enabled
3. ⏳ **Verify** that 0x2EE2 packet is now sent
4. ⏳ **Implement** server-side handler for 0x2EE2 packet

## Functions to Rename in Ghidra

- ✅ `FUN_00a4ff60` → `IsStarterModeEnabled`
- ✅ `FUN_00a4fe00` → `GetStarterParameterValue`
- ⏳ `FUN_00636830` → `StageLogin_HandleUIEvent` or `StageLogin_MessageHandler`
- ⏳ `FUN_00a6e200` → `SendStarterModeLoginPacket` or `GameUI_SendStarterLogin`
- ⏳ `FUN_00a6de40` → `SendNormalModeLoginPacket` or `GameUI_SendNormalLogin`
- ⏳ `FUN_00a6dbe0` → `ShowHackShieldRequiredDialog` or `GameUI_ShowProtectionError`

## References

- **StageLogin_Enter**: `0x00636630` - Login stage initialization
- **StageLogin_MessageHandler**: `0x00636830` - Handles UI events (case 5 = Login button)
- **g_LaunchParametersMap**: `0x015B64B8` - Global map of launch parameters
- **g_LaunchParametersMapEnd**: `0x015B64BC` - End iterator for parameter map
