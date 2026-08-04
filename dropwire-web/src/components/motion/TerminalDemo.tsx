import React, { useState, useRef, useEffect } from 'react';

export default function TerminalDemo() {
  const [history, setHistory] = useState([
    { type: 'input', text: 'dropwire help' },
    { type: 'output', text: 'DropWire v0.1.2\nSecure P2P File & Folder Transfer\n\nUsage:\n  dropwire send <file_or_dir>\n  dropwire receive <room-code>' }
  ]);
  const [input, setInput] = useState('');
  const scrollRef = useRef<HTMLDivElement>(null);

  const handleCommand = () => {
    if (!input.trim()) return;

    const cmd = input.trim();
    const newHistory = [...history, { type: 'input', text: cmd }];
    setInput('');
    setHistory(newHistory);

    // Simulate response delay
    setTimeout(() => {
      let output = '';
      if (cmd.startsWith('dropwire send')) {
        output = '[+] Generating ephemeral keys...\n[+] Connected to relay.\nRoom Code: 7-purple-lion\n[*] Waiting for receiver...';
      } else if (cmd.startsWith('dropwire receive') || cmd.startsWith('dropwire recv')) {
        output = '[+] Authenticating with SPAKE2...\n[+] Handshake complete. Connected directly via LAN.\n[⬇] Downloading... (100%)\n[✔] Verified BLAKE3 integrity. File saved.';
      } else if (cmd === 'clear') {
        setHistory([]);
        return;
      } else if (cmd === 'dropwire') {
        output = 'DropWire v0.1.1\nUsage:\n  dropwire send <path>\n  dropwire receive <room-code>';
      } else {
        output = `command not found: ${cmd.split(' ')[0]}\nTry: dropwire send ./files or clear`;
      }
      setHistory(prev => [...prev, { type: 'output', text: output }]);
    }, 500);
  };

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [history]);

  return (
    <div className="bg-[#111111] rounded-2xl w-full max-w-2xl border border-white/10 shadow-2xl overflow-hidden text-left font-mono text-sm sm:text-base flex flex-col h-[400px]">
      <div className="flex items-center gap-2 px-4 py-3 bg-[#222222] border-b border-white/10">
        <div className="w-3 h-3 rounded-full bg-[#FF3D00]"></div>
        <div className="w-3 h-3 rounded-full bg-[#FFB800]"></div>
        <div className="w-3 h-3 rounded-full bg-[#00B060]"></div>
        <span className="ml-2 text-white/50 text-xs font-mono font-bold tracking-widest">INTERACTIVE TERMINAL</span>
      </div>

      <div ref={scrollRef} className="p-4 overflow-y-auto flex-1 space-y-3 custom-scrollbar" style={{ scrollbarWidth: 'thin' }}>
        {history.map((line, i) => (
          <div key={i} className="space-y-1">
            {line.type === 'input' ? (
              <div className="flex items-center gap-2 text-[#0052FF]">
                <span className="font-bold shrink-0">~/project</span>
                <span className="text-white shrink-0">$</span>
                <span className="text-[#00B060]">{line.text}</span>
              </div>
            ) : (
              <div className="text-white/70 whitespace-pre-wrap pl-4 text-xs sm:text-sm">
                {line.text.includes('Room Code:') ? (
                  <>
                    {line.text.split('Room Code:')[0]}
                    <span className="text-[#FFB800] font-bold">Room Code:{line.text.split('Room Code:')[1].split('\n')[0]}</span>
                    {line.text.includes('\n[*]') && '\n[*] Waiting for receiver...'}
                  </>
                ) : (
                  line.text
                )}
              </div>
            )}
          </div>
        ))}
        
        <div className="flex items-center gap-2 text-[#0052FF] mt-2 group">
          <span className="font-bold shrink-0">~/project</span>
          <span className="text-white shrink-0">$</span>
          <div className="relative w-full flex items-center h-6">
            <input
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  handleCommand();
                }
              }}
              className="bg-transparent border-none outline-none text-[#00B060] w-full h-full font-mono focus:ring-0 p-0 z-10"
              spellCheck={false}
              autoComplete="off"
            />
            {/* Blinking cursor when input is empty */}
            {!input && (
              <div className="w-2 h-4 bg-[#00B060] animate-pulse absolute left-0 z-0"></div>
            )}
          </div>
        </div>
      </div>
      <div className="bg-[#222222] px-4 py-2 text-[10px] text-white/40 uppercase tracking-widest text-center border-t border-white/5 font-bold">
        Try typing "dropwire send ./my_folder" or "clear"
      </div>
    </div>
  );
}
