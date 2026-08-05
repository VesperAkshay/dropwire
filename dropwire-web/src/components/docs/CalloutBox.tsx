import React from 'react';
import { Info } from 'lucide-react';

interface CalloutBoxProps {
  type: 'info' | 'warning' | 'tip';
  title?: string;
  children: React.ReactNode;
}

export function CalloutBox({ type, title, children }: CalloutBoxProps) {
  let styles = '';
  let Icon = Info;
  let defaultTitle = 'Info';

  switch (type) {
    case 'info':
      styles = 'bg-[#0052FF]/10 border-[#0052FF] text-[#0052FF]';
      defaultTitle = 'Info';
      break;
    case 'warning':
      styles = 'bg-[#FF3D00]/10 border-[#FF3D00] text-[#FF3D00]';
      defaultTitle = 'Warning';
      break;
    case 'tip':
      styles = 'bg-[#00B060]/10 border-[#00B060] text-[#00B060]';
      defaultTitle = 'Tip';
      break;
  }

  return (
    <div className={`p-6 rounded-3xl border-2 ${styles} my-6 shadow-sm`}>
      <div className="flex items-center gap-3 font-display font-black uppercase text-xl mb-2">
        <Icon size={24} weight="bold" />
        {title || defaultTitle}
      </div>
      <div className="font-sans font-medium text-[#0B1016]/80 text-lg leading-relaxed">
        {children}
      </div>
    </div>
  );
}
