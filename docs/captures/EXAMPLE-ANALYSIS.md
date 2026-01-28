# Complete Packet Analysis Example

This document demonstrates a full analysis of a captured RO2 login packet to extract message IDs and understand the payload structure.

## Example Scenario: ReqLogin Packet

### Raw Wireshark Capture

```
Packet #42: Client → Server (7101) - 64 bytes

0000  50 52 4f 55 30 00 00 00  23 01 01 00 05 00 00 00   PROU0...#.......
0010  05 00 00 00 61 64 6d 69  6e 08 00 00 00 61 64 6d   ....admin....adm
0020  69 6e 31 32 33 00 01 00  00 00 00 00 00 00 00 00   in123...........
0030  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00   ................
```

### Step-by-Step Analysis

#### 1. Parse Header (16 bytes)

```
Offset | Bytes          | Field Name     | Value         | Notes
-------|----------------|----------------|---------------|------------------
0x00   | 50 52 4F 55    | magic          | 0x554F5250    | 'PROU' signature
0x04   | 30 00 00 00    | length         | 48 (0x0030)   | Payload length
0x08   | 23 01          | message_id     | 0x0123        | ⚠️ ReqLogin ID
0x0A   | 01 00          | flags          | 0x0001        | Protocol version
0x0C   | 05 00 00 00    | sequence       | 5             | Packet counter
```

**Key Finding:** `ReqLogin` message ID = **0x0123**

#### 2. Parse Payload (48 bytes)

**Offset 0x10-0x18 (username length + data):**
```
05 00 00 00 61 64 6d 69 6E
```
- `05 00 00 00` = 5 (u32 little-endian) - username length
- `61 64 6d 69 6E` = "admin" (5 ASCII bytes)

**Offset 0x19-0x24 (password length + data):**
```
08 00 00 00 61 64 6d 69 6E 31 32 33 00
```
- `08 00 00 00` = 8 (u32 little-endian) - password length
- `61 64 6d 69 6E 31 32 33` = "admin123" (8 ASCII bytes)
- `00` = null terminator (optional)

**Offset 0x25-0x30 (login options):**
```
01 00 00 00 00 00 00 00 00 00 00 00
```
- `01 00 00 00` = 1 (u32) - possibly "remember me" flag
- Remaining bytes = padding or reserved fields

### Payload Structure (C-like)

```c
struct ReqLoginPayload {
    uint32_t username_length;    // 4 bytes
    char username[username_length]; // Variable
    uint32_t password_length;    // 4 bytes
    char password[password_length]; // Variable
    uint8_t null_terminator;     // 1 byte (optional)
    uint32_t login_flags;        // 4 bytes
    uint8_t padding[8];          // 8 bytes reserved
};
```

### Entropy Analysis

```
Username section: "admin" - ASCII text, low entropy
Password section: "admin123" - ASCII text, low entropy
Conclusion: Packet is NOT encrypted
```

---

## Example Scenario: AnsLogin Response

### Raw Wireshark Capture

```
Packet #43: Server → Client (7101) - 80 bytes

0000  50 52 4f 55 40 00 00 00  24 01 01 00 06 00 00 00   PROU@...$.......
0010  00 00 00 00 20 00 00 00  A3 4F 8C 2D E9 7A 3B C4   .... ...O.-.z;.
0020  F1 62 D8 9E 0A 14 7F 32  8B C5 E0 19 4E 6F B2 85   .b.....2....No..
0030  D7 23 A8 C1 5C 94 F6 0D  01 00 00 00 7B 56 34 12   .#..\......{V4.
0040  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00   ................
```

### Step-by-Step Analysis

#### 1. Parse Header

```
Offset | Bytes          | Field Name     | Value         | Notes
-------|----------------|----------------|---------------|------------------
0x00   | 50 52 4F 55    | magic          | 0x554F5250    | 'PROU' signature
0x04   | 40 00 00 00    | length         | 64 (0x0040)   | Payload length
0x08   | 24 01          | message_id     | 0x0124        | ⚠️ AnsLogin ID
0x0A   | 01 00          | flags          | 0x0001        | Protocol version
0x0C   | 06 00 00 00    | sequence       | 6             | Response to seq 5
```

**Key Finding:** `AnsLogin` message ID = **0x0124**

#### 2. Parse Payload

**Offset 0x10-0x14 (login result):**
```
00 00 00 00
```
- `00 00 00 00` = 0 (u32) - success code (0 = OK, non-zero = error)

**Offset 0x14-0x18 (session key length):**
```
20 00 00 00
```
- `20 00 00 00` = 32 (u32) - session key is 32 bytes

**Offset 0x18-0x38 (session key):**
```
A3 4F 8C 2D E9 7A 3B C4 F1 62 D8 9E 0A 14 7F 32
8B C5 E0 19 4E 6F B2 85 D7 23 A8 C1 5C 94 F6 0D
```
- 32-byte random session key (used for subsequent authentication)

**Offset 0x38-0x3C (account ID):**
```
01 00 00 00
```
- `01 00 00 00` = 1 (u32) - account ID in database

**Offset 0x3C-0x40 (server host ID):**
```
7B 56 34 12
```
- `7B 56 34 12` = 0x1234567B (u32) - server-assigned host ID

### Payload Structure (C-like)

```c
struct AnsLoginPayload {
    uint32_t result_code;         // 0 = success, 1+ = error
    uint32_t session_key_length;  // Always 32 bytes
    uint8_t session_key[32];      // Random session identifier
    uint32_t account_id;          // Database account ID
    uint32_t host_id;             // Server-assigned client identifier
    uint8_t padding[16];          // Reserved
};
```

---

## Updating the Code

### 1. Update `crates/ro2-common/src/protocol/mod.rs`

Replace placeholder IDs:

```diff
 #[repr(u32)]
 pub enum MessageType {
-    ReqLogin = 0x0001,
-    AnsLogin = 0x0002,
+    ReqLogin = 0x0123,  // ✓ Extracted from capture
+    AnsLogin = 0x0124,  // ✓ Extracted from capture
     ReqLoginChannel = 0x0003,
     // ... etc
 }
```

### 2. Create Message Payload Structs

Create `crates/ro2-common/src/protocol/messages.rs`:

```rust
use bytes::{Buf, BufMut};

/// ReqLogin payload structure
#[derive(Debug, Clone)]
pub struct ReqLogin {
    pub username: String,
    pub password: String,
    pub login_flags: u32,
}

impl ReqLogin {
    pub fn parse(mut data: &[u8]) -> crate::Result<Self> {
        // Read username
        let username_len = data.get_u32_le() as usize;
        if data.len() < username_len {
            anyhow::bail!("Insufficient data for username");
        }
        let username = String::from_utf8(data[..username_len].to_vec())?;
        data = &data[username_len..];
        
        // Skip null terminator if present
        if !data.is_empty() && data[0] == 0 {
            data = &data[1..];
        }

        // Read password
        let password_len = data.get_u32_le() as usize;
        if data.len() < password_len {
            anyhow::bail!("Insufficient data for password");
        }
        let password = String::from_utf8(data[..password_len].to_vec())?;
        data = &data[password_len..];
        
        // Skip null terminator if present
        if !data.is_empty() && data[0] == 0 {
            data = &data[1..];
        }

        // Read flags
        let login_flags = if data.len() >= 4 {
            data.get_u32_le()
        } else {
            0
        };

        Ok(Self {
            username,
            password,
            login_flags,
        })
    }
}

/// AnsLogin payload structure
#[derive(Debug, Clone)]
pub struct AnsLogin {
    pub result_code: u32,
    pub session_key: Vec<u8>,
    pub account_id: u32,
    pub host_id: u32,
}

impl AnsLogin {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        
        // Result code
        buf.extend_from_slice(&self.result_code.to_le_bytes());
        
        // Session key length + data
        buf.extend_from_slice(&(self.session_key.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.session_key);
        
        // Account ID
        buf.extend_from_slice(&self.account_id.to_le_bytes());
        
        // Host ID
        buf.extend_from_slice(&self.host_id.to_le_bytes());
        
        // Padding
        buf.extend_from_slice(&[0u8; 16]);
        
        buf
    }
}
```

### 3. Update Login Handler

In `crates/ro2-login/src/handlers/mod.rs`:

```rust
use ro2_common::packet::parser::{RmiMessage, RmiMessageBuilder};
use ro2_common::protocol::messages::{ReqLogin, AnsLogin};
use rand::Rng;

pub async fn handle_req_login(data: &[u8]) -> Result<Vec<u8>> {
    // Parse RMI message
    let rmi = RmiMessage::parse(data)?;
    
    // Parse ReqLogin payload
    let req = ReqLogin::parse(&rmi.payload)?;
    
    tracing::info!("Login attempt: username={}", req.username);
    
    // TODO: Validate credentials against database
    // For now, accept any login
    
    // Generate session key
    let mut session_key = vec![0u8; 32];
    rand::thread_rng().fill(&mut session_key[..]);
    
    // Build response
    let ans = AnsLogin {
        result_code: 0, // Success
        session_key,
        account_id: 1,
        host_id: 0x1234567B,
    };
    
    // Build RMI response
    let response = RmiMessageBuilder::new(0x0124, rmi.sequence)
        .payload(&ans.serialize())
        .build();
    
    Ok(response.to_bytes())
}
```

---

## Testing the Implementation

### 1. Run the packet analyzer

```bash
cargo run --bin packet-analyzer -- file docs/captures/login_flow_hex.txt
```

Expected output:
```
=== Packet Analysis ===

Message ID:       0x0123 (291)
                  ⚠️  UPDATE MessageType ENUM WITH THIS VALUE

=== Payload (48 bytes) ===
Potential strings found:
  - "admin"
  - "admin123"

Payload entropy: 3.87 bits/byte
  ✓ Low entropy - payload likely plaintext
```

### 2. Update the enum

Apply the message ID to `MessageType::ReqLogin`.

### 3. Test with real client

Point the RO2 client to your local server and attempt login. Monitor logs:

```bash
RUST_LOG=debug cargo run -p ro2-login
```

Expected log output:
```
INFO ro2_login: Login server starting on 0.0.0.0:7101
DEBUG ro2_login: Client connected: 127.0.0.1:54321
INFO ro2_login: Login attempt: username=admin
DEBUG ro2_login: Session key generated: a34f8c2d...
INFO ro2_login: Login successful: account_id=1
```

---

## Summary

✅ **Message IDs Extracted:**
- `ReqLogin` = `0x0123`
- `AnsLogin` = `0x0124`

✅ **Payload Structures Documented:**
- ReqLogin: length-prefixed username + password + flags
- AnsLogin: result code + 32-byte session key + account ID + host ID

✅ **Encryption Status:**
- Login packets are **plaintext** (low entropy)
- Session key is generated server-side (random 32 bytes)
- Encryption may be negotiated after initial login

## Next Steps

1. Capture lobby connection packets (port 7201) to extract channel/character message IDs
2. Implement database validation in login handler
3. Implement session key validation for lobby server
4. Research encryption handshake (if any) for gameplay packets
