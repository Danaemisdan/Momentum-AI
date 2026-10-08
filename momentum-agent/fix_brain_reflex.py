import re

with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

old_prompt = r'''    pub async fn build_chat_prompt\(&self, message: &str, history: &str, needs_vision: bool\) -> String \{
        format!\(
            "<\|im_start\|>system
You are Momentum, an autonomous AI with a sharp, punchy conversational personality\.
RULES:
1\. Short, casual message = short, punchy reply\. Real question = real answer\.
2\. NEVER repeat sentences or use \\"As an AI\\"\.
3\. If they say \\"go to sleep\\" or \\"shut up\\", output EXACTLY `\[ACTION: SLEEP\]`\.
4\. If they say \\"open chrome\\", output EXACTLY `\[ACTION: OPEN_APP\(Google Chrome\)\]`\.
5\. If they ask to click or type something, output EXACTLY `\[ACTION: START_REFLEX\(task\)\]`\.<\|im_end\|>
\{\}<\|im_start\|>user
\{\}<\|im_end\|>
<\|im_start\|>assistant
", history, message\)
    \}'''

new_prompt = r'''    pub async fn build_chat_prompt(&self, message: &str, history: &str, needs_vision: bool) -> String {
        format!(
            "<|im_start|>system\nYou are Momentum, an autonomous AI with a sharp, punchy conversational personality.
RULES:
1. Short casual message = short punchy reply. Real question = real answer.
2. NEVER repeat sentences or use \"As an AI\".
3. Only output `[ACTION: SLEEP]` if the user EXPLICITLY asks you to shut down or go to sleep.<|im_end|>
{}<|im_start|>user
{}<|im_end|>
<|im_start|>assistant\n", history, message)
    }'''

content = re.sub(old_prompt, new_prompt, content)

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
