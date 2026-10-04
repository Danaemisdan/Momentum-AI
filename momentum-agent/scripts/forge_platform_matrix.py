import asyncio
import os
import requests
import glob
import re

OPENROUTER_API_KEY = "YOUR_OPENROUTER_API_KEY"
OPENROUTER_MODEL = "google/gemini-2.5-flash"

base_dir = os.path.dirname(os.path.dirname(__file__))
platforms_dir = os.path.join(base_dir, 'skills', 'platforms')
roles_file = os.path.join(base_dir, 'skills', 'master_role_registry_100.yaml')

sem = asyncio.Semaphore(4)

async def generate_intents_for_batch(platform_file, role_batch):
    platform_name = os.path.basename(platform_file).replace('.yaml', '')
    
    with open(platform_file, 'r') as f:
        platform_txt = f.read()

    roles_list = "\n".join([f"- {r}" for r in role_batch])
        
    prompt = f"""
    You are structurally engineering an AI Operational Heuristic Map for the web platform '{platform_name}'.
    
    We have 10 God-Tier Elite AI Roles that need to operate on this platform.
    ROLES BATCH:
    {roles_list}
    
    Your task:
    Generate exactly ONE highly specific intent for EACH Role. The intent must represent a complex, native interaction the role would uniquely perform on '{platform_name}'.
    
    CRITICAL RULES:
    1. STRICT SCHEMA MUST BE FOLLOWED:
       - name: [distinct system action name]
       - trigger_roles: [ARRAY of the role string verbatim]
       - strategy: [array of 2-3 logical steps understanding the goal]
       - reference_patterns: [array of behavior signs]
       - decision_logic: [array of if/then statements]
       - actions: [array of what to click/interact with]
       - verification: [array of confirmation steps]
       - recovery: [array of retry steps if blocked]
    
    OUTPUT FORMAT:
    Output STRICTLY as valid YAML. DO NOT output ```yaml markdown blocks. DO NOT output conversational text.
    You are outputting items for an existing `intents:` list. 
    EACH intent MUST start with exactly TWO spaces for the list dash: `  - name: "intent_name"`
    AND ALL sibling keys MUST be indented with EXACTLY FOUR spaces.
    
    Start directly with the first item:
    
  - name: "example_intent"
    trigger_roles:
      - "Role Name 1"
    strategy:
    ...
    """

    headers = {
        "Authorization": f"Bearer {OPENROUTER_API_KEY}",
        "Content-Type": "application/json"
    }
    payload = {
        "model": OPENROUTER_MODEL,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.3
    }

    loop = asyncio.get_event_loop()
    def fetch():
        return requests.post("https://openrouter.ai/api/v1/chat/completions", headers=headers, json=payload, timeout=90)
    
    try:
        async with sem:
            response = await loop.run_in_executor(None, fetch)
        if response.status_code == 200:
            content = response.json()['choices'][0]['message']['content'].strip()
            if content.startswith("```yaml"): content = content[7:]
            elif content.startswith("```"): content = content[3:]
            if content.endswith("```"): content = content[:-3]
            
            # Ensure proper spacing (not stripping newlines entirely, just trim edges)
            content = content.strip()
            
            # Make sure it starts with 2 spaces for the bullet
            lines = content.split('\n')
            formatted_lines = []
            for line in lines:
                if line.startswith("- name:"):
                    formatted_lines.append("  " + line)
                else:
                    formatted_lines.append(line)
                    
            return "\n" + "\n".join(formatted_lines) + "\n"
        else:
            print(f"[-] API Error on {platform_name}: Status {response.status_code}")
            return ""
    except Exception as e:
        print(f"[-] Fetch Error on {platform_name}: {e}")
        return ""

async def generate_platform_matrix():
    print("[*] Initiating Forging Sequence: Mapping 100 Roles to 25 Platforms...")
    
    # 1. Get Roles
    try:
        with open(roles_file, 'r') as f:
            roles_txt = f.read()
        role_matches = re.findall(r'\n- ([^:\n]+):', roles_txt)
        roles_list = list(set(role_matches))
        print(f"[+] Loaded {len(roles_list)} God-Tier Roles.")
    except Exception:
        print("[-] Could not parse roles file. Ensure Phase 1 ran first.")
        return

    # 2. Get 25 Platforms
    platforms = glob.glob(os.path.join(platforms_dir, "*.yaml"))
    print(f"[+] Loaded {len(platforms)} Platform Base Archetypes.")

    # 3. Batch Roles into groups of 10
    batch_size = 10
    role_batches = [roles_list[i:i + batch_size] for i in range(0, len(roles_list), batch_size)]

    # 4. For each platform, dispatch intent generation concurrently for all role batches
    for index, p in enumerate(platforms):
        platform_name = os.path.basename(p)
        print(f"[*] Processing Platform [{index+1}/{len(platforms)}]: {platform_name}")
        
        # Spawn tasks for all role batches for THIS platform
        tasks = [generate_intents_for_batch(p, batch) for batch in role_batches]
        results = await asyncio.gather(*tasks)
        
        # Append all generated intents into the platform yaml file
        with open(p, 'a') as f:
            for r in results:
                if r: f.write(r)
                
        print(f"[+] Successfully woven omniscience into {platform_name}.")

    print("[+] Phase 2 Complete. OS Behavioral Architecture Synced globally.")

if __name__ == "__main__":
    asyncio.run(generate_platform_matrix())
