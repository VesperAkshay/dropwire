<div align="center">
  <h1>Security Architecture</h1>
  <p><b>An in-depth look at how DropWire keeps your data mathematically secure.</b></p>
  <a href="./README.md">← Back to README</a>
</div>

<br/>

Security isn't an afterthought—it is the foundational premise of DropWire. Every byte is symmetrically encrypted, authenticated, and mathematically verified. The network, your ISP, and even the DropWire relay server cannot read or tamper with your data.

## 🔑 1. Password-Authenticated Key Exchange (PAKE)
Authentication and key generation in DropWire utilize **SPAKE2** (Simple Password-Authenticated Key Exchange). 
- **Zero-Knowledge:** The password (code phrase) is never transmitted over the network in plaintext.
- **Brute-Force Immune:** SPAKE2 ensures that even if a passive attacker records the entire handshake over the signaling relay, they cannot run offline brute-force attacks against the password. The attacker is mathematically forced into executing exactly one online guess per connection attempt.
- **Ephemeral Keys:** Each session dynamically derives a unique cryptographic key.

## 🛡️ 2. Stream Encryption & Authenticated Data
Every transferred chunk and protocol control frame is encrypted using **ChaCha20Poly1305** (an Authenticated Encryption with Associated Data cipher).
- **Absolute Secrecy:** Only the Sender and Receiver hold the derived session key. 
- **Tamper-Proof:** The chunk index and file sequence metadata are bound tightly as AAD (Additional Authenticated Data). If an attacker attempts to swap, reorder, or manipulate encrypted chunks in transit, the ChaCha20Poly1305 decryption will strictly fail.

## 🧱 3. Mitigation of System Attacks
DropWire is aggressively hardened against resource exhaustion, denial-of-service, and remote code execution vulnerabilities:
- **Path Traversal Protection:** All incoming paths are sanitized. Nested directory escapes (`../../etc/passwd`) are strictly neutralized. Payloads are rigidly confined to the target output directory.
- **Memory & Storage Bounds:** DropWire enforces a hard limit of `16 MiB` on remote manifest bytes and caps the internal `.dwstate` chunk processing at `1,000,000` chunks (~1TB transfer maximum). This completely defeats compression bombs and memory exhaustion attacks.
- **Integer Overflow Immunity:** Safe 64-bit casting protects against 32-bit platform wrapping exploits during manifest allocations.
- **Relay Hardening:** The DropWire Relay server utilizes a concurrent `Semaphore` lock to enforce a strict boundary on accepted TCP connections, neutralizing SYN floods and file-descriptor exhaustion.

## 🔍 4. Integrity & Hashing
- We use **BLAKE3** to mathematically verify every single chunk in real-time as it arrives. 
- When a transfer completes, an overall directory/file BLAKE3 hash is verified against the initial signed manifest. If there is a single byte discrepancy, the CLI throws a Hash Mismatch error and safely rejects the transfer.
