import React, { useState, useEffect } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { ModeSlider } from './ModeSlider';
import { DocsSidebar } from './DocsSidebar';
import { CliDocs, cliSidebarGroups } from './CliDocs';
import { TuiDocs, tuiSidebarGroups } from './TuiDocs';

export function DocsContainer() {
  const [mode, setMode] = useState<'cli' | 'tui'>('cli');
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
    // Check URL params
    const params = new URLSearchParams(window.location.search);
    const m = params.get('mode');
    if (m === 'tui') setMode('tui');
    else if (m === 'cli') setMode('cli');
    else {
      // Check localStorage
      const saved = localStorage.getItem('dropwire-docs-mode');
      if (saved === 'tui') setMode('tui');
    }
  }, []);

  const handleModeChange = (newMode: 'cli' | 'tui') => {
    setMode(newMode);
    localStorage.setItem('dropwire-docs-mode', newMode);
    
    // Update URL without reload
    const url = new URL(window.location.href);
    url.searchParams.set('mode', newMode);
    window.history.pushState({}, '', url.toString());
  };

  if (!mounted) return <div className="min-h-screen flex items-center justify-center font-display text-2xl">Loading Documentation...</div>;

  const isCli = mode === 'cli';
  const sidebarGroups = isCli ? cliSidebarGroups : tuiSidebarGroups;

  return (
    <div className="p-4 sm:p-8 lg:p-12">
      <div className="max-w-[1440px] mx-auto px-6 sm:px-12 py-12 bg-[#F4EFEA]/95 backdrop-blur-2xl rounded-[3rem] border-2 border-[#D9D2C9] shadow-2xl min-h-[calc(100vh-8rem)]">
        
        {/* Header & Toggle */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 mb-16 border-b-2 border-[#D9D2C9] pb-12">
          <div className="space-y-4">
          <span className="text-xs font-mono text-[#0052FF] uppercase tracking-widest font-black">DOCUMENTATION HUB</span>
          <h1 className="text-5xl sm:text-7xl font-display font-black text-[#0B1016] uppercase tracking-tight leading-none">
            DropWire Docs
          </h1>
          <p className="text-lg text-[#0B1016]/80 font-sans font-medium max-w-xl">
            Everything you need to know about the DropWire CLI and TUI, all in one place.
          </p>
        </div>

        <div className="shrink-0">
          <ModeSlider mode={mode} onChange={handleModeChange} />
        </div>
      </div>

      {/* Main Content Area */}
      <div className="flex gap-16 relative">
        <DocsSidebar groups={sidebarGroups} />

        <div className="flex-1 min-w-0">
          <AnimatePresence mode="wait" initial={false}>
            {isCli ? (
              <motion.div
                key="cli"
                initial={{ x: -20, opacity: 0 }}
                animate={{ x: 0, opacity: 1 }}
                exit={{ x: 20, opacity: 0 }}
                transition={{ duration: 0.3 }}
              >
                <CliDocs />
              </motion.div>
            ) : (
              <motion.div
                key="tui"
                initial={{ x: 20, opacity: 0 }}
                animate={{ x: 0, opacity: 1 }}
                exit={{ x: -20, opacity: 0 }}
                transition={{ duration: 0.3 }}
              >
                <TuiDocs />
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
    </div>
  );
}
