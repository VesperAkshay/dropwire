<div align="center">
  <h1>DropWire CLI Reference</h1>
  <p><b>Comprehensive guide to every command and flag available in DropWire.</b></p>
  <a href="./README.md">← Back to README</a>
  <img src="./assets/commands.gif" alt="Commands Workflow" width="100%" />
</div>

<br/>

## 🌐 Global Usage

Every DropWire execution follows this structure:

```bash
dropwire <COMMAND> [OPTIONS]
```

---

## 📤 1. `send`
Initiates a file or directory transfer. If you don't provide a custom code, DropWire will securely auto-generate a random 3-word code phrase.

**Usage:**
```bash
dropwire send <FILE_OR_DIR> [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|---|---|---|
| `-c, --code <CODE>` | Provide a custom code phrase instead of auto-generating one. | Auto-generated |
| `-s, --streams <NUM>` | Set the number of parallel TCP multiplexing streams. Higher numbers can increase throughput on high-bandwidth links. | `4` |
| `-r, --relay <URL>` | Override the signaling relay server address. | Config file, or `ws://relay.dropwire.tyes.dev:9010` |
| `--no-lan` | Disable local network (UDP multicast) peer discovery and force WAN routing via relay. | `false` |

**Examples:**
```bash
# Basic send (auto-generates a code)
dropwire send ./my_video.mp4

# Send an entire project folder
dropwire send ./my_project/

# Send using a custom code phrase
dropwire send ./my_project --code secret-project-123

# Send using a local relay server and 8 parallel streams
dropwire send archive.zip --streams 8 --relay ws://127.0.0.1:9010
```

---

## 📥 2. `receive`
Receives a file or directory using the code phrase provided by the sender.

**Automatic Resume:** If a transfer is interrupted (network drop, Ctrl+C, laptop sleep), simply re-run the exact same `receive` command in the same directory. DropWire will detect the `.dwstate` state file and seamlessly resume from where it left off — no special flags needed.

**Usage:**
```bash
dropwire receive <CODE> [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|---|---|---|
| `-o, --out <DIR>` | Specify the output directory where the received payload should be saved. | `~/Downloads/Dropwire` |
| `-r, --relay <URL>` | Override the signaling relay server address. | Config file, or `ws://relay.dropwire.tyes.dev:9010` |
| `--no-lan` | Disable local network (UDP multicast) peer discovery and force WAN routing via relay. | `false` |

**Examples:**
```bash
# Basic receive into default Downloads folder
dropwire receive secret-project-123

# Receive into a specific folder
dropwire receive secret-project-123 --out ./incoming

# Receive using a local relay server
dropwire receive secret-project-123 --relay ws://127.0.0.1:9010

# Resume an interrupted transfer (just re-run the same command)
dropwire receive secret-project-123
```

---

## 📡 3. `relay`
Starts a self-hosted DropWire relay server. This server facilitates WAN discovery, SPAKE2 PAKE handshakes (via WebSockets), and handles TCP stream stapling for peers that cannot connect directly over LAN. **The relay cannot read your data.**

**Usage:**
```bash
dropwire relay [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|---|---|---|
| `--bind <IP:PORT>` | The bind address for the TCP stream stapling server. | `0.0.0.0:9009` |
| `--ws-bind <IP:PORT>` | The bind address for the WebSocket signaling server. | `0.0.0.0:9010` |

**Examples:**
```bash
# Start relay on default ports (9009 and 9010)
dropwire relay

# Start relay on custom ports
dropwire relay --bind 0.0.0.0:8000 --ws-bind 0.0.0.0:8001
```

---

## ⚙️ 4. `config`
Manages persistent CLI configuration. Settings are stored in a JSON file at the OS-specific config directory.

- **Linux/macOS:** `~/.config/dropwire/config.json`
- **Windows:** `%APPDATA%\dropwire\config.json`

**Usage:**
```bash
dropwire config <ACTION> [KEY] [VALUE]
```

**Actions:**
| Action | Description | Example |
|---|---|---|
| `show` | Display all current configuration values and config file location. | `dropwire config show` |
| `set <KEY> <VALUE>` | Set a configuration value persistently. | `dropwire config set relay ws://my-server:9010` |

**Available Keys:**
| Key | Description | Default |
|---|---|---|
| `relay` | Default relay server URL used by `send` and `receive` when `--relay` is not passed. | `ws://relay.dropwire.tyes.dev:9010` |

**Relay Resolution Priority (highest wins):**
1. `--relay` flag passed directly to the command
2. Value saved in `config.json`
3. Built-in default: `ws://relay.dropwire.tyes.dev:9010`

**Examples:**
```bash
# View current config
dropwire config show

# Set a custom default relay
dropwire config set relay wss://my-private-relay.example.com:9010
```
