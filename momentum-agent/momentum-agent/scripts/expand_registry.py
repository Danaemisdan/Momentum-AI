import asyncio
import os
import requests
import re
import yaml

OPENROUTER_API_KEY = "YOUR_OPENROUTER_API_KEY"
OPENROUTER_MODEL = "google/gemini-2.5-flash"

base_dir = os.path.dirname(os.path.dirname(__file__))
registry_path = os.path.join(base_dir, 'skills', 'master_role_registry.yaml')
backup_path = os.path.join(base_dir, 'skills', 'master_role_registry_backup.yaml')

if not os.path.exists(registry_path):
    print("[-] Registry not found.")
    exit(1)

# Safely copy current for backup
with open(registry_path, 'r') as f:
    text = f.read()
    with open(backup_path, 'w') as bf:
        bf.write(text)

# We parse the raw YAML into chunks so we retain the comments
# We assume each role starts with "  - Name:" or "  - RoleName:"
# But some have "  - Role:\n      name: ..."
chunks = re.split(r'(\n- [A-Za-z0-9_ &]+:|\n- Role:)', text)

async def upgrade_role(block_title, block_body):
    raw_yaml = block_title + block_body
    if len(raw_yaml.strip()) < 20:
        return raw_yaml # Not a valid role block
    if "example_workflows:" not in raw_yaml and "reference_patterns:" not in raw_yaml:
        return raw_yaml # Skip headers/footers
        
    prompt = f"""
    You are structurally upgrading an AI Operational Role Schema.
    INPUT SCHEMA:
    {raw_yaml}
    
    CRITICAL RULES:
    1. RENAME the role entirely to a highly sophisticated, expansive, meta-agent title. If it is standard like "Teacher", change it to "Knowledge Synthesizer & Educational Pedagogy Specialist". The exact role name must sound like a God-Tier elite autonomous AI division.
    2. Expand the `description:` into a massive, dense semantic paragraph defining exactly what cross-functional behaviors the agent executes. It must be big enough and complex enough for the AI logic router to pick it up perfectly.
    3. WIPE OUT any weak, niche, or irrelevant URLs from `websites:`. ADD elite, high-value relevant domains (e.g. AWS, Stripe, GitHub, Notion, LinkedIn, Figma, etc depending on the role).
    4. Generate a massive `example_workflows:` array (DO NOT use reference_patterns). We need MORE examples. Make 8 to 12 explicitly distinct highly complex 'intent' and 'steps' arrays.
    5. Append exactly: `decision_logic`, `constraints`, `verification`, and `recovery` loops with 2-3 logical sentences each matching an apex OS Architecture.
    
    OUTPUT strictly as valid YAML. Your output MUST START WITH exactly the new role definition title (e.g. `- Elite Systems Engineer:`) using a leading hyphen and a colon. Maintain 2-space physical indents inside the block! No markdown tags. No raw text wrapping.
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
            # Clean possible markdown
            if content.startswith("```yaml"): content = content[7:]
            elif content.startswith("```"): content = content[3:]
            if content.endswith("```"): content = content[:-3]
            return "\n" + content.strip() + "\n"
        else:
            print(f"[-] API Error for {block_title.strip()}: Status {response.status_code}")
            return raw_yaml
    except Exception as e:
        print(f"[-] Fetch Error for {block_title.strip()}: {e}")
        return raw_yaml

async def process_all():
    print("[*] Initiating Mass Semantic Expansion across 100 Roles...")
    
    # Text is split into [preceding, delimiter1, body1, delimiter2, body2, ...]
    if not text.startswith("-"):
        # Header is chunks[0]
        final_yaml = chunks[0]
        start_idx = 1
    else:
        final_yaml = ""
        start_idx = 0
        
    tasks = []
    indices = []
    
    # Batch the execution to not bottleneck API
    for i in range(start_idx, len(chunks)-1, 2):
        block_title = chunks[i]
        block_body = chunks[i+1]
        
        if "description:" in block_body:
            indices.append(i)
            # Cap parallelization to avoid getting instantly rate limited by OpenRouter
            tasks.append(upgrade_role(block_title, block_body))
            
    print(f"[+] Found {len(tasks)} roles to upgrade mathematically.")
    
    # Process in batches of 10
    batch_size = 5
    results_map = {}
    
    for i in range(0, len(tasks), batch_size):
        batch = tasks[i:i+batch_size]
        print(f"[*] Upgrading Batch {i//batch_size + 1} ({len(batch)} Roles)...")
        batch_results = await asyncio.gather(*batch)
        for j, res in enumerate(batch_results):
            results_map[indices[i+j]] = res
            
    # Reconstruct
    for i in range(start_idx, len(chunks)-1, 2):
        if i in results_map:
            final_yaml += results_map[i]
        else:
            final_yaml += chunks[i] + chunks[i+1]
            
    with open(registry_path, 'w') as f:
        f.write(final_yaml)
        
    print("[+] Master Semantic Registry Upgrade Complete. Successfully wiped irrelevant URLs and multiplied Examples.")

if __name__ == "__main__":
    asyncio.run(process_all())
