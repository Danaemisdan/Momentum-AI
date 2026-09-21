import asyncio
import os
import requests
import json
import uuid

OPENROUTER_API_KEY = "YOUR_OPENROUTER_API_KEY"
OPENROUTER_MODEL = "google/gemini-2.5-flash"

base_dir = os.path.dirname(os.path.dirname(__file__))
registry_path = os.path.join(base_dir, 'skills', 'master_role_registry_100.yaml')

DIVISIONS = [
    "Executive Management & Strategic Command",
    "Backend Infrastructure & Scalability Systems",
    "Frontend UX & Immersive Interface Design",
    "AI/ML Engineering & Neural Architecture",
    "Data Science & Big Data Analytics",
    "Cybersecurity & Threat Intelligence Ops",
    "DevOps, MLOps & Continuous Delivery",
    "Product Management & Growth Hacking",
    "Sales, Revenue Generation & Market Arbitrage",
    "Legal, Compliance & Risk Arbitration"
]

async def forge_division_roles(division_name):
    prompt = f"""
    You are structurally engineering an AI Operational Role Schema for an Apex Autonomous Agent OS.
    
    Task: Generate EXACTLY 10 highly sophisticated, unique, god-tier AI meta-agent Roles that belong in the '{division_name}' division.
    
    CRITICAL RULES FOR EACH ROLE:
    1. The name MUST be an expansive, meta-agent title (e.g., 'Autonomous Knowledge Acquisition & Pedagogy Structurer', 'Meta-Architect of System Engineering', 'Algorithmic Trend & Market Sentiment Harvester'). Do not use simple humans titles like 'Teacher' or 'Developer'.
    2. Expand the `description:` into a massive, dense semantic paragraph defining exactly what cross-functional behaviors and meta-cognitive skills the agent executes. It must be big enough and complex enough for our cognitive router to context-switch into.
    3. Under `websites:`, ADD 6-8 elite, high-value relevant HTTP target domains for that profession that they use to scour for info.
    4. Generate an `example_workflows:` array. Make 6 explicitly distinct highly complex target tasks. Each must have an 'intent' string and a 'steps' array of strings.
    5. Append exactly the following fields at the end of every role: `decision_logic`, `constraints`, `verification`, and `recovery`. Fill them with 2-3 logical strategy sentences matching an apex OS mental loop.
    
    OUTPUT FORMAT:
    Output STRICTLY as valid YAML. No markdown tags like ```yaml. No conversational text.
    It MUST be formatted perfectly like this, using exactly 2-space indentation:
    
# ===== {division_name.upper()} =====
- Role Name 1:
    description: "..."
    websites:
      - "..."
    example_workflows:
      - intent: "..."
        steps:
          - "..."
    decision_logic: "..."
    constraints: "..."
    verification: "..."
    recovery: "..."
- Role Name 2:
    ...
    """

    headers = {
        "Authorization": f"Bearer {OPENROUTER_API_KEY}",
        "Content-Type": "application/json"
    }

    payload = {
        "model": OPENROUTER_MODEL,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.4
    }

    print(f"[*] Dispatching prompt for division: {division_name}...")
    
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
            print(f"[+] Division '{division_name}' fully materialized (~10 roles forged).")
            return content.strip() + "\n\n"
        else:
            print(f"[-] API Error for {division_name}: Status {response.status_code}")
            return ""
    except Exception as e:
        print(f"[-] Fetch Error for {division_name}: {e}")
        return ""

sem = asyncio.Semaphore(3)

async def generate_all():
    print("[*] Initiating Forging Sequence: 100 God-Tier AI Roles...")
    
    with open(registry_path, 'w') as f:
        f.write("roles:\n")
        
    tasks = [forge_division_roles(div) for div in DIVISIONS]
    results = await asyncio.gather(*tasks)
    
    master_yaml = ""
    for r in results:
        if r:
            master_yaml += r
            
    with open(registry_path, 'a') as f:
        f.write(master_yaml)

    print("[+] Master Role Registry OS generation complete! Written to master_role_registry_100.yaml.")

if __name__ == "__main__":
    asyncio.run(generate_all())
