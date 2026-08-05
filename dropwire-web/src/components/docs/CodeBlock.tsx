import React, { useState } from 'react';
import { Copy, Check } from 'lucide-react';

interface CodeBlockProps {
  code: string;
  language?: string;
}

export function CodeBlock({ code, language = 'bash' }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const copyToClipboard = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="relative group rounded-3xl bg-[#0B1016] p-6 border-2 border-[#222C37] shadow-xl overflow-hidden font-mono text-sm sm:text-base my-6">
      <div className="absolute top-4 right-4 z-10">
        <button
          onClick={copyToClipboard}
          className="p-2 rounded-xl bg-[#222C37] text-white hover:bg-[#0052FF] transition-colors shadow-sm"
          title="Copy code"
        >
          {copied ? <Check size={16} /> : <Copy size={16} />}
        </button>
      </div>
      <div className="overflow-x-auto text-[#EBE5DC]">
        <pre className="whitespace-pre">
          <code>{code}</code>
        </pre>
      </div>
    </div>
  );
}
