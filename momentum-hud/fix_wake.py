import re
with open('src/app/page.tsx', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('''        if (!isVisibleRef.current && (lower.includes("momentum") || lower.includes("hey momentum"))) {
            setIsVisible(true);
            invoke("show_window_now");
            
            if (ttsWsRef.current?.readyState === WebSocket.OPEN) {
               const greetings = ["Yeah?", "I'm here.", "Go ahead.", "I'm listening.", "What's up?", "Mm-hmm?"];
               const greeting = greetings[Math.floor(Math.random() * greetings.length)];
               ttsWsRef.current.send(greeting);
               displayDialogue(greeting);
            }
            return;
        }''', '''        if (!isVisibleRef.current && (lower.includes("momentum") || lower.includes("hey momentum"))) {
            setIsVisible(true);
            invoke("show_window_now");
            
            // Only drop the message if they ONLY said the wake word
            if (lower === "momentum" || lower === "hey momentum" || lower === "momentum." || lower === "hey momentum.") {
                if (ttsWsRef.current?.readyState === WebSocket.OPEN) {
                   const greetings = ["Yeah?", "I'm here.", "Go ahead.", "I'm listening.", "What's up?", "Mm-hmm?"];
                   const greeting = greetings[Math.floor(Math.random() * greetings.length)];
                   ttsWsRef.current.send(greeting);
                   displayDialogue(greeting);
                }
                return;
            }
        }''')

with open('src/app/page.tsx', 'w', encoding='utf-8') as f:
    f.write(content)
