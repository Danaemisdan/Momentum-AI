import asyncio
import os
import requests
import glob

OPENROUTER_API_KEY = "YOUR_OPENROUTER_API_KEY"
OPENROUTER_MODEL = "google/gemini-2.5-flash"

base_dir = os.path.dirname(os.path.dirname(__file__))
platforms_dir = os.path.join(base_dir, 'skills', 'platforms')

async def upgrade_intents(platform_name, intents_yaml):
    if len(intents_yaml.strip()) < 5:
        return intents_yaml
        
    prompt = f"""
    You are upgrading the 'intents' block of a Web Agent's heuristic map for platform '{platform_name}'.
    
    CURRENT INTENTS YAML:
    {intents_yaml}
    
    RULES:
    1. KEEP the existing `name` and `trigger_roles` and format.
    2. Upgrade the interaction logic by converting `interaction_patterns` to `reference_patterns` (or keep both).
    3. You MUST ADD missing cognitive fields for EVERY intent strictly adhering to this schema structure:
       - strategy: [array of 2-3 logical steps understanding the goal]
       - reference_patterns: [array of 2-3 patterns on how humans look for this]
       - decision_logic: [array of 2-3 conditional checks, e.g. 'if A -> do X']
       - actions: [array of what exact interactibles to trigger/click]
       - verification: [array of 1-2 steps to confirm it worked]
       - recovery: [array of 1-2 steps to retry if failed]
    
    OUTPUT exactly the RAW updated YAML block for the intents. DO NOT output ```yaml markdown blocks. DO NOT output the `platform` or `interactibles` blocks. Keep your output perfectly indented (e.g. starting with `  - name:`).
    """

    headers = {
        "Authorization": f"Bearer {OPENROUTER_API_KEY}",
        "Content-Type": "application/json"
    }

    payload = {
        "model": OPENROUTER_MODEL,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.2
    }

    loop = asyncio.get_event_loop()
    def fetch():
        return requests.post("https://openrouter.ai/api/v1/chat/completions", headers=headers, json=payload, timeout=45)
    
    try:
        response = await loop.run_in_executor(None, fetch)
        if response.status_code == 200:
            content = response.json()['choices'][0]['message']['content'].strip()
            if content.startswith("```yaml"): content = content[7:]
            elif content.startswith("```"): content = content[3:]
            if content.endswith("```"): content = content[:-3]
            return "\n" + content.strip() + "\n"
        else:
            print(f"[-] API Error for {platform_name}: Status {response.status_code}")
            return intents_yaml
    except Exception as e:
        print(f"[-] Fetch Error for {platform_name}: {e}")
        return intents_yaml

async def process_file(file_path):
    with open(file_path, 'r') as f:
        text = f.read()

    # Split to protect interactibles from LLM character limits
    delim = "\nintents:\n"
    if delim not in text:
        delim = "\nintents:"
        if delim not in text:
            print(f"[!] No intents found in {os.path.basename(file_path)}, skipping.")
            return

    parts = text.split(delim, 1)
    if len(parts) == 2:
        top_half = parts[0]
        intents_half = parts[1]
        
        platform_name = os.path.basename(file_path).replace('.yaml', '')
        
        print(f"[*] Upgrading platform logic: {platform_name}")
        upgraded_intents = await upgrade_intents(platform_name, intents_half)
        
        final_text = top_half + "\nintents:\n" + upgraded_intents
        
        with open(file_path, 'w') as f:
            f.write(final_text)
        print(f"[+] Successfully upgraded {platform_name}")

async def process_all():
    print("[*] Initiating Mass Cognitive Upgrade across all Web Platforms...")
    files = glob.glob(os.path.join(platforms_dir, "*.yaml"))
    
    # Process concurrently mapped
    tasks = [process_file(f) for f in files]
    
    # Run in batches of 5 to avoid quick rate limits
    batch_size = 5
    for i in range(0, len(tasks), batch_size):
        await asyncio.gather(*tasks[i:i+batch_size])

    print("[+] Master Platform Upgrade Complete. OS Architecture synced.")

if __name__ == "__main__":
    asyncio.run(process_all())
