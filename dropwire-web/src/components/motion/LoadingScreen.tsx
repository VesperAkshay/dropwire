import React, { useEffect, useState } from 'react';

export default function LoadingScreen() {
  const [stage, setStage] = useState<'init' | 'title' | 'subtitle' | 'hold' | 'fade' | 'done'>('init');

  useEffect(() => {
    // Only play once per session
    if (typeof window !== 'undefined') {
      const hasLoaded = sessionStorage.getItem('DropWire_loaded');
      if (hasLoaded) {
        setStage('done');
        return;
      }
    }

    // 1. Tiny 100ms start to reduce starting wait
    const startTimer = setTimeout(() => {
      setStage('title');
    }, 100);

    // 2. DropWire. reveals over 1200ms (100ms -> 1300ms)
    const titleTimer = setTimeout(() => {
      setStage('subtitle');
    }, 1300);

    // 3. Subtitle P2P ENCRYPTED ENGINE reveals over 1000ms (1300ms -> 2300ms)
    const subTimer = setTimeout(() => {
      setStage('hold');
    }, 2300);

    // 4. Hold full text on screen for exactly 1.0 second (2300ms -> 3300ms)
    const fadeTimer = setTimeout(() => {
      setStage('fade');
    }, 3300);

    // 5. Dissolve merge into homepage (3300ms -> 4300ms)
    const doneTimer = setTimeout(() => {
      sessionStorage.setItem('DropWire_loaded', 'true');
      setStage('done');
    }, 4300);

    return () => {
      clearTimeout(startTimer);
      clearTimeout(titleTimer);
      clearTimeout(subTimer);
      clearTimeout(fadeTimer);
      clearTimeout(doneTimer);
    };
  }, []);

  if (stage === 'done') return null;

  const isTitleRevealed = stage === 'subtitle' || stage === 'hold' || stage === 'fade';
  const isSubRevealed = stage === 'hold' || stage === 'fade';

  return (
    <div
      className={`fixed inset-0 z-[9999] bg-[#F4EFEA] text-[#0B1016] flex flex-col items-center justify-center p-8 transition-opacity duration-1000 ease-in-out pointer-events-none ${
        stage === 'fade' ? 'opacity-0' : 'opacity-100'
      }`}
    >
      <div
        className={`text-left space-y-4 max-w-5xl transition-all duration-1000 ease-in-out ${
          stage === 'fade' ? 'scale-98 opacity-0 filter blur-md' : 'scale-100 opacity-100 filter blur-none'
        }`}
      >
        {/* 1. DropWire. (Reveals first from left-to-right, hidden during init) */}
        <div className="relative overflow-hidden py-2">
          <h1
            className="font-display text-7xl sm:text-[10rem] font-black tracking-tighter text-black uppercase leading-none select-none transition-all duration-[1200ms] ease-[cubic-bezier(0.65,0,0.35,1)]"
            style={{
              clipPath: isTitleRevealed ? 'inset(0 0% 0 0)' : 'inset(0 100% 0 0)',
            }}
          >
            DropWire.
          </h1>
        </div>

        {/* 2. P2P ENCRYPTED ENGINE (Strictly hidden until DropWire. finishes) */}
        <div className="relative overflow-hidden pl-2 py-1">
          <p
            className="font-mono text-sm sm:text-2xl text-black/60 font-bold tracking-[0.25em] uppercase transition-all duration-[1000ms] ease-[cubic-bezier(0.65,0,0.35,1)]"
            style={{
              clipPath: isSubRevealed ? 'inset(0 0% 0 0)' : 'inset(0 100% 0 0)',
            }}
          >
            P2P ENCRYPTED ENGINE
          </p>
        </div>
      </div>
    </div>
  );
}

