import React, { useState } from 'react';

const STAGES = [
  {
    id: 1,
    name: 'Disk Reader',
    tech: 'Async File I/O',
    desc: 'High-throughput asynchronous file reading using Tokio to stream bytes directly from disk without blocking the main thread.',
    icon: '💾',
    color: 'bg-[#0052FF] text-white',
  },
  {
    id: 2,
    name: 'Chunker',
    tech: 'Dynamic Chunking',
    desc: 'Splits large multi-gigabyte files into deterministic chunk blocks for seamless streaming and resume functionality.',
    icon: '🧩',
    color: 'bg-[#FFB800] text-black',
  },
  {
    id: 3,
    name: 'Hasher',
    tech: 'BLAKE3 Integrity',
    desc: 'Generates ultra-fast parallel cryptographic BLAKE3 hashes for every chunk to guarantee payload integrity.',
    icon: '🔒',
    color: 'bg-[#FF3D00] text-white',
  },
  {
    id: 4,
    name: 'Compressor',
    tech: 'Zstd Compression',
    desc: 'Compresses the payload stream on the fly to maximize WAN throughput and reduce overall byte transfers.',
    icon: '⚡',
    color: 'bg-[#00B060] text-white',
  },
  {
    id: 5,
    name: 'Security Protocol',
    tech: 'SPAKE2 & ChaCha20',
    desc: 'Password-authenticated key exchange followed by ChaCha20Poly1305 symmetric encryption for all payloads.',
    icon: '🔑',
    color: 'bg-[#9D00FF] text-white',
  },
  {
    id: 6,
    name: 'Direct Transport',
    tech: 'Multiplexed TCP',
    desc: 'Streams encrypted payload frames over multiplexed TCP transport resilient to connection drops and limits.',
    icon: '📡',
    color: 'bg-[#0052FF] text-white',
  },
  {
    id: 7,
    name: 'Peer Receiver',
    tech: 'Verify & Write',
    desc: 'Verifies BLAKE3 chunk hashes, decrypts the ChaCha20 payload, and writes clean bytes directly to receiver storage.',
    icon: '📥',
    color: 'bg-[#00B060] text-white',
  },
];

export default function PipelineInteractive() {
  const [activeStage, setActiveStage] = useState(5);
  const current = STAGES.find((s) => s.id === activeStage) || STAGES[0];

  return (
    <div className="space-y-8">
      <div className="space-y-2">
        <span className="text-xs font-mono font-black uppercase text-[#0052FF] tracking-wider">
          PIPELINE SPECIFICATION
        </span>
        <h3 className="text-3xl font-black font-display text-[#0B1016] uppercase tracking-tight">
          High-Performance Zero-Trust Data Flow
        </h3>
        <p className="text-sm font-sans text-[#0B1016]/80 font-semibold max-w-xl">
          Click any stage of the streaming engine to inspect how DropWire handles data from your disk to the remote peer.
        </p>
      </div>

      {/* Stepper Node Grid */}
      <div className="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-7 gap-3">
        {STAGES.map((s) => {
          const isActive = s.id === activeStage;
          return (
            <button
              key={s.id}
              onClick={() => setActiveStage(s.id)}
              className={`p-4 rounded-2xl border-2 transition-all text-left flex flex-col justify-between h-32 shadow-sm ${
                isActive
                  ? `${s.color} border-black shadow-xl scale-105 ring-4 ring-[#FFB800]/40`
                  : 'bg-[#FFFFFF] text-[#0B1016] border-[#D9D2C9] hover:border-[#0052FF]'
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="text-2xl">{s.icon}</span>
                <span className="font-mono text-xs font-black opacity-80">
                  {s.id}
                </span>
              </div>
              <div>
                <p className="font-display font-black text-sm leading-tight uppercase">
                  {s.name}
                </p>
                <p className="text-[10px] font-mono opacity-80 mt-1 truncate">
                  Stage {s.id}
                </p>
              </div>
            </button>
          );
        })}
      </div>

      {/* Active Stage Detail Drawer */}
      <div className="bg-[#FFFFFF] p-6 sm:p-8 rounded-[2rem] border-2 border-[#0052FF] card-shadow space-y-4">
        <div className="flex items-center justify-between border-b-2 border-[#F4EFEA] pb-4">
          <div className="flex items-center gap-3">
            <span className="px-3 py-1 bg-[#0052FF] text-white text-xs font-mono font-black uppercase rounded-full">
              STAGE {current.id} OF 7
            </span>
            <h4 className="text-2xl font-black font-display text-[#0B1016] uppercase">
              {current.name}
            </h4>
          </div>
          <span className="px-4 py-1.5 bg-[#FFB800] text-black font-mono text-xs font-black rounded-full uppercase shadow-sm">
            {current.tech}
          </span>
        </div>
        <p className="text-base font-sans font-bold text-[#0B1016]/90 leading-relaxed">
          {current.desc}
        </p>
      </div>
    </div>
  );
}

