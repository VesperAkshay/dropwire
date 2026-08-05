import React from 'react';

export interface FlagData {
  flag: string;
  short?: string;
  type: string;
  defaultVal: string;
  desc: string;
}

interface FlagTableProps {
  flags: FlagData[];
}

export function FlagTable({ flags }: FlagTableProps) {
  return (
    <div className="overflow-x-auto my-6 rounded-3xl border-2 border-[#D9D2C9] bg-white shadow-lg">
      <table className="w-full text-left border-collapse">
        <thead>
          <tr className="bg-[#F4EFEA] border-b-2 border-[#D9D2C9]">
            <th className="p-4 font-display font-black text-[#0B1016] uppercase text-sm">Flag</th>
            <th className="p-4 font-display font-black text-[#0B1016] uppercase text-sm">Short</th>
            <th className="p-4 font-display font-black text-[#0B1016] uppercase text-sm">Type</th>
            <th className="p-4 font-display font-black text-[#0B1016] uppercase text-sm">Default</th>
            <th className="p-4 font-display font-black text-[#0B1016] uppercase text-sm">Description</th>
          </tr>
        </thead>
        <tbody className="font-sans font-medium text-[#0B1016]/80">
          {flags.map((f, i) => (
            <tr key={i} className="border-b border-[#EBE5DC] last:border-b-0 hover:bg-[#F4EFEA]/50 transition-colors">
              <td className="p-4"><code className="font-mono text-[#0052FF] bg-[#0052FF]/10 px-2 py-1 rounded-md text-sm font-bold">{f.flag}</code></td>
              <td className="p-4">{f.short ? <code className="font-mono font-bold">{f.short}</code> : '—'}</td>
              <td className="p-4 text-sm">{f.type}</td>
              <td className="p-4 text-sm">{f.defaultVal}</td>
              <td className="p-4">{f.desc}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
