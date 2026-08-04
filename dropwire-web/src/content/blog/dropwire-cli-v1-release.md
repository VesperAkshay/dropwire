---
title: "Announcing DropWire CLI v1.0: The Future of P2P File Transfer"
description: "DropWire CLI v1.0 is officially live. Featuring automatic folder streaming, zero-configuration resume capabilities, and a new custom configuration engine."
pubDate: 2026-08-04
author: "VesperAkshay"
---

# Announcing DropWire CLI v1.0: The Future of P2P File Transfer

Today, we are thrilled to announce the official release of the **DropWire CLI**. 

We built DropWire because we were tired of the modern file-transfer status quo. Sending a 50GB project folder to a coworker shouldn't require paying for cloud storage, waiting hours for an upload to finish, and generating temporary links.

DropWire changes the paradigm. It is a completely zero-trust, end-to-end encrypted, peer-to-peer file transfer engine written in Rust.

## What's New in the Release

With this release, we have solidified DropWire as a professional, enterprise-grade tool. Here are the core capabilities you can start using today:

### 1. Automatic Folder Streaming
You no longer need to zip or tarball your directories before sending them. You can now pass an entire directory directly to the CLI:
```bash
dropwire send ./my_massive_project_folder
```
DropWire uses virtualized chunking to instantly traverse your directory structure, serialize it, and stream it securely over the wire. The receiver's client will perfectly reconstruct the folder hierarchy on their end on the fly.

### 2. Zero-Configuration Automatic Resume
Internet connections drop. Laptops go to sleep. With DropWire, failed transfers are a thing of the past. 
If your transfer dies at 99%, you do not need to restart from 0%. DropWire utilizes a highly memory-efficient cryptographic bitmap (`.dwstate`) to track every chunk you've received. 
To resume, simply run the **exact same receive command** again. DropWire will instantly negotiate with the sender to stream only the missing chunks. No special flags required.

### 3. The New Config Engine & Custom Relays
DropWire defaults to using our public signaling relay (`ws://relay.dropwire.tyes.dev:9010`) to help peers find each other across the internet. However, DropWire is designed for complete autonomy. 
You can run your own relay using the `dropwire relay` command, and then permanently save it to your CLI using our brand new config engine:
```bash
dropwire config set relay wss://my-private-relay.com
```

## Get Started
DropWire is incredibly lightweight, compiled natively, and ready to fly.

[Read the Documentation](/docs/getting-started) or [Download the CLI](/download) to securely transfer your first file in seconds.
