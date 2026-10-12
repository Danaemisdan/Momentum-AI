"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { AgentState } from "@/components/AgentFace";
import React from "react";

export function useAgentVoice(setAgentState: React.Dispatch<React.SetStateAction<AgentState>>) {
  const [isReady, setIsReady] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  
  // AudioContext strictly manages the raw Float32 arrays returned from the Python RVC Pipeline!
  const audioCtxRef = useRef<AudioContext | null>(null);
  const nextStartTimeRef = useRef<number>(0);
  
  // Track all actively playing web-audio source nodes to cleanly flush them on interrupt.
  const sourceNodesRef = useRef<Set<AudioBufferSourceNode>>(new Set());

  useEffect(() => {
    if (typeof window === "undefined") return;
    let isShuttingDown = false;

    // Connect to the localized Python Piper+RVC Orchestrator
    const connectRVC = () => {
      const ws = new WebSocket("ws://127.0.0.1:8000/tts");
      
      ws.onopen = () => {
        setIsReady(true);
        console.log("RVC Neural Pipeline Linked 🎭⚡");
      };

      // When pure PCM bytecode streams back from Python, we schedule it synchronously.
      ws.onmessage = async (event) => {
        // Drop any non-binary text payloads instantly to stop arrayBuffer() crashes!
        if (typeof event.data === "string") {
          console.warn("Received text via Audio WS Stream, dropping chunk: ", event.data);
          return;
        }

        if (!audioCtxRef.current) {
          audioCtxRef.current = new (window.AudioContext || (window as any).webkitAudioContext)();
        }

        const ctx = audioCtxRef.current;
        if (ctx.state === "suspended") {
            try { await ctx.resume(); } catch(e) {}
        }

        try {
          // Decode the Python Web Audio Blob directly into a Float32 DSP Array!
          const arrayBuffer = await event.data.arrayBuffer();
          const audioBuffer = await ctx.decodeAudioData(arrayBuffer);

          const source = ctx.createBufferSource();
          source.buffer = audioBuffer;
          
          // Boost native Web Audio TTS Gain by +250% (fixes low volume issues)
          const gainNode = ctx.createGain();
          gainNode.gain.value = 2.5; 

          source.connect(gainNode);
          gainNode.connect(ctx.destination);
          
          sourceNodesRef.current.add(source);

          // Force the Agent Avatar mouth to stay open!
          setAgentState("speaking");

          // Calculate seamless overlap time to prevent any jittering between 5-word Pipeline chunks!
          const currentTime = ctx.currentTime;
          // Add a microscopic 20ms crossfade buffer against RVC artifacts
          const startTime = Math.max(currentTime, nextStartTimeRef.current); 

          source.start(startTime);
          nextStartTimeRef.current = startTime + audioBuffer.duration;

          source.onended = () => {
            sourceNodesRef.current.delete(source);
            if (sourceNodesRef.current.size === 0 && ctx.currentTime >= nextStartTimeRef.current - 0.1) {
              setAgentState((prev: any) => prev === "speaking" ? "idle" : prev);
            }
          };

        } catch (e) {
          console.error("RVC Byte Decoding Error", e);
        }
      };

      ws.onclose = () => {
        setIsReady(false);
        // Force aggressive reconnection if Python microservice rebooted
        if (!isShuttingDown) {
          reconnectTimeoutRef.current = setTimeout(connectRVC, 2000);
        }
      };

      wsRef.current = ws;
    };

    connectRVC();

    return () => {
      isShuttingDown = true;
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.onclose = null;
      }
      wsRef.current?.close();
    };
  }, []);

  const speakChunk = useCallback((text: string) => {
    const cleanText = text.trim();
    if (!cleanText || !wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;

    // Dispatch the exact 5-10 word clause to the Python Piper loop instantly.
    wsRef.current.send(cleanText);
  }, []);

  const initAudio = useCallback(() => {
    if (typeof window === "undefined") return;
    try {
        if (!audioCtxRef.current) {
            audioCtxRef.current = new (window.AudioContext || (window as any).webkitAudioContext)();
        }
        if (audioCtxRef.current.state === "suspended") {
            audioCtxRef.current.resume();
        }
    } catch(e) {}
  }, []);

  const stopSynthesis = useCallback(() => {
    // 1. Terminate all active buffered 5-word chunks currently loaded in the RAM AudioContext!
    sourceNodesRef.current.forEach(source => {
      try { source.stop(); } catch(e) {}
    });
    sourceNodesRef.current.clear();
    
    // 2. Clear the time-offset scheduler so the next chunk starts instantly!
    if (audioCtxRef.current) {
       nextStartTimeRef.current = audioCtxRef.current.currentTime;
    }
  }, []);

  return { speakChunk, stopSynthesis, initAudio, isReady };
}
