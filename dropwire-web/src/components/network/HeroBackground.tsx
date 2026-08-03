import React, { useEffect, useRef } from 'react';

const COLORS = ['#0052FF', '#FFB800', '#FF3D00', '#00B060', '#9D00FF'];
// Adjusted for the white card background in the Hero section
const BG_COLOR = '#FFFFFF';

function easeInOutExpo(x: number): number {
  return x === 0 ? 0 : x === 1 ? 1 : x < 0.5 ? Math.pow(2, 20 * x - 10) / 2 : (2 - Math.pow(2, -20 * x + 10)) / 2;
}

export default function HeroBackground() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) return;

    let width = 0;
    let height = 0;
    let nodes: any[] = [];
    let pulses: any[] = [];
    let packets: any[] = [];
    let animationFrameId: number;

    const mouse = { x: -1000, y: -1000, targetX: -1000, targetY: -1000 };

    const handleMouseMove = (e: MouseEvent) => {
      // Offset by the canvas bounding rect to get correct coordinates within the hero card
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
      phaseX: number;
      phaseY: number;
      speedX: number;
      speedY: number;
      radius: number;
      isHub: boolean;
      connections: Node[];

      constructor(id: number, w: number, h: number) {
        this.id = id;
        this.baseX = Math.random() * w;
        this.baseY = Math.random() * h;
        this.z = Math.random() * 1.4 + 0.1;
        this.x = this.baseX;
        this.y = this.baseY;
        this.phaseX = Math.random() * Math.PI * 2;
        this.phaseY = Math.random() * Math.PI * 2;
        this.speedX = (Math.random() * 0.0005) + 0.0002;
        this.speedY = (Math.random() * 0.0005) + 0.0002;
        this.radius = (Math.random() * 1.5 + 0.5) * this.z;
        this.isHub = Math.random() > 0.95;
        if (this.isHub) this.radius *= 3;
        this.connections = [];
      }

      update(time: number) {
        const driftX = Math.sin(time * this.speedX + this.phaseX) * (40 * this.z);
        const driftY = Math.cos(time * this.speedY + this.phaseY) * (40 * this.z);
        const centerX = width / 2;
        const centerY = height / 2;
        const parallaxX = ((mouse.x - centerX) * 0.05) * this.z;
        const parallaxY = ((mouse.y - centerY) * 0.05) * this.z;

        let magneticX = 0;
        let magneticY = 0;
        const dx = mouse.x - (this.baseX + driftX + parallaxX);
        const dy = mouse.y - (this.baseY + driftY + parallaxY);
        const dist = Math.sqrt(dx * dx + dy * dy);
        
        const lensRadius = 300;
        if (dist < lensRadius) {
          const force = Math.pow((lensRadius - dist) / lensRadius, 2);
          magneticX = -(dx / dist) * force * 50 * this.z;
          magneticY = -(dy / dist) * force * 50 * this.z;
        }

        this.x = this.baseX + driftX + parallaxX + magneticX;
        this.y = this.baseY + driftY + parallaxY + magneticY;
      }

      draw(ctx: CanvasRenderingContext2D) {
        ctx.beginPath();
        ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
        const alpha = Math.min(1, this.z * 0.8 + 0.2);
        ctx.fillStyle = `rgba(11, 16, 22, ${alpha})`;
        ctx.fill();
      }
    }

    class Pulse {
      x: number;
      y: number;
      color: string;
      radius: number;
      maxRadius: number;
      life: number;
      decay: number;

      constructor(x: number, y: number, color: string) {
        this.x = x;
        this.y = y;
        this.color = color;
        this.radius = 0;
        this.maxRadius = Math.random() * 20 + 15;
        this.life = 1;
        this.decay = Math.random() * 0.02 + 0.015;
      }
      update() {
        this.radius += (this.maxRadius - this.radius) * 0.1;
        this.life -= this.decay;
      }
      draw(ctx: CanvasRenderingContext2D) {
        if (this.life <= 0) return;
        ctx.beginPath();
        ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
        ctx.strokeStyle = this.color;
        ctx.lineWidth = this.life * 2;
        ctx.globalAlpha = this.life;
        ctx.stroke();
        ctx.globalAlpha = 1;
      }
    }

    class PacketStreak {
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
        this.speed = Math.random() * 0.005 + 0.002;
        this.color = COLORS[Math.floor(Math.random() * COLORS.length)];
        this.length = Math.random() * 0.15 + 0.05;
      }

      pickNextNode(node: Node) {
        if (node.connections.length === 0) return node;
        return node.connections[Math.floor(Math.random() * node.connections.length)];
      }

      update(pulsesArray: Pulse[]) {
        this.progress += this.speed;
        if (this.progress >= 1) {
          pulsesArray.push(new Pulse(this.endNode.x, this.endNode.y, this.color));
          this.startNode = this.endNode;
          this.endNode = this.pickNextNode(this.startNode);
          this.progress = 0;
        }
      }

      draw(ctx: CanvasRenderingContext2D) {
        if (this.startNode === this.endNode) return;
        const easeHead = easeInOutExpo(Math.min(1, this.progress + this.length));
        const easeTail = easeInOutExpo(Math.max(0, this.progress));

        const hx = this.startNode.x + (this.endNode.x - this.startNode.x) * easeHead;
        const hy = this.startNode.y + (this.endNode.y - this.startNode.y) * easeHead;
        
        const tx = this.startNode.x + (this.endNode.x - this.startNode.x) * easeTail;
        const ty = this.startNode.y + (this.endNode.y - this.startNode.y) * easeTail;

        const gradient = ctx.createLinearGradient(tx, ty, hx, hy);
        gradient.addColorStop(0, 'rgba(255,255,255,0)');
        gradient.addColorStop(0.8, this.color);
        gradient.addColorStop(1, '#ffffff');

        ctx.beginPath();
        ctx.moveTo(tx, ty);
        ctx.lineTo(hx, hy);
        ctx.strokeStyle = gradient;
        ctx.lineWidth = (this.startNode.z + this.endNode.z) * 1.0;
        ctx.lineCap = 'round';
        ctx.stroke();
      }
    }

    const initNetwork = () => {
      const rect = canvas.getBoundingClientRect();
      width = rect.width;
      height = rect.height;
      const dpr = window.devicePixelRatio || 1;
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      ctx.scale(dpr, dpr);

      nodes = [];
      packets = [];
      pulses = [];

      // Highly dense network fit for hero
      const numNodes = Math.floor((width * height) / 8000);

      for (let i = 0; i < numNodes; i++) {
        nodes.push(new Node(i, width, height));
      }

      nodes.forEach(node => {
        let distances = nodes
          .filter(n => n.id !== node.id)
          .map(n => ({
            node: n,
            dist: Math.hypot(node.baseX - n.baseX, node.baseY - n.baseY)
          }))
          .sort((a, b) => a.dist - b.dist);
        
        node.connections = distances.slice(0, Math.floor(Math.random() * 3) + 3).map(d => d.node);
      });

      for (let i = 0; i < numNodes * 0.8; i++) {
        packets.push(new PacketStreak(nodes[Math.floor(Math.random() * nodes.length)]));
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

      ctx.fillStyle = BG_COLOR;
      ctx.fillRect(0, 0, width, height);

      nodes.forEach(node => node.update(time));

      ctx.lineWidth = 0.6;
      nodes.forEach(node => {
        node.connections.forEach(target => {
          if (node.id > target.id) return;
          
          const dx = node.x - target.x;
          const dy = node.y - target.y;
          const dist = Math.sqrt(dx * dx + dy * dy);
          
          if (dist < 200) {
            const avgZ = (node.z + target.z) / 2;
            const alpha = (1 - (dist / 200)) * (avgZ * 0.3);
            
            ctx.beginPath();
            ctx.moveTo(node.x, node.y);
            ctx.lineTo(target.x, target.y);
            ctx.strokeStyle = `rgba(11, 16, 22, ${alpha})`;
            ctx.stroke();
          }
        });
      });

      nodes.forEach(node => node.draw(ctx));

      ctx.globalCompositeOperation = 'source-over'; 
      packets.forEach(packet => {
        packet.update(pulses);
        packet.draw(ctx);
      });

      for (let i = pulses.length - 1; i >= 0; i--) {
        pulses[i].update();
        if (pulses[i].life <= 0) {
          pulses.splice(i, 1);
        } else {
          pulses[i].draw(ctx);
        }
      }

      animationFrameId = requestAnimationFrame(animate);
    };

    // Delay initialization to ensure DOM layout is complete
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
