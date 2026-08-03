import React, { useState, useEffect } from 'react';

const COLORS = ['#0052FF', '#FFB800', '#FF3D00', '#00B060', '#9D00FF'];

export default function InteractiveMosaic() {
  const [activeColors, setActiveColors] = useState<{ [key: number]: string }>({});
  const [tiles, setTiles] = useState<number[]>([]);

  useEffect(() => {
    // Generate exactly 480 tiles (12 rows for 40 columns)
    setTiles(Array.from({ length: 480 }));
  }, []);

  const handleTileHover = (index: number) => {
    const randomColor = COLORS[Math.floor(Math.random() * COLORS.length)];
    setActiveColors((prev) => ({
      ...prev,
      [index]: randomColor,
    }));
  };

  return (
    <div className="relative w-full rounded-none border-y border-[#D9D2C9] bg-[#F4EFEA] overflow-hidden shadow-none">
      {/* Background Interactive Tile Grid (Small perfect square tiles using arbitrary Tailwind columns) */}
      <div className="w-full grid grid-cols-[repeat(20,minmax(0,1fr))] sm:grid-cols-[repeat(40,minmax(0,1fr))] gap-0">
        {tiles.map((_, i) => {
          const color = activeColors[i];
          return (
            <div
              key={i}
              onMouseEnter={() => handleTileHover(i)}
              className="w-full aspect-square border-r border-b border-[#D9D2C9]/60 transition-colors duration-300 ease-out cursor-pointer"
              style={{
                backgroundColor: color || 'transparent',
              }}
            />
          );
        })}
      </div>

      {/* High-Contrast Overlay Giant DropWire Title - BOTTOM LEFT PLACEMENT EXACTLY LIKE FIX.PNG */}
      <div className="absolute bottom-0 left-0 w-full z-10 pointer-events-none pb-4 pl-4 sm:pb-8 sm:pl-8 flex items-end">
        <span 
          className="font-display font-black text-[5rem] sm:text-[12rem] tracking-tighter text-black uppercase select-none drop-shadow-sm"
          style={{ lineHeight: '0.75' }}
        >
          DropWire.
        </span>
      </div>
    </div>
  );
}

