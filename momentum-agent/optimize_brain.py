import re
with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Optimize context window and batch size for low-end CPU
content = content.replace('.with_n_ctx(Some(std::num::NonZeroU32::new(8192).unwrap()))', '.with_n_ctx(Some(std::num::NonZeroU32::new(2048).unwrap()))')
content = content.replace('.with_n_batch(4096)', '.with_n_batch(512)')

# 2. Fix prompt hallucination by removing specific app names from the system prompt
bad_prompt = 'you MUST output EXACTLY the phrase `[ACTION: OPEN_APP(App Name)]` where App Name is the exact formal macOS application name (e.g., "Google Chrome", "Visual Studio Code", "System Settings", "Notes", "Spotify"). For example: "Opening Chrome for you now. [ACTION: OPEN_APP(Google Chrome)]"'
good_prompt = 'you MUST output EXACTLY the phrase `[ACTION: OPEN_APP(App Name)]` where App Name is the exact formal application name. For example: "Opening app for you now. [ACTION: OPEN_APP(App Name)]"'
content = content.replace(bad_prompt, good_prompt)

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
