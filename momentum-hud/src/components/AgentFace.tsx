"use client";

import { motion, AnimatePresence, useMotionValue, useSpring, useTransform } from "framer-motion";
import { cn } from "@/lib/utils";
import { useEffect, useState } from "react";

export type AgentState = "idle" | "thinking" | "speaking" | "happy" | "surprised" | "listening" | "error" | "sleeping" | "laughing" | "crying" | "angry" | "coughing" | "sneezing";

interface AgentFaceProps {
  state: AgentState;
  isShuttered?: boolean;
  isVoiceMode?: boolean;
  className?: string;
}

// Absolutely Minimalist "Tall Eye" Setup + Sleep Mode
const EyeStates = {
  idle: { height: 64, width: 20, borderRadius: 10, rotate: 0, y: 0, transition: { type: "spring", bounce: 0.3, duration: 0.5 } },
  listening: { height: 70, width: 28, borderRadius: 14, rotate: 0, y: 0, transition: { type: "spring", bounce: 0.4, duration: 0.4 } },
  thinking: { height: 64, width: 20, borderRadius: 10, rotate: 0, y: -12, x: 0, transition: { type: "spring", bounce: 0.2, duration: 0.6 } },
  speaking: { height: 54, width: 24, borderRadius: 12, rotate: 0, y: 0, transition: { type: "spring", bounce: 0.2, duration: 0.4 } },
  happy: { height: 12, width: 48, borderRadius: 6, rotate: 0, y: -8, transition: { type: "spring", bounce: 0.5, duration: 0.5 } },
  surprised: { height: 60, width: 60, borderRadius: 30, rotate: 0, y: -12, transition: { type: "spring", bounce: 0.6, duration: 0.4 } },
  error: { height: 8, width: 40, borderRadius: 4, rotate: 0, y: 4, transition: { type: "spring", bounce: 0.3, duration: 0.4 } },
  sleeping: { height: 4, width: 28, borderRadius: 2, rotate: 0, y: 0, transition: { type: "spring", bounce: 0.2, duration: 0.8 } },
  laughing: { height: 14, width: 44, borderRadius: 7, rotate: 0, y: -10, transition: { type: "spring", bounce: 0.7, duration: 0.3 } },
  crying: { height: 10, width: 30, borderRadius: 5, rotate: 0, y: 15, transition: { type: "spring", bounce: 0.1, duration: 0.8 } },
  angry: { height: 12, width: 50, borderRadius: 4, rotate: 0, y: 8, transition: { type: "spring", bounce: 0.6, duration: 0.2 } },
  coughing: { height: 40, width: 40, borderRadius: 20, rotate: 0, y: 0, transition: { type: "spring", bounce: 0.8, duration: 0.1 } },
  sneezing: { height: 4, width: 40, borderRadius: 2, rotate: 0, y: -20, transition: { type: "spring", bounce: 0.2, duration: 0.1 } }
};

export function AgentFace({ state, isShuttered = false, isVoiceMode = false, className }: AgentFaceProps) {
  const currentEye = EyeStates[state] || EyeStates.idle;
  
  const [isBlinking, setIsBlinking] = useState(false);
  
  // Physics-based Cursor Tracking
  const mouseX = useMotionValue(0);
  const mouseY = useMotionValue(0);
  
  // Spring dampers for cinematic smoothing
  const smoothX = useSpring(mouseX, { damping: 30, stiffness: 200, mass: 0.5 });
  const smoothY = useSpring(mouseY, { damping: 30, stiffness: 200, mass: 0.5 });
  
  // Transform screen pixels into a tiny offset (-20px to +20px max)
  const eyeOffsetX = useTransform(smoothX, [-1000, 1000], [-30, 30]);
  const eyeOffsetY = useTransform(smoothY, [-1000, 1000], [-30, 30]);

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      // Calculate delta from center of the screen
      const centerX = window.innerWidth / 2;
      const centerY = window.innerHeight / 2;
      const dx = e.clientX - centerX;
      const dy = e.clientY - centerY;
      
      const distance = Math.sqrt(dx * dx + dy * dy);
      
      // Only track if cursor is near the face (within ~400px radius)
      if (distance < 400) {
         mouseX.set(dx);
         mouseY.set(dy);
      } else {
         mouseX.set(0);
         mouseY.set(0);
      }
    };

    window.addEventListener("mousemove", handleMouseMove);
    return () => window.removeEventListener("mousemove", handleMouseMove);
  }, [mouseX, mouseY]);
  
  useEffect(() => {
    let timeoutId: NodeJS.Timeout;
    const blinkCycle = () => {
      if (state !== "happy" && state !== "error" && state !== "sleeping" && !isShuttered) {
        setIsBlinking(true);
        setTimeout(() => setIsBlinking(false), 140);
      }
      timeoutId = setTimeout(blinkCycle, Math.random() * 4000 + 2000);
    };
    timeoutId = setTimeout(blinkCycle, 2000);
    return () => clearTimeout(timeoutId);
  }, [state, isShuttered]);

  const getLeftRotate = () => {
    if (state === "happy" || state === "laughing") return 15;   
    if (state === "error" || state === "crying") return -20; // Sad droop   
    if (state === "angry") return 25; // Angry slant
    return currentEye.rotate;
  };

  const getRightRotate = () => {
    if (state === "happy" || state === "laughing") return -15;  
    if (state === "error" || state === "crying") return 20; // Sad droop 
    if (state === "angry") return -25; // Angry slant
    return currentEye.rotate;
  };

  return (
    <div
      className={cn(
        "relative w-[280px] h-[280px] rounded-[3rem] overflow-hidden flex items-center justify-center bg-[#070708] border-[4px] border-[#1a1a1c]",
        "shadow-[0_20px_50px_rgba(0,0,0,0.5),inset_0_0_60px_rgba(0,0,0,0.8)]",
        className
      )}
      style={{
        boxShadow: `0 0 60px rgba(255,255,255,0.1), inset 0 0 50px rgba(0,0,0,0.9)`
      }}
    >
      <div className="absolute inset-x-0 top-0 h-[40%] bg-gradient-to-b from-[#ffffff10] to-transparent rounded-full blur-[2px] pointer-events-none transform -translate-y-4 scale-x-110" />
      <div className="absolute inset-0 bg-[url('/noise.svg')] opacity-[0.08] mix-blend-overlay pointer-events-none"></div>

      {/* Physics Wrapper for Parallax Eyes */}
      <motion.div 
        style={{ x: state === "sleeping" || isShuttered || isVoiceMode ? 0 : eyeOffsetX, y: state === "sleeping" || isShuttered || isVoiceMode ? 0 : eyeOffsetY }}
        className="relative z-10 flex items-center justify-center gap-10"
      >
        <motion.div
           animate={(isBlinking ? { height: 4, transition: { duration: 0.1 } } : { ...currentEye, rotate: getLeftRotate() }) as any}
          className="bg-white shadow-xl origin-center"
          style={{ boxShadow: `0 0 30px rgba(255,255,255,0.6), 0 0 60px rgba(255,255,255,0.3)` }}
        />
        <motion.div
           animate={(isBlinking ? { height: 4, transition: { duration: 0.1 } } : { ...currentEye, rotate: getRightRotate() }) as any}
          className="bg-white shadow-xl origin-center"
          style={{ boxShadow: `0 0 30px rgba(255,255,255,0.6), 0 0 60px rgba(255,255,255,0.3)` }}
        />
      </motion.div>

      {/* Annoyance Shutter Blast Doors */}
      <AnimatePresence>
        {isShuttered && (
          <>
            <motion.div 
              initial={{ y: "-100%" }} animate={{ y: "0%" }} exit={{ y: "-100%" }} 
              transition={{ type: "spring", bounce: 0.2, stiffness: 200, damping: 20 }}
              className="absolute top-0 left-0 w-full h-[50.5%] bg-[#1a1a1c] z-50 border-b-4 border-[#333] shadow-2xl flex items-end justify-center pb-2"
            >
              <div className="w-1/4 h-1.5 bg-black/40 rounded-full mb-1"></div>
            </motion.div>
            <motion.div 
              initial={{ y: "100%" }} animate={{ y: "0%" }} exit={{ y: "100%" }} 
              transition={{ type: "spring", bounce: 0.2, stiffness: 200, damping: 20 }}
              className="absolute bottom-0 left-0 w-full h-[50.5%] bg-[#1a1a1c] z-50 border-t-4 border-[#222] shadow-2xl flex items-start justify-center pt-2"
            >
              <div className="w-1/4 h-1.5 bg-black/40 rounded-full mt-1"></div>
            </motion.div>
          </>
        )}
      </AnimatePresence>

    </div>
  );
}
