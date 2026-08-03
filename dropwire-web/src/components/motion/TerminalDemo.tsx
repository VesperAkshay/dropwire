import React, { useState, useEffect } from 'react';

export default function TerminalDemo() {
  const [tab, setTab] = useState<'sender' | 'receiver'>('sender');
  const [copied, setCopied] = useState(false);
  const [progress, setProgress] = useState(65);

  useEffect(() => {
    const timer = setInterval(() => {
      setProgress((prev) => (prev >= 100 ? 20 : prev + 5));
    }, 1200);
    return () => clearInterval(timer);
  }, []);

  const handleCopy = () => {
    navigator.clipboard.writeText('cargo install DropWire-cli');
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="w-full bg-[#FFFFFF] border-2 border-[#0052FF] rounded-[2.5rem] p-6 shadow-2xl space-y-6">
      {/* Terminal Window Header */}
      <div className="flex items-center justify-between border-b-2 border-[#F4EFEA] pb-4">
        <div className="flex items-center gap-2">
          <div className="w-3.5 h-3.5 rounded-full bg-[#FF3D00]" />
          <div className="w-3.5 h-3.5 rounded-full bg-[#FFB800]" />
          <div className="w-3.5 h-3.5 rounded-full bg-[#00B060]" />
          <span className="ml-3 font-mono text-xs font-black text-[#0B1016]">
            DropWire — interactive terminal engine
          </span>
        </div>

        {/* Sender / Receiver Toggle Tabs */}
        <div className="flex items-center gap-1 bg-[#F4EFEA] p-1.5 rounded-full border border-[#D9D2C9]">
          <button
            onClick={() => setTab('sender')}
            className={`px-4 py-1.5 rounded-full text-xs font-display font-black transition-all ${
              tab === 'sender'
                ? 'bg-[#0052FF] text-white shadow-md'
                : 'text-[#0B1016]/70 hover:text-black'
            }`}
          >
            Sender Mode
          </button>
          <button
            onClick={() => setTab('receiver')}
            className={`px-4 py-1.5 rounded-full text-xs font-display font-black transition-all ${
              tab === 'receiver'
                ? 'bg-[#0052FF] text-white shadow-md'
                : 'text-[#0B1016]/70 hover:text-black'
            }`}
          >
            Receiver Mode
          </button>
        </div>
      </div>

      {/* Terminal Console Surface */}
      <div className="bg-[#F4EFEA] border-2 border-[#D9D2C9] rounded-[2rem] p-6 font-mono text-xs text-[#0B1016] space-y-5">
        {tab === 'sender' ? (
          <>
            <div className="flex items-center gap-2">
              <span className="text-[#0052FF] font-black text-sm">$</span>
              <span className="font-bold text-[#0B1016] text-sm">DropWire send ./archive-2026.tar.zst</span>
            </div>

            <div className="bg-[#FFFFFF] p-4 rounded-2xl border-2 border-[#FFB800] space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-[#0B1016]/70 uppercase tracking-wider">Generated Room Code</span>
                <span className="px-3 py-1 bg-[#00B060] text-white text-[10px] font-black uppercase rounded-full">
                  Active P2P Listener
                </span>
              </div>
              <p className="font-mono text-xl sm:text-2xl font-black text-[#0052FF]">
                happy-dog-42
              </p>
              <div className="text-[11px] text-[#0B1016]/70 flex flex-wrap gap-4 font-bold">
                <span>Protocol: Direct Encrypted P2P</span>
                <span>·</span>
                <span>Integrity: Cryptographically Verified</span>
              </div>
            </div>

            {/* Live Progress Bar */}
            <div className="space-y-2 pt-2">
              <div className="flex justify-between text-xs font-bold">
                <span className="text-[#0B1016]">Streaming P2P Data...</span>
                <span className="text-[#0052FF] font-black">{progress}%</span>
              </div>
              <div className="w-full bg-[#EBE5DC] h-4 rounded-full overflow-hidden p-0.5 border border-[#D9D2C9]">
                <div
                  className="bg-[#0052FF] h-full rounded-full transition-all duration-500 shadow-lg"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <div className="flex justify-between text-[11px] font-bold text-[#0B1016]/70 pt-1">
                <span>Speed: 142.5 MB/s</span>
                <span>Transferred: 273.0 / 420.0 MB</span>
              </div>
            </div>
          </>
        ) : (
          <>
            <div className="flex items-center gap-2">
              <span className="text-[#FF3D00] font-black text-sm">$</span>
              <span className="font-bold text-[#0B1016] text-sm">DropWire receive happy-dog-42</span>
            </div>

            <div className="bg-[#FFFFFF] p-4 rounded-2xl border-2 border-[#00B060] space-y-2">
              <span className="text-xs font-bold text-[#00B060] uppercase">Peer Connected</span>
              <p className="font-mono text-sm font-bold text-[#0B1016]">
                Connected to Sender (Peer Fingerprint: 8f4a1c98e...)
              </p>
              <p className="text-xs text-[#0B1016]/70">
                Receiving: <strong className="text-[#0052FF]">archive-2026.tar.zst</strong> (420 MB)
              </p>
            </div>

            <div className="space-y-2 pt-2">
              <div className="flex justify-between text-xs font-bold">
                <span className="text-[#0B1016]">Writing Verified Chunks to Disk...</span>
                <span className="text-[#00B060] font-black">{progress}%</span>
              </div>
              <div className="w-full bg-[#EBE5DC] h-4 rounded-full overflow-hidden p-0.5 border border-[#D9D2C9]">
                <div
                  className="bg-[#00B060] h-full rounded-full transition-all duration-500 shadow-lg"
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>
          </>
        )}
      </div>

      {/* Copy Snippet Footer */}
      <div className="flex items-center justify-between bg-[#F4EFEA] p-3.5 rounded-2xl border border-[#D9D2C9]">
        <span class="font-mono text-xs font-bold text-[#0B1016]/80">Ready to try? Install CLI from cargo:</span>
        <button
          onClick={handleCopy}
          className="px-4 py-2 bg-black text-white hover:bg-[#0052FF] rounded-xl font-mono text-xs font-bold transition-all shadow-md"
        >
          {copied ? 'Copied to Clipboard!' : 'cargo install DropWire-cli 📋'}
        </button>
      </div>
    </div>
  );
}

