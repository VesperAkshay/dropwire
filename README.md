<div align="center">
  <img src="./assets/logo.svg" alt="DropWire Logo" width="600" />
  <br/>
  <p><b>A blazingly fast, serverless, end-to-end encrypted P2P file-transfer CLI.</b></p>
  
  <p>
    <a href="https://github.com/VesperAkshay/dropwire/releases"><img src="https://img.shields.io/github/v/release/VesperAkshay/dropwire?color=00B060&label=version" alt="Version"></a>
    <a href="https://github.com/VesperAkshay/dropwire/actions"><img src="https://img.shields.io/github/actions/workflow/status/VesperAkshay/dropwire/ci.yml?branch=main&label=build&color=0052FF" alt="Build"></a>
    <a href="https://github.com/VesperAkshay/dropwire/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-Proprietary-FF3D00.svg" alt="License"></a>
  </p>
  
  <img src="./assets/demo.gif" alt="DropWire Terminal Demo" width="100%" />
</div>

DropWire lets you securely send files and massive directories of any size directly between machines over the internet using a simple code phrase. No accounts, no port-forwarding, and absolutely no limits. Inspired by `magic-wormhole` and `croc`, but architected for maximum bandwidth multiplexing and vast repository transfers.

---

## ✨ Features

- **No Accounts or Setup:** Just install and use. Authentication is based on a zero-knowledge proof derived from a shared secret code phrase.
- **Continuous Virtual Chunking:** Send 100,000 tiny files just as seamlessly (and fast) as a single 100GB video file.
- **End-to-End Encrypted (E2EE):** Every byte is symmetrically encrypted via `ChaCha20Poly1305`. The network (and the fallback relay) cannot read your data. [Read our Security Architecture](./SECURITY.md).
- **Resilient & Resume-ready:** Internet dropped at 99%? Just rerun the command. DropWire reads its deterministic `.dwstate` and resumes instantly where it left off.
- **NAT Traversal:** Attempts direct P2P first. If both users are behind strict NATs/firewalls, traffic is securely routed through a relay—but the relay remains completely blind to the contents.

## 📦 Installation

### macOS / Linux (Homebrew)
*Coming soon...*

### Windows (Scoop)
*Coming soon...*

### From Source
Ensure you have [Rust](https://rustup.rs/) installed, then run:

```bash
cargo install --git https://github.com/VesperAkshay/dropwire.git
```

## ⚡ Usage

DropWire is designed to be incredibly simple to use. Here is a quick start:

**To send a folder:**
```bash
dropwire send /path/to/your/folder
```

**To receive:**
```bash
dropwire receive 7-purple-monkey
```

📖 **[View the full Commands Reference](./COMMANDS_REFERENCE.md)** for advanced options like custom codes, parallel streams, and self-hosted relays.

## 🔒 Security Architecture

Security isn't an afterthought—it's the foundation of DropWire. We utilize SPAKE2 for zero-knowledge key exchange and ChaCha20Poly1305 for all payload encryption, paired with strict memory bounds and BLAKE3 integrity hashing.

🛡️ **[Read the complete Security Architecture document](./SECURITY.md)** for an in-depth breakdown of our cryptographic hardening.

## 🤝 Contributing

We highly encourage community feedback, bug reports, and feature requests. Please note that DropWire is closed source and we do not accept external pull requests at this time.

📝 **[Read the Contributing Guidelines](./CONTRIBUTING.md)**

## 📜 License

This software is Proprietary and Closed Source. See the **[LICENSE](./LICENSE)** file for more information.
