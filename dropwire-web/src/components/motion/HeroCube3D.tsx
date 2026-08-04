import { useEffect, useRef } from 'react';
import Atropos from 'atropos';
import 'atropos/css';

export default function HeroCube3D() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const atroposInstance = Atropos({
      el: containerRef.current,
      activeOffset: 40,
      shadowScale: 1,
      shadow: false,
      rotateXMax: 15,
      rotateYMax: 15,
      highlight: false,
    });

    return () => {
      atroposInstance.destroy();
    };
  }, []);

  return (
    <div className="relative flex justify-center items-center py-6 w-full h-full">
      <div ref={containerRef} className="atropos my-atropos w-full max-w-[28rem] sm:max-w-lg aspect-square">
        <div className="atropos-scale w-full h-full">
          <div className="atropos-rotate w-full h-full">
            
            {/* The Neo-Brutalist Card */}
            <div className="atropos-inner p-8 bg-white rounded-[2.5rem] border-4 border-black relative overflow-hidden group w-full h-full shadow-[16px_16px_0_0_#000]">
              
              {/* Layer -5: Background Blueprint Grid */}
              <div 
                className="absolute inset-0 opacity-40 pointer-events-none bg-grid-pattern-light"
                data-atropos-offset="-5"
              />

              {/* Layer 0: Holographic Scanline (CSS Animated) */}
              <div 
                className="absolute left-0 right-0 h-1.5 bg-[#0052FF] shadow-[0_0_20px_4px_#0052FF] opacity-70 z-10 animate-[scanline_3s_linear_infinite] pointer-events-none"
                data-atropos-offset="0"
                style={{ top: '-10%' }}
              />

              {/* Layer 8: Decorative Pill */}
              <div 
                className="absolute top-8 left-8 px-5 py-2 bg-[#FFB800] border-2 border-black rounded-full text-sm font-display font-black text-black tracking-wide shadow-[4px_4px_0_0_#000] z-20"
                data-atropos-offset="8"
              >
                P2P ENCRYPTED
              </div>

              {/* Layer 10: The Logo (Deep Parallax) */}
              <div className="absolute inset-0 flex justify-center items-center z-30" data-atropos-offset="10">
                <img 
                  src="/brand/dropwire-logo-v2.png" 
                  alt="DropWire Logo" 
                  className="w-48 h-48 sm:w-64 sm:h-64 object-contain drop-shadow-[0_10px_15px_rgba(0,0,0,0.15)] transition-transform duration-500 group-hover:scale-105"
                />
              </div>

              {/* Layer 15: Geometric Data Blocks (Popping out) */}
              <div className="absolute inset-0 pointer-events-none z-40" data-atropos-offset="15">
                <div className="absolute top-[20%] right-12 w-6 h-6 bg-[#00B060] border-2 border-black shadow-[3px_3px_0_0_#000] animate-[bounce_3s_infinite]" />
                <div className="absolute bottom-[25%] left-10 w-8 h-8 bg-[#FF3D00] border-2 border-black rounded-full shadow-[3px_3px_0_0_#000] animate-[bounce_4s_infinite_reverse]" />
                <div className="absolute top-[50%] left-8 w-4 h-4 bg-[#AB54F7] border-2 border-black shadow-[2px_2px_0_0_#000]" />
              </div>

              {/* Layer 3: Bottom Label */}
              <div className="absolute bottom-10 left-0 right-0 text-center z-20" data-atropos-offset="3">
                <p className="text-base font-display font-black uppercase tracking-widest text-black">Zero-Knowledge Peer Stream</p>
                <div className="inline-block mt-3 px-4 py-1.5 bg-black text-[#00B060] text-xs font-mono font-bold rounded-xl shadow-md border border-white/10">
                  Multiplexed TCP · SPAKE2 · BLAKE3
                </div>
              </div>

            </div>
          </div>
        </div>
      </div>
      
      {/* Required CSS for Scanline */}
      <style>{`
        @keyframes scanline {
          0% { transform: translateY(0); opacity: 0; }
          10% { opacity: 0.7; }
          90% { opacity: 0.7; }
          100% { transform: translateY(35rem); opacity: 0; }
        }
      `}</style>
    </div>
  );
}

