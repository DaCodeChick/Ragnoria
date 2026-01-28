# WinMain Deep Dive - Ragnarok Online 2 Client Analysis

**Binary:** `Rag2.exe`  
**Entry Point:** `WinMain @ 0x00A502F0`  
**Analysis Date:** 2026-01-27  
**Methodology:** Top-down traversal from WinMain following call graph

---

## Executive Summary

The RO2 client follows a sophisticated initialization sequence with extensive logging, ProudNet networking middleware, and a message-driven game loop. This analysis traces the execution flow from `WinMain` through all critical subsystems to understand how the client connects to servers and processes game messages.

### Key Findings

1. **Logging System**: 14 separate log files for different subsystems
2. **ProudNet Integration**: 75+ ProudNet-specific functions identified
3. **Network Architecture**: Three-tier connection system (Login → Lobby → World)
4. **Game Loop**: Windows message pump with frame-rate management
5. **Error Handling**: Comprehensive localization and error reporting

---

## WinMain Initialization Sequence

### Phase 1: Logging System Setup

**Function:** `LoggingSystem_Initialize()`

The client initializes **14 different log files** for comprehensive debugging:

| Log ID | File Path | Purpose |
|--------|-----------|---------|
| 1 | `./Log/OSAPIError.log` | Windows API error tracking |
| 2 | `./Log/Game.log` | General game events |
| 3 | *(empty path)* | Reserved/disabled |
| 4 | *(empty path)* | Reserved/disabled |
| 5 | *(empty path)* | Reserved/disabled |
| 6 | *(empty path)* | Reserved/disabled |
| 7 | `./Log/BattleLog.tbl` | Combat encounter logs |
| 8 | `./Log/DPSLog.tbl` | Damage-per-second statistics |
| 9 | *(empty path)* | Reserved |
| 10 | *(empty path)* | Reserved |
| 11 | `./Log/ActorSystemLog.txt` | Actor/entity system |
| 12 | `./Log/SkillSystemLog.txt` | Skill execution logs |
| 13 | `./Log/FontLog.txt` | Font rendering logs |
| 14 | `./Log/CombatLog.txt` | General combat events |

#### BattleLog.tbl Structure

Header: `Time\tAttacker\tDefender\tAttack_Type\tCombat_Type\tColor_Tendency_Type\tDamage\tString_Text\tSkillName\r\n`

This is a **tab-delimited table** format, suggesting data analysis or replay capabilities.

#### DPSLog.tbl Structure

Header: `NPC_ID\tNPC_NAME\tNPC_LEVEL\tCOMBAT_TIME\tNORMAL_ATTACK_COUNT\tNORMAL_ATTACK_DAMAGE\tNORMAL_ATTACK_DPS\tSKILL_ATTACK_COUNT\tSKILL_ATTACK_DAMAGE\tSKILL_ATTACK_DPS\tDOT_ATTACK_COUNT\tDOT_ATTACK_DAMAGE\tDOT_ATTACK_DPS\tTOTAL_DAMAGE\tTOTAL_DPS\r\n`

**Analysis:** This provides detailed combat metrics broken down by:
- Normal attacks vs. skill attacks vs. damage-over-time
- Per-NPC statistics
- Total DPS calculations

### Phase 2: Application Metadata Loading

```c
LoadStringW(hInstance, 0x67, &local_a08, 0x200);  // Load app title
LoadStringW(hInstance, 0x6d, &local_608, 0x200);  // Load app class name

ApplicationMetadata* appMeta = GetApplicationMetadataInstance();
appMeta->title = local_a08;   // Offset +0x58
appMeta->className = local_608; // Offset +0x74
```

**Resource IDs:**
- `0x67` (103): Application window title
- `0x6d` (109): Window class name

### Phase 3: Command Line Processing

```c
// Convert command line from Unicode to ANSI
LoggingSystem_GetLogger()->GetCodePageConverter()->ConvertUnicodeToAnsi(
    lpCmdLine, 0x200, &local_208, 0x200
);

ParseGameLaunchArguments(&local_208);
```

**Critical Function:** `GetGameServerIPFromCommandLine()`  
Returns pointer to IP address string from command line arguments.

```c
char* serverIP = GetGameServerIPFromCommandLine();
if (strlen(serverIP) == 0) {
    // ERROR: No server IP specified
    PTR_FUN_015a5244(&DAT_013dfcec, "ERROR", 0);  // Error handler
} else {
    strcpy_s(g_GameServerIPAddress, 0x20, serverIP);
}
```

**Global Variable Identified:**
- `g_GameServerIPAddress @ 0x015c9088` (estimated based on context)

### Phase 4: Protection Systems

```c
if (CheckGameProtectionEnabled()) {
    InitializeGameProtection();
}

if (CheckProtectionSystemEnabled()) {
    InitializeProtectionSystem();
}
```

**Analysis:** Two-layer protection:
1. Game-specific anti-cheat
2. General protection system (possibly nProtect GameGuard or similar)

### Phase 5: Graphics & Subsystems

```c
InitializeGameConfiguration();

if (ValidateGameLaunchParameters()) {
    DWORD seed = timeGetTime();
    srand(seed);  // Initialize RNG with system time
    
    InitializeGraphicsEngine(hInstance);
    InitializeGameSubsystems();
    InitializeGameTimers();
    
    if (InitializeMainWindow(hInstance)) {
        // Main game loop setup
    }
}
```

#### InitializeGameSubsystems Details

**File:** `dict.lex` (dictionary/lexicon file)  
**Size:** `0x118` bytes header + variable data  

```c
void InitializeGameSubsystems(void) {
    HFILE hFile = _lopen("dict.lex", O_RDONLY);
    if (hFile != -1) {
        _hread(hFile, &DAT_015c3678, 0x118);  // Read header
        
        // Validate header
        if (DAT_015c3784 == DAT_015c3780 + DAT_015c377c * 2) {
            DAT_015c3790 = malloc(DAT_015c3788);  // Allocate dictionary buffer
            _hread(hFile, DAT_015c3790, DAT_015c3788);  // Read dictionary data
            _DAT_015c378c = 1;  // Mark as initialized
        }
        _lclose(hFile);
    }
}
```

**Global Variables:**
- `DAT_015c3678`: Dictionary header structure
- `DAT_015c3780`: Lexicon entry count
- `DAT_015c377c`: Entry size multiplier
- `DAT_015c3784`: Calculated size (validation)
- `DAT_015c3788`: Total data size
- `DAT_015c3790`: Dictionary data buffer (pointer)
- `_DAT_015c378c`: Initialization flag

### Phase 6: Game Timer Setup

```c
GameTimer* timers = GetGameTimersInstance();

// Timer 0: Frame rate timer
GameTimer_SetValue(timers[0], _FLOAT_DefaultFrameRate);  // ~60.0 FPS

// Timer 1: Update timer
GameTimer_SetValue(timers[1], _FLOAT_DefaultFrameRate);

// Timer 2: Fixed timestep
GameTimer_SetValue(timers[2], 1.0);
```

**Structure:** `GetGameTimersInstance()` returns array of `GameTimer*`

---

## Main Game Loop

```c
MSG msg;
msg.message = 0;
PeekMessageW(&msg, NULL, 0, 0, 0);  // Initialize message pump

while (msg.message != WM_QUIT) {  // 0x12 = WM_QUIT
    if (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE)) {
        // Special handling for input window
        if (!g_InputSystemEnabled && 
            g_InputWindow != NULL && 
            (msg.message >= WM_KEYFIRST && msg.message <= WM_KEYLAST)) {
            // Redirect keyboard input to dedicated input window
            SendMessageW(g_InputWindow, msg.message, msg.wParam, msg.lParam);
        }
        
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    } else {
        MainGameLoopUpdate();  // Core game logic
        Sleep(0);  // Yield to other threads
    }
}
```

**Global Variables:**
- `g_InputSystemEnabled @ 0x015a1c10` (boolean)
- `g_InputWindow @ 0x015b17a8` (HWND)

---

## ProudNet Network Architecture

### Discovered ProudNet Functions (75 total)

#### Core Functions

| Function | Address | Purpose |
|----------|---------|---------|
| `ProudNet_FatalError` | 0x00EFD530 | Fatal error handler |
| `InitializeProudNetHeader` | 0x00F0E420 | Initialize packet header |
| `FinalizeProudNetHeader` | 0x00F0E4A0 | Finalize packet for send |
| `SetProudNetReadPointer` | 0x00EFFFE0 | Set buffer read position |
| `ReadProudNetOpcode` | 0x00F01180 | Extract message opcode |

#### Encryption Functions

| Function | Address | Purpose |
|----------|---------|---------|
| `ProudNet_InitializeAESKey` | 0x00F36420 | Set up AES encryption key |
| `ProudNet_AES_EncryptBuffer` | 0x00F37720 | Encrypt buffer with AES |
| `ProudNet_DecryptWithAES` | 0x00F37CC0 | Decrypt buffer with AES |
| `ProudNet_EncryptPacket_Method1` | 0x00F37D30 | Packet encryption (method 1) |
| `ProudNet_CompressPacket` | 0x00F3B650 | Compress packet data |

#### Protocol Handlers

| Function | Address | Opcode | Purpose |
|----------|---------|--------|---------|
| `HandleProudNet_0x1F_ConnectTCP` | 0x00F3BC70 | 0x1F | TCP connection established |
| `HandleProudNet_0x20_DisconnectTCP` | 0x00F3BCA0 | 0x20 | TCP disconnection |
| `HandleProudNet_0x0B_VersionCheck` | 0x00F3C390 | 0x0B | Version validation |

**Analysis:** ProudNet uses **opcode-based message dispatch**. Opcodes in the `0x0B`, `0x1F`, `0x20` range are system-level ProudNet messages (not game-specific RMI calls).

### Game-Specific Network Functions

| Function | Address | Params | Purpose |
|----------|---------|--------|---------|
| `EstablishGameServerConnection` | 0x0044DD80 | 4 | Connect to game server |
| `ProcessGameServerConnectionHandler` | 0x006A6380 | 1 | Handle connection state |
| `HandleGameServerDisconnection` | 0x006A6960 | 1 | Clean up on disconnect |
| `CreateGameNetworkConnection` | 0x00994DA0 | 11 | Create network connection object |
| `Network_IsConnectionActive` | 0x00A3ECD0 | 0 | Check connection status |

---

## ProcessGameServerConnectionHandler Deep Dive

**Function:** `ProcessGameServerConnectionHandler @ 0x006A6380`  
**Signature:** `undefined4 __thiscall ProcessGameServerConnectionHandler(void* this)`

### Purpose

This is the **main connection state machine** that validates preconditions before establishing network connections. It checks:

1. **Game State**: Is the game in an active state?
2. **UI State**: Are UI dialogs blocking connection?
3. **Proximity**: Is player too close to other players? (proximity check)
4. **Inventory**: Does player have required items?

### Key Logic Flow

```c
if (!IsGameStateActive(g_GameState)) {
    return 1;  // Abort if game not active
}

// Check if connection dialog is open
if (this->dialogWindow != NULL) {
    wchar_t* dialogName = this->dialogWindow->name;
    
    if (dialogName != L"") {
        if (UI_IsDialogOpen(g_UIManager, dialogName)) {
            // Show error: dialog still open
            ShowLocalizedError(0x1DFC, 0x07D0);  // IDs from LocalizationManager
            return 1;
        }
        
        // Check player inventory/containers
        int containerCount1 = GameState_GetContainerCount1(g_GameState);
        int containerCount2 = GameState_GetContainerCount2(g_GameState);
        
        bool hasItems = (containerCount1 > 0) || (containerCount2 > 0);
        
        if (this->itemRequirement != 0) {
            longlong itemCount = GetItemCount(this->itemRequirement);
            if (itemCount > 0) {
                hasItems = true;
            }
        }
        
        // If no required items, show error
        if (!hasItems) {
            if (/* both flags check */) {
                ShowLocalizedError(0x22D2, 0x07D0);
            } else {
                ShowLocalizedError(0x2D5B, 0x07D0);
            }
            return 1;
        }
        
        // Proximity check - iterate through nearby players
        PlayerList* players = GetPlayerList();
        for (Player* player : players) {
            if (player->state == 2 && IsPlayerOnline(player)) {
                Player* localPlayer = GetLocalPlayer();
                
                // Calculate distance
                Vec3 localPos = localPlayer->GetPosition();
                Vec3 playerPos = player->GetPosition();
                Vec3 delta = playerPos - localPos;
                float distance = VectorLength(delta);
                
                // Check proximity threshold
                if (distance < g_ProximityDistance) {
                    // Too close! Show error
                    ShowLocalizedError(0x21E4, 0x07D0);
                    CreateGameNetworkConnection(/* show error dialog */);
                    return 1;
                }
            }
        }
        
        // All checks passed - initiate connection
        InitiateServerConnection(this->targetServer);
        ShowLocalizedMessage(0x22CF, 0x07D0);  // "Connecting..."
        CreateGameNetworkConnection(/* success params */);
    }
}

// Default error: missing parameters
ShowLocalizedError(0x2264, 0x07D0);
CreateGameNetworkConnection(/* error dialog */);
return 1;
```

### Localization String IDs

| ID (Hex) | ID (Dec) | Context |
|----------|----------|---------|
| 0x1DFC | 7676 | Error: Dialog still open |
| 0x07D0 | 2000 | Generic title/header |
| 0x22D2 | 8914 | Error: Missing items (variant 1) |
| 0x2D5B | 11611 | Error: Missing items (variant 2) |
| 0x21E4 | 8676 | Error: Too close to another player |
| 0x22CF | 8911 | Info: Connecting to server |
| 0x2264 | 8804 | Error: Missing connection parameters |

### Global Variables Referenced

- `g_GameState @ 0x015C9088`: Main game state manager
- `g_UIManager @ 0x015B17A8`: UI system manager
- `g_ProximityDistance`: Minimum distance threshold for connections

---

## CreateGameNetworkConnection Analysis

**Function:** `CreateGameNetworkConnection @ 0x00994DA0`  
**Signature:** `int __cdecl CreateGameNetworkConnection(void* param_1, wchar_t* param_2, wchar_t* param_3, uint param_4, float param_5, int param_6, undefined4 param_7, int param_8, int param_9, int param_10, undefined4* param_11)`

### Parameters (11 total)

| Param | Type | Purpose (Inferred) |
|-------|------|-------------------|
| param_1 | `void*` | Localization string ID (or context) |
| param_2 | `wchar_t*` | Dialog title text |
| param_3 | `wchar_t*` | Dialog message text |
| param_4 | `uint` | Dialog type/flags (0x21, 0x24, etc.) |
| param_5 | `float` | Callback function pointer (cast as float!) |
| param_6 | `int` | Additional flags |
| param_7 | `undefined4` | Reserved/optional parameter |
| param_8 | `int` | Connection timeout? |
| param_9 | `int` | Retry count? |
| param_10 | `int` | Result code (-1 = error, -0x80 = special) |
| param_11 | `undefined4*` | Output parameter (connection object?) |

**Note:** The `float` parameter casting `code*` is unusual - likely a quirk of the function signature or decompiler interpretation.

### Common Call Patterns

```c
// Error dialog
CreateGameNetworkConnection(
    (void*)0x2264,              // String ID
    L"Error Title",             // Title
    L"Error Message",           // Message
    0x21,                       // MB_ICONERROR | MB_OK
    0.0,                        // No callback
    0,                          // No flags
    NULL,                       // Reserved
    0,                          // No timeout
    0,                          // No retries
    -1,                         // Error result
    NULL                        // No output
);

// Success with callback
CreateGameNetworkConnection(
    (void*)0x1,                 // Success context
    L"Connecting...",           // Title
    L"Please wait",             // Message
    0x24,                       // Custom dialog type
    (float)HandleGamePacket_0x1001_SystemMessage,  // Callback
    0,
    NULL,
    0,
    0,
    -0x80,                      // Special result code
    NULL
);
```

---

## Critical Global Variables Map

| Address | Name | Type | Purpose |
|---------|------|------|---------|
| 0x015B53AC | `g_NetworkManager_ProudNetClient` | `ProudNet::NetClient*` | Main ProudNet client instance |
| 0x015C9088 | `g_GameStateManager` | `GameState*` | Game state singleton |
| 0x015D7708 | `g_RendererInstance` | `Renderer*` | Graphics renderer |
| 0x015A1C10 | `g_ShutdownRequested` | `bool` | Shutdown flag |
| 0x015B17A8 | `g_UIManager` | `UIManager*` | UI system manager |
| 0x015A5244 | `PTR_FUN_015a5244` | `FatalErrorHandler*` | Fatal error callback |
| 0x015A52B0 | `PTR_FUN_015a52b0` | `FatalErrorHandler*` | Error handler 2 |
| `TBD` | `g_InputSystemEnabled` | `bool` | Input system active flag |
| `TBD` | `g_InputWindow` | `HWND` | Input handling window |
| `TBD` | `g_GameServerIPAddress` | `char[32]` | Target server IP address |
| `TBD` | `g_ProximityDistance` | `float` | Minimum proximity for actions |

---

## Message Dispatch Architecture

### System Messages (ProudNet Internal)

These are **ProudNet protocol messages**, not game-specific:

- `0x0B`: Version check
- `0x1F`: TCP connection established
- `0x20`: TCP disconnect

### Game Messages (RMI Calls)

Based on function name `HandleGamePacket_0x1001_SystemMessage`, we can infer:

- `0x1001`: System message (chat/notification)
- Higher message IDs likely game-specific (movement, combat, inventory, etc.)

**Format:** `0xABCD` where:
- `0xA000` range: System/core messages
- `0xB000` range: Combat messages (speculation)
- `0xC000` range: Social messages (speculation)
- etc.

---

## Next Steps for Analysis

### Immediate Priorities

1. **Rename Global Variables**
   - Use Ghidra to rename all `DAT_*` globals with descriptive names
   - Document structure layouts for key globals

2. **Map ProudNet Message IDs**
   - Analyze all 75 ProudNet functions
   - Create comprehensive message ID table
   - Correlate with packet captures

3. **Extract LocalizationManager Strings**
   - Dump all localization IDs (0x1DFC, 0x22CF, etc.)
   - Build string table for error messages
   - Helps understand game flow

4. **Analyze `MainGameLoopUpdate()`**
   - This is where frame logic happens
   - Likely calls network receive/process
   - Update game state, rendering, physics

5. **Deep Dive on `EstablishGameServerConnection`**
   - Parameters: What gets passed?
   - Does it use ProudNet directly?
   - Connection handshake sequence

### Long-term Goals

1. **Reconstruct ProudNet RMI Interface**
   - Identify all RMI proxies and stubs
   - Map message IDs to handler functions
   - Build complete message catalog

2. **Extract Packet Structures**
   - Find serialization/deserialization code
   - Document packet layouts for each message type
   - Validate against Wireshark captures

3. **Map Game State Machine**
   - Understand state transitions
   - Login → Character Select → In-Game
   - How does state affect message handling?

4. **Reverse Engineer Encryption**
   - Analyze `ProudNet_InitializeAESKey`
   - Determine AES mode (CBC, CTR, GCM?)
   - RSA key exchange protocol

---

## Appendix: Function Signature Reference

### Functions Needing Rename

| Current Name | Suggested Rename | Address | Rationale |
|--------------|------------------|---------|-----------|
| `FUN_00995900` | `UI_IsDialogOpen` | 0x00995900 | Checks if named dialog is visible |
| `FUN_006ae6f0` | `GetItemCount` | 0x006AE6F0 | Returns item count for ID |
| `FUN_006aec70` | `InitiateServerConnection` | 0x006AEC70 | Starts connection process |
| `FUN_0044b110` | `VectorLength` | 0x0044B110 | Calculates 3D vector magnitude |
| `FUN_00464c40` | `ReleasePlayerReference` | 0x00464C40 | Decrements player ref count |
| `FUN_004d8bd0` | `CleanupPlayerIterator` | 0x004D8BD0 | Iterator cleanup |
| `FUN_004d8400` | `AdvancePlayerIterator` | 0x004D8400 | Move to next player |

### Structures to Define

```c
// Application Metadata (partial)
struct ApplicationMetadata {
    // ... (unknown fields 0x00-0x57)
    wchar_t* windowTitle;      // +0x58
    // ... (unknown fields 0x5C-0x73)
    wchar_t* windowClassName;  // +0x74
    // ...
};

// Game State (partial)
struct GameState {
    // ... (unknown layout)
    // Contains container counts, player state, etc.
};

// GameTimer
struct GameTimer {
    float value;
    // ... (unknown fields)
};

// Dialog Window
struct DialogWindow {
    // ... (unknown fields)
    wchar_t* name;  // +0x138 (offset from struct base)
    // ...
};
```

---

## Summary

The RO2 client is a **well-engineered MMORPG client** with:

- **Sophisticated logging** for debugging and analytics
- **ProudNet middleware** for networking (75+ related functions)
- **Comprehensive error handling** with localized messages
- **State validation** before connections (UI, inventory, proximity checks)
- **Message-driven architecture** with Windows message pump
- **Protection systems** for anti-cheat

The traversal from `WinMain` reveals a clear initialization sequence and exposes the main game loop structure. Key next steps involve mapping all global variables, renaming functions systematically, and extracting message ID mappings from both Ghidra analysis and packet captures.

**This document provides the foundation for implementing an accurate server emulator by understanding exactly how the client expects servers to behave.**
