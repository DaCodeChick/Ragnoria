# Function Analysis - Rag2.exe

Complete documentation of reverse-engineered functions related to login authentication.

---

## Login Packet Dispatcher: `DispatchLoginAuthPackets`

### Overview
- **Address**: 0x00E552E0
- **Size**: 13,765 bytes (0x35C5)
- **Calling Convention**: __thiscall
- **Parameters**: 3
  - `this` (void*): Object instance pointer
  - `param_1` (void*): Packet context
  - `param_2` (undefined4): Additional flags

### Function Signature
```c
void __thiscall DispatchLoginAuthPackets(void *this, void *param_1, undefined4 param_2)
```

### Purpose
Primary dispatcher for client-initiated login packets. Routes incoming packets based on opcode to appropriate handler delegates via virtual function calls.

### Architecture
- Large switch statement on packet opcode
- Each case deserializes packet data
- Calls handler via virtual function at specific vtable offset
- Includes extensive logging and performance monitoring
- Validates packet context before processing

### Handled Opcodes

#### 0x2EE1: ReqVersionCheck
- **Offset in vtable**: +0x24
- **Deserializer**: `DeserializeReqVersionCheck`
- **Purpose**: Client version validation
- **Validator**: Calls `ValidateProudNetPacketContext` before processing

#### 0x2EE2: ReqLogin (CRITICAL)
- **Offset in vtable**: +0x20
- **Deserializer**: `DeserializeReqLogin`
- **Payload Size**: 209 bytes (0xD1)
- **Total Packet**: 211 bytes (opcode + payload)
- **Contains**: Username, password, client version, platform ID
- **Security**: Must be AES encrypted (wrapped in 0x25/0x26)

#### 0x2EE3: ReqGraLogin
- **Offset in vtable**: +0x24
- **Deserializer**: `DeserializeReqGraLogin`
- **Purpose**: Gravity-specific login variant
- **Note**: Platform-specific authentication method

#### 0x2EE4: ReqChannelList
- **Offset in vtable**: +0x24
- **Deserializer**: None (empty packet)
- **Purpose**: Request available server channels

#### 0x2EE5: ReqLoginChannel
- **Offset in vtable**: +0x2C
- **Deserializer**: `DeserializeChannelId`
- **Purpose**: Connect to specific game channel
- **Payload**: Channel ID (uint32)

#### 0x2EE6: ReqLogOut
- **Offset in vtable**: +0x30
- **Deserializer**: None
- **Purpose**: Disconnect from authentication server

#### 0x2EE7: SendPacket (Generic)
- **Offset in vtable**: +0x34
- **Deserializer**: `DeserializeByteArrayWrapper`
- **Purpose**: Generic packet forwarding
- **Payload**: Variable-length byte array

#### 0x2EE8: ReqServerStatus
- **Offset in vtable**: +0x38
- **Deserializer**: None
- **Purpose**: Request server/channel status information

#### 0x2EE9: ReqSecondPassword
- **Offset in vtable**: +0x3C
- **Deserializer**: None
- **Purpose**: Initiate second password (2FA) prompt

#### 0x2EEA: ReqInputSecondPassword
- **Offset in vtable**: +0x40
- **Deserializer**: `DeserializeReqInputSecondPassword`
- **Purpose**: Submit second password for validation
- **Security**: Multi-factor authentication

### Logging Behavior
- Controlled by global flag: `g_ProudNet_VerboseLogging`
- Logs include: opcode, connection ID, timestamp, packet contents
- Performance monitoring via `GetTickCount()` before/after handler
- Wide string formatting for Korean/international character support

### Error Handling
- Validates packet context for most sensitive operations
- Throws `ThrowInvalidPacketException()` for malformed packets
- Delegates can return boolean success/failure
- Failed validation prevents handler execution

### Code Sample (Decompiled)
```c
case 0x2ee2:  // ReqLogin
    InitializePacketContext(local_118);
    local_114 = GetConnectionId(param_1);
    local_118[0] = GetPacketFlags((int)param_1);
    local_ec = param_2;
    
    // Deserialize 209-byte login payload
    DeserializeReqLogin(local_14, local_1f0);
    
    // Call handler via virtual function at offset +0x20
    (**(code **)(**(int **)((int)this + 8) + 0x24))();
    
    // Verbose logging if enabled
    if ((*(char *)((int)this + 0xd) != '\0') && (*(char *)((int)this + 0xc) == '\0')) {
        InitializeEmptyWideString(&local_1f4);
        LogReqLogin();
        // ... logging code ...
        (**(code **)(*(int *)this + 0x14))(local_20, 0x2ee2);
        ReleaseWideStringRef(&local_1f4);
    }
    
    // Performance monitoring
    if ((*(char *)((int)this + 0xc) == '\0') && (*(char *)((int)this + 0xe) != '\0')) {
        local_c0 = GetTickCount();
    }
    
    // Execute handler (returns success/failure)
    local_c1 = (**(code **)(*(int *)this + 0x20))(local_20);
    
    // If handler returns false, call error handler
    if (local_c1 == '\0') {
        (**(code **)(**(int **)((int)this + 8) + 0x20))();
    }
    
    // Log execution time
    if ((*(char *)((int)this + 0xc) == '\0') && (*(char *)((int)this + 0xe) != '\0')) {
        local_208 = GetTickCount();
        local_20c = local_208 - local_c0;  // Execution time in milliseconds
        (**(code **)(*(int *)this + 0xc))();
    }
    break;
```

---

## Acknowledgment Dispatcher: `DispatchAckPackets_0x30D5_0x30DC`

### Overview
- **Address**: 0x00E58940
- **Calling Convention**: __thiscall
- **Parameters**: 3

### Purpose
Routes server response packets (acknowledgments) to client-side handlers.

### Handled Opcodes

#### 0x30D4: AckVersionCheck
- **Deserializer**: `DeserializeAckVersionCheck`
- **Purpose**: Version validation result from server
- **Handler**: Offset +0x24 in vtable

#### 0x30D5: AckLogin (CRITICAL)
- **Deserializer**: `DeserializeAckLogin`
- **Payload Size**: 80 bytes (0x50)
- **Total Packet**: 82 bytes
- **Contains**: Login result, account ID, session keys, server info
- **Handler**: Offset +0x24 in vtable

#### 0x30D7: AckChannelList
- **Deserializer**: `DeserializeChannelList`
- **Purpose**: List of available game channels
- **Handler**: Offset +0x24 in vtable

#### 0x30D8: AckLoginChannel
- **Deserializer**: `DeserializeChannelList`
- **Purpose**: Channel connection confirmation
- **Handler**: Offset +0x28 in vtable

#### 0x30DA: AnsSecondPassword
- **Deserializer**: `DeserializeAnsSecondPassword`
- **Purpose**: Second password prompt response
- **Handler**: Offset +0x30 in vtable

#### 0x30DB: AnsInputSecondPassword
- **Deserializer**: `DeserializeAnsInputSecondPassword`
- **Purpose**: Second password validation result
- **Handler**: Offset +0x34 in vtable

### Code Sample (Decompiled)
```c
case 0x30d5:  // AckLogin
    InitializePacketContext(local_80);
    local_7c = GetConnectionId((void *)param_1);
    local_80[0] = GetPacketFlags(param_1);
    local_54 = param_2;
    
    // Initialize buffer for 80-byte response
    FUN_00e5a1c0(local_90);
    
    // Deserialize 80-byte login response
    DeserializeAckLogin(local_14, local_90);
    
    // Call handler
    (**(code **)(**(int **)((int)this + 8) + 0x24))();
    
    // ... logging and performance monitoring ...
    
    break;
```

---

## Login Request Builder: `SendReqLogin`

### Overview
- **Address**: 0x00E52FE0
- **Size**: 173 bytes
- **Calling Convention**: __thiscall
- **Return Type**: undefined1 (boolean success/failure)
- **Parameters**: 4
  - `this` (void*): Object instance
  - `param_1` (undefined4): Connection flags
  - `param_2` (undefined4): Priority/routing flags
  - `param_3` (undefined4*): Login data structure pointer

### Function Signature
```c
undefined1 __thiscall SendReqLogin(void *this, undefined4 param_1, 
                                    undefined4 param_2, undefined4 *param_3)
```

### Purpose
Constructs and sends the 0x2EE2 login authentication packet. This is the function that the client calls when the user clicks the Login button.

### Critical Finding
**NO CALLERS FOUND** - This function is invoked via function pointer or virtual function table, making it impossible to trace back to the UI button handler directly. This suggests:
- COM interface method
- Event handler callback
- Virtual function override
- UI framework binding

### Implementation Details
```c
undefined1 __thiscall SendReqLogin(void *this, undefined4 param_1, 
                                    undefined4 param_2, undefined4 *param_3)
{
    undefined1 uVar1;
    uint uVar2;
    undefined4 extraout_EDX;
    undefined1 local_2c [28];
    int *local_10;
    code *pcStack_c;
    int local_8;
    
    // Stack canary for buffer overflow protection
    local_8 = -1;
    pcStack_c = FUN_012e6ec8;
    local_10 = ExceptionList;
    uVar2 = g_StackCanary ^ (uint)&stack0xfffffffc;
    ExceptionList = &local_10;
    
    // Create packet buffer
    Proud::CMessage::CreatePacketBuffer((CMessage *)local_2c);
    local_8 = 0;
    
    // Initialize packet
    InitializePacketBuffer(local_2c);
    
    // Write opcode 0x2EE2
    WritePacketOpcode(local_2c, extraout_EDX, 0x2ee2);
    
    // Serialize 209-byte login payload from param_3
    SerializeReqLoginPacket(local_2c, param_3);
    
    // Send packet through network layer
    // Offset +0xC in vtable = send function
    uVar1 = (**(code **)(*(int *)this + 0xc))
                (&param_1, 1, param_2, local_2c, 
                 PTR_u_ReqLogin_015a5c4c, 0x2ee2, uVar2);
    
    // Cleanup
    local_8 = -1;
    Proud::CMessage::FinalizePacketBuffer((CMessage *)local_2c);
    ExceptionList = local_10;
    
    return uVar1;  // Returns success/failure
}
```

### Payload Structure (param_3)
The `param_3` pointer points to a structure containing:
- Account username (likely null-terminated string)
- Password hash or encrypted password
- Client version information
- Platform identifier (Steam, AeriaGames, etc.)
- Session tokens
- Hardware fingerprint (optional)

Total serialized size: **209 bytes (0xD1)**

### Network Layer Call
```c
(**(code **)(*(int *)this + 0xc))(&param_1, 1, param_2, local_2c, 
                                    PTR_u_ReqLogin_015a5c4c, 0x2ee2, uVar2)
```
This virtual function call sends the packet:
- `param_1`: Connection flags (reliability, encryption, etc.)
- `1`: Priority level
- `param_2`: Routing flags
- `local_2c`: Packet buffer
- `PTR_u_ReqLogin_015a5c4c`: Wide string "ReqLogin" for logging
- `0x2ee2`: Opcode for logging/debugging
- `uVar2`: Stack canary validation

---

## ProudNet Protocol Dispatcher: `DispatchProudNetProtocolPackets`

### Overview
- **Address**: 0x00F43FF0
- **Purpose**: Routes ProudNet protocol layer packets (opcodes 0x01-0x32)
- **Critical Feature**: Recursive processing for encryption/compression

### Key Protocol Opcodes

#### 0x01: KeepAlive
- No validation
- Connection maintenance ping

#### 0x04: EncryptionHandshake (CRITICAL)
- **Validator**: `ValidateProudNetPacketContext`
- **Handler**: `ProudNet_HandleServerPublicKey_0x04`
- **Purpose**: RSA-1024 key exchange for AES setup

#### 0x07: VersionCheck
- ProudNet layer version validation
- Different from game-layer version check (0x2EE1)

#### 0x1B: Heartbeat
- Connection keepalive with timestamp
- Requires acknowledgment

#### 0x1C: KeepAlive Ping
- Faster ping for connection monitoring

#### 0x25/0x26: EncryptedPacket (CRITICAL)
- **Purpose**: AES-encrypted payload wrapper
- **Processing**: 
  1. Decrypt using `Proud::CNetCoreImpl::ProcessMessage_Encrypted`
  2. **Recursively** call `DispatchProudNetProtocolPackets` with decrypted data
  3. This allows nested encryption and routing to game layer

```c
case 0x25:
case 0x26:
    InitializeProudNetHeader(decryptedBuffer);
    
    // Decrypt the packet
    opcodeReadSuccess = Proud::CNetCoreImpl::ProcessMessage_Encrypted(
        *(CNetCoreImpl **)((int)this + 0x5c),
        protocolOpcode,
        packetBuffer_00,
        (CMessage *)decryptedBuffer
    );
    
    if (opcodeReadSuccess) {
        // RECURSIVE CALL with decrypted data
        uVar4 = DispatchProudNetProtocolPackets(this, contextParams, 
                                                  (CMessage *)decryptedBuffer);
        dispatchResult = (char)uVar4;
    }
    
    FinalizeProudNetHeader(decryptedBuffer);
    break;
```

This recursive architecture allows:
- Game packets (0x0000, 0x2EE2) wrapped in 0x25/0x26
- Multiple layers of encryption
- Transparent decryption before game logic sees data

---

## Deserialization Functions

### DeserializeReqLogin
- **Address**: 0x00E5A430
- **Parameters**: 2
  - `param_1`: CMessage* (packet buffer)
  - `param_2`: Output structure pointer
- **Purpose**: Extracts 209-byte login payload from packet

```c
void __thiscall DeserializeReqLogin(CMessage *msg, void *output)
{
    // Read exactly 209 bytes (0xD1) from packet stream
    Proud::CMessage::ReadBytesFromBuffer(msg, output, 0xD1);
}
```

### DeserializeAckLogin
- **Address**: 0x00E5A530
- **Parameters**: 2
- **Purpose**: Extracts 80-byte login response

```c
void __thiscall DeserializeAckLogin(CMessage *msg, void *output)
{
    // Read exactly 80 bytes (0x50) from packet stream
    Proud::CMessage::ReadBytesFromBuffer(msg, output, 0x50);
}
```

### SerializeReqLoginPacket
- **Purpose**: Writes login data structure to packet buffer
- **Method**: Raw memcpy of 209-byte structure

---

## Supporting Functions

### ValidateProudNetPacketContext
- **Purpose**: Validates packet metadata before processing
- **Checks**: Connection ID, sequence numbers, timestamps
- **Used By**: Most sensitive packet handlers

### GetConnectionId
- **Purpose**: Extracts connection ID from packet context
- **Returns**: uint32 connection identifier

### InitializePacketContext
- **Purpose**: Sets up packet processing context
- **Includes**: Connection ID, flags, timestamps

### GetTickCount
- **Purpose**: Windows API for millisecond-precision timestamps
- **Used For**: Performance monitoring, latency measurement

---

## Unknown/Missing Functions

### 0x0000 Handler (NOT FOUND)
**Expected**: Function that processes incoming 0x0000 game handshake response

**Searched For**:
- Pattern: "0000", "Handshake", "InitialConnect"
- Cross-references: To/from all known dispatchers
- String references: "0x0000", error messages

**Possible Explanations**:
1. **Inlined**: Compiler optimized it into calling function
2. **Obfuscated**: Name doesn't match expected pattern
3. **COM Interface**: Hidden in COM vtable dispatch
4. **Doesn't Exist**: Maybe 0x0000 isn't supposed to be handled?

### Login Button Handler (NOT FOUND)
**Expected**: UI callback that calls `SendReqLogin`

**Why It's Hard to Find**:
- COM/DirectX UI framework
- Event-driven callback system
- Function pointers in vtables
- Possibly in external DLL (UI library)

---

## Next Steps for Function Analysis

1. **Dynamic Analysis**: Use x64dbg to set breakpoint on `SendReqLogin` and work backwards when Login button is clicked
2. **vtable Mapping**: Document all virtual function tables to understand delegate architecture
3. **String Cross-Reference**: Find UI strings like "Login", "Username", "Password" and trace to handlers
4. **COM Analysis**: Investigate COM interfaces for UI event bindings

