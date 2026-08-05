import React from 'react';
import { CodeBlock } from './CodeBlock';
import { FlagTable } from './FlagTable';
import { CalloutBox } from './CalloutBox';
import type { SidebarGroup } from './DocsSidebar';

export const cliSidebarGroups: SidebarGroup[] = [
  {
    title: 'Getting Started',
    items: [
      { id: 'cli-installation', label: 'Installation' },
      { id: 'cli-quickstart', label: 'Quick Start' }
    ]
  },
  {
    title: 'Commands',
    items: [
      { id: 'cli-send', label: 'dropwire send' },
      { id: 'cli-receive', label: 'dropwire receive' },
      { id: 'cli-relay', label: 'dropwire relay' },
      { id: 'cli-config', label: 'dropwire config' }
    ]
  },
  {
    title: 'Concepts',
    items: [
      { id: 'cli-lan', label: 'LAN Discovery' },
      { id: 'cli-fallback', label: 'Relay Fallback' },
      { id: 'cli-resume', label: 'Resumable Transfers' }
    ]
  }
];

export function CliDocs() {
  return (
    <div className="space-y-16 pb-32">
      {/* ───────────────────────────────────────────────────────── */}
      <section id="cli-installation">
        <h2 className="text-3xl font-display font-black uppercase text-[#0B1016] mb-4">Installation</h2>
        <p className="text-lg font-medium text-[#0B1016]/80 mb-4">
          Install the DropWire CLI on your machine to start sending and receiving files instantly from your terminal.
        </p>
        
        <h3 className="text-xl font-bold mt-8 mb-2">Windows</h3>
        <CodeBlock code={`# Install via PowerShell
irm https://install.dropwire.dev/windows | iex`} />

        <h4 className="text-lg font-bold mt-4 mb-2 text-[#0B1016]/80">Uninstall (Windows)</h4>
        <CodeBlock code={`irm https://install.dropwire.dev/windows-uninstall | iex`} />

        <h3 className="text-xl font-bold mt-8 mb-2">macOS / Linux</h3>
        <CodeBlock code={`# Install via curl
curl -sSL https://install.dropwire.dev | sh

# Or with Cargo (build from source):
cargo install dropwire`} />

        <h4 className="text-lg font-bold mt-4 mb-2 text-[#0B1016]/80">Uninstall (macOS / Linux)</h4>
        <CodeBlock code={`curl -sSL https://install.dropwire.dev/uninstall | sh`} />
      </section>

      {/* ───────────────────────────────────────────────────────── */}
      <section id="cli-quickstart">
        <h2 className="text-3xl font-display font-black uppercase text-[#0B1016] mb-4">Quick Start</h2>
        <p className="text-lg font-medium text-[#0B1016]/80 mb-4">
          Sending a file is as simple as running one command. DropWire will automatically generate a memorable room code for the receiver.
        </p>
        
        <h3 className="text-xl font-bold mt-8 mb-2">Machine A (Sender)</h3>
        <CodeBlock code={`dropwire send ./photo.jpg
# Output: Code: 7-guitar-revenge`} />

        <h3 className="text-xl font-bold mt-8 mb-2">Machine B (Receiver)</h3>
        <CodeBlock code={`dropwire receive 7-guitar-revenge
# File saved to ~/Downloads/Dropwire/`} />

        <CalloutBox type="info" title="Zero-Knowledge Encryption">
          All files are End-to-End Encrypted using ChaCha20Poly1305. The relay server (if used) routes encrypted frames but never sees your filenames or file content.
        </CalloutBox>
      </section>

      {/* ───────────────────────────────────────────────────────── */}
      <section id="cli-send">
        <h2 className="text-3xl font-display font-black uppercase text-[#0B1016] mb-4">dropwire send</h2>
        <p className="text-lg font-medium text-[#0B1016]/80 mb-4">
          Sends a file or directory over an encrypted P2P channel. When sending a directory, DropWire automatically respects <code>.gitignore</code> and <code>.ignore</code> files to skip build artifacts.
        </p>

        <CodeBlock code={`dropwire send <FILE_OR_DIR> [OPTIONS]`} />

        <h3 className="text-xl font-bold mt-8 mb-2">Flags</h3>
        <FlagTable flags={[
          { flag: '--code', short: '-c', type: 'String', defaultVal: 'Auto-generated', desc: 'Custom room code phrase (e.g. secret-code-123)' },
          { flag: '--streams', short: '-s', type: 'Number', defaultVal: '4', desc: 'Number of parallel TCP streams to multiplex' },
          { flag: '--relay', short: '-r', type: 'URL', defaultVal: 'Config value', desc: 'Override the signaling relay server URL' },
          { flag: '--no-lan', type: 'Boolean', defaultVal: 'false', desc: 'Skip local UDP peer discovery and force WAN routing via relay' }
        ]} />

        <h3 className="text-xl font-bold mt-8 mb-2">Examples</h3>
        <CodeBlock code={`# Send a folder using 8 parallel streams
dropwire send ./project/ --streams 8

# Send with a custom code
dropwire send report.pdf --code top-secret

# Force relay (no LAN attempt)
dropwire send video.mp4 --no-lan`} />
      </section>

      {/* ───────────────────────────────────────────────────────── */}
      <section id="cli-receive">
        <h2 className="text-3xl font-display font-black uppercase text-[#0B1016] mb-4">dropwire receive</h2>
        <p className="text-lg font-medium text-[#0B1016]/80 mb-4">
          Connects to a sender using a room code phrase and downloads the payload.
        </p>

        <CodeBlock code={`dropwire receive <CODE> [OPTIONS]`} />

        <h3 className="text-xl font-bold mt-8 mb-2">Flags</h3>
        <FlagTable flags={[
          { flag: '--out', short: '-o', type: 'Path', defaultVal: '~/Downloads/Dropwire', desc: 'Output directory destination' },
          { flag: '--relay', short: '-r', type: 'URL', defaultVal: 'Config value', desc: 'Override the signaling relay server URL' },
          { flag: '--no-lan', type: 'Boolean', defaultVal: 'false', desc: 'Skip local UDP peer discovery' }
        ]} />
      </section>

      {/* ───────────────────────────────────────────────────────── */}
      <section id="cli-relay">
        <h2 className="text-3xl font-display font-black uppercase text-[#0B1016] mb-4">dropwire relay</h2>
        <p className="text-lg font-medium text-[#0B1016]/80 mb-4">
          Runs a self-hosted zero-knowledge signaling and TCP stream relay server for when LAN discovery isn't possible.
        </p>

        <CodeBlock code={`dropwire relay [OPTIONS]`} />

        <h3 className="text-xl font-bold mt-8 mb-2">Flags</h3>
        <FlagTable flags={[
          { flag: '--bind', type: 'IP:PORT', defaultVal: '0.0.0.0:9009', desc: 'TCP stream server bind address' },
          { flag: '--ws-bind', type: 'IP:PORT', defaultVal: '0.0.0.0:9010', desc: 'WebSocket signaling server address' }
        ]} />
      </section>

      {/* ───────────────────────────────────────────────────────── */}
      <section id="cli-config">
        <h2 className="text-3xl font-display font-black uppercase text-[#0B1016] mb-4">dropwire config</h2>
        <p className="text-lg font-medium text-[#0B1016]/80 mb-4">
          Manages persistent CLI configuration stored in your OS config directory.
        </p>

        <CodeBlock code={`dropwire config show
dropwire config set <KEY> <VALUE>`} />

        <h3 className="text-xl font-bold mt-8 mb-2">Keys</h3>
        <FlagTable flags={[
          { flag: 'relay', type: 'URL', defaultVal: 'ws://relay.dropwire.tyes.dev:9010', desc: 'Default relay server' },
          { flag: 'no_lan', type: 'Boolean', defaultVal: 'false', desc: 'Always skip LAN discovery' },
          { flag: 'download_dir', type: 'Path', defaultVal: '~/Downloads/Dropwire', desc: 'Default download folder' },
          { flag: 'parallel_streams', type: 'Number', defaultVal: '4', desc: 'Default stream count' },
          { flag: 'chunk_size_kb', type: 'Number', defaultVal: '1024', desc: 'Chunk size in KB' }
        ]} />
      </section>

      {/* ───────────────────────────────────────────────────────── */}
      <section id="cli-lan">
        <h2 className="text-3xl font-display font-black uppercase text-[#0B1016] mb-4">LAN Discovery</h2>
        <p className="text-lg font-medium text-[#0B1016]/80 mb-4">
          DropWire always attempts to find peers locally before using the relay server to guarantee maximum possible speed.
        </p>
        <CodeBlock code={`Machine A (Sender)                 Machine B (Receiver)
      │                                    │
      ├──── UDP Multicast Announce ────────►│
      │                                    │
      │◄─── TCP Direct Connection ─────────┤
      │                                    │
      └══════ Direct P2P (No Relay) ═══════┘`} language="text" />
      </section>

      {/* ───────────────────────────────────────────────────────── */}
      <section id="cli-fallback">
        <h2 className="text-3xl font-display font-black uppercase text-[#0B1016] mb-4">Relay Fallback</h2>
        <p className="text-lg font-medium text-[#0B1016]/80 mb-4">
          If LAN discovery times out after 15 seconds (or fails due to a firewall), DropWire automatically cascades to the Relay server via TCP. You can force the relay immediately by passing <code>--no-lan</code>.
        </p>
      </section>

      {/* ───────────────────────────────────────────────────────── */}
      <section id="cli-resume">
        <h2 className="text-3xl font-display font-black uppercase text-[#0B1016] mb-4">Resumable Transfers</h2>
        <p className="text-lg font-medium text-[#0B1016]/80 mb-4">
          When receiving, DropWire saves a <code>.dwstate</code> file tracking every received chunk with a BLAKE3 hash. 
        </p>
        <CalloutBox type="tip" title="Zero-Config Resume">
          If a transfer is interrupted, simply run the exact same <code>receive</code> command in the same directory. DropWire will auto-detect the <code>.dwstate</code> file and resume downloading only the missing chunks!
        </CalloutBox>
      </section>
      
    </div>
  );
}
