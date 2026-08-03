# DropWire Product Context

## Overview
DropWire is a fast, encrypted, peer-to-peer file transfer engine written in Rust. It eliminates cloud server dependencies, user accounts, and file size restrictions by streaming encrypted data directly between sender and receiver endpoints over TCP multiplexed transport with SPAKE2 mutual authentication.

## Target Audience
- Developers and system administrators needing TCPk terminal file transfers.
- Technical users requiring non-custodial, end-to-end encrypted file sharing.
- Privacy-conscious individuals moving large datasets across local networks (LAN via mDNS) or zero-knowledge internet relays.

## Key Features
- **P2P TCP Transport:** Direct multiplexed TCP streaming.
- **SPAKE2 Mutual Authentication:** Unencrypted data never touches the network.
- **BLAKE3 Merkle Proofs:** Per-chunk integrity verification and instant resumability.
- **zstd Compression:** Real-time stream compression.
- **Ratatui TUI & Headless CLI:** Rich interactive terminal UI and scriptable CLI mode.
