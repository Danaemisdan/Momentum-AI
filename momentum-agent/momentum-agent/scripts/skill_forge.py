import asyncio
import os
import sys
import json
import re
import requests
from urllib.parse import urlparse
from playwright.async_api import async_playwright

# REQUIRED: pip install playwright requests
# RUN: playwright install chromium

OPENROUTER_API_KEY = "YOUR_OPENROUTER_API_KEY"
OPENROUTER_MODEL = "google/gemini-2.5-flash"

base_dir = os.path.dirname(os.path.dirname(__file__))
PROFILE_DIR = os.path.join(base_dir, '.momentum_chrome_profile')

def get_registry_urls():
    """Extracts all target websites from the Role registry autonomously."""
    registry_path = os.path.join(base_dir, 'skills', 'master_role_registry.yaml')
    unique_urls = set()
    if not os.path.exists(registry_path):
        print("[-] Master Role Registry not found.")
        return []
        
    with open(registry_path, 'r', encoding='utf-8') as f:
        for line in f:
            stripped = line.strip()
            if stripped.startswith("- http"):
                url = stripped[2:].strip()
                unique_urls.add(url)
    return list(unique_urls)

async def extract_interactibles(page, url: str):
    """Browses the URL using persistent auth and pulls deep accessibility elements natively."""
    try:
        print(f"[*] Authenticated Crawling: {url}...")
        await page.goto(url, wait_until="networkidle", timeout=25000)
        
        js_extractor = """
        () => {
            const elements = [...document.querySelectorAll('button, a, input, select, [role="button"], [role="link"]')];
            return elements.map(el => {
                const rect = el.getBoundingClientRect();
                const isVisible = rect.width > 0 && rect.height > 0 && window.getComputedStyle(el).visibility !== 'hidden';
                if (!isVisible) return null;
                
                return {
                    tag: el.tagName.toLowerCase(),
                    name: el.innerText?.trim() || el.value?.trim() || el.getAttribute('aria-label') || el.getAttribute('placeholder') || 'unnamed',
                    href: el.getAttribute('href') || 'none',
                    type: el.getAttribute('type') || 'none'
                };
            }).filter(i => i && i.name && i.name !== 'unnamed');
        }
        """
        raw_elements = await page.evaluate(js_extractor)
        
        unique_nodes = []
        seen = set()
        for el in raw_elements:
            # We enforce deep subpage mapping by including hrefs
            ident = f"{el['tag']}-{el['name']}-{el['href']}"
            if ident not in seen:
                seen.add(ident)
                unique_nodes.append(el)
        return unique_nodes
    except Exception as e:
        print(f"[-] Failed to crawl {url}: {e}")
        return []

def prompt_openrouter(platform_name, interactibles):
    """Pumps the massive DOM dict to OpenRouter enforcing generic interaction heuristics."""
    print(f"[*] Synthesizing Authenticated Heuristics via OpenRouter ({OPENROUTER_MODEL})...")
    
    prompt = f"""
    You are an AI generating an exact physical Platform Capability Schema for: {platform_name}.
    We use this to train an autonomous browser agent. This page may be an authenticated internal dashboard (behind a login wall).
    
    Here is the exact dictionary of 100% real, scraped functional interactibles from this specific deep-page:
    {json.dumps(interactibles, indent=2)}
    
    RULES:
    1. Organize the scraped interactibles into logical groups (e.g., 'primary_navigation', 'search_components'). 
    2. PAY STRICT ATTENTION TO HREFS. If an href points to a subdomain or internal route (like '/channels/@me'), document what that route physically achieves.
    3. DO NOT USE CSS SELECTORS. Only reference the "name", "tag", and "href".
    4. GENERALIZE ALL PERSONAL DATA: If you see specific friend names, your own profile name (e.g., 'Danny K'), or specific chat names, strip them. Replace them with universal tags like '[User Profile]' or '[Connection Name]'. The YAML must be universal across all accounts.
    5. Generate up to 10 core 'intents' based on what authenticated humans generally do internally here.
    6. PLATFORM CONTEXT: Write a clear 'platform_context' string explaining exactly what this website is, why it exists, and what human intent it serves globally. (e.g., "This platform functions as a peer-to-peer B2B directory primarily used for talent sourcing...").
    
    CRITICAL INSTRUCTION:
    LLMs are Thinkers, not Executors. DO NOT write hard step-by-step clicks like "click id=123".
    You must output loose `reference_patterns` (e.g., "search for export buttons") to guide the Rust Execution Engine.
    You must never assume an element exists. 
    
    Output the result as PURE YAML text (no markdown wrapping).
    Schema layout MUST rigidly follow this format structure:
    platform: string
    platform_context: string
    interactibles: dict
    intents:
      - name: string
        reference_patterns:
          - string
        decision_logic:
          - string
        constraints:
          - "never assume element exists"
          - "always verify before acting"
        verification:
          - string
        recovery:
          - string
    """

    headers = {
        "Authorization": f"Bearer {OPENROUTER_API_KEY}",
        "Content-Type": "application/json",
        "HTTP-Referer": "https://momentum-agent.io",
        "X-Title": "Momentum Skill Forge"
    }

    payload = {
        "model": OPENROUTER_MODEL,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.2
    }

    response = requests.post("https://openrouter.ai/api/v1/chat/completions", headers=headers, json=payload)
    if response.status_code != 200:
        print(f"[!] OpenRouter API Error: {response.text}")
        return None
        
    data = response.json()
    yaml_output = data['choices'][0]['message']['content'].strip()
    
    if yaml_output.startswith("```yaml"):
        yaml_output = yaml_output.replace("```yaml", "", 1)
    if yaml_output.endswith("```"):
        yaml_output = yaml_output[:-3]
        
    return yaml_output.strip()

async def auth_setup():
    """Opens a persistent chrome window for manual credential entry."""
    print("[*] Launching Interactive Session for Authentication Walls...")
    os.makedirs(PROFILE_DIR, exist_ok=True)
    
    async with async_playwright() as p:
        context = await p.chromium.launch_persistent_context(
            user_data_dir=PROFILE_DIR, 
            headless=False,
            viewport={'width': 1280, 'height': 800}
        )
        print("\n" + "="*80)
        print(">>> BROWSER LAUNCHED IN PERSISTENT MODE <<<")
        print("Please manually log in to critical websites (e.g., LinkedIn, Discord, X, Upwork).")
        print("Check all 'Remember Me' boxes. ")
        print("When you are fully authenticated across the board, press ENTER in this terminal.")
        print("="*80 + "\n")
        
        input("Press ENTER to save authentication state and close... ")
        await context.close()
    
    print("[✔] Global Authentication State saved successfully to `.momentum_chrome_profile`.")

async def mass_forge(interactive=False):
    """Autonomously loops over all URLs utilizing the authenticated persistent context."""
    print("[*] Starting Momentum Mass Authenticated Skill Forger...")
    urls = get_registry_urls()
    print(f"[+] Found {len(urls)} unique websites to map.")
    
    if not urls:
        return
        
    plat_dir = os.path.join(base_dir, 'skills', 'platforms')
    os.makedirs(plat_dir, exist_ok=True)
    os.makedirs(PROFILE_DIR, exist_ok=True)
    
    async with async_playwright() as p:
        context = await p.chromium.launch_persistent_context(
            user_data_dir=PROFILE_DIR, 
            headless=True
        )
        page = context.pages[0] if context.pages else await context.new_page()
        
        for idx, url in enumerate(urls):
            domain = urlparse(url).netloc
            safe_filename = re.sub(r'[^a-zA-Z0-9_\-]', '_', domain) + ".yaml"
            file_path = os.path.join(plat_dir, safe_filename)
            
            if os.path.exists(file_path):
                print(f"[{idx+1}/{len(urls)}] Skipping {domain} (Already Forged)")
                continue

            if interactive:
                # Blocking input to pause the asyncio loop explicitly
                choice = input(f"[{idx+1}/{len(urls)}] Forge {domain}? [y=yes, s=skip, q=quit]: ").strip().lower()
                if choice == 'q':
                    print("[-] Forging Aborted.")
                    break
                elif choice == 's':
                    print(f"[-] Dropped {domain}.")
                    continue
                elif choice != 'y' and choice != '':
                    print(f"[-] Unknown input, dropping {domain}.")
                    continue
                
            interactibles = await extract_interactibles(page, url)
            if not interactibles:
                continue
                
            yaml_payload = prompt_openrouter(domain, interactibles)
            if yaml_payload:
                with open(file_path, 'w') as f:
                    f.write(yaml_payload)
                print(f"[✔] [{idx+1}/{len(urls)}] Forged Heuristic Map -> {safe_filename}")
                
        await context.close()
    print("\n[+] Mass Forging Cycle Complete.")

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--auth":
        asyncio.run(auth_setup())
    elif len(sys.argv) > 1 and sys.argv[1] == "--interactive":
        asyncio.run(mass_forge(interactive=True))
    else:
        asyncio.run(mass_forge(interactive=False))
