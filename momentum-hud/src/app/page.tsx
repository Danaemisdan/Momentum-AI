"use client";

import { useEffect, useState, useRef, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
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

  // Notch Keyboard / Text Chat state
  const [isKeyboardOpen, setIsKeyboardOpen] = useState(false);
  const [inputText, setInputText] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const embeddedTextareaRef = useRef<HTMLTextAreaElement>(null);
  const closeTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const suggestions = [
    { label: "Summarize screen", query: "What do you see on my screen right now?" },
    { label: "Play Spotify", query: "Play music on Spotify" },
    { label: "Open Chrome", query: "Open Google Chrome" },
    { label: "Check status", query: "How is everything running?" },
  ];

  // Native Global Notch Cursor Tracker (Summons with zero clicks)
  useEffect(() => {
    let unlistenFn: (() => void) | null = null;
    listen("notch-hover", (event: any) => {
      const isHovered = event.payload as boolean;
      if (isVisibleRef.current) {
        // Momentum is active on screen; textbox is already embedded below Momentum's face
        return;
      }
      if (isHovered) {
        setIsKeyboardOpen(true);
        setTimeout(() => textareaRef.current?.focus(), 50);
      } else {
        // Only close if user isn't typing
        if (!inputText.trim()) {
          setIsKeyboardOpen(false);
          textareaRef.current?.blur();
        }
      }
    }).then((unlisten) => {
      unlistenFn = unlisten;
    });

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, [inputText]);
  

  
  const hideTimeout = useRef<NodeJS.Timeout | null>(null);
  const dialogueTimerRef = useRef<NodeJS.Timeout | null>(null);
  const isTransitioning = useRef(false);
  
  const ttsWsRef = useRef<WebSocket | null>(null);
  const agentWsRef = useRef<WebSocket | null>(null);
  const chatHistoryRef = useRef<{role: string, text: string}[]>([]);
  const speechBufferRef = useRef("");
  const streamedTokensCountRef = useRef(0);
  const inactivityTimerRef = useRef<NodeJS.Timeout | null>(null);

  const resetInactivityTimer = useCallback(() => {
      if (inactivityTimerRef.current) clearTimeout(inactivityTimerRef.current);
      inactivityTimerRef.current = setTimeout(() => {
          setIsVisible(false);
      }, 10000); // 10 second inactivity timeout
  }, []);

  const handleSubmitText = useCallback((e?: React.FormEvent) => {
    if (e) e.preventDefault();
    const query = inputText.trim();
    if (!query) return;

    setInputText("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
    if (embeddedTextareaRef.current) {
      embeddedTextareaRef.current.style.height = "auto";
    }
    setIsKeyboardOpen(false);
    textareaRef.current?.blur();
    embeddedTextareaRef.current?.blur();

    // Only when Enter is pressed does Momentum come in and start answering!
    setIsVisible(true);
    resetInactivityTimer();

    chatHistoryRef.current.push({ role: "user", text: query });
    const payload = JSON.stringify({
        history: chatHistoryRef.current,
        message: query
    });

    if (agentWsRef.current?.readyState === WebSocket.OPEN) {
        agentWsRef.current.send(payload);
    } else {
        try {
            const ws = new WebSocket("ws://127.0.0.1:44444/chat");
            ws.onopen = () => ws.send(payload);
            agentWsRef.current = ws;
        } catch(e) {}
    }
  }, [inputText, resetInactivityTimer]);

  const handleMouseEnterBar = useCallback(() => {
    if (closeTimeoutRef.current) {
      clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = null;
    }
    setIsKeyboardOpen(true);
    setTimeout(() => {
      if (isVisibleRef.current) {
        embeddedTextareaRef.current?.focus();
      } else {
        textareaRef.current?.focus();
      }
    }, 60);
  }, []);

  const handleMouseLeaveBar = useCallback(() => {
    // Only auto-close if input is empty
    if (!inputText.trim()) {
      if (closeTimeoutRef.current) clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = setTimeout(() => {
        setIsKeyboardOpen(false);
        textareaRef.current?.blur();
        embeddedTextareaRef.current?.blur();
      }, 280);
    }
  }, [inputText]);

  const displayDialogue = useCallback((text: string) => {
      setCurrentDialogue(text);
      currentDialogueRef.current = text;
      
      if (inactivityTimerRef.current) clearTimeout(inactivityTimerRef.current); // Pause timer while speaking
      
      if (dialogueTimerRef.current) clearTimeout(dialogueTimerRef.current);
      // Dynamic timeout based on text length to solve vanishing too early
      const ms = Math.max(3000, Math.min(10000, 3000 + text.length * 70));
      dialogueTimerRef.current = setTimeout(() => {
          setCurrentDialogue("");
          currentDialogueRef.current = "";
          resetInactivityTimer(); // Resume timer after speaking
      }, ms);
  }, [resetInactivityTimer]);

  const handleVoiceTrigger = useCallback(() => {
    setIsKeyboardOpen(false);
    setIsVisible(true);
    resetInactivityTimer();
    
    if (ttsWsRef.current?.readyState === WebSocket.OPEN) {
      const greeting = "I'm listening, go ahead.";
      ttsWsRef.current.send(greeting);
      displayDialogue(greeting);
    }
  }, [displayDialogue, resetInactivityTimer]);

  const handleExecuteSuggestion = useCallback((query: string) => {
    setIsKeyboardOpen(false);
    setInputText("");
    if (textareaRef.current) textareaRef.current.style.height = "auto";
    if (embeddedTextareaRef.current) embeddedTextareaRef.current.style.height = "auto";
    setIsVisible(true);
    resetInactivityTimer();

    chatHistoryRef.current.push({ role: "user", text: query });
    const payload = JSON.stringify({
      history: chatHistoryRef.current,
      message: query
    });

    if (agentWsRef.current?.readyState === WebSocket.OPEN) {
      agentWsRef.current.send(payload);
    } else {
      try {
        const ws = new WebSocket("ws://127.0.0.1:44444/chat");
        ws.onopen = () => ws.send(payload);
        agentWsRef.current = ws;
      } catch(e) {}
    }
  }, [resetInactivityTimer]);

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
            if (typeof event.data === "string") {
                const data = JSON.parse(event.data);
                if (data.type === "caption") {
                    displayDialogue(data.text);
                }
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
  }, [displayDialogue]);



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
                      streamedTokensCountRef.current += parsed.t.length;
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
                              displayDialogue(chunk);
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
              } else if (parsed.type === "action" || parsed.type === "thought" || parsed.type === "clarify") {
                  const wasStreamed = streamedTokensCountRef.current > 0;
                  streamedTokensCountRef.current = 0; // Reset for next message

                  // Final catch-all for any remaining text in buffer when stream generation completes
                  if (speechBufferRef.current.trim().length > 0) {
                      let chunk = speechBufferRef.current.trim();
                      if (chunk.includes("[ACTION: IGNORE]")) {
                          chunk = chunk.replace("[ACTION: IGNORE]", "").trim();
                      }
                      if (chunk.includes("[ACTION: SLEEP]") || chunk.includes("{SLEEP}")) {
                          setIsVisible(false);
                          setTimeout(() => invoke("hide_window_now"), 800);
                          chunk = chunk.replace("[ACTION: SLEEP]", "").replace("{SLEEP}", "").trim();
                      }
                      if (chunk.length > 0) {
                          chatHistoryRef.current.push({ role: "agent", text: chunk });
                          displayDialogue(chunk);
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

                  // Handle direct action speech or questions ONLY IF tokens were not already streamed
                  if (!wasStreamed) {
                      let directSpeech = parsed.speech;
                      if (!directSpeech && parsed.action && typeof parsed.action === "object") {
                          if ("talk" in parsed.action && parsed.action.talk?.speech) {
                              directSpeech = parsed.action.talk.speech;
                          } else if ("ask" in parsed.action && parsed.action.ask?.question) {
                              directSpeech = parsed.action.ask.question;
                          }
                      }

                      if (directSpeech && directSpeech.trim().length > 0) {
                          let text = directSpeech.trim();
                          if (text.includes("[ACTION: IGNORE]")) {
                              text = text.replace("[ACTION: IGNORE]", "").trim();
                          }
                          if (text.length > 0) {
                              chatHistoryRef.current.push({ role: "agent", text });
                              displayDialogue(text);
                              if (ttsWsRef.current?.readyState === WebSocket.OPEN) {
                                  ttsWsRef.current.send(text);
                              }
                              if (!isVisibleRef.current) {
                                  setIsVisible(true);
                                  invoke("show_window_now");
                              }
                          }
                      }
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
      const wakeRegex = /^(?:hey\s+|yo\s+|ok\s+|okay\s+)?momentum\b[,.]?\s*(.*)$/i;
      const wakeMatch = transcript.match(wakeRegex);

      if (!isVisibleRef.current && (wakeMatch !== null || lower.includes("momentum"))) {
          setIsVisible(true);
          invoke("show_window_now");
          
          const remainder = wakeMatch ? wakeMatch[1].trim() : transcript.replace(/^(?:hey\s+|yo\s+|ok\s+|okay\s+)?momentum\b[,.]?\s*/i, "").trim();
          
          if (!remainder || remainder.length < 3) {
             if (ttsWsRef.current?.readyState === WebSocket.OPEN) {
                const greetings = ["Yeah?", "I'm here.", "Go ahead.", "I'm listening.", "What's up?"];
                const greeting = greetings[Math.floor(Math.random() * greetings.length)];
                ttsWsRef.current.send(greeting);
                displayDialogue(greeting);
             }
             return;
          }
          transcript = remainder;
      }

      // Conversational Routing (Stream to LLM)
      if (isFinal) {
          let finalTranscript = transcript;
          
          // On Interruption: clear active dialogue immediately to feel responsive
          if (currentDialogueRef.current.trim() !== "") {
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
    <main className="flex flex-col h-screen w-screen items-center justify-start pt-0 bg-transparent select-none overflow-hidden">
      
      {/* Dynamic Notch Island Container (Translucent Liquid Glass) — Active only when Momentum is off-screen */}
      {!isVisible && (
        <div 
          onMouseEnter={handleMouseEnterBar}
          onMouseLeave={handleMouseLeaveBar}
          className="w-full flex flex-col items-center pt-0 pb-2 z-50 select-none cursor-default"
        >
          {/* 1. Resting Notch Lip (Subtle rounded glass capsule) */}
          {!isKeyboardOpen && (
            <div className="flex flex-col items-center cursor-pointer pt-0 transition-opacity duration-300">
              <div className="w-14 h-1.5 bg-white/30 hover:bg-white/60 rounded-full backdrop-blur-md shadow-[0_2px_8px_rgba(255,255,255,0.25)] transition-all duration-300" />
            </div>
          )}

          {/* 2. Expanded Liquid Glass Chat & Voice Unit (Animated Entrance & Exit with Spring Physics) */}
          <AnimatePresence>
            {isKeyboardOpen && (
              <motion.div
                initial={{ opacity: 0, y: -28, scale: 0.94, filter: "blur(16px)" }}
                animate={{ opacity: 1, y: 4, scale: 1, filter: "blur(0px)" }}
                exit={{ opacity: 0, y: -28, scale: 0.94, filter: "blur(16px)" }}
                transition={{ type: "spring", stiffness: 380, damping: 28, mass: 0.7 }}
                className="w-full flex flex-col items-center px-4"
              >
                {/* Main Frosted Translucent Liquid Glass Bar */}
                <form 
                  onSubmit={handleSubmitText}
                  className="relative flex items-center w-full max-w-[430px] rounded-[32px] bg-black/50 hover:bg-black/60 focus-within:bg-black/65 backdrop-blur-[50px] saturate-[180%] border border-white/20 focus-within:border-white/35 shadow-[0_20px_50px_rgba(0,0,0,0.55),inset_0_1px_1px_rgba(255,255,255,0.25),inset_0_0_15px_rgba(255,255,255,0.03)] px-4 py-2.5 transition-all duration-300"
                >
                  {/* Dynamic Auto-Expanding Textarea (Same Font & Style as Momentum) */}
                  <textarea
                    ref={textareaRef}
                    rows={1}
                    value={inputText}
                    onChange={(e) => {
                      setInputText(e.target.value);
                      e.target.style.height = "auto";
                      e.target.style.height = `${Math.min(e.target.scrollHeight, 120)}px`;
                    }}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        handleSubmitText();
                      } else if (e.key === "Escape") {
                        setIsKeyboardOpen(false);
                        textareaRef.current?.blur();
                      }
                    }}
                    placeholder="Ask or command..."
                    className="bg-transparent text-white text-[14px] font-medium tracking-wide leading-relaxed outline-none w-full placeholder-white/40 resize-none max-h-[120px] overflow-y-auto py-0.5 drop-shadow-sm"
                  />

                  {/* Right Controls: Thicker Rounded Mic & Enter Pods */}
                  <div className="relative flex items-center space-x-2 ml-2 shrink-0 self-end mb-0.5">
                    {/* Thick Rounded Voice Mode Button */}
                    <button
                      type="button"
                      onClick={handleVoiceTrigger}
                      title="Activate Voice Mode"
                      className="flex items-center justify-center w-8 h-8 rounded-full bg-white/[0.1] hover:bg-white/[0.22] border border-white/20 text-white/90 hover:text-white transition-all duration-200 active:scale-95 shadow-sm"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" className="w-4 h-4">
                        <rect x="9" y="2" width="6" height="12" rx="3" />
                        <path d="M5 10a7 7 0 0 0 14 0" />
                        <line x1="12" y1="17" x2="12" y2="21" />
                        <line x1="8" y1="21" x2="16" y2="21" />
                      </svg>
                    </button>

                    {/* Thick Rounded Enter Button */}
                    <button
                      type="submit"
                      title="Send (Enter)"
                      className="flex items-center justify-center w-8 h-8 rounded-full bg-white/[0.1] hover:bg-white/[0.22] border border-white/20 text-white/90 hover:text-white transition-all duration-200 active:scale-95 shadow-sm"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.6" strokeLinecap="round" strokeLinejoin="round" className="w-3.5 h-3.5">
                        <polyline points="9 10 4 15 9 20" />
                        <path d="M20 4v7a4 4 0 0 1-4 4H4" />
                      </svg>
                    </button>
                  </div>
                </form>

                {/* Vertical Suggestions List (Translucent Glass & Rounded Pill) */}
                {inputText.trim().length === 0 && (
                  <motion.div
                    initial={{ opacity: 0, y: -8, scale: 0.98 }}
                    animate={{ opacity: 1, y: 0, scale: 1 }}
                    exit={{ opacity: 0, y: -8, scale: 0.98 }}
                    transition={{ duration: 0.2 }}
                    className="w-full max-w-[430px] mt-2 rounded-[28px] bg-black/50 backdrop-blur-[50px] saturate-[180%] border border-white/15 shadow-[0_16px_40px_rgba(0,0,0,0.5),inset_0_1px_1px_rgba(255,255,255,0.15)] p-2 flex flex-col space-y-0.5"
                  >
                    <div className="px-3 py-1 text-[10px] uppercase font-mono tracking-wider text-white/35">
                      Suggestions
                    </div>
                    {suggestions.map((item, idx) => (
                      <button
                        key={idx}
                        type="button"
                        onClick={() => handleExecuteSuggestion(item.query)}
                        className="w-full text-left px-3.5 py-2.5 rounded-2xl hover:bg-white/[0.1] active:bg-white/[0.16] text-white/85 hover:text-white transition-all duration-150 flex items-center justify-between group"
                      >
                        <span className="text-[13px] font-medium tracking-wide">{item.label}</span>
                        <span className="text-[10px] font-mono text-white/40 group-hover:text-white/80 bg-white/[0.08] px-1.5 py-0.5 rounded-md transition-colors">
                          ↵
                        </span>
                      </button>
                    ))}
                  </motion.div>
                )}
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      )}

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
      <div className={`relative flex flex-col items-center justify-center pt-2 transition-all duration-700 ease-[cubic-bezier(0.34,1.56,0.64,1)] ${
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
        <div key={currentDialogue} className={`mt-[14px] w-[340px] px-5 py-3 rounded-3xl bg-black/50 backdrop-blur-[40px] border border-white/10 shadow-[0_8px_32px_rgba(0,0,0,0.6)] ${
           currentDialogue ? "animate-pop-in block" : "hidden"
        }`}>
            <p className="text-white text-base font-medium tracking-wide text-center leading-relaxed drop-shadow-md">
                {currentDialogue}
            </p>
        </div>

        {/* Collapsible Liquid Glass Chat Unit below Momentum — Pops out ONLY on hover */}
        {isVisible && (
          <div 
            onMouseEnter={handleMouseEnterBar}
            onMouseLeave={handleMouseLeaveBar}
            className="w-full flex flex-col items-center pt-2 pb-2 z-50 select-none cursor-default"
          >
            {/* 1. Subtle Resting Capsule under Momentum when not hovered */}
            {!isKeyboardOpen && (
              <div 
                onClick={() => {
                  setIsKeyboardOpen(true);
                  setTimeout(() => embeddedTextareaRef.current?.focus(), 60);
                }}
                className="flex flex-col items-center cursor-pointer pt-1 transition-opacity duration-300"
              >
                <div className="w-14 h-1.5 bg-white/30 hover:bg-white/60 rounded-full backdrop-blur-md shadow-[0_2px_8px_rgba(255,255,255,0.25)] transition-all duration-300" />
              </div>
            )}

            {/* 2. Expanded Liquid Glass Chat & Voice Unit — Pops down ONLY on hover */}
            <AnimatePresence>
              {isKeyboardOpen && (
                <motion.div
                  initial={{ opacity: 0, y: -14, scale: 0.95, filter: "blur(16px)" }}
                  animate={{ opacity: 1, y: 0, scale: 1, filter: "blur(0px)" }}
                  exit={{ opacity: 0, y: -14, scale: 0.95, filter: "blur(16px)" }}
                  transition={{ type: "spring", stiffness: 380, damping: 28, mass: 0.7 }}
                  className="w-full max-w-[430px] flex flex-col items-center px-2"
                >
                  {/* Main Frosted Translucent Liquid Glass Bar */}
                  <form 
                    onSubmit={handleSubmitText}
                    className="relative flex items-center w-full rounded-[32px] bg-black/50 hover:bg-black/60 focus-within:bg-black/65 backdrop-blur-[50px] saturate-[180%] border border-white/20 focus-within:border-white/35 shadow-[0_20px_50px_rgba(0,0,0,0.55),inset_0_1px_1px_rgba(255,255,255,0.25),inset_0_0_15px_rgba(255,255,255,0.03)] px-4 py-2.5 transition-all duration-300"
                  >
                    {/* Dynamic Auto-Expanding Textarea (Same Font & Style as Momentum) */}
                    <textarea
                      ref={embeddedTextareaRef}
                      rows={1}
                      value={inputText}
                      onChange={(e) => {
                        setInputText(e.target.value);
                        resetInactivityTimer();
                        e.target.style.height = "auto";
                        e.target.style.height = `${Math.min(e.target.scrollHeight, 100)}px`;
                      }}
                      onKeyDown={(e) => {
                        if (e.key === "Enter" && !e.shiftKey) {
                          e.preventDefault();
                          handleSubmitText();
                        } else if (e.key === "Escape") {
                          setIsKeyboardOpen(false);
                          embeddedTextareaRef.current?.blur();
                        }
                      }}
                      placeholder="Ask or command Momentum..."
                      className="bg-transparent text-white text-[14px] font-medium tracking-wide leading-relaxed outline-none w-full placeholder-white/40 resize-none max-h-[100px] overflow-y-auto py-0.5 drop-shadow-sm"
                    />

                    {/* Right Controls: Thicker Rounded Mic & Enter Pods */}
                    <div className="relative flex items-center space-x-2 ml-2 shrink-0 self-end mb-0.5">
                      {/* Thick Rounded Voice Mode Button */}
                      <button
                        type="button"
                        onClick={handleVoiceTrigger}
                        title="Activate Voice Mode"
                        className="flex items-center justify-center w-8 h-8 rounded-full bg-white/[0.1] hover:bg-white/[0.22] border border-white/20 text-white/90 hover:text-white transition-all duration-200 active:scale-95 shadow-sm"
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" className="w-4 h-4">
                          <rect x="9" y="2" width="6" height="12" rx="3" />
                          <path d="M5 10a7 7 0 0 0 14 0" />
                          <line x1="12" y1="17" x2="12" y2="21" />
                          <line x1="8" y1="21" x2="16" y2="21" />
                        </svg>
                      </button>

                      {/* Thick Rounded Enter Button */}
                      <button
                        type="submit"
                        title="Send (Enter)"
                        className="flex items-center justify-center w-8 h-8 rounded-full bg-white/[0.1] hover:bg-white/[0.22] border border-white/20 text-white/90 hover:text-white transition-all duration-200 active:scale-95 shadow-sm"
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.6" strokeLinecap="round" strokeLinejoin="round" className="w-3.5 h-3.5">
                          <polyline points="9 10 4 15 9 20" />
                          <path d="M20 4v7a4 4 0 0 1-4 4H4" />
                        </svg>
                      </button>
                    </div>
                  </form>

                  {/* Vertical Suggestions List (Translucent Glass & Rounded Pill) */}
                  {inputText.trim().length === 0 && (
                    <motion.div
                      initial={{ opacity: 0, y: -6, scale: 0.98 }}
                      animate={{ opacity: 1, y: 0, scale: 1 }}
                      exit={{ opacity: 0, y: -6, scale: 0.98 }}
                      transition={{ duration: 0.2 }}
                      className="w-full max-w-[430px] mt-2 rounded-[28px] bg-black/50 backdrop-blur-[50px] saturate-[180%] border border-white/15 shadow-[0_16px_40px_rgba(0,0,0,0.5),inset_0_1px_1px_rgba(255,255,255,0.15)] p-2 flex flex-col space-y-0.5"
                    >
                      <div className="px-3 py-1 text-[10px] uppercase font-mono tracking-wider text-white/35">
                        Suggestions
                      </div>
                      {suggestions.map((item, idx) => (
                        <button
                          key={idx}
                          type="button"
                          onClick={() => handleExecuteSuggestion(item.query)}
                          className="w-full text-left px-3.5 py-2.5 rounded-2xl hover:bg-white/[0.1] active:bg-white/[0.16] text-white/85 hover:text-white transition-all duration-150 flex items-center justify-between group"
                        >
                          <span className="text-[13px] font-medium tracking-wide">{item.label}</span>
                          <span className="text-[10px] font-mono text-white/40 group-hover:text-white/80 bg-white/[0.08] px-1.5 py-0.5 rounded-md transition-colors">
                            ↵
                          </span>
                        </button>
                      ))}
                    </motion.div>
                  )}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        )}

      </div>
      
    </main>
  );
}
