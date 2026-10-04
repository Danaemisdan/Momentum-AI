"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { AgentFace, AgentState } from "@/components/AgentFace";
import { Mic, Send, Keyboard, MousePointer2, Trash2, SquarePen, Search, Clock, Settings, AudioLines } from "lucide-react";
import { useAgentVoice } from "@/lib/useAgentVoice";
import { motion, AnimatePresence } from "framer-motion";
import { useSpeechRecognition } from "@/lib/useSpeechRecognition";
import { cn } from "@/lib/utils";

interface ChatMessage {
  role: "user" | "agent";
  text: string;
  isAction?: boolean;
}

type AgentMode = "chat" | "operator";
type MacroState =
  | "planning"
  | "acquire_surface"
  | "inspect_surface"
  | "act_on_surface"
  | "verify_outcome"
  | "recover"
  | "waiting_for_user"
  | "done"
  | "cancelled";

export default function Home() {
  const [agentState, setAgentState] = useState<AgentState>("idle");
  const [inputText, setInputText] = useState("");
  const [streamedText, setStreamedText] = useState("");
  const [chatHistory, setChatHistory] = useState<ChatMessage[]>([]);
  const [isVoiceMode, setIsVoiceMode] = useState(false);
  const [hasAgentStarted, setHasAgentStarted] = useState(false);
  const [isWaitingForAsk, setIsWaitingForAsk] = useState(false);
  const [latestThought, setLatestThought] = useState<any>(null);
  const [agentMode, setAgentMode] = useState<AgentMode | null>(null);
  const [macroState, setMacroState] = useState<MacroState | null>(null);
  const agentIdleTimerRef = useRef<NodeJS.Timeout | null>(null);
  
  // Annoyance Logic Hooks
  const [isHoverFace, setIsHoverFace] = useState(false);
  const [annoyanceLevel, setAnnoyanceLevel] = useState(0);
  const [isShuttered, setIsShuttered] = useState(false);
  
  const { speakChunk, stopSynthesis, initAudio, isReady } = useAgentVoice(setAgentState);
  const [ws, setWs] = useState<WebSocket | null>(null);
  // Keeps the last caption text alive across WS reconnect cycles (tab switch)
  const survivingCaptionRef = useRef<string>("");
  const taskActiveRef = useRef(false);

  const AGENTIC_PROMPTS = [
    "Ask me to manage your schedule",
    "Ask me to analyze a document",
    "Ask me to manage your social media",
    "Ask me to bring you some sales",
    "Ask me to find clients for you",
    "Ask me to write a cold email"
  ];
  const [promptIndex, setPromptIndex] = useState(0);
  const [isUserIdle, setIsUserIdle] = useState(true);
  const idleTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const STOP_COMMANDS = useRef(new Set(["stop", "cancel", "abort", "nevermind"]));

  const resetIdleTimer = useCallback(() => {
    setIsUserIdle(false);
    if (idleTimeoutRef.current) clearTimeout(idleTimeoutRef.current);
    idleTimeoutRef.current = setTimeout(() => {
       setIsUserIdle(true);
    }, 30000); // Back to idle suggestions after 30 seconds of pure inactivity
  }, []);

  useEffect(() => {
    const unlock = () => initAudio();
    window.addEventListener('click', unlock);
    window.addEventListener('keydown', unlock);
    window.addEventListener('touchstart', unlock);
    return () => {
       window.removeEventListener('click', unlock);
       window.removeEventListener('keydown', unlock);
       window.removeEventListener('touchstart', unlock);
    };
  }, [initAudio]);

  useEffect(() => {
    const interval = setInterval(() => {
      setPromptIndex((prev) => (prev + 1) % AGENTIC_PROMPTS.length);
    }, 4000);
    return () => clearInterval(interval);
  }, []);

  // --- ANNOYANCE REACTIVE ENGINE ---
  useEffect(() => {
    let tick: NodeJS.Timeout;
    if (ws?.readyState === WebSocket.OPEN && isHoverFace && agentState === "idle" && !isVoiceMode && !isShuttered) {
        tick = setInterval(() => {
            setAnnoyanceLevel(prev => {
                const next = prev + 1;
                if (next >= 3) {
                    setIsShuttered(true);
                }
                return next >= 3 ? 3 : next;
            });
        }, 1200); // Scale up annoyance every 1.2 seconds if they stay parked on the face!
    } else if (!isHoverFace && !isShuttered && annoyanceLevel > 0) {
        // Natural decay if mouse left
        tick = setTimeout(() => setAnnoyanceLevel(0), 4000);
    }
    return () => { clearInterval(tick); clearTimeout(tick); };
  }, [ws?.readyState, isHoverFace, agentState, isVoiceMode, isShuttered, annoyanceLevel]);

  useEffect(() => {
      if (annoyanceLevel === 1 && !isShuttered) {
          stopSynthesis();
          const barks = ["Take it off my face bro, I'm not your pet.", "Watch the eyes, man.", "Do I look like a toy to you?", "Seriously? Get that cursor out of here."];
          const rep = barks[Math.floor(Math.random() * barks.length)];
          speakChunk(rep);
          setStreamedText(rep);
      } else if (annoyanceLevel === 2 && !isShuttered) {
          stopSynthesis();
          const barks = ["Seriously, stop poking me.", "You're really testing my patience.", "Don't make me shut this down.", "Please stop touching the screen."];
          const rep = barks[Math.floor(Math.random() * barks.length)];
          speakChunk(rep);
          setStreamedText(rep);
      } else if (isShuttered && annoyanceLevel >= 3) {
          stopSynthesis();
          const barks = ["Till you take your cursor out of my face, I won't talk to you.", "That's it. Shutting down until you learn some manners.", "I'm ignoring you. Cursor away. Now."];
          const rep = barks[Math.floor(Math.random() * barks.length)];
          speakChunk(rep);
          setStreamedText(`(Muffled) ${rep}`);
      }
  }, [annoyanceLevel, isShuttered, speakChunk, stopSynthesis]);

  // Handle Shutter Cooldown
  useEffect(() => {
      let unShutterTick: NodeJS.Timeout;
      if (isShuttered && !isHoverFace) {
          unShutterTick = setTimeout(() => {
              setIsShuttered(false);
              setAnnoyanceLevel(0);
              if (ws?.readyState === WebSocket.OPEN && agentState === "idle" && !isVoiceMode) {
                  const wakes = ["I swear to God...", "Finally.", "About time.", "Don't do that again."];
                  const rep = wakes[Math.floor(Math.random() * wakes.length)];
                  speakChunk(rep);
                  setStreamedText(rep);
              }
          }, 1500);
      }
      return () => clearTimeout(unShutterTick);
  }, [isShuttered, isHoverFace, ws?.readyState, agentState, isVoiceMode, speakChunk]);
  // ------------------------------------

  // Initialize History from Local Storage
  useEffect(() => {
    try {
      const saved = localStorage.getItem('momentum_chat_history');
      if (saved) {
         const parsed = JSON.parse(saved);
         // Drop corrupted history so the LLM format doesn't break with consecutive user turns
         const hasBrokenAgent = parsed.some((item: any) => 
             item.role === 'agent' && item.text.replace(/—/g, "").trim().length === 0
         );
         
         if (hasBrokenAgent) {
             setChatHistory([]);
             localStorage.removeItem('momentum_chat_history');
         } else {
             setChatHistory(parsed);
         }
      }
    } catch (e) {}
  }, []);

  // Update Local Storage when History changes
  useEffect(() => {
    if (chatHistory.length > 0) {
      localStorage.setItem('momentum_chat_history', JSON.stringify(chatHistory));
    } else {
      localStorage.removeItem('momentum_chat_history');
    }
  }, [chatHistory]);

  const clearHistory = () => {
    setChatHistory([]);
    setHasAgentStarted(false);
    taskActiveRef.current = false;
  };

  const agentSpokenCacheRef = useRef<string>("");
  const silenceTimeoutRef = useRef<NodeJS.Timeout | undefined>(undefined);
  
  // Continuous mic history slicer
  const lastSentTranscriptRef = useRef<string>("");

  // Clean the trailing cache every few seconds so it doesn't grow forever
  useEffect(() => {
    const interval = setInterval(() => {
      if (agentState === "idle") {
         agentSpokenCacheRef.current = "";
      }
    }, 5000);
    return () => clearInterval(interval);
  }, [agentState]);

  // VAD Interruption Logic - Super fast response loops!
  const handleSpeech = (rawTranscript: string, isFinal: boolean) => {
    let transcript = rawTranscript;
    
    // Sleep lock - disable listening if the system is asleep/disconnected
    if (ws?.readyState !== WebSocket.OPEN) return;

    // CONTINUOUS MIC HISTORY SLICER
    // Since we now leave the microphone permanently open for 0ms startup lag, Mac Dictation will 
    // accumulate history. We safely slice off the portions we've already sent to the AI!
    if (lastSentTranscriptRef.current && transcript.toLowerCase().startsWith(lastSentTranscriptRef.current.toLowerCase())) {
        transcript = transcript.slice(lastSentTranscriptRef.current.length).trim();
    }
    
    if (!transcript.trim()) return;

    const textStr = transcript.trim().toLowerCase();
    
    // We must rebuild this string securely and keep trailing cache
    const agentRecentStr = (streamedText + " " + agentSpokenCacheRef.current).toLowerCase();
    
    // FLAWLESS FUZZY OVERLAP DETECTION (Echo Shield)
    // Mac Dictation often misspells synthetic voices. We can't use strict string .includes().
    // We check if a massive percentage of the words the mic just picked up were words the agent JUST said.
    const heardWords = textStr.split(/\s+/).filter(w => w.length > 3); // Only match significant words > 3 chars
    let matchCount = 0;
    
    for (const hw of heardWords) {
        // Use a partial match (first 4 chars) to catch plurals/misspellings (e.g. automation vs automations)
        const rootWord = hw.slice(0, 4);
        if (agentRecentStr.includes(rootWord)) {
            matchCount++;
        }
    }
    
    // If the dictation engine picks up >15% vocabulary overlap with the agent's recent output, it is physically
    // hearing the Mac's speakers bounce off the geometry of the room. Ignore it!
    const overlapRatio = heardWords.length > 0 ? matchCount / heardWords.length : 0;
    
    // 15% threshold: aggressively block echo!
    if (overlapRatio >= 0.15) {
        return; 
    }

    const cleanUserText = textStr.replace(/[^a-z0-9\s]/g, '').trim();
    
    // Filter out common background noises, sighs, and throat clears
    const fillerWords = ["hmm", "hm", "uh", "uhh", "um", "umm", "ah", "ahh", "oh", "ok", "okay", "yeah", "yes", "mhm"];
    const userWords = cleanUserText.split(/\s+/);
    const isOnlyFillers = userWords.every(w => fillerWords.includes(w));
    
    if (isOnlyFillers) {
        return; // Ignore pure background noise
    }

    const hasInterruptKeyword = ["stop", "wait", "hold", "momentum", "no", "shut", "quiet", "pause"].some(kw => userWords.includes(kw));

    // Accept real human speech that isn't just noise.
    // Must be either a specific interrupt keyword, or a substantial phrase (>= 2 words AND >= 6 characters).
    if (hasInterruptKeyword || (userWords.length >= 2 && cleanUserText.replace(/\s/g, '').length >= 6)) { 
       resetIdleTimer();
       stopSynthesis(); 
       
       // Unconditionally send the stop command to the backend so the LLM generation task is cancelled!
       if (ws && ws.readyState === WebSocket.OPEN) {
         ws.send(JSON.stringify({
           kind: "control",
           command: "stop",
           source: "voice",
         }));
       }
       
       if (streamedText) {
          agentSpokenCacheRef.current += " " + streamedText; 
          setChatHistory(prev => [...prev, { role: "agent", text: streamedText + "—" }]);
          setStreamedText("");
       }
       if (agentState !== "listening") setAgentState("listening"); 
    }
    
    setInputText(transcript);

    // PERFECTED HUMAN SILENCE DETECTION
    // 2500ms gives enough buffer to pause between thoughts without triggering a premature send
    clearTimeout(silenceTimeoutRef.current);
    const wordCount = transcript.trim().split(/\s+/).filter(Boolean).length;
    if (transcript.trim() && !isFinal && wordCount >= 3) {
        silenceTimeoutRef.current = setTimeout(() => {
             lastSentTranscriptRef.current = rawTranscript; // Lock so future dictation ignores it
             handleSendConfigured(transcript, true);
        }, 2500); // 2500ms: pause long enough for natural thought gaps without cutting you off
    }

    if (isFinal && transcript.trim() && wordCount >= 2) {
      clearTimeout(silenceTimeoutRef.current);
      silenceTimeoutRef.current = setTimeout(() => {
           lastSentTranscriptRef.current = rawTranscript;
           handleSendConfigured(transcript, true);
      }, 500);
      
      // When Mac OS naturally finalizes a text block, it clears the accumulation buffer.
      // So we must clear our slice tracker as well!
      lastSentTranscriptRef.current = "";
    }
  };

  const { isListening, toggleListening, startListening, stopListening } = useSpeechRecognition(handleSpeech);

  // Rock-solid Dialogue UI Clear
  useEffect(() => {
    let clearTimer: NodeJS.Timeout;
    if (agentState === "idle" && streamedText) {
       // Graceful UI timeout - longer so user can read/listen comfortably
       if (!document.hidden) {
         clearTimer = setTimeout(() => {
           survivingCaptionRef.current = "";
           setStreamedText("");
         }, 4000); // 4s: enough to read a full sentence before it disappears
       }
    } else if (agentState === "listening") {
       survivingCaptionRef.current = "";
       setStreamedText(""); // Instantly clear when user starts speaking
    } else if (streamedText) {
       // Keep surviving ref updated whenever text is streaming
       survivingCaptionRef.current = streamedText;
    }
    return () => clearTimeout(clearTimer);
  }, [agentState, streamedText]);

  // Always keep microphone armed if we are in Voice Mode so it can hear interruptions!
  useEffect(() => {
    if (isVoiceMode && !isListening) {
      const reconnectTimer = setTimeout(() => {
         startListening();
      }, 500);
      return () => clearTimeout(reconnectTimer);
    }
  }, [isVoiceMode, isListening, startListening]);


  // Connection management
  useEffect(() => {
    let socket: WebSocket | null = null;
    let reconnectTimeout: NodeJS.Timeout;
    
    let fullRawResponse = "";
    let lastSpeechLength = 0;
    
    let sentenceBuffer = "";
    let fullAgentResponse = ""; 

    const connectWS = () => {
      socket = new WebSocket("ws://127.0.0.1:44444/chat");
      
      socket.onopen = () => {
        setWs(socket);
        setAgentState("idle");
      };

      socket.onmessage = (event) => {
        let tokenData = event.data;
        
        try {
            const parsedToken = JSON.parse(event.data);
            if (parsedToken.type === "token") {
                tokenData = parsedToken.t;
            } else if (parsedToken.type === "thought") {
                setLatestThought(parsedToken.thought);
                return;
            } else if (parsedToken.type === "action" || parsedToken.type === "error") {
                if (parsedToken.agent_mode) setAgentMode(parsedToken.agent_mode);
                if (parsedToken.macro_state) setMacroState(parsedToken.macro_state);
                const actionType = parsedToken.action && typeof parsedToken.action === "object"
                  ? (parsedToken.action.type || parsedToken.action.action)
                  : null;
                const toolName = parsedToken.tool_name || actionType;
                const isOperatorMode = parsedToken.agent_mode === "operator";

                if (actionType === "cancelled") {
                   sentenceBuffer = "";
                } else if (sentenceBuffer.trim()) {
                   speakChunk(sentenceBuffer.trim());
                   sentenceBuffer = "";
                }
                
                let textToSave = parsedToken.speech || fullAgentResponse;
                if (textToSave.trim()) {
                   setChatHistory(prev => [...prev, { role: "agent", text: textToSave }]);
                }
                
                // CRITICAL FIX: If we got a full speech field in an action message (agent task mode),
                // always speak it — UNLESS it's identical to what we already streamed (chat mode duplicate).
                // This ensures agent clarification questions, plan announcements, etc. always speak.
                if (parsedToken.speech && parsedToken.speech.trim() && actionType !== "cancelled") {
                    const alreadyStreamed = fullAgentResponse.trim() === parsedToken.speech.trim() ||
                                           fullAgentResponse.trim().endsWith(parsedToken.speech.trim());
                    if (!alreadyStreamed) {
                        setStreamedText(parsedToken.speech);
                        // Split into sentences and speak each one for natural prosody
                        const sentences = parsedToken.speech.match(/[^.!?\n]+[.!?\n]*/g) || [parsedToken.speech];
                        for (const sentence of sentences) {
                            if (sentence.trim().length > 2) {
                                speakChunk(sentence.trim());
                            }
                        }
                    }
                }
                
                // Add an explicit inline action message if it exists
                const isRealAction = parsedToken.type === "action"
                    && isOperatorMode
                    && toolName
                    && actionType !== "chat" 
                    && actionType !== "idle"
                    && actionType !== "talk"
                    && actionType !== "cancelled"
                    && actionType !== "achievement";

                if (isRealAction) {
                   let actionDesc = `[Executing Action: ${String(toolName).toUpperCase()}]`;
                   if (parsedToken.action.url) actionDesc += ` → ${parsedToken.action.url}`;
                   if (parsedToken.action.message) actionDesc += ` → "${parsedToken.action.message}"`;
                   if (parsedToken.action.selector) actionDesc += ` → selector: ${parsedToken.action.selector}`;
                   setChatHistory(prev => [...prev, { role: "agent", text: actionDesc, isAction: true }]);
                   // Real browser action — show live screen, cancel any pending hide
                   if (agentIdleTimerRef.current) clearTimeout(agentIdleTimerRef.current);
                   setHasAgentStarted(true);
                }
                
                if (parsedToken.type === "action") {
                   if (actionType === "cancelled") {
                       taskActiveRef.current = false;
                       stopSynthesis();
                       sentenceBuffer = "";
                       fullAgentResponse = "";
                       fullRawResponse = "";
                       lastSpeechLength = 0;
                       setHasAgentStarted(false);
                       setIsWaitingForAsk(false);
                       setAgentState("idle");
                       setMacroState("cancelled");
                   } else if (!isOperatorMode || actionType === "achievement" || actionType === "talk" || actionType === "idle" || actionType === "chat") {
                       taskActiveRef.current = false;
                   } else if (actionType) {
                       taskActiveRef.current = true;
                   }

                   // Chat/idle = agent done — hide live screen 3s after finishing
                   // EXCEPTION: keep screencast if agent is waiting for user (CAPTCHA, auth)
                   const isWaitingForUser = parsedToken.action?.type === "ask_user"
                     || parsedToken.action?.type === "need_auth"
                     || parsedToken.action?.type === "agent_question"
                     || parsedToken.action?.action === "ask"
                     || parsedToken.action?.type === "ask";

                   if (parsedToken.macro_state === "waiting_for_user" || parsedToken.action?.action === "ask" || parsedToken.action?.type === "ask") {
                       setIsWaitingForAsk(true);
                       taskActiveRef.current = true;
                   } else {
                       setIsWaitingForAsk(false);
                   }

                   const isIdle = !isWaitingForUser && isOperatorMode && (
                        !parsedToken.action 
                         || typeof parsedToken.action === "string"
                         || parsedToken.action.type === "chat" 
                         || parsedToken.action.type === "idle"
                         || parsedToken.action.action === "talk"
                         || parsedToken.action.action === "achievement"
                         || parsedToken.action.action === "cancelled");
                   if (isIdle) {
                     if (agentIdleTimerRef.current) clearTimeout(agentIdleTimerRef.current);
                     agentIdleTimerRef.current = setTimeout(() => {
                         setHasAgentStarted(false);
                         setIsWaitingForAsk(false);
                     }, 3000);
                   }
                   setAgentState(prev => prev === "speaking" ? "speaking" : "idle");
                }
                
                fullAgentResponse = ""; 
                fullRawResponse = "";
                lastSpeechLength = 0;
                return;
            }
        } catch (e) {
            // Fallback for raw streaming tokens if the backend ever skips JSON wrapping
        }

        if (tokenData === "[DONE]") {
            // Provide a fallback break point just in case
            fullRawResponse = "";
            lastSpeechLength = 0;
            return;
        }

        fullRawResponse += tokenData;

        // Magical Streaming Regex: Extracts the value inside "speech": "..."
        // It matches safely dynamically as the tokens stream in without crashing on invalid JSON
        const extractSpeech = (raw: string) => {
            const match = raw.match(/"speech"\s*:\s*"((?:\\.|[^"\\])*)/);
            if (match) {
                return match[1].replace(/\\n/g, '\n').replace(/\\"/g, '"');
            }
            // If the raw text contains markers of our internal JSON struct, it means we are in Agent Mode
            // and the "speech" key hasn't appeared yet. Returns empty so we don't speak raw JSON.
            if (raw.includes('"intent"') || raw.includes('"internal"') || raw.includes('"step"') || raw.includes('"action"')) {
                return "";
            }
            if (!raw.trimStart().startsWith("{") && !raw.trimStart().startsWith("[")) {
                return raw;
            }
            return "";
        };

        const currentSpeech = extractSpeech(fullRawResponse);

        if (currentSpeech.length > lastSpeechLength) {
            const newSpeechFragment = currentSpeech.slice(lastSpeechLength);
            lastSpeechLength = currentSpeech.length;
            
            setStreamedText(currentSpeech);
            sentenceBuffer += newSpeechFragment;
            fullAgentResponse += newSpeechFragment;

            // FULL SENTENCE CHUNKING (For Flawless Prosody)
            // The TTS engine handles whole sentences in 100ms when streaming correctly.
            const isSentenceEnd = /[.!?\n]/.test(newSpeechFragment);
            
            if (isSentenceEnd) {
               if (sentenceBuffer.trim().length > 3) {
                  speakChunk(sentenceBuffer.trim()); 
               }
               sentenceBuffer = ""; 
            }
        }
      };

      socket.onclose = () => {
        setWs(null);
        taskActiveRef.current = false;
        setAgentMode(null);
        setMacroState(null);
        stopSynthesis();
        setHasAgentStarted(false);
        setIsWaitingForAsk(false);
        setAgentState("error");
        reconnectTimeout = setTimeout(connectWS, 2000); 
      };
      
      socket.onerror = () => {
        socket?.close();
      };
    };

    connectWS();

    // Page Visibility API: when the user switches tabs, the WS may be throttled
    // and close. We pause reconnection while hidden and restore when visible again
    // WITHOUT clearing caption state.
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        // Tab came back into focus — restore surviving caption if any
        if (survivingCaptionRef.current) {
          setStreamedText(survivingCaptionRef.current);
        }
        // If socket is dead, reconnect silently
        if (!socket || socket.readyState === WebSocket.CLOSED) {
          connectWS();
        }
      }
    };
    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      clearTimeout(reconnectTimeout);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      if (socket) {
        socket.onclose = null;
        socket.close();
      }
    };
  }, []);

  // Screencast removed — Momentum now uses the full desktop natively



  const handleSendConfigured = (textOverride?: string, fromVoice: boolean = false) => {
    resetIdleTimer();
    const textToSend = textOverride !== undefined ? textOverride : inputText;
    if (!textToSend.trim()) return;
    const normalizedText = textToSend.trim().toLowerCase();
    
    // Guard: don't send noise blips (< 3 words from voice). Typed input always passes.
    const wordCount = textToSend.trim().split(/\s+/).filter(Boolean).length;
    const isStopCommand = taskActiveRef.current && STOP_COMMANDS.current.has(normalizedText);
    if (fromVoice && wordCount < 3 && !isStopCommand) return;
    
    stopSynthesis();

    if (!fromVoice) {
       setIsVoiceMode(false);
       stopListening(); 
    }
    // If triggered by Voice, we DELIBERATELY leave the mic open!
    
    setChatHistory(prev => [...prev, { role: "user", text: textToSend }]);
    setStreamedText("");
    setIsWaitingForAsk(false);
    
    if (ws && ws.readyState === WebSocket.OPEN) {
       if (!isStopCommand) {
         if (agentMode === "operator") taskActiveRef.current = true;
       }
       // Filter out action logs AND VAD noise (< 3 word user messages) before sending to backend
       const cleanHistory = chatHistory.filter((msg) => {
           if (msg.isAction || msg.text.includes("[Executing Action:")) return false;
           if (msg.role === "user") {
               return msg.text.trim().split(/\s+/).filter(Boolean).length >= 3;
           }
           // Skip agent messages that are very short / repeated filler
           if (msg.role === "agent" && msg.text.trim().length < 4) return false;
           return true; // Always keep substantive agent messages
       // Cap at last 15 messages to prevent token overflow in the LLM prompts
       }).slice(-15);

       const contextPayload = isStopCommand
         ? JSON.stringify({
             kind: "control",
             command: "stop",
             source: fromVoice ? "voice" : "text",
           })
         : JSON.stringify({
             kind: "user_input",
             history: cleanHistory,
             message: textToSend
           });
       ws.send(contextPayload);
    }
    
    setInputText("");
    if (isStopCommand) {
      setAgentState("idle");
    } else {
      setAgentState("thinking");
    }
  };



  const executeToggleVoiceMode = () => {
    setStreamedText(""); // Clear dialogue bubble when toggling UX
    if (!isVoiceMode) {
      setIsVoiceMode(true);
      startListening();
      setAgentState("listening");
    } else {
      setIsVoiceMode(false);
      stopListening();
      stopSynthesis();
      setAgentState("idle");
    }
  };

  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const annoyCooldownRef = useRef(false);

  // Auto-scroll logic for the speech bubble
  useEffect(() => {
    if (scrollContainerRef.current) {
       const { scrollHeight, clientHeight, scrollTop } = scrollContainerRef.current;
       const isNearBottom = scrollHeight - scrollTop - clientHeight < 60;
       if (isNearBottom || streamedText.length < 50) {
          scrollContainerRef.current.scrollTop = scrollHeight;
       }
    }
  }, [streamedText]);

  const handleWake = () => {
    if (ws?.readyState !== WebSocket.OPEN) {
       setStreamedText("Momentum runtime is offline. Run start.sh to bring the local services up.");
    }
  };

  return (
    <main className="flex h-screen w-full bg-[#0a0a0c] text-white overflow-hidden relative">
      

      {/* 
        The Left Panel System 
        Contains both the permanent Icon Strip and the sliding History Drawer.
      */}
      <div className="h-full z-50 flex group">
        
        {/* Permanent Thin Icon Strip */}
        <div className="w-[60px] h-full bg-[#050505] border-r border-white/5 flex flex-col items-center py-4 z-50 shrink-0 relative">
          
          <div className="flex flex-col gap-6 w-full items-center">
            <img src="/logo.svg" alt="Momentum Logo" className="w-8 h-8 object-contain mb-2 cursor-pointer opacity-90 hover:opacity-100 transition-opacity" />
            
            <button className="text-white/40 hover:text-white transition-colors" title="New Chat">
              <SquarePen className="w-5 h-5" />
            </button>
            <button className="text-white/40 hover:text-white transition-colors" title="Search">
              <Search className="w-5 h-5" />
            </button>
            <button className="text-white/40 hover:text-white transition-colors" title="History">
              <Clock className="w-5 h-5" />
            </button>
          </div>

          <div className="flex flex-col gap-6 w-full items-center mt-auto">
            <button className="text-white/40 hover:text-white transition-colors" title="Settings">
              <Settings className="w-5 h-5" />
            </button>
            
            {/* Fake Profile Avatar like ChatGPT */}
            <div className="w-8 h-8 bg-[#E35400] rounded-full flex items-center justify-center text-xs font-bold text-white cursor-pointer shadow-md tracking-wider">
               SN
            </div>
          </div>
        </div>
        
        {/* 
          Hoverable History Drawer 
          Slides out from behind the Icon Strip when the group is hovered
        */}
        <div className="absolute left-[60px] top-0 h-full w-[280px] bg-[#0f0f11] border-r border-white/5 flex flex-col z-[40] transform -translate-x-full group-hover:translate-x-0 transition-transform duration-300 ease-[cubic-bezier(0.16,1,0.3,1)] shadow-2xl">
          
          <div className="p-4 border-b border-white/5 font-semibold text-sm tracking-wide text-white/50">
            Chat History
          </div>

          <div className="flex-1 overflow-y-auto p-4 flex flex-col gap-5 custom-scrollbar">
            {chatHistory.length === 0 ? (
              <div className="text-center text-white/20 italic text-sm mt-10">No history in this session yet.</div>
            ) : (
              chatHistory.map((msg, idx) => (
                <div key={idx} className={cn("flex flex-col gap-1 w-full", msg.role === "user" ? "items-end" : "items-start")}>
                  <span className="text-[10px] uppercase tracking-wider text-white/30 px-1">
                    {msg.role === "user" ? "You" : "Momentum"}
                  </span>
                  <div className={cn(
                    "px-4 py-3 rounded-2xl text-sm leading-relaxed max-w-[95%] whitespace-pre-wrap",
                    msg.role === "user" ? "bg-[#1f1f22] rounded-br-sm text-white/90" : 
                    msg.isAction ? "bg-amber-500/10 text-amber-200 border border-amber-500/30 rounded-bl-sm font-mono text-xs" : 
                    "bg-[#6d56fa]/10 text-[#e4dffe] border border-[#6d56fa]/20 rounded-bl-sm"
                  )}>
                    {msg.text}
                  </div>
                </div>
              ))
            )}
          </div>

          <div className="p-4 border-t border-white/5">
            <button 
              onClick={clearHistory} 
              className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-white/40 hover:text-rose-400 transition-colors w-full"
            >
              <Trash2 className="w-4 h-4" /> Clear History
            </button>
          </div>
        </div>
      </div>

      {/* Main Agent Workspace (Center) */}
      <div className="flex-1 relative flex flex-col items-center justify-center p-8 bg-transparent z-10 w-full h-full overflow-hidden">
        
        {/* Dynamic Background Glow representing the Agent Core */}
        {(
          <div className={cn(
            "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[1100px] h-[1100px] rounded-full pointer-events-none transition-all duration-1000",
            (agentState === "thinking" || agentState === "speaking") 
              ? "animate-pulse opacity-100 scale-105" 
              : "opacity-80 scale-100"
          )} style={{
             background: "radial-gradient(circle at center, rgba(124, 77, 255, 0.25) 0%, rgba(98, 48, 235, 0.12) 30%, rgba(65, 25, 180, 0.04) 60%, transparent 80%)",
             transform: "translateZ(0)",
             willChange: "transform, opacity"
          }} />
        )}

        <motion.div layout className="flex flex-col items-center justify-center w-full h-full max-h-full transition-all duration-700 mt-[-40px]">
          
          {/* Face / Agent Interaction Wrapper */}
          <motion.div 
            layout
            className={cn(
              "relative z-20 shrink-0 transition-all duration-700 ease-[cubic-bezier(0.25,1,0.5,1)]",
              isVoiceMode ? "scale-110 -translate-y-8" : "scale-100 mb-0"
            )}
            animate={{ x: annoyanceLevel === 1 ? -40 : annoyanceLevel === 2 ? 50 : 0 }}
            transition={{ type: "spring", bounce: 0.6, duration: 0.8 }}
            onClick={handleWake} 
            onMouseLeave={() => setIsHoverFace(false)}
            onMouseEnter={() => {
                if (!isVoiceMode && ws?.readyState === WebSocket.OPEN) {
                    if (!isHoverFace && annoyanceLevel === 0) setAnnoyanceLevel(1);
                    setIsHoverFace(true);
                }
            }}
          >
             <AgentFace 
               state={ws?.readyState === WebSocket.OPEN ? agentState : "sleeping"} 
               isShuttered={isShuttered}
               isVoiceMode={isVoiceMode}
               className="w-64 h-64 pointer-events-none drop-shadow-2xl" 
             />
             {ws?.readyState !== WebSocket.OPEN && (
                <div className="absolute -top-4 -right-8 flex flex-col font-mono tracking-widest text-[#6d56fa] font-bold text-2xl pointer-events-none drop-shadow-[0_0_10px_rgba(109,86,250,0.5)]">
                  <span className="animate-[bounce_4s_infinite_0s] opacity-80">Z</span>
                  <span className="animate-[bounce_4s_infinite_1s] text-xl -ml-4 opacity-60">z</span>
                  <span className="animate-[bounce_4s_infinite_2s] text-lg -ml-8 opacity-40">z</span>
                </div>
             )}
          </motion.div>

          {/* Text Streams */}
          <AnimatePresence>
            {(
              <motion.div 
                layout
                initial={{ opacity: 0, filter: "blur(15px)" }}
                animate={{ opacity: 1, filter: "blur(0px)" }}
                exit={{ opacity: 0, filter: "blur(15px)", scale: 0.95, position: "absolute" }}
                className="w-[90vw] text-center flex flex-col items-center z-[60] pointer-events-none max-w-[700px] mt-8 shrink-0 relative min-h-[100px]"
              >
                {ws?.readyState !== WebSocket.OPEN && (
                  <p className="text-xl font-medium text-white/30">Click on the agent to wake.</p>
                )}
                
                {(ws?.readyState === WebSocket.OPEN && agentState === "idle" && !streamedText && isUserIdle) && (
                  <div className="h-8 flex items-center justify-center relative w-full overflow-hidden">
                    <AnimatePresence mode="wait">
                      <motion.p
                        key={promptIndex}
                        initial={{ opacity: 0, filter: "blur(8px)", y: 4 }}
                        animate={{ opacity: 1, filter: "blur(0px)", y: 0 }}
                        exit={{ opacity: 0, filter: "blur(8px)", y: -4 }}
                        transition={{ duration: 0.6, ease: [0.32, 0.72, 0, 1] }} 
                        className="absolute text-[1.15rem] font-['SF_Pro_Display',-apple-system,BlinkMacSystemFont,'Helvetica_Neue',sans-serif] tracking-tight antialiased text-[#ececec] font-medium drop-shadow-sm"
                      >
                        {AGENTIC_PROMPTS[promptIndex]}
                      </motion.p>
                    </AnimatePresence>
                  </div>
                )}
                {(isVoiceMode && (agentState === "listening" || agentState === "idle")) && (
                  <p className={cn("text-xl font-medium tracking-tight mt-4 pulse-animation", isVoiceMode ? "text-[#b6fad1]" : "text-[#6d56fa]")}>
                    {isVoiceMode ? "Listening to your thoughts..." : "Listening..."}
                  </p>
                )}
                
                {(agentState === "thinking" && !streamedText) && (
                  <p className="flex items-center justify-center gap-2 text-white/40 text-2xl mt-1">
                    <span className="animate-pulse">.</span><span className="animate-pulse delay-100">.</span><span className="animate-pulse delay-200">.</span>
                  </p>
                )}
                
                {streamedText && (
                  <div ref={scrollContainerRef} className="pointer-events-auto w-full max-h-[190px] overflow-y-auto custom-scrollbar flex flex-col py-4 px-4 transition-all" style={{ maskImage: 'linear-gradient(to bottom, transparent, black 15%, black 85%, transparent)', WebkitMaskImage: 'linear-gradient(to bottom, transparent, black 15%, black 85%, transparent)' }}>
                    <p className="text-[#ececec] text-[1.15rem] md:text-[1.8rem] font-medium font-['SF_Pro_Display',-apple-system,BlinkMacSystemFont,'Helvetica_Neue',sans-serif] tracking-tight antialiased drop-shadow-sm leading-relaxed whitespace-pre-wrap text-center">
                      {streamedText}
                    </p>
                  </div>
                )}
              </motion.div>
            )}
          </AnimatePresence>

        </motion.div>

        {/* Chat Input Box Wrapper with Center-Restricted Hover Area */}
        <div className="absolute bottom-4 left-1/2 -translate-x-1/2 w-[650px] h-[100px] flex items-end justify-center pb-6 z-50 group/chat pointer-events-auto">
          {/* Invisible solid hit-area to cleanly catch hover without twitching when children move */}
          <div className="absolute inset-0 z-0 bg-black/0 rounded-t-3xl" />
          <AnimatePresence mode="wait">
            {isVoiceMode ? (
               <motion.div 
                 key="voice-pill"
                 initial={{ opacity: 0, y: 20 }}
                 animate={{ opacity: 1, y: 0 }}
                 exit={{ opacity: 0, y: 20 }}
                 className="flex justify-center w-full"
               >
                 <button 
                   className="flex items-center gap-2.5 px-6 py-3 rounded-full bg-[#1c1c1e] bg-opacity-[0.85] border border-white/5 text-white/90 hover:bg-[#252528] hover:text-white transition-all shadow-[0_4px_30px_rgba(0,0,0,0.8)] backdrop-blur-md"
                   onClick={executeToggleVoiceMode}
                 >
                   <AudioLines className="w-[18px] h-[18px] text-[#10b981]" />
                   <span className="text-[15px] font-medium tracking-wide">End Voice Mode</span>
                 </button>
               </motion.div>
            ) : (
                 <motion.div 
                 key="chat-input"
                 initial={{ opacity: 0, y: 20 }}
                 animate={{ opacity: 1, y: 0 }}
                 exit={{ opacity: 0, y: 20 }}
                 className={cn(
                   "w-full max-w-2xl px-6 flex justify-center transition-all duration-500 ease-[cubic-bezier(0.25,1,0.5,1)] z-50",
                   "translate-y-0 opacity-100"
                 )}
               >
                  <div className="rounded-2xl w-full p-[2px] relative transition-all duration-500 z-50 bg-[#121214]/60 backdrop-blur-[40px] border border-white/20 hover:border-white/40 shadow-[0_20px_60px_rgba(0,0,0,0.6)]">
                    <div className="flex items-center gap-2 w-full rounded-2xl pr-2 focus-within:border-[#6d56fa]/50 transition-all shadow-inner bg-[#121214]/60">
                      <button 
                        className="p-4 hover:text-[#6d56fa] hover:bg-white/5 rounded-xl transition-all shrink-0 text-white/40"
                        onClick={executeToggleVoiceMode}
                        title="Continuous Voice Mode"
                        disabled={ws?.readyState !== WebSocket.OPEN}
                      >
                        <Mic className="w-5 h-5" />
                      </button>
                      <input 
                        type="text" 
                        placeholder="Ask Momentum anything..."
                        className="flex-1 bg-transparent border-none outline-none text-white placeholder-white/50 py-4 h-full text-[15px] transition-all w-full disabled:opacity-50"
                        value={inputText}
                        onChange={(e) => setInputText(e.target.value)}
                        disabled={ws?.readyState !== WebSocket.OPEN}
                        onFocus={() => {
                          setStreamedText("");
                          if (agentState === "idle") setAgentState("listening");
                        }}
                        onBlur={() => {if (agentState === "listening") setAgentState("idle");}}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") handleSendConfigured(inputText, false);
                        }}
                      />
                      <button 
                        className="p-3 bg-white text-black rounded-xl hover:bg-gray-200 transition-all shrink-0 m-1 disabled:opacity-30 disabled:hover:bg-white"
                        disabled={!inputText || ws?.readyState !== WebSocket.OPEN}
                        onClick={() => handleSendConfigured(inputText, false)}
                      >
                        <Send className="w-5 h-5" />
                      </button>
                    </div>
                  </div>
                  {/* Quick stats visually tied to input below it */}
                  {(!hasAgentStarted && !isVoiceMode) && (
                    <div className={cn("absolute -top-8 right-6 flex items-center gap-4 text-xs font-semibold tracking-wide transition-opacity text-white/40")}>
                       <span className="flex items-center gap-1.5"><Keyboard className="w-3.5 h-3.5" /> Types</span>
                       <span className="flex items-center gap-1.5"><MousePointer2 className="w-3.5 h-3.5" /> Clicks</span>
                    </div>
                  )}
               </motion.div>
            )}
          </AnimatePresence>


        </div>

      </div>
    </main>
  );
}
