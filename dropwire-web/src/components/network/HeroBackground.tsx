import React, { useEffect, useRef } from 'react';

const COLORS = ['#0052FF', '#FFB800', '#FF3D00', '#00B060', '#AB54F7'];
const BG_COLOR = '#FFFFFF';

export default function HeroBackground() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d', { alpha: false, desynchronized: true });
    if (!ctx) return;

    let width = 0;
    let height = 0;
    let nodes: any[] = [];
    let packets: any[] = [];
    let animationFrameId: number;

    const mouse = { x: -1000, y: -1000, targetX: -1000, targetY: -1000 };

    const handleMouseMove = (e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      mouse.targetX = e.clientX - rect.left;
      mouse.targetY = e.clientY - rect.top;
    };
    const handleMouseOut = () => {
      mouse.targetX = width / 2;
      mouse.targetY = height / 2;
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseout', handleMouseOut);

    class Node {
      id: number;
      baseX: number;
      baseY: number;
      z: number;
      x: number;
      y: number;
      phase: number;
      speed: number;
      radius: number;
      connections: Node[];
      color: string;

      constructor(id: number, w: number, h: number) {
        this.id = id;
        this.baseX = Math.random() * w;
        this.baseY = Math.random() * h;
        this.z = Math.random() * 1.5 + 0.5;
        this.x = this.baseX;
        this.y = this.baseY;
        this.phase = Math.random() * Math.PI * 2;
        this.speed = (Math.random() * 0.0002) + 0.0001;
        this.radius = (Math.random() * 1.5 + 0.5) * this.z;
        this.connections = [];
        this.color = COLORS[Math.floor(Math.random() * COLORS.length)];
      }

      update(time: number, centerX: number, centerY: number) {
        const driftX = Math.sin(time * this.speed + this.phase) * (40 * this.z);
        const driftY = Math.sin(time * this.speed * 2 + this.phase) * (30 * this.z);
        
        const parallaxX = ((mouse.x - centerX) * 0.03) * this.z;
        const parallaxY = ((mouse.y - centerY) * 0.03) * this.z;

        let magneticX = 0;
        let magneticY = 0;
        const dx = mouse.x - (this.baseX + driftX + parallaxX);
        const dy = mouse.y - (this.baseY + driftY + parallaxY);
        const dist = dx * dx + dy * dy; // Use squared distance for performance
        
        const lensRadiusSq = 90000; // 300 * 300
        if (dist < lensRadiusSq) {
          const actualDist = Math.sqrt(dist);
          const force = Math.pow((300 - actualDist) / 300, 2);
          magneticX = -(dx / actualDist) * force * 50 * this.z;
          magneticY = -(dy / actualDist) * force * 50 * this.z;
        }

        this.x = this.baseX + driftX + parallaxX + magneticX;
        this.y = this.baseY + driftY + parallaxY + magneticY;
      }

      draw(ctx: CanvasRenderingContext2D) {
        ctx.beginPath();
        ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(11, 16, 22, ${Math.min(0.6, this.z * 0.3)})`;
        ctx.fill();
        
        // Fast fake aura (no gradients, just transparent circles)
        ctx.beginPath();
        ctx.arc(this.x, this.y, this.radius * 3, 0, Math.PI * 2);
        ctx.fillStyle = `${this.color}15`; // ~8% opacity hex
        ctx.fill();
      }
    }

    class DataStream {
      startNode: Node;
      endNode: Node;
      progress: number;
      speed: number;
      color: string;
      length: number;

      constructor(startNode: Node) {
        this.startNode = startNode;
        this.endNode = this.pickNextNode(startNode);
        this.progress = 0;
        this.speed = Math.random() * 0.003 + 0.002;
        this.color = startNode.color;
        this.length = Math.random() * 0.2 + 0.1;
      }

      pickNextNode(node: Node) {
        if (node.connections.length === 0) return node;
        return node.connections[Math.floor(Math.random() * node.connections.length)];
      }

      update() {
        this.progress += this.speed;
        if (this.progress >= 1) {
          this.startNode = this.endNode;
          this.endNode = this.pickNextNode(this.startNode);
          this.progress = 0;
          this.color = this.startNode.color;
        }
      }

      getPointOnCurve(p0: {x: number, y: number}, p1: {x: number, y: number}, p2: {x: number, y: number}, t: number) {
        const u = 1 - t;
        const tt = t * t;
        const uu = u * u;
        return {
          x: uu * p0.x + 2 * u * t * p1.x + tt * p2.x,
          y: uu * p0.y + 2 * u * t * p1.y + tt * p2.y
        };
      }

      draw(ctx: CanvasRenderingContext2D, time: number) {
        if (this.startNode === this.endNode) return;

        const mx = (this.startNode.x + this.endNode.x) / 2;
        const my = (this.startNode.y + this.endNode.y) / 2;
        const angle = Math.atan2(this.endNode.y - this.startNode.y, this.endNode.x - this.startNode.x);
        
        // Distance approximation is faster than Math.hypot
        const dx = this.endNode.x - this.startNode.x;
        const dy = this.endNode.y - this.startNode.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        
        const offset = Math.sin(time * 0.001 + this.startNode.id) * dist * 0.3;
        const cx = mx + Math.cos(angle + Math.PI/2) * offset;
        const cy = my + Math.sin(angle + Math.PI/2) * offset;

        const p0 = {x: this.startNode.x, y: this.startNode.y};
        const p1 = {x: cx, y: cy};
        const p2 = {x: this.endNode.x, y: this.endNode.y};

        const headT = Math.min(1, this.progress + this.length);
        const tailT = Math.max(0, this.progress);
        
        const head = this.getPointOnCurve(p0, p1, p2, headT);
        const tail = this.getPointOnCurve(p0, p1, p2, tailT);

        // Solid streak (faster than gradient)
        ctx.beginPath();
        ctx.moveTo(tail.x, tail.y);
        ctx.quadraticCurveTo(
          this.getPointOnCurve(p0, p1, p2, (headT + tailT)/2).x, 
          this.getPointOnCurve(p0, p1, p2, (headT + tailT)/2).y, 
          head.x, head.y
        );
        ctx.strokeStyle = this.color;
        ctx.lineWidth = (this.startNode.z + this.endNode.z) * 1.5;
        ctx.globalAlpha = 0.6; // Fake gradient fade out
        ctx.lineCap = 'round';
        ctx.stroke();
        ctx.globalAlpha = 1.0;
        
        // Fast dot (no shadowBlur!)
        ctx.beginPath();
        ctx.arc(head.x, head.y, this.startNode.z * 1.5, 0, Math.PI * 2);
        ctx.fillStyle = '#0B1016';
        ctx.fill();
        ctx.beginPath();
        ctx.arc(head.x, head.y, this.startNode.z * 3, 0, Math.PI * 2);
        ctx.fillStyle = `${this.color}66`;
        ctx.fill();
      }
    }

    const initNetwork = () => {
      const rect = canvas.getBoundingClientRect();
      width = rect.width;
      height = rect.height;
      // Cap devicePixelRatio to 1.5 maximum for performance
      const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      ctx.scale(dpr, dpr);

      nodes = [];
      packets = [];

      // Reduced node density for 60fps performance
      const numNodes = Math.floor((width * height) / 25000);

      for (let i = 0; i < numNodes; i++) {
        nodes.push(new Node(i, width, height));
      }

      nodes.forEach(node => {
        let distances = nodes
          .filter(n => n.id !== node.id)
          .map(n => ({
            node: n,
            dist: (node.baseX - n.baseX) ** 2 + (node.baseY - n.baseY) ** 2 // Squared distance is faster
          }))
          .sort((a, b) => a.dist - b.dist);
        
        node.connections = distances.slice(0, Math.floor(Math.random() * 2) + 1).map(d => d.node);
      });

      // Fewer packets for performance
      for (let i = 0; i < numNodes * 0.8; i++) {
        packets.push(new DataStream(nodes[Math.floor(Math.random() * nodes.length)]));
      }
      
      mouse.x = width / 2;
      mouse.y = height / 2;
      mouse.targetX = width / 2;
      mouse.targetY = height / 2;
    };

    let time = 0;
    const animate = () => {
      mouse.x += (mouse.targetX - mouse.x) * 0.1;
      mouse.y += (mouse.targetY - mouse.y) * 0.1;
      time += 16;
      const centerX = width / 2;
      const centerY = height / 2;

      ctx.fillStyle = BG_COLOR;
      ctx.fillRect(0, 0, width, height);

      nodes.forEach(node => node.update(time, centerX, centerY));

      // Draw Base Organic Web (Swaying Curves)
      ctx.lineWidth = 1;
      nodes.forEach(node => {
        node.connections.forEach(target => {
          if (node.id > target.id) return;
          
          const dx = target.x - node.x;
          const dy = target.y - node.y;
          const distSq = dx * dx + dy * dy;
          
          if (distSq < 90000) { // 300 * 300
            const dist = Math.sqrt(distSq);
            const mx = (node.x + target.x) / 2;
            const my = (node.y + target.y) / 2;
            const angle = Math.atan2(dy, dx);
            
            const offset = Math.sin(time * 0.001 + node.id) * dist * 0.3;
            const cx = mx + Math.cos(angle + Math.PI/2) * offset;
            const cy = my + Math.sin(angle + Math.PI/2) * offset;

            const alpha = (1 - (dist / 300)) * 0.15;
            
            ctx.beginPath();
            ctx.moveTo(node.x, node.y);
            ctx.quadraticCurveTo(cx, cy, target.x, target.y);
            ctx.strokeStyle = `rgba(11, 16, 22, ${alpha})`;
            ctx.stroke();
          }
        });
      });

      nodes.forEach(node => node.draw(ctx));

      packets.forEach(packet => {
        packet.update();
        packet.draw(ctx, time);
      });

      animationFrameId = requestAnimationFrame(animate);
    };

    const timeout = setTimeout(() => {
      initNetwork();
      animate();
    }, 100);

    const handleResize = () => {
      initNetwork();
    };
    window.addEventListener('resize', handleResize);

    return () => {
      clearTimeout(timeout);
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseout', handleMouseOut);
      window.removeEventListener('resize', handleResize);
      cancelAnimationFrame(animationFrameId);
    };
  }, []);

  return (
    <canvas 
      ref={canvasRef} 
      className="absolute inset-0 w-full h-full pointer-events-none z-0"
      style={{ display: 'block', pointerEvents: 'none' }}
    />
  );
}
