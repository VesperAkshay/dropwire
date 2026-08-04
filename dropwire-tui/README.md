# DropWire TUI (`dropwirex`)

A gorgeous, interactive Terminal User Interface for DropWire, a blazingly fast E2E encrypted P2P file-transfer tool.

Built entirely in Rust using `ratatui`, `dropwirex` provides a seamless visual experience over the powerful zero-copy engine of DropWire.

## Features

- **Interactive File Browser:** Navigate your file system naturally. Select multiple files and folders at once using `[Space]` for bulk batch transfers.
- **Transfer Dashboard:** Monitor active transfers with a live-updating sparkline speed chart and precise progress bars.
- **Transfer History:** Every successful and failed transfer is logged to a local `history.json` file. Press `[H]` from the main menu to review past activity.
- **Custom Themes:** The terminal should match your aesthetic. Press `[C]` to enter the Config Editor, where you can toggle between beautiful color schemes including Cyberpunk, Matrix, Nord, and Monochrome.
- **Persistent Configuration:** Edit default LAN rules, maximum parallel streams, chunk sizes, and download directories directly from the terminal UI. Configuration automatically persists to `dropwire-config.toml`.

## Controls

| Key           | Action |
|---------------|--------|
| `↑` / `↓`     | Navigate files and menus |
| `Enter`       | Open directory / Select option |
| `Backspace`   | Go up a directory / Back |
| `Space`       | Select multiple files for transfer |
| `s` or `S`    | **Send** selected file(s) |
| `r` or `R`    | Open **Receive** input menu |
| `h` or `H`    | Open **Transfer History** |
| `c` or `C`    | Open **Config Editor** |
| `Esc` or `q`  | Quit application |

## Installation

Ensure you have Rust installed. From the root workspace:

```bash
cargo install --path ./dropwire-tui
```

Run with:
```bash
dropwirex
```
