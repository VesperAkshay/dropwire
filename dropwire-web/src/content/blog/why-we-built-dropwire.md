---
title: "Why I Built DropWire: No More Cloud, No More Waiting"
description: "I'm Akshay Patel, and I built DropWire — a Rust-powered, peer-to-peer encrypted file transfer tool — because I was tired of sending my private files through servers I don't control."
pubDate: 2026-08-01
author: "Akshay Patel"
---

# Why I Built DropWire

My name is Akshay Patel, and I built DropWire.

It started with a simple frustration. I was trying to send a large video file to a friend sitting right next to me. Every tool I tried forced me to choose between convenience and privacy — upload to Google Drive, paste a WeTransfer link, hope the file isn't too large. My data was bouncing off servers in data centers I'll never see, owned by companies I'll never audit.

That felt wrong. It still feels wrong.

## The Problem I Was Solving

Cloud-based file sharing has three problems that nobody talks about honestly:

1. **Your data leaves your machine.** Even if it's encrypted in transit, it sits on someone else's server.
2. **You're dependent on internet speed**, even when the person you're sharing with is two meters away.
3. **You need accounts, links, and permissions** just to hand someone a file.

File transfer should be as simple and direct as handing someone a USB drive. Except faster. And encrypted.

## The Solution: DropWire

I wrote DropWire in Rust because performance and safety are non-negotiable at the transport layer. The architecture is straightforward but powerful:

- **SPAKE2 key exchange** — a cryptographic handshake that generates a unique ephemeral session key. Even if someone intercepts the connection, they get nothing useful.
- **Direct P2P streaming** — when both machines are on the same network, the data never leaves the LAN.
- **Zero-knowledge relay** — when you're on different networks, a relay server forwards the encrypted stream. The relay sees only ciphertext. It never knows what you're sending.
- **BLAKE3 integrity proofs** — every chunk is verified. If a transfer breaks, it resumes exactly where it stopped.

## Why "DropWire"?

The name is literal. You drop files over a wire — a direct, encrypted connection between two machines. No detours.

## Where It's Going

DropWire started as a CLI tool. Right now it's open-source and free to use. I'm building toward a full Desktop App with a drag-and-drop interface so anyone — not just developers — can use it.

Eventually, DropWire will have a commercial tier for teams and enterprises. But the core mission never changes: your files, your machines, nobody else in between.

If you want to try it today, [download the CLI](/download) or [read the docs](/docs).

— Akshay Patel ([@VesperAkshay](https://github.com/VesperAkshay))
