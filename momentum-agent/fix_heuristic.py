import re

with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

old_heuristic = r'''    pub fn heuristic_mode\(message: &str\) -> Option<AgentMode> \{
        let normalized = message\.trim\(\)\.to_lowercase\(\);
        if normalized\.is_empty\(\) \{
            return Some\(AgentMode::Chat\);
        \}

        let chat_phrases = \[
            "hi", "hello", "hey", "how are you", "make something up", "what's up",
            "whats up", "thank you", "thanks",
        \];

        if chat_phrases\.iter\(\)\.any\(\|phrase\| normalized == \*phrase\) \{
            return Some\(AgentMode::Chat\);
        \}

        // Removed hardcoded intent heuristics to let the AI natively decide routing!
        None
    \}'''

new_heuristic = '''    pub fn heuristic_mode(message: &str) -> Option<AgentMode> {
        let normalized = message.trim().to_lowercase();
        if normalized.is_empty() {
            return Some(AgentMode::Chat);
        }

        let operator_phrases = [
            "search", "google", "look up", "find me", "buy", "purchase",
            "open a tab", "navigate to", "download"
        ];

        if operator_phrases.iter().any(|phrase| normalized.contains(phrase)) {
            return Some(AgentMode::Operator);
        }

        // Default to Chat for fast conversational latency (skips slow LLM classification)
        Some(AgentMode::Chat)
    }'''

content = re.sub(old_heuristic, new_heuristic, content)

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
