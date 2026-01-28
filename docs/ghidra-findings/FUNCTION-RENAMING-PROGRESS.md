# Network Infrastructure Function Renaming Progress

## Session 4 - Network Tracing and Systematic Renaming

### Functions Renamed (14 total)

#### NetworkManager Core (5 functions)
| Original Name | New Name | Address | Purpose |
|---------------|----------|---------|---------|
| `FUN_00a36de0` | `InitNetworkManager` | 0x00a36de0 | Allocates and initializes NetworkManager singleton (0x74 bytes) |
| `FUN_00a420d0` | `NetworkManager_Constructor` | 0x00a420d0 | NetworkManager constructor, sets up vtable |
| `FUN_00a35e90` | `NetworkManager_PreGameUpdate` | 0x00a35e90 | Network update phase 1 (vtable+0x24) - receives messages before game logic |
| `FUN_00a306d0` | `NetworkManager_PostGameUpdate` | 0x00a306d0 | Network update phase 2 (vtable+0x28) - sends messages after game logic |
| `FUN_00a30660` | `GetNetworkManager` | 0x00a30660 | Returns NetworkManager singleton pointer |

#### Message Handling (4 functions)
| Original Name | New Name | Address | Purpose |
|---------------|----------|---------|---------|
| `FUN_00a3e2b0` | `NetworkManager_HandleWindowMessage` | 0x00a3e2b0 | Handles custom Windows messages 0x6623, 0x6624 (vtable+0x08) |
| `FUN_00a3d300` | `Handle_NetworkErrorMessage` | 0x00a3d300 | Displays network error dialog (message 0x6623) |
| `FUN_00a34240` | `Handle_NetworkErrorMessage2` | 0x00a34240 | Displays localized network error (message 0x6624) |
| `FUN_00a30640` | `ShutdownNetworkManager` | 0x00a30640 | Cleans up and frees NetworkManager |

#### VTable Placeholders (2 functions)
| Original Name | New Name | Address | Purpose |
|---------------|----------|---------|---------|
| `FUN_00a30710` | `NetworkManager_VTable_0x34` | 0x00a30710 | Called during cleanup (vtable+0x34) |
| `FUN_00a30720` | `NetworkManager_VTable_0x38` | 0x00a30720 | Called during cleanup (vtable+0x38) |

#### Utility Functions (3 functions from earlier session)
| Original Name | New Name | Address | Purpose |
|---------------|----------|---------|---------|
| `FUN_00a4c300` | `MainGameLoopUpdate` | 0x00a4c300 | Main game loop update function |
| `FUN_00a4b7c0` | `MainWindowProcedure` | 0x00a4b7c0 | Windows message procedure |
| `FUN_00a4c2a0` | `HandleGameShutdownRequest` | 0x00a4c2a0 | Initiates shutdown sequence |

---

## NetworkManager VTable Structure

**VTable Address:** `0x013df2f8`  
**Object Size:** 0x74 bytes (116 bytes)

### Complete VTable Layout

| Offset | Address | Status | Function Name |
|--------|---------|--------|---------------|
| +0x00 | 0x00a36e60 | ❌ Not renamed | Destructor |
| +0x04 | 0x00a34160 | ❌ Not renamed | Initialization (called after constructor) |
| +0x08 | 0x00a3e2b0 | ✅ Renamed | `NetworkManager_HandleWindowMessage` |
| +0x0c | 0x00a422b0 | ❌ Not renamed | Unknown |
| +0x10 | 0x00a42410 | ❌ Not renamed | Unknown |
| +0x14 | 0x00a306a0 | ❌ Not renamed | Takes hInstance parameter |
| +0x18 | 0x00a3c530 | ❌ Not renamed | Takes hInstance, hWnd parameters |
| +0x1c | 0x00a306c0 | ❌ Not renamed | Unknown |
| +0x20 | 0x00a42630 | ❌ Not renamed | Unknown |
| +0x24 | 0x00a35e90 | ✅ Renamed | `NetworkManager_PreGameUpdate` |
| +0x28 | 0x00a306d0 | ✅ Renamed | `NetworkManager_PostGameUpdate` |
| +0x2c | 0x00a36040 | ❌ Not renamed | Unknown |
| +0x30 | 0x00a41b80 | ❌ Not renamed | Unknown |
| +0x34 | 0x00a30710 | ✅ Renamed | `NetworkManager_VTable_0x34` |
| +0x38 | 0x00a30720 | ✅ Renamed | `NetworkManager_VTable_0x38` |
| +0x3c | 0x00a30700 | ❌ Not renamed | Unknown |
| +0x40 | 0x00a43570 | ❌ Not renamed | Unknown |
| +0x44 | 0x00a428e0 | ❌ Not renamed | Unknown |
| +0x48 | 0x00a43540 | ❌ Not renamed | Unknown |
| +0x4c | 0x00a42260 | ❌ Not renamed | Unknown |
| +0x50 | 0x00a392c0 | ❌ Not renamed | Unknown |

**Progress:** 4/21 vtable functions renamed (19%)

---

## Initialization Chain

```
WinMain @ 0x00A502F0
  │
  ├─> InitializeGameSubsystems @ 0x00ef89f0
  │     └─> (loads dict.lex dictionary file)
  │
  └─> InitializeMainWindow @ 0x00a4c660
        │
        ├─> InitNetworkManager @ 0x00a36de0
        │     ├─> operator_new(0x74)
        │     ├─> NetworkManager_Constructor @ 0x00a420d0
        │     └─> Sets g_NetworkManager_ProudNetClient @ 0x015B53AC
        │
        ├─> NetworkManager.vtable[0x04] @ 0x00a34160  (initialization)
        ├─> NetworkManager.vtable[0x14] @ 0x00a306a0  (setup with hInstance)
        └─> NetworkManager.vtable[0x18] @ 0x00a3c530  (setup with hWnd)
```

---

## Network Update Flow

```
MainGameLoopUpdate @ 0x00a4c300
  │
  ├─> (various timer and manager updates)
  │
  ├─> NetworkManager_PreGameUpdate @ 0x00a35e90  (vtable+0x24)
  │     └─> Receives incoming network messages
  │         Queues for processing
  │
  ├─> GameStateManager_Update (processes game logic)
  │
  └─> NetworkManager_PostGameUpdate @ 0x00a306d0  (vtable+0x28)
        └─> Sends outgoing network messages
            Flushes network buffers
```

---

## Message Processing

### Windows Message Handler
**Function:** `MainWindowProcedure @ 0x00a4b7c0`

Forwards all messages to:
```c
NetworkManager.vtable[0x08] @ 0x00a3e2b0  (NetworkManager_HandleWindowMessage)
```

### Custom Network Messages
- **0x6623**: Network error message (string) → `Handle_NetworkErrorMessage`
- **0x6624**: Network error message (localization ID) → `Handle_NetworkErrorMessage2`

---

## ProudNet Strings Identified (31 total)

Key function strings found in binary:

| String | Address | Likely Purpose |
|--------|---------|----------------|
| `"Proud::CNetClientWorker::ProcessMessage_ProudNetLayer"` | 0x01458560 | Main message processor |
| `"Proud::CNetCoreImpl::Send_SecureLayer"` | 0x01457b50 | Encrypted send |
| `"Proud::CNetCoreImpl::ProcessMessage_Encrypted"` | 0x01457c90 | Encrypted receive |
| `"Proud::CFastSocket::Connect"` | 0x014599ec | Socket connection |
| `"Proud::CFastSocket::IssueRecv"` | 0x01459a24 | Async receive |
| `"Proud::CFastSocket::IssueSend"` | 0x01459b3c | Async send |

**Note:** These strings are referenced in ProudNet library code (0x00f0xxxx range). Need to trace calls to these functions.

---

## Next Steps

### Immediate Priority

1. **Analyze NetworkManager_PreGameUpdate and PostGameUpdate**
   - These currently just call other manager updates
   - Need to find where actual socket receive/send happens
   - Likely delegates to internal ProudNet client object

2. **Find ProudNet Client Object**
   - NetworkManager probably contains a ProudNet::CNetClient member
   - Look for calls to functions in 0x00f0xxxx range from NetworkManager
   - Trace to actual message dispatch code

3. **Locate Message Dispatch Table**
   - Find switch/case or function pointer array
   - Maps message IDs (0x0B, 0x1F, 0x20, 0x1001+) to handlers
   - Likely in ProudNet::CNetClientWorker::ProcessMessage_ProudNetLayer

4. **Rename Remaining VTable Functions**
   - 17 functions still need renaming
   - Analyze each to determine purpose
   - Follow calling patterns from game code

### Medium Priority

5. **Input Message Handlers**
   - Already identified in MainWindowProcedure:
     - `HandleKeyDownMessage`
     - `HandleKeyUpMessage`
     - `HandleMouseMoveMessage`
     - `HandleLeftButtonDownMessage`
     - `HandleRightButtonDownMessage`
     - etc.
   - These need to be found and renamed

6. **ProudNet Internal Functions**
   - Functions in 0x00f0xxxx range (563 functions)
   - Focus on core message processing first
   - Then encryption/compression
   - Finally socket operations

### Lower Priority

7. **Game Message Handlers**
   - Handlers for RMI messages (0x1001+)
   - Likely in separate files/classes
   - Will need extensive analysis

---

## Global Variables Confirmed

| Variable | Address | Type | Description |
|----------|---------|------|-------------|
| `g_NetworkManager_ProudNetClient` | 0x015B53AC | `void*` | NetworkManager singleton (allocated as 0x74 bytes) |
| `g_MainWindowHandle` | Unknown | `HWND` | Main window handle |
| `g_UIManager` | 0x015B17A8 | `void*` | UI manager singleton |
| `g_ShutdownRequested` | 0x015A1C10 | `bool` | Shutdown flag |
| `g_InputSystemEnabled` | Unknown | `int` | Input routing state (0=normal, 2=dragging) |
| `_g_MouseDragStartX` | Unknown | `int` | Mouse drag origin X |
| `_g_MouseDragStartY` | Unknown | `int` | Mouse drag origin Y |

---

## Ghidra MCP Status

**Binary:** Rag2.exe  
**Session:** Active  
**Renaming Progress:** 14 functions renamed  
**Analysis Documents:** 
- `docs/ghidra-findings/WINMAIN-ANALYSIS.md` (600 lines)
- `docs/ghidra-findings/GAME-LOOP-ANALYSIS.md` (750 lines)

---

## Strategy Notes

The NetworkManager appears to be a **wrapper/facade** around ProudNet's internal client. The actual network processing likely happens in ProudNet code (0x00f0xxxx range). 

To find message dispatch:
1. Look for calls from NetworkManager into 0x00f0xxxx functions
2. Trace to ProcessMessage_ProudNetLayer
3. Find message ID extraction and dispatch table

The two-phase update model (Pre/Post game) currently just updates various managers - the actual socket I/O is probably in a background thread or completion port that ProudNet manages internally.
