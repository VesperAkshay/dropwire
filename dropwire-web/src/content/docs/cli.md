---
title: "CLI Command Reference"
description: "Comprehensive guide to every command and flag available in the DropWire CLI."
order: 2
---

# CLI Command Reference

The DropWire CLI is designed to be minimal and scriptable. It exposes three primary commands: `send`, `receive`, and `relay`.

```bash
dropwire <COMMAND> [OPTIONS]
```

---

## 1. `dropwire send`

Initiates a secure file or directory transfer and waits for a receiver to connect.

**Usage:**
```bash
dropwire send <FILE_OR_DIR> [OPTIONS]
```

**Options:**
- `-c, --code <CODE>`  
  Provide a custom code phrase instead of allowing the engine to auto-generate a random 3-word phrase.
- `-s, --streams <NUM>`  
  Set the number of parallel TCP multiplexing streams (default: `4`). Increasing this can help saturate high-bandwidth WAN connections.
- `-r, --relay <URL>`  
  Override the default signaling relay server address (default: `ws://dropwire.tyes.dev:9010`).
- `--no-lan`  
  Disable local network UDP multicast peer discovery. Forces the connection to route over the WAN internet relay.

**Example:**
```bash
dropwire send ./confidential_data.zip --streams 8 --code secret-project-123
```

---

## 2. `dropwire receive`

Connects to a sender using a shared room code and downloads the payload.

**Automatic Resume:** DropWire features zero-configuration resumability. If a transfer is interrupted, simply run the exact same `receive` command again in the same directory. The engine will read the `.dropwire-partial` state file and instantly negotiate to resume downloading only the missing chunks.

**Usage:**
```bash
dropwire receive <CODE> [OPTIONS]
```

**Options:**
- `-o, --out <DIR>`  
  Specify the output directory where the received files should be saved (default: `~/Downloads/Dropwire`).
- `-r, --relay <URL>`  
  Override the default signaling relay server address.
- `--no-lan`  
  Disable local network peer discovery.

**Example:**
```bash
dropwire receive secret-project-123 --out ./incoming-files
```

---

## 3. `dropwire relay`

Starts a self-hosted DropWire relay server. This is only necessary if you wish to host your own infrastructure. Relay servers facilitate WAN discovery and handle TCP stream stapling for peers blocked by restrictive NATs. **Relays cannot decrypt or read your file data.**

**Usage:**
```bash
dropwire relay [OPTIONS]
```

**Options:**
- `--bind <IP:PORT>`  
  The bind address for the TCP stream stapling server (default: `0.0.0.0:9009`).
- `--ws-bind <IP:PORT>`  
  The bind address for the WebSocket signaling server (default: `0.0.0.0:9010`).

**Example:**
```bash
dropwire relay --bind 0.0.0.0:8080 --ws-bind 0.0.0.0:8081
```
