---
title: "System Architecture"
description: "A deep dive into the DropWire data pipeline, multiplexed transport, and memory limits."
order: 4
---

# System Architecture

DropWire isn't just a wrapper around a network socket; it is a highly optimized streaming pipeline designed to handle everything from massive 100GB video archives to directories containing hundreds of thousands of tiny files.

## The Data Pipeline

When you initiate a transfer, data flows through a strict, multi-stage pipeline:

1. **Async File I/O:** Tokio asynchronously streams bytes directly from your disk. This prevents the application from blocking the main thread, allowing for high-throughput reads on fast NVMe SSDs.
2. **Dynamic Chunking:** The stream is broken into discrete, deterministic blocks. This ensures that memory consumption remains bounded regardless of the total file size.
3. **BLAKE3 Hashing:** As chunks are created, they are hashed using the parallel BLAKE3 algorithm.
4. **Zstd Compression:** The chunk payload is compressed on the fly to reduce network bandwidth.
5. **ChaCha20 Encryption:** Finally, the compressed chunk is authenticated and symmetrically encrypted.

On the receiver's end, this pipeline runs in exact reverse: Decrypt → Decompress → Verify Hash → Write to Disk. If the BLAKE3 hash does not match, the chunk is immediately discarded.

## Multiplexed TCP Transport

Moving data over long-distance WAN connections is notoriously susceptible to packet loss, latency spikes, and bandwidth throttling. 

DropWire utilizes **Multiplexed TCP streaming**. Instead of pushing all data through a single socket, DropWire spawns multiple parallel TCP streams (defaulting to 4) between the sender and receiver. This allows the engine to bypass single-stream bandwidth limits imposed by ISPs and saturates the maximum available network capacity.

## Defeating Malicious Payloads

DropWire is designed to be completely zero-trust. You should be able to receive a file from an untrusted peer without risking your machine's stability.

- **Compression Bombs:** DropWire enforces strict expansion ratios and memory allocation bounds during the decompression phase. If a peer attempts to send a 10-byte chunk that decompresses into 10GB of garbage data, the engine immediately terminates the connection.
- **Path Traversal (Directory Escapes):** Incoming directory structures are strictly sanitized. Filenames containing `../` or absolute paths (like `/etc/passwd`) are neutralized before they can be written to disk.
- **OOM (Out-of-Memory) Attacks:** By streaming in fixed-size chunks, DropWire's memory footprint is strictly bounded. An attacker cannot force the receiver to load a massive payload entirely into RAM.
