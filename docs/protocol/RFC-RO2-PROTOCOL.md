# RFC-RO2-001: Ragnarok Online 2 Network Protocol Specification

**Status:** Draft  
**Version:** 0.1.0  
**Date:** January 2026  
**Authors:** Ragnoria Project Contributors

---

## Abstract

This document specifies the network protocol used by Ragnarok Online 2, including the ProudNet RMI (Remote Method Invocation) framework, packet structures, message types, and encryption mechanisms. This specification is derived through reverse engineering of the client binary (Rag2.exe) using Ghidra static analysis and network traffic capture.

## Status of This Memo

This document provides information for the implementation of Ragnarok Online 2 server emulators. Distribution of this memo is unlimited for educational and research purposes.

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Terminology](#2-terminology)
3. [Protocol Overview](#3-protocol-overview)
4. [ProudNet RMI Framework](#4-proudnet-rmi-framework)
5. [Packet Structure](#5-packet-structure)
6. [Message Types](#6-message-types)
7. [Encryption and Security](#7-encryption-and-security)
8. [Login Server Protocol](#8-login-server-protocol)
9. [Lobby Server Protocol](#9-lobby-server-protocol)
10. [World Server Protocol](#10-world-server-protocol)
11. [Error Codes](#11-error-codes)
12. [Security Considerations](#12-security-considerations)
13. [References](#13-references)
14. [Appendices](#14-appendices)

---

## 1. Introduction

Ragnarok Online 2 is a massively multiplayer online role-playing game (MMORPG) that uses a proprietary network protocol built on the ProudNet game networking library developed by Nettention.

This specification documents the protocol for the purpose of creating server emulators and understanding the game's network architecture.

### 1.1. Scope

This document covers:
- Network packet structure and binary format
- ProudNet RMI message system
- Three-tier server architecture (Login, Lobby, World)
- Encryption and authentication mechanisms
- Complete message catalog reverse-engineered from client

### 1.2. Reverse Engineering Methodology

Information in this specification was obtained through:
1. Static analysis using Ghidra SRE (Software Reverse Engineering)
2. Network traffic capture using Wireshark
3. Binary analysis of Rag2.exe client executable
4. String table extraction and cross-referencing

---

## 2. Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

**ProudNet RMI**: Remote Method Invocation framework by Nettention  
**Host ID**: Unique identifier for connected client  
**Session Key**: Temporary encryption key issued post-authentication  
**Message ID**: Numeric identifier for RMI message types  
**Packet**: Complete unit of network transmission  
**RMI Proxy**: Client-side object for remote method invocation  
**RMI Stub**: Server-side object that receives method calls

---

## 3. Protocol Overview

### 3.1. Architecture

RO2 uses a three-tier server architecture:

```
[Client] <---> [Login Server:7101] <---> [Database]
             |
             v
          [Lobby Server:7201] <---> [Database]
             |
             v
          [World Server:7401] <---> [Database]
```

Each server handles specific responsibilities:
- **Login Server**: Authentication and session key generation
- **Lobby Server**: Channel selection and character management
- **World Server**: Game world simulation and gameplay

### 3.2. Connection Flow

```
1. Client connects to Login Server (TCP port 7101)
2. Client sends ReqLogin with credentials
3. Server validates and responds with AnsLogin + session key
4. Client disconnects from Login Server
5. Client connects to Lobby Server (TCP port 7201)
6. Client sends ReqLoginChannel with session key
7. Server validates session and responds with AnsLoginChannel
8. Client selects character and channel
9. Client connects to World Server (TCP port 7401)
10. Gameplay commences
```

### 3.3. Server Ports

| Server Type | Port | Protocol | Purpose                    |
|-------------|------|----------|----------------------------|
| Login       | 7101 | TCP      | Authentication             |
| Lobby       | 7201 | TCP      | Character/Channel select   |
| World       | 7401 | TCP      | Gameplay simulation        |

All servers use TCP for reliable, ordered delivery of game messages.

---

## 4. ProudNet RMI Framework

### 4.1. RMI Proxy/Stub Pattern

ProudNet implements a distributed object model using the Proxy/Stub pattern:

- **Proxy (Client-side)**: Marshals method calls into network packets
- **Stub (Server-side)**: Unmarshals packets and invokes actual methods

From Ghidra analysis, the following structures were identified:
- `IRmiProxy` - Interface for client-side RMI proxies
- `IRmiStub` - Interface for server-side RMI stubs
- `IRmiHost` - Host container for RMI endpoints

### 4.2. Message Naming Convention

All RMI messages follow a strict naming convention:

| Prefix | Direction | Purpose                        | Example            |
|--------|-----------|--------------------------------|--------------------|
| Req    | C → S     | Client request                 | ReqLogin           |
| Ans    | S → C     | Server answer (response)       | AnsLogin           |
| Nfy    | S → C     | Server notification (push)     | NfyServerTime      |
| Ack    | S → C     | Server acknowledgment          | AckServerStatus    |

### 4.3. Message Categories

Messages are organized by functional area:
- **Authentication**: Login, logout, session management
- **Character**: Creation, deletion, selection
- **Channel**: Server selection, channel switching
- **Inventory**: Item management, equipment
- **Combat**: Skill usage, damage calculation
- **Social**: Party, guild, chat, friends
- **Economy**: Trading, shops, auction house

For the complete message catalog, see [Appendix A](./appendices/message-catalog.md).

---

## 5. Packet Structure

### 5.1. PacketHeader Format (16 bytes)

From Ghidra analysis of `PacketHeader` structure at offset 0x00:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          VTable Pointer                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Source IP Address                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Source Port          |  Addr Flags   |   Reserved    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                            Host ID                             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Field Descriptions:**

- **VTable (4 bytes, offset 0x00)**: Pointer to virtual function table (C++ object implementation detail)
- **Source IP (4 bytes, offset 0x04)**: IPv4 address of sender in network byte order
- **Source Port (2 bytes, offset 0x08)**: TCP/UDP port of sender
- **Address Flags (1 byte, offset 0x0A)**: Bitfield for address properties
- **Reserved (1 byte, offset 0x0B)**: Reserved for future use, MUST be zero
- **Host ID (4 bytes, offset 0x0C)**: Unique identifier assigned by server to client connection

### 5.2. PacketBuffer Format (25 bytes)

From Ghidra analysis of `PacketBuffer` structure:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Buffer Data Ptr                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          Buffer Size                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Current Data Ptr                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          Current Size                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Allocated Size                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Read Position                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Buffer Flags  |
+-+-+-+-+-+-+-+-+
```

**Field Descriptions:**

- **Buffer Data (4 bytes, offset 0x00)**: Pointer to actual buffer memory
- **Buffer Size (4 bytes, offset 0x04)**: Total size of buffer in bytes
- **Current Data (4 bytes, offset 0x08)**: Pointer to current read/write position
- **Current Size (4 bytes, offset 0x0C)**: Number of bytes currently used
- **Allocated Size (4 bytes, offset 0x10)**: Total allocated memory
- **Read Position (4 bytes, offset 0x14)**: Current read cursor position
- **Buffer Flags (1 byte, offset 0x18)**: Control flags for buffer behavior

### 5.3. NetworkPacket Format (44 bytes)

From Ghidra analysis of `NetworkPacket` structure:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Buffer Data Ptr                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          Buffer Size                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Buffer Capacity                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Buffer Offset                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Read Pointer                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Write Pointer                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Buffer Flags  |  Reserved 1   |  Reserved 2   |  Reserved 3   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          Packet Type                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                                |
+                        PacketHeader (16 bytes)                 +
|                          (see section 5.1)                     |
+                                                                +
|                                                                |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Notable Fields:**

- **Packet Type (4 bytes, offset 0x1C)**: Message type identifier
- **PacketHeader (16 bytes, offset 0x20)**: Embedded header structure

### 5.4. CompletePacket Format (48 bytes)

From Ghidra analysis of `CompletePacket` structure:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                                |
+                      PacketBuffer (25 bytes)                   +
|                         (see section 5.2)                      |
+                                                                +
|                                                                |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Padding 1     |           Padding 2           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          Packet Type                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                                |
+                      PacketHeader (16 bytes)                   +
|                         (see section 5.1)                      |
+                                                                +
|                                                                |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

This appears to be the highest-level packet container used for transmission.

---

## 6. Message Types

### 6.1. Request (Req) Messages

Client-initiated requests MUST use the "Req" prefix. The server SHOULD respond with a corresponding "Ans" message.

**Examples:**
- `ReqLogin` - Authenticate with username/password
- `ReqLoginChannel` - Request lobby server access
- `ReqServerStatus` - Query available servers
- `ReqChannelList` - Get available game channels

### 6.2. Answer (Ans) Messages

Server responses to Req messages MUST use the "Ans" prefix.

**Examples:**
- `AnsLogin` - Login response with result code
- `AnsLoginChannel` - Lobby authentication response
- `AnsServerStatus` - Server status information
- `AnsChannelList` - Available channel list

### 6.3. Notify (Nfy) Messages

Server-initiated notifications MUST use the "Nfy" prefix. These are asynchronous push notifications that do not require a client request.

**Examples:**
- `NfyServerTime` - Server time synchronization
- `NfyServerTimeToLoginPC` - Login server time notification
- `NfyChannelDisconnect` - Channel disconnection notice

### 6.4. Acknowledgment (Ack) Messages

Server acknowledgments MUST use the "Ack" prefix. These confirm receipt of client messages.

**Examples:**
- `AckServerStatus` - Server status query acknowledged
- `AckLogin` - Login attempt acknowledged
- `AckChannelList` - Channel list request acknowledged

For the complete message catalog (200+ messages), see [Appendix A: Message Catalog](./appendices/message-catalog.md).

---

## 7. Encryption and Security

### 7.1. Encryption Algorithms

Based on client analysis, RO2 uses:
- **AES**: Symmetric encryption for session data
- **RSA**: Asymmetric encryption for key exchange

### 7.2. Authentication Flow

```
1. Client connects to Login Server
2. Server sends RSA public key
3. Client generates AES session key
4. Client encrypts session key with RSA public key
5. Client sends encrypted session key to server
6. Server decrypts session key with RSA private key
7. All subsequent communication encrypted with AES session key
```

### 7.3. Session Key Management

- Session keys MUST be unique per connection
- Session keys SHOULD expire after logout or timeout
- Session keys MUST be validated when connecting to Lobby/World servers

---

## 8. Login Server Protocol

### 8.1. Connection Establishment

The client initiates a TCP connection to port 7101.

### 8.2. Authentication Flow

```
Client                           Server
   |                                |
   |-------- ReqLogin ------------->|
   |  (username, password)          |
   |                                |
   |<------- AnsLogin --------------|
   |  (result, session_key)         |
   |                                |
   |---- ReqServerStatus ---------->|
   |                                |
   |<--- AckServerStatus -----------|
   |  (server_list, channel_info)   |
   |                                |
```

### 8.3. Key Messages

- **ReqLogin**: Username, password, client version
- **AnsLogin**: Result code, session key (if successful)
- **ReqServerStatus**: Query for available servers
- **AckServerStatus**: List of lobby/world servers
- **AnsPlayerLoginKey**: Session key distribution
- **NfyServerTimeToLoginPC**: Time synchronization

For detailed message structures, see [Appendix B: Login Messages](./appendices/login-messages.md).

---

## 9. Lobby Server Protocol

### 9.1. Session Validation

The client connects to port 7201 with the session key obtained from login server.

### 9.2. Channel Selection Flow

```
Client                           Server
   |                                |
   |--- ReqLoginChannel ----------->|
   |  (session_key)                 |
   |                                |
   |<-- AnsLoginChannel ------------|
   |  (character_list)              |
   |                                |
   |--- ReqChannelList ------------>|
   |                                |
   |<-- AckChannelListInGame -------|
   |  (available_channels)          |
   |                                |
   |--- ReqChannelMove ------------>|
   |  (channel_id)                  |
   |                                |
   |<-- AnsChannelMove -------------|
   |  (world_server_address)        |
   |                                |
```

### 9.3. Key Messages

- **ReqLoginChannel**: Session key validation
- **AnsLoginChannel**: Character list
- **ReqChannelList**: Query available channels
- **AckChannelListInGame**: Channel availability
- **ReqChannelMove**: Request channel switch
- **AnsChannelMove**: World server connection info

---

## 10. World Server Protocol

### 10.1. Game World Connection

The client connects to port 7401 to enter the game world.

### 10.2. Scope (Proof of Concept)

For the initial implementation, the world server will:
- Accept connections from authenticated clients
- Handle basic presence (player spawn)
- Future: Movement, chat, combat, etc.

---

## 11. Error Codes

From client string analysis:

| Error String               | Likely Code | Meaning                    |
|----------------------------|-------------|----------------------------|
| Login_Ok                   | 0           | Authentication successful  |
| Login_Failed               | 1           | Generic authentication failure |
| LoginFail_ACCOUNT_BLOCK    | 2           | Account banned             |
| LoginFail_ACCOUNT_LOGGING  | 3           | Already logged in          |
| LoginFail_PASSWORD_ERROR   | 4           | Invalid password           |
| LoginFail_ACCOUNT_ERROR    | 5           | Account does not exist     |
| LoginFail_IP_ERROR         | 6           | IP address blocked         |
| LoginFail_LOCALSYS_ERROR   | 7           | Internal server error      |
| LoginFail_WrongLoginMethod | 8           | Invalid login method       |
| Channel_NotExist           | 100         | Invalid channel ID         |
| Channel_Full               | 101         | Channel at capacity        |
| Server_System_Error        | 500         | Internal server error      |
| Server_Inpected            | 503         | Server under inspection    |

**Note:** Actual numeric codes must be determined through packet capture analysis.

---

## 12. Security Considerations

### 12.1. Authentication

- Passwords MUST be hashed before transmission (client-side)
- Server MUST use bcrypt or argon2 for password storage
- Session keys MUST be cryptographically random

### 12.2. Rate Limiting

- Login attempts SHOULD be rate-limited per IP
- Failed authentication SHOULD trigger exponential backoff

### 12.3. Input Validation

- All client input MUST be validated before processing
- Message size limits MUST be enforced to prevent DoS

### 12.4. Encryption

- AES encryption SHOULD use at least 128-bit keys
- RSA key exchange SHOULD use at least 2048-bit keys

---

## 13. References

- **Ghidra Software Reverse Engineering**: https://ghidra-sre.org/
- **ProudNet Game Networking Library**: Nettention Corp.
- **Ragnarok Online 2 Client**: Rag2.exe (Gravity Interactive)
- **RFC 2119**: Key words for use in RFCs to Indicate Requirement Levels

---

## 14. Appendices

- [Appendix A: Complete Message Catalog](./appendices/message-catalog.md) - All 200+ RMI messages
- [Appendix B: Login Messages](./appendices/login-messages.md) - Detailed login flow structures
- [Appendix C: Ghidra Findings](../ghidra-findings.md) - Raw reverse engineering notes
- [Appendix D: Packet Captures](./appendices/packet-captures.md) - Wireshark analysis (TBD)
- [Appendix E: Implementation Notes](./appendices/implementation-notes.md) - Rust-specific guidance

---

**End of RFC-RO2-001**
