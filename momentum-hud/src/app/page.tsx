"use client";

import { useEffect, useState, useRef, useCallback } from "react";
// Tauri APIs dynamic wrappers for Next.js SSR compatibility
const invoke = async (...args: any[]) => {
  if (typeof window !== "undefined") {
    const core = await import("@tauri-apps/api/core");
    return core.invoke(args[0], args[1]);
  }
};
const listen = async (...args: any[]) => {
  if (typeof window !== "undefined") {
    const event = await import("@tauri-apps/api/event");
    return event.listen(args[0], args[1]);
  }
  return () => {};
};

export default function Home() {
  const [status, setStatus] = useState({ agent: false, audio: false, vision: false });
  const [showSplash, setShowSplash] = useState(true);
  const [isVisible, setIsVisible] = useState(false);
  const isVisibleRef = useRef(false);
  useEffect(() => { isVisibleRef.current = isVisible; }, [isVisible]);
  const [currentDialogue, setCurrentDialogue] = useState("");
  const currentDialogueRef = useRef("");
  

  
  const hideTimeout = useRef<NodeJS.Timeout | null>(null);
  const dialogueTimerRef = useRef<NodeJS.Timeout | null>(null);
  const isTransitioning = useRef(false);
  
  const ttsWsRef = useRef<WebSocket | null>(null);
  const agentWsRef = useRef<WebSocket | null>(null);
  const chatHistoryRef = useRef<{role: string, text: string}[]>([]);
  const speechBufferRef = useRef("");
  const inactivityTimerRef = useRef<NodeJS.Timeout | null>(null);

  const resetInactivityTimer = useCallback(() => {
      if (inactivityTimerRef.current) clearTimeout(inactivityTimerRef.current);
      inactivityTimerRef.current = setTimeout(() => {
          setIsVisible(false);
          setTimeout(() => invoke("hide_window_now"), 800);
      }, 10000); // 10 second inactivity timeout
  }, []);

  useEffect(() => {
      if (isVisible) {
          resetInactivityTimer();
      } else {
          if (inactivityTimerRef.current) clearTimeout(inactivityTimerRef.current);
      }
  }, [isVisible, resetInactivityTimer]);

  // 1. Initialize Kokoro TTS WebSocket (Persistent with Reconnect)
  useEffect(() => {
    let ws: WebSocket | null = null;
    let isMounted = true;
    
    const connect = () => {
      ws = new WebSocket("ws://127.0.0.1:8000/tts");
      ws.onmessage = async (event) => {
        try {
            const data = JSON.parse(event.data);
            if (data.type === "caption") {
                displayDialogue(data.text);
            }
        } catch(e) {}
      };
      ws.onclose = () => {
        if (isMounted) setTimeout(connect, 2000);
      };
      ttsWsRef.current = ws;
    };
    
    connect();
    
    return () => {
      isMounted = false;
      if (ws) ws.close();
    };
  }, []);

  const displayDialogue = useCallback((text: string) => {
      setCurrentDialogue(text);
      currentDialogueRef.current = text;
      
      if (inactivityTimerRef.current) clearTimeout(inactivityTimerRef.current); // Pause timer while speaking
      
      if (dialogueTimerRef.current) clearTimeout(dialogueTimerRef.current);
      // Dynamic timeout based on text length to solve vanishing too early
      const ms = Math.max(2000, Math.min(8000, 2000 + text.length * 60));
      dialogueTimerRef.current = setTimeout(() => {
          setCurrentDialogue("");
          currentDialogueRef.current = "";
          resetInactivityTimer(); // Resume timer after speaking
      }, ms);
  }, [resetInactivityTimer]);

  // 2. Initialize LLM Agent WebSocket (Persistent with Reconnect)
  useEffect(() => {
    let ws: WebSocket | null = null;
    let isMounted = true;
    
    const connect = () => {
      ws = new WebSocket("ws://127.0.0.1:44444/chat");
      ws.onmessage = (event) => {
          try {
              const parsed = JSON.parse(event.data);
              
              if (parsed.type === "token") {
                  if (parsed.t) {
                      speechBufferRef.current += parsed.t;
                      
                      // Instant sentence-level TTS dispatch (Ultra-low latency)
                      if (/[.!?]\s/.test(speechBufferRef.current) || /[.!?]$/.test(speechBufferRef.current)) {
                          let chunk = speechBufferRef.current.trim();
                          
                          // Handle Actions in stream
                          if (chunk.includes("[ACTION: IGNORE]")) {
                              chunk = chunk.replace("[ACTION: IGNORE]", "").trim();
                              speechBufferRef.current = "";
                              if (chunk.length === 0) return;
                          }
                          
                          if (chunk.includes("[ACTION: SLEEP]") || chunk.includes("{SLEEP}")) {
                              setIsVisible(false);
                              setTimeout(() => invoke("hide_window_now"), 800);
                              chunk = chunk.replace("[ACTION: SLEEP]", "").replace("{SLEEP}", "").trim();
                          }
                          
                          if (chunk.length > 0) {
                              chatHistoryRef.current.push({ role: "agent", text: chunk });
                              if (ttsWsRef.current?.readyState === WebSocket.OPEN) {
                                  ttsWsRef.current.send(chunk);
                              }
                              
                              if (!isVisibleRef.current) {
                                  setIsVisible(true);
                                  invoke("show_window_now");
                              }
                          }
                          speechBufferRef.current = ""; // Reset buffer after sending
                      }
                  }
              } else if (parsed.type === "action") {
                  // Final catch-all for any remaining text in buffer when generation completes
                  if (speechBufferRef.current.trim().length > 0) {
                      let chunk = speechBufferRef.current.trim();
                      if (chunk.includes("[ACTION: IGNORE]")) {
                          chunk = chunk.replace("[ACTION: IGNORE]", "").trim();
                          if (chunk.length === 0) {
                              speechBufferRef.current = "";
                              return;
                          }
                      }
                      if (chunk.includes("[ACTION: SLEEP]") || chunk.includes("{SLEEP}")) {
                          setIsVisible(false);
                          setTimeout(() => invoke("hide_window_now"), 800);
                          chunk = chunk.replace("[ACTION: SLEEP]", "").replace("{SLEEP}", "").trim();
                      }
                      if (chunk.length > 0) {
                          chatHistoryRef.current.push({ role: "agent", text: chunk });
                          if (ttsWsRef.current?.readyState === WebSocket.OPEN) {
                              ttsWsRef.current.send(chunk);
                          }
                          if (!isVisibleRef.current) {
                              setIsVisible(true);
                              invoke("show_window_now");
                          }
                      }
                      speechBufferRef.current = "";
                  }
              }
          } catch(e) {}
      };
      ws.onclose = () => {
        if (isMounted) setTimeout(connect, 2000);
      };
      agentWsRef.current = ws;
    };
    
    connect();
    
    return () => {
      isMounted = false;
      if (ws) ws.close();
    };
  }, [displayDialogue]);

  // 3. Voice Activity Detection & Conversational Routing
  const handleSpeech = useCallback((transcript: string, isFinal: boolean) => {
      if (!transcript.trim()) return;
      const lower = transcript.toLowerCase();
      
      // Filter out common Whisper hallucinations on silence
      const trimmed = lower.replace(/[^a-z]/g, "");
      if (trimmed === "you" || trimmed === "thankyou" || lower === "[silence]") {
          return;
      }
      
      if (isVisible) {
          resetInactivityTimer();
      }
      
      // Wake Word Activation
      if (!isVisibleRef.current && (lower.includes("momentum") || lower.includes("hey momentum"))) {
          setIsVisible(true);
          invoke("show_window_now");
          
          if (ttsWsRef.current?.readyState === WebSocket.OPEN) {
             const greetings = ["Yeah?", "I'm here.", "Go ahead.", "I'm listening.", "What's up?", "Mm-hmm?"];
             const greeting = greetings[Math.floor(Math.random() * greetings.length)];
             ttsWsRef.current.send(greeting);
             displayDialogue(greeting);
          }
          return;
      }

      // Conversational Routing (Stream to LLM)
      if (isFinal) {
          let finalTranscript = transcript;
          
          // Interruption Context Injection
          if (currentDialogueRef.current.trim() !== "") {
              finalTranscript = `[System Note: You were just interrupted by the user mid-sentence. React naturally and conversationally (e.g. 'yeah?', 'go ahead') before addressing their input.] User says: ${transcript}`;
              // Clear the dialogue immediately on screen to feel responsive
              setCurrentDialogue("");
              currentDialogueRef.current = "";
          }
          
          chatHistoryRef.current.push({ role: "user", text: finalTranscript });
          if (agentWsRef.current?.readyState === WebSocket.OPEN) {
              agentWsRef.current.send(JSON.stringify({
                  history: chatHistoryRef.current,
                  message: finalTranscript
              }));
          }
      }
  }, []);

  // Connect to Python Backend STT (Persistent with Reconnect)
  useEffect(() => {
      let ws: WebSocket | null = null;
      let isMounted = true;
      
      const connect = () => {
        ws = new WebSocket("ws://127.0.0.1:8000/stt");
        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                handleSpeech(data.transcript, data.isFinal);
            } catch(e) {}
        };
        ws.onclose = () => {
          if (isMounted) setTimeout(connect, 2000);
        };
      };
      
      connect();
      
      return () => {
        isMounted = false;
        if (ws) ws.close();
      };
  }, [handleSpeech]);

  // Handle System Tray manual toggle
  useEffect(() => {
    let isMounted = true;
    const unlistenPromise = listen("visibility-toggle", () => {
      if (!isMounted) return;
      if (isTransitioning.current) return;
      isTransitioning.current = true;
      setTimeout(() => { isTransitioning.current = false; }, 1000);

      setIsVisible((prev) => {
        const next = !prev;
        if (next) {
          if (hideTimeout.current) {
            clearTimeout(hideTimeout.current);
            hideTimeout.current = null;
          }
          displayDialogue("I'm here.");
          setTimeout(() => {
            if (ttsWsRef.current && ttsWsRef.current.readyState === WebSocket.OPEN) {
              ttsWsRef.current.send("I'm here.");
            }
          }, 200);
        } else {
          displayDialogue("Going to sleep.");
          setTimeout(() => {
            if (ttsWsRef.current && ttsWsRef.current.readyState === WebSocket.OPEN) {
              ttsWsRef.current.send("Going to sleep.");
            }
          }, 200);
          hideTimeout.current = setTimeout(() => {
            invoke("hide_window_now");
          }, 800);
        }
        return next;
      });
    });

    return () => {
      isMounted = false;
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [displayDialogue]);

  // Splash screen transition
  useEffect(() => {
    const timer = setTimeout(() => {
      setShowSplash(false);
    }, 2000); // 2 second splash screen
    return () => clearTimeout(timer);
  }, []);

  // Status polling
  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const res = await invoke("get_status");
        setStatus(res as any);
      } catch (e) {
        console.error(e);
      }
    };

    fetchStatus();
    const interval = setInterval(fetchStatus, 2000);
    return () => clearInterval(interval);
  }, []);

  const isOnline = status.agent;

  return (
    <main className="flex flex-col h-screen w-screen items-center justify-start pt-[20px] bg-transparent select-none overflow-hidden">
      
      {/* Splash Screen (Delta Logo) */}
      <div 
        className={`absolute top-[20px] w-full h-[120px] flex items-center justify-center pointer-events-none transition-all duration-1000 ease-in-out ${
          showSplash ? "opacity-100 scale-100" : "opacity-0 scale-50"
        }`}
      >
        <div className="w-16 h-16 drop-shadow-[0_0_15px_rgba(255,255,255,0.5)]">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 375 375" className="w-full h-full text-white">
            <defs><clipPath id="clip-splash"><path d="M 45 64 L 330 64 L 330 311 L 45 311 Z" /></clipPath></defs>
            <g clipPath="url(#clip-splash)"><path fill="currentColor" d="M 187.5 64.3 L 329.7 310.6 L 187.5 239.4 L 45.3 310.6 Z" /></g>
          </svg>
        </div>
      </div>

      {/* Momentum Container */}
      <div className={`relative flex flex-col items-center justify-center transition-all duration-700 ease-[cubic-bezier(0.34,1.56,0.64,1)] ${
          showSplash ? "opacity-0 scale-150 blur-xl" : (isVisible ? "opacity-100 scale-100 translate-y-0 blur-0" : "opacity-0 scale-100 -translate-y-[150px] blur-md pointer-events-none")
      }`}>
        
        {/* Momentum Face */}
        <div 
          data-tauri-drag-region
          className="animate-breathe relative flex items-center justify-center w-[120px] h-[120px] bg-[#0A0A0C] rounded-[32px] cursor-grab active:cursor-grabbing border border-white/5 drop-shadow-[0_10px_25px_rgba(0,0,0,0.5)]"
        >
          {/* Glowing Eyes */}
          <div data-tauri-drag-region className="animate-blink flex space-x-[20px] pointer-events-none">
            <div className={`w-[14px] rounded-full bg-white transition-all duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)] ${
              isOnline ? "h-[42px] drop-shadow-[0_0_15px_rgba(255,255,255,0.9)]" : "h-[10px] opacity-40 drop-shadow-none"
            }`} />
            <div className={`w-[14px] rounded-full bg-white transition-all duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)] delay-75 ${
              isOnline ? "h-[42px] drop-shadow-[0_0_15px_rgba(255,255,255,0.9)]" : "h-[10px] opacity-40 drop-shadow-none"
            }`} />
          </div>
        </div>

        {/* Liquid Glass Dialogue Box */}
        <div key={currentDialogue} className={`mt-[20px] w-[340px] px-6 py-4 rounded-3xl bg-black/50 backdrop-blur-[40px] border border-white/10 shadow-[0_8px_32px_rgba(0,0,0,0.6)] ${
           currentDialogue ? "animate-pop-in block" : "hidden"
        }`}>
            <p className="text-white text-lg font-medium tracking-wide text-center leading-relaxed drop-shadow-md">
                {currentDialogue}
            </p>
        </div>

      </div>
      
    </main>
  );
}
