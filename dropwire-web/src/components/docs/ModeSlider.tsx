import React from 'react';
import { motion } from 'framer-motion';

interface ModeSliderProps {
  mode: 'cli' | 'tui';
  onChange: (mode: 'cli' | 'tui') => void;
}

export function ModeSlider({ mode, onChange }: ModeSliderProps) {
  return (
    <div className="relative inline-flex items-center p-1 bg-[#EBE5DC] rounded-full shadow-inner border-2 border-[#D9D2C9]">
      <button
        onClick={() => onChange('cli')}
        className={`relative z-10 px-8 py-3 text-lg font-display font-black uppercase rounded-full transition-colors ${
          mode === 'cli' ? 'text-white' : 'text-[#0B1016]/60 hover:text-[#0B1016]'
        }`}
      >
        CLI Docs
      </button>

      <button
        onClick={() => onChange('tui')}
        className={`relative z-10 px-8 py-3 text-lg font-display font-black uppercase rounded-full transition-colors ${
          mode === 'tui' ? 'text-white' : 'text-[#0B1016]/60 hover:text-[#0B1016]'
        }`}
      >
        TUI Docs
      </button>

      {/* Animated Pill Background */}
      <motion.div
        className="absolute top-1 bottom-1 w-[calc(50%-4px)] bg-[#0052FF] rounded-full shadow-md pointer-events-none"
        animate={{
          left: mode === 'cli' ? '4px' : '50%',
        }}
        transition={{ type: 'spring', stiffness: 400, damping: 30 }}
      />
    </div>
  );
}
