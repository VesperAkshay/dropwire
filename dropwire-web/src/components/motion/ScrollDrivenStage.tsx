import React, { useRef } from 'react';
import { motion, useScroll, useTransform } from 'framer-motion';

interface Props {
  children: React.ReactNode;
  className?: string;
}

export default function ScrollDrivenStage({ children, className = '' }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({
    target: containerRef,
    offset: ['start end', 'end start'],
  });

  const y = useTransform(scrollYProgress, [0, 1], [30, -30]);
  const opacity = useTransform(scrollYProgress, [0, 0.2, 0.8, 1], [0.6, 1, 1, 0.6]);
  const scale = useTransform(scrollYProgress, [0, 0.3], [0.96, 1]);

  return (
    <div ref={containerRef} className={className}>
      <motion.div style={{ y, opacity, scale }} transition={{ ease: 'easeOut' }}>
        {children}
      </motion.div>
    </div>
  );
}
