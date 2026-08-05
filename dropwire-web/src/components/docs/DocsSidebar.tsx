import React, { useEffect, useState } from 'react';

export interface SidebarGroup {
  title: string;
  items: { id: string; label: string }[];
}

interface DocsSidebarProps {
  groups: SidebarGroup[];
}

export function DocsSidebar({ groups }: DocsSidebarProps) {
  const [activeId, setActiveId] = useState<string>('');

  useEffect(() => {
    const handleScroll = () => {
      const headings = Array.from(document.querySelectorAll('h2, h3'));
      let current = '';
      for (const heading of headings) {
        const top = heading.getBoundingClientRect().top;
        if (top < 150) {
          current = heading.id;
        }
      }
      if (current) {
        setActiveId(current);
      }
    };

    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <nav className="w-64 shrink-0 hidden md:block sticky top-32 max-h-[calc(100vh-128px)] overflow-y-auto pr-6 pb-12 custom-scrollbar">
      {groups.map((group, idx) => (
        <div key={idx} className="mb-8 last:mb-0">
          <h4 className="text-xs font-mono font-black uppercase text-[#0B1016]/40 tracking-widest mb-3">
            {group.title}
          </h4>
          <ul className="space-y-1 border-l-2 border-[#EBE5DC]">
            {group.items.map((item) => {
              const isActive = activeId === item.id;
              return (
                <li key={item.id}>
                  <a
                    href={`#${item.id}`}
                    className={`block pl-4 py-1.5 text-sm font-medium transition-all ${
                      isActive
                        ? 'text-[#0052FF] font-bold border-l-2 border-[#0052FF] -ml-[2px]'
                        : 'text-[#0B1016]/60 hover:text-[#0B1016] hover:border-l-2 hover:border-[#0B1016]/20 hover:-ml-[2px]'
                    }`}
                  >
                    {item.label}
                  </a>
                </li>
              );
            })}
          </ul>
        </div>
      ))}
    </nav>
  );
}
