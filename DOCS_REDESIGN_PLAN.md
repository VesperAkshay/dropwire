# Docs Page Redesign — Complete Plan

> Full specification for the new `/docs` CLI + TUI unified documentation page.
> Stack: Astro 7 + React 19 + Tailwind CSS v4 + Framer Motion

---

## 1. Big Picture Concept

Replace the current flat docs grid (which just links to markdown articles) with a
**single, immersive, interactive docs page** that lives at `/docs`.

The page is split into two major modes:

```
┌──────────────────────────────────────────────────────────────┐
│                     DropWire Docs                            │
│                                                              │
│           ┌────────────┐    ┌─────────────┐                 │
│           │  CLI  ●    │    │     TUI     │   ← Slider/Tab  │
│           └────────────┘    └─────────────┘                 │
│                                                              │
│  [ Left Sidebar Nav ]   [ Main Content Area ]               │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

When the user slides to **CLI**: every CLI command, flag, install guide, config docs.
When the user slides to **TUI**: every screen, keybinding, theme, config option.

The slider animates content in/out using Framer Motion.
The left sidebar is a sticky nav that deep-links to every section on the page.

---

## 2. Page Layout Architecture

```
/docs
├── PageHero                  (Title + Pill Tags + Slider Toggle)
│
├── SidebarNav (sticky left)  (Auto-generated from section IDs)
│
└── MainContent (right)
    │
    ├── [CLI MODE]
    │   ├── Section: Installation
    │   ├── Section: Quick Start
    │   ├── Section: send command
    │   ├── Section: receive command
    │   ├── Section: relay command
    │   ├── Section: config command
    │   ├── Section: Global Flags
    │   ├── Section: Configuration File
    │   ├── Section: LAN Discovery
    │   ├── Section: Relay Fallback
    │   └── Section: Resumable Transfers
    │
    └── [TUI MODE]
        ├── Section: Installation
        ├── Section: Launching the TUI
        ├── Section: Boot Animation
        ├── Section: File Browser
        ├── Section: Sidebar (Places)
        ├── Section: Receive Screen
        ├── Section: Transfer Dashboard
        ├── Section: Config Editor
        ├── Section: History Viewer
        ├── Section: All Keybindings
        └── Section: Themes
```

---

## 3. Slider Toggle Component (The Hero Feature)

### Design:
- A large, pill-shaped toggle with two segments: **CLI** and **TUI**
- The active segment has a solid filled background (Units Blue `#0052FF`)
- When switched, Framer Motion animates the background pill sliding across
- Below the toggle, a `framer-motion` `AnimatePresence` fades/slides the content out and the new content in (slide from left to right for CLI→TUI, slide right to left for TUI→CLI)
- The URL updates: `/docs?mode=cli` and `/docs?mode=tui` so links are shareable

### Behaviour:
- Default mode: `cli`
- Remembers last choice in `localStorage`
- Deep anchor links like `/docs?mode=tui#keybindings` jump directly to that section

---

## 4. Left Sidebar Navigation

### Design:
- Sticky, 260px wide, on the left
- Collapses into a floating bottom drawer on mobile
- Sections auto-highlighted as you scroll (IntersectionObserver)
- Grouped by category with bold group headers

### CLI Sidebar Groups:
```
▸ Getting Started
    Installation
    Quick Start

▸ Commands
    send
    receive
    relay
    config

▸ Concepts
    LAN Discovery
    Relay Fallback
    Resumable Transfers
    Git-Ignore Support
    Encryption Model

▸ Configuration
    Config File Location
    All Config Keys
    Priority Hierarchy
```

### TUI Sidebar Groups:
```
▸ Getting Started
    Installation
    Launching

▸ Screens
    Boot Animation
    File Browser
    Receive Input
    Transfer Dashboard
    Config Editor
    History Viewer

▸ Controls
    All Keybindings

▸ Customization
    Themes
    Config Options
```

---

## 5. Full CLI Content Specification

### Section: Installation

**Windows:**
```powershell
# GitHub Releases (recommended)
winget install dropwire

# Or download binary from:
# https://github.com/VesperAkshay/dropwire/releases/latest
```

**macOS / Linux:**
```bash
curl -sSL https://install.dropwire.dev | sh

# Or with Cargo (build from source):
cargo install dropwire
```

**Verify install:**
```bash
dropwire --version
```

---

### Section: Quick Start

```bash
# SENDER (Machine A)
dropwire send ./photo.jpg
# Output: Code: 7-guitar-revenge

# RECEIVER (Machine B)
dropwire receive 7-guitar-revenge
# File saved to ~/Downloads/Dropwire/
```

Callout box: *Files are E2E encrypted. The relay server never sees the content.*

---

### Section: `send` Command

**Syntax:**
```bash
dropwire send <FILE_OR_DIR> [OPTIONS]
```

**Arguments:**

| Argument | Type | Required | Description |
|---|---|---|---|
| `FILE_OR_DIR` | Path | ✅ Yes | File or folder to transfer |

**Flags:**

| Flag | Short | Type | Default | Description |
|---|---|---|---|---|
| `--code` | `-c` | String | Auto-generated | Custom room code (`{n}-{word}-{word}` format) |
| `--streams` | `-s` | Number | `4` | Number of parallel TCP streams |
| `--relay` | `-r` | URL | Config value | Override relay server URL |
| `--no-lan` | — | Bool | `false` | Skip LAN discovery, force relay routing |

**Examples:**
```bash
# Send a file (auto code)
dropwire send report.pdf

# Send with custom code
dropwire send report.pdf --code my-secret-code

# Send a folder using 8 parallel streams
dropwire send ./project/ --streams 8

# Force relay (no LAN attempt)
dropwire send video.mp4 --no-lan

# Self-hosted relay
dropwire send photo.jpg --relay ws://192.168.1.10:9010
```

**Git Integration callout:**
When sending a directory, DropWire automatically reads `.gitignore` and `.ignore` files
and skips build artifacts (`node_modules/`, `target/`, `.git/`, `dist/`).

---

### Section: `receive` Command

**Syntax:**
```bash
dropwire receive <CODE> [OPTIONS]
```

**Arguments:**

| Argument | Type | Required | Description |
|---|---|---|---|
| `CODE` | String | ✅ Yes | Code phrase from sender |

**Flags:**

| Flag | Short | Type | Default | Description |
|---|---|---|---|---|
| `--out` | `-o` | Path | `~/Downloads/Dropwire` | Output directory |
| `--relay` | `-r` | URL | Config value | Override relay server URL |
| `--no-lan` | — | Bool | `false` | Skip LAN discovery |

**Examples:**
```bash
# Basic receive
dropwire receive 7-guitar-revenge

# Custom output directory
dropwire receive 7-guitar-revenge --out ~/Desktop

# Resume an interrupted transfer (run same command again)
dropwire receive 7-guitar-revenge
# DropWire auto-detects .dwstate and resumes from where it left off
```

**Resumability callout:**
If a transfer is interrupted (network drop, power loss), running the exact same
`receive` command in the same directory automatically resumes from where it stopped.
No data re-downloaded.

---

### Section: `relay` Command

**Syntax:**
```bash
dropwire relay [OPTIONS]
```

**Flags:**

| Flag | Type | Default | Description |
|---|---|---|---|
| `--bind` | IP:PORT | `0.0.0.0:9009` | TCP stream server bind address |
| `--ws-bind` | IP:PORT | `0.0.0.0:9010` | WebSocket signaling server address |

**Examples:**
```bash
# Run relay on default ports
dropwire relay

# Custom ports
dropwire relay --bind 0.0.0.0:5001 --ws-bind 0.0.0.0:5002

# Run with Docker (example)
docker run -p 9009:9009 -p 9010:9010 dropwire/relay
```

**Zero-Knowledge callout:**
The relay server only routes SPAKE2 handshake tokens and encrypted frames.
It **cannot** read, decrypt, or log file content or filenames.

---

### Section: `config` Command

**Syntax:**
```bash
dropwire config <ACTION> [KEY] [VALUE]
```

**Actions:**

| Action | Description |
|---|---|
| `show` | Print all current config values and config file location |
| `set <KEY> <VALUE>` | Persist a setting to config file |

**Config Keys:**

| Key | Type | Default | Description |
|---|---|---|---|
| `relay` | URL | `ws://relay.dropwire.tyes.dev:9010` | Default relay server |
| `no_lan` | bool | `false` | Always skip LAN discovery |
| `download_dir` | Path | `~/Downloads/Dropwire` | Default download folder |
| `default_mode` | String | `browser` | TUI start mode (`browser` or `receive`) |
| `parallel_streams` | Number | `4` | Default stream count |
| `chunk_size_kb` | Number | `1024` | Chunk size in KB |
| `theme` | String | `cyberpunk` | TUI theme (`cyberpunk`, `matrix`, `nord`, `monochrome`) |

**Examples:**
```bash
# Show all current settings
dropwire config show

# Set a self-hosted relay
dropwire config set relay ws://192.168.1.10:9010

# Always use 8 streams
dropwire config set parallel_streams 8

# Disable LAN by default
dropwire config set no_lan true

# Set default download folder
dropwire config set download_dir ~/Desktop/received
```

**Config File Location:**
- **Linux/macOS**: `~/.config/dropwire/config.json`
- **Windows**: `%APPDATA%\dropwire\config.json`

---

### Section: LAN Discovery

Diagram card explaining how LAN discovery works:
```
Machine A (Sender)                 Machine B (Receiver)
      │                                    │
      ├──── UDP Multicast Announce ────────►│
      │                                    │
      │◄─── TCP Direct Connection ─────────┤
      │                                    │
      └══════ Direct P2P (No Relay) ═══════┘
```

- DropWire binds a local TCP listener and announces over UDP multicast
- If a peer is found on LAN within 15 seconds → Direct connection (maximum speed, zero relay)
- If not → Automatic fallback to relay

---

### Section: Relay Fallback

Priority order when connecting:
1. **LAN Direct** — UDP multicast → Direct TCP (fastest)
2. **Relay** — Encrypted frames routed through relay (works everywhere)

Override with `--no-lan` to skip step 1.

---

### Section: Resumable Transfers

- When receiving, DropWire saves a `.dwstate` file tracking every received chunk
- State file contains a BLAKE3 hash per chunk for integrity verification
- On resume: only missing chunks are re-requested from sender
- `.dwstate` is deleted automatically upon successful transfer completion

---

## 6. Full TUI Content Specification

### Section: Installation

**Windows:**
```powershell
# Download from GitHub Releases:
# https://github.com/VesperAkshay/dropwire/releases/latest
# Download: dropwirex-windows-x64.exe
# Rename to dropwirex.exe and add to PATH
```

**macOS / Linux:**
```bash
curl -sSL https://install.dropwire.dev/tui | sh

# Or build from source:
git clone https://github.com/VesperAkshay/dropwire.git
cd dropwire-tui
cargo build --release
```

---

### Section: Launching the TUI

```bash
dropwirex
```

That's it. The TUI auto-reads `~/.config/dropwire/config.json` and launches
in the mode set by `default_mode` (default: `browser`).

---

### Section: Boot Animation

On launch, DropWire X plays a cinematic particle assembly intro:

- Hundreds of scattered Braille dots animate across the screen
- Over 2.5 seconds, they assemble using cubic easing into the **DROPWIRE X** ASCII logo
- At 2.5s the logo snaps into crisp, perfectly rendered text
- Holds for 1 second then transitions into the File Browser
- Press **any key** to skip immediately *(coming soon)*

---

### Section: File Browser

The main screen. Two-pane layout:

```
┌──────────────┬─────────────────────────────────┐
│  PLACES      │  EXPLORER                       │
│              │                                 │
│  🏠 Home     │  📁 Documents                   │
│  🖥 Desktop  │  📁 Downloads                   │
│  📄 Docs     │  📁 node_modules                │
│  ⬇ Downloads │  📄 report.pdf                  │
│  🖼 Pictures │  📄 photo.jpg                   │
│  🎵 Music    │                                 │
│  🎬 Videos   │                                 │
│  ──────────  │                                 │
│  💾 C:\      │                                 │
│  💾 D:\      │                                 │
└──────────────┴─────────────────────────────────┘
```

**Sidebar (Places):**
- Home directory
- Desktop, Documents, Downloads, Pictures, Music, Videos
- All detected Windows drives (C:\, D:\, E:\...) or `/` on Unix
- Press `←` / `Tab` to focus sidebar, navigate with `↑`/`↓`, `Enter` to jump

**File Explorer:**
- Directories sorted first, then files
- `..` (Parent Directory) at top for navigating up
- Press `→` or `Tab` to focus file list
- Icons: 📁 Directories, 📄 Files
- Active pane border glows with animated RGB color

---

### Section: Receive Input Screen

Activated with `r` / `R` from the File Browser.

- Large centered input field with animated border
- Animated Braille spinner shows the app is ready (`⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`)
- Blinking block cursor (`█`) in the input field
- Placeholder: `e.g. 7-guitar-revenge`
- Press `Enter` to connect and start receiving
- Press `Esc` to cancel back to File Browser

---

### Section: Transfer Dashboard

Shown during both send and receive operations.

**Displays:**
- 🟡 `SENDING MODE` (Gold) or 🔵 `RECEIVING MODE` (Cyan) badge
- Animated activity spinner
- Room code phrase (e.g. `7-guitar-revenge`)
- Real-time engine status (`Connecting...`, `Transferring...`, `Verifying...`)
- Live progress bar with completion percentage and speed (`MB/s` or `KB/s`)
- Verified bytes counter vs total bytes

**Controls:**
- `Esc` — Abort transfer and return to File Browser

---

### Section: Config Editor

Activated with `c` / `C` from the File Browser.

**Editable settings:**

| Setting | Type | Default | Description |
|---|---|---|---|
| Relay URL | Text | `ws://relay.dropwire.tyes.dev:9010` | Relay server address |
| Disable LAN | Toggle | Off | Force relay routing |
| Download Directory | Text | `~/Downloads/Dropwire` | Where received files go |
| Default Mode | Cycle | `browser` | Start in browser or receive mode |
| Parallel Streams | Number | `4` | TCP stream count (4–16) |
| Chunk Size (KB) | Number | `1024` | Per-chunk transfer size |
| Theme | Cycle | `cyberpunk` | Color theme |

**Controls:**
- `↑` / `↓` — Navigate setting rows
- `Enter` — Edit text field / Toggle boolean / Cycle option
- `Esc` — Save all settings and return to File Browser (auto-writes to config file)

---

### Section: History Viewer

Activated with `h` / `H` from the File Browser.

Displays a scrollable log of all past transfers:

| Date | Direction | File | Size | Speed |
|---|---|---|---|---|
| 2026-08-05 23:10 | ⬆ SENT | report.pdf | 12 MB | 45 MB/s |
| 2026-08-04 18:22 | ⬇ RECV | photo.zip | 320 MB | 38 MB/s |

**Controls:**
- `↑` / `↓` — Scroll history
- `Esc` / `q` — Return to File Browser

History is persisted at `~/.config/dropwire/history.json`.

---

### Section: All Keybindings

**Full keybinding reference table — every key, every screen.**

| Screen | Key | Action |
|---|---|---|
| All screens | `q` / `Q` | Quit application |
| All screens | `Esc` | Back / Cancel / Abort / Save & Exit |
| File Browser | `↑` / `↓` | Navigate in active pane |
| File Browser | `←` | Focus Places sidebar |
| File Browser | `→` | Focus Explorer file list |
| File Browser | `Tab` | Toggle between sidebar and file list |
| File Browser | `Enter` | Open directory / Jump to sidebar location |
| File Browser | `Space` | Select / deselect file for batch transfer |
| File Browser | `s` / `S` | Send selected file(s) |
| File Browser | `r` / `R` | Open Receive Input screen |
| File Browser | `h` / `H` | Open History viewer |
| File Browser | `c` / `C` | Open Config editor |
| Receive Input | `Any char` | Type room code |
| Receive Input | `Backspace` | Delete last character |
| Receive Input | `Enter` | Submit code and start transfer |
| Receive Input | `Esc` | Cancel, return to File Browser |
| Transfer Dashboard | `Esc` | Abort transfer, return to File Browser |
| Config Editor | `↑` / `↓` | Navigate setting rows |
| Config Editor | `Enter` | Edit field / Toggle / Cycle option |
| Config Editor | `Char` | Type while editing a text field |
| Config Editor | `Backspace` | Delete while editing a text field |
| Config Editor | `Esc` | Save all settings, return to File Browser |
| History Viewer | `↑` / `↓` | Scroll history entries |
| History Viewer | `Esc` / `q` | Return to File Browser |

---

### Section: Themes

Four built-in visual themes selectable from Config Editor or via `dropwire config set theme <name>`.

| Theme | Primary | Accent | Feel |
|---|---|---|---|
| `cyberpunk` | Gold `#FFB800` | Lavender `#AA90B3` | Neon noir terminal |
| `matrix` | Green `#00FF00` | Dark Green `#009600` | Hacker / The Matrix |
| `nord` | Arctic Blue `#88C0D0` | Slate `#81A1C1` | Clean, cool, minimal |
| `monochrome` | White `#FFFFFF` | Gray `#CCCCCC` | High-contrast grayscale |

All themes feature **animated RGB border pulsing** on active panes —
the border color slowly breathes and shifts hue using a real-time sine wave.

---

## 7. Component Breakdown (What to Build)

| Component | File | Description |
|---|---|---|
| `DocsPage.astro` | `src/pages/docs/index.astro` | Main page shell |
| `ModeSlider.tsx` | `src/components/docs/ModeSlider.tsx` | CLI / TUI animated slider toggle |
| `DocsSidebar.tsx` | `src/components/docs/DocsSidebar.tsx` | Sticky left nav with scroll spy |
| `CliDocs.tsx` | `src/components/docs/CliDocs.tsx` | All CLI content sections |
| `TuiDocs.tsx` | `src/components/docs/TuiDocs.tsx` | All TUI content sections |
| `CodeBlock.tsx` | `src/components/docs/CodeBlock.tsx` | Syntax-highlighted code block with copy button |
| `FlagTable.tsx` | `src/components/docs/FlagTable.tsx` | Reusable CLI flags table |
| `KeybindTable.tsx` | `src/components/docs/KeybindTable.tsx` | Keybinding reference table |
| `CalloutBox.tsx` | `src/components/docs/CalloutBox.tsx` | Info / Warning / Tip callout cards |

---

## 8. Animations Plan (Framer Motion)

| Interaction | Animation |
|---|---|
| CLI → TUI switch | Content slides out left, new content slides in from right (300ms ease) |
| TUI → CLI switch | Content slides out right, new content slides in from left (300ms ease) |
| Page load | Sections fade-in staggered (50ms delay per section) |
| Sidebar nav click | Smooth scroll to section with offset for sticky header |
| Sidebar active indicator | Animated left border slides between active items |
| Code block copy | Button shows ✓ checkmark for 2 seconds |

---

## 9. Mobile Responsiveness

- Below `768px`: Sidebar collapses into a bottom drawer with a toggle button
- Slider toggle shrinks to fit mobile width
- Code blocks scroll horizontally
- Tables become scrollable cards on small screens

---

## 10. File Breakdown — What Changes vs What's New

| File | Change Type | Action |
|---|---|---|
| `src/pages/docs/index.astro` | 🔄 Replace | Full rewrite as interactive page |
| `src/pages/docs/[...id].astro` | ✅ Keep | Existing article reader stays for old links |
| `src/content/docs/cli.md` | ✅ Keep | Can keep for SEO / old links |
| `src/components/docs/` | ✨ New folder | All new React components |
| `src/lib/site.ts` | 🔄 Minor edit | Update nav to reflect new docs page |

---

*Plan written: 2026-08-06*
*Ready to implement on approval.*
