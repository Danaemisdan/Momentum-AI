"use client";

import { useState, useEffect, useRef, useCallback } from "react";

declare global {
  interface Window {
    SpeechRecognition: any;
    webkitSpeechRecognition: any;
  }
}

export function useSpeechRecognition(
  onSpeechDetected: (transcript: string, isFinal: boolean) => void
) {
  const [isListening, setIsListening] = useState(false);
  const recognitionRef = useRef<any>(null);
  const shouldListenRef = useRef(false);

  // We use a mutable ref for the callback so the closure inside the speech listener 
  // ALWAYS has access to the most up-to-date state (like the WebSocket and agentState)
  // without needing to aggressively unmount and remount the recognizer.
  const handleSpeechRef = useRef(onSpeechDetected);
  useEffect(() => { handleSpeechRef.current = onSpeechDetected; }, [onSpeechDetected]);

  useEffect(() => {
    if (typeof window === "undefined") return;

    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SpeechRecognition) {
      console.warn("Speech Recognition API not supported in this browser.");
      return;
    }

    const recognition = new SpeechRecognition();
    
    // FALSE ensures the browser natively detects when you stop speaking, granting lightning-fast 'isFinal' response queries!
    recognition.continuous = false;
    
    // interimResults lets us halt the TTS the *second* the user starts mumbling
    recognition.interimResults = true;
    recognition.lang = "en-US";
    
    recognition.onstart = () => setIsListening(true);
    
    recognition.onerror = (event: any) => {
      if (event.error !== "no-speech") {
         console.warn("VAD Execution Error - Restarting Engine", event.error);
      }
    };

    recognition.onend = () => {
      setIsListening(false);
      // FLAWLESS 10-millisecond instant reboot. Eliminates the 500ms deaf window between sentences!
      if (shouldListenRef.current) {
        setTimeout(() => {
          try { recognition.start(); } catch(e){}
        }, 10);
      }
    };

    recognition.onresult = (event: any) => {
      let interimTranscript = '';
      let finalTranscript = '';

      for (let i = event.resultIndex; i < event.results.length; ++i) {
        if (event.results[i].isFinal) {
          finalTranscript += event.results[i][0].transcript;
        } else {
          interimTranscript += event.results[i][0].transcript;
        }
      }

      if (finalTranscript) {
        handleSpeechRef.current(finalTranscript, true);
      } else if (interimTranscript) {
        handleSpeechRef.current(interimTranscript, false);
      }
    };

    recognitionRef.current = recognition;

    return () => {
      recognition.stop();
    };
  }, []);

  const startListening = useCallback(() => {
    shouldListenRef.current = true;
    try {
      if (!isListening) {
        // Only request the hardware AEC stream ONCE to prevent Apple microphone flickering!
        if (!(window as any)._hackyAecStream && navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
            navigator.mediaDevices.getUserMedia({ 
                audio: { echoCancellation: true, noiseSuppression: true, autoGainControl: true } 
            }).then((stream) => { (window as any)._hackyAecStream = stream; })
            .catch(e => console.error("Hardware AEC binding failed", e));
        }
        recognitionRef.current?.start();
      }
    } catch(e) {}
  }, [isListening]);

  const stopListening = useCallback(() => {
    shouldListenRef.current = false;
    try {
      recognitionRef.current?.stop();
      if ((window as any)._hackyAecStream) {
         ((window as any)._hackyAecStream as MediaStream).getTracks().forEach(t => t.stop());
         (window as any)._hackyAecStream = null;
      }
    } catch(e) {}
  }, []);

  const toggleListening = useCallback(() => {
    if (isListening) {
      stopListening();
    } else {
      setTimeout(() => startListening(), 50);
    }
  }, [isListening, startListening, stopListening]);

  return { isListening, toggleListening, startListening, stopListening };
}
