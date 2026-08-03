import { useEffect, useRef } from 'react';
import Atropos from 'atropos';
import 'atropos/css';

export default function HeroCube3D() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const atroposInstance = Atropos({
      el: containerRef.current,
      activeOffset: 50,
      shadowScale: 1.05,
      shadow: true,
      shadowOffset: 40,
      shadowOpacity: 0.4,
      rotateXMax: 15,
      rotateYMax: 15,
      highlight: true,
    });

    return () => {
      atroposInstance.destroy();
    };
  }, []);

  return (
    <div className="relative flex justify-center items-center py-6">
      {/* Background Gold Radial Glow */}
      <div className="absolute inset-0 bg-gradient-radial from-[rgba(225,184,101,0.2)] via-transparent to-transparent blur-3xl rounded-full pointer-events-none" />
      
      <div ref={containerRef} className="atropos my-atropos w-full max-w-sm sm:max-w-md">
        <div className="atropos-scale">
          <div className="atropos-rotate">
            <div className="atropos-inner p-8 bg-[#161D24]/80 backdrop-blur-md rounded-2xl border border-[rgba(124,101,66,0.4)] shadow-2xl relative overflow-hidden group">
              {/* Subtle grid pattern inside card */}
              <div className="absolute inset-0 bg-grid-pattern opacity-30 pointer-events-none" />
              
              {/* Decorative floating chip */}
              <div 
                className="absolute top-4 left-4 px-3 py-1 bg-[#0B1016]/90 border border-[#E1B865]/40 rounded-full text-xs font-mono text-[#F3D578] tracking-wide"
                data-atropos-offset="5"
              >
                P2P ENCRYPTED
              </div>

              {/* Cube Image with Parallax Offset */}
              <div className="flex justify-center items-center py-6" data-atropos-offset="10">
                <img 
                  src="/brand/cube-logo.png" 
                  alt="DropWire 3D Hero Cube" 
                  className="w-48 h-48 sm:w-60 sm:h-60 object-contain drop-shadow-[0_20px_35px_rgba(225,184,101,0.35)] transition-transform duration-500 group-hover:scale-105"
                />
              </div>

              {/* Bottom Label */}
              <div className="text-center pt-2" data-atropos-offset="3">
                <p className="text-xs uppercase tracking-widest text-[#AA90B3] font-semibold">Zero-Knowledge Peer Stream</p>
                <p className="text-sm font-mono text-[#FEF7E4] font-medium mt-0.5">Multiplexed TCP · SPAKE2 · BLAKE3</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

