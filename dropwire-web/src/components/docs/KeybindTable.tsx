import React from 'react';

export interface KeybindData {
  screen: string;
  key: string;
  action: string;
}

interface KeybindTableProps {
  keybinds: KeybindData[];
}

export function KeybindTable({ keybinds }: KeybindTableProps) {
  return (
    <div className="overflow-x-auto my-6 rounded-3xl border-2 border-[#D9D2C9] bg-white shadow-lg">
      <table className="w-full text-left border-collapse">
        <thead>
          <tr className="bg-[#F4EFEA] border-b-2 border-[#D9D2C9]">
            <th className="p-4 font-display font-black text-[#0B1016] uppercase text-sm">Screen</th>
            <th className="p-4 font-display font-black text-[#0B1016] uppercase text-sm">Key</th>
            <th className="p-4 font-display font-black text-[#0B1016] uppercase text-sm">Action</th>
          </tr>
        </thead>
        <tbody className="font-sans font-medium text-[#0B1016]/80">
          {keybinds.map((k, i) => (
            <tr key={i} className="border-b border-[#EBE5DC] last:border-b-0 hover:bg-[#F4EFEA]/50 transition-colors">
              <td className="p-4 font-bold">{k.screen}</td>
              <td className="p-4">
                <kbd className="font-mono text-[#0B1016] bg-[#EBE5DC] px-2 py-1 rounded-md text-sm font-bold border border-[#D9D2C9] shadow-sm">
                  {k.key}
                </kbd>
              </td>
              <td className="p-4">{k.action}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
