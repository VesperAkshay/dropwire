---
title: "Getting Started"
description: "Install the DropWire CLI and securely transfer your first file in seconds."
order: 1
---

# Getting Started with DropWire

DropWire is a command-line tool that lets you transfer files directly between computers. It eliminates the need to upload files to a third-party cloud service like Google Drive or WeTransfer, ensuring your data remains completely private and transfers as fast as your network allows.

## Installation

You can install DropWire directly from your terminal using our quick installation script:

```bash
curl -sS https://dropwire.tyes.dev/install.sh | sh
```

Verify your installation by checking the version:

```bash
dropwire --version
```

## Your First Transfer

DropWire works using a sender/receiver model. No accounts or setup are required.

### 1. Send a File

On the sending machine, specify the file (or folder) you want to share:

```bash
dropwire send ./project-assets.tar.gz
```

DropWire will lock the file, establish a secure context, and output a random, human-readable room code (e.g., `happy-dog-42`). It will then wait for a peer to connect.

### 2. Receive the File

On the receiving machine, join the session using the room code provided by the sender:

```bash
dropwire receive happy-dog-42
```

### What happens next?

1. **Discovery:** DropWire automatically searches your local network (LAN/Wi-Fi) using UDP Multicast to see if the sender is nearby. If they aren't, it falls back to a secure internet relay.
2. **Authentication:** The two machines securely authenticate each other using the room code (SPAKE2). 
3. **Transfer:** The file is streamed, verified, and saved to your local disk.
4. **Automatic Resume:** If your connection drops halfway, don't panic! Just run the exact same `receive` command again. DropWire will automatically detect the `.dropwire-partial` state file and instantly resume the transfer right where it left off.
