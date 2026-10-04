# Momentum — Complete Product & Architecture Document

**Version:** 1.0  
**Date:** March 2026  
**Status:** Active — governs every build and design decision

---

## 1. The Vision

Momentum is a fully local, cross-platform AI personal assistant that controls a real Chrome browser and does anything a human being would do on a computer. It is not a chatbot. It is not a script runner. It is a thinking, learning, adapting agent that you talk to in plain English and it executes.

> **"If I say go make money, it should just research properly, become literally a human being in earning money with the resources it has."**

The product goal is simple: an agent that acts like an extremely capable human assistant sitting at your computer. It knows what it has access to, it plans before it acts, it asks when it doesn't know something, it learns from every session, and it never stops because one task is done — it's always listening, always alive.

---

## 2. Core Principles (Non-Negotiable)

These are not preferences. These are rules. Every design decision must serve these.

### 2.1 Continuous & Alive
The agent is NEVER in a stopped state. It starts on boot, enters Idle mode, and stays alive until you kill the process. When a task is done, it returns to Idle — not shut down. It is always listening. If you give it a new task 3 days into a session, it picks up like you never left.

### 2.2 Self-Aware
The agent knows:
- What accounts it has on which platforms
- What files are on the disk and where
- What apps are running on the PC
- What the PC's hardware specs are and what it can run
- What actions it can and cannot take in the browser
- Its own limitations (can't do camera, mic, can't make purchases without explicit per-session approval)

This is NOT optional. Without self-awareness, the agent blindly hits login pages for accounts it doesn't have, tries to use resources it can't access, and hallucinates capabilities it lacks.

### 2.3 One Brain, All Tasks
There is NO intent classifier. NO `is_chat()` function. NO separate "conversation pipeline" vs "action pipeline." One cognitive loop reads the world and decides whether to talk, navigate, click, or reflect — all from the same LLM call. The LLM makes this decision every single step.

### 2.4 Human-Like, Not Robotic
- Conversational tone at all times — even while executing tasks, the agent narrates naturally
- UI interactions are stealth-grade: full CDP event chains (mousemove → mouseenter → mousedown → mouseup → click), not just `.click()` dispatches
- Agent plans before acting, clarifies before assuming, reflects when stuck
- No unnecessary steps — every action moves directly toward the intent

### 2.5 Fast
- Known tasks (verified skills) execute on a fast-path: no LLM planning, just element identification. ~5-10x faster.
- Models are the lightest possible for the quality needed
- Total RAM footprint: under 6GB on 8GB RAM machines
- Vision model is lazy-loaded (only when actually needed)

### 2.6 Zero External Brands Visible
No user-facing mention of Llama, Qwen, llama.cpp, Ollama, or any third-party model anywhere. The product is Momentum. The LLM is "Momentum Engine." The vision model is "Momentum Vision." Even the GGUF file is named `momentum-engine-3b.gguf`.

### 2.7 Cross-Platform Day One
Mac (Apple Silicon + Intel) and Windows (CPU, CUDA GPU) from day one. No "Mac first, Windows later." Both targets accounted for in every architectural decision.

---

## 3. What It Can Do (Full Capability List)

### 3.1 Browser Control — Universal
The agent can do ANYTHING a human can do in a Chrome browser tab:

| Action | How |
|---|---|
| Navigate to any URL | CDP `Page.navigate` |
| Click any element | Full CDP mouse event chain (move → enter → over → down → up → click) |
| Type text | CDP `Input.insertText` (natural, undetectable) |
| Press keyboard shortcuts | CDP `Input.dispatchKeyEvent` (Ctrl+C, Cmd+V, Enter, Tab, Escape, arrows, etc.) |
| Click and hold | CDP `mousedown` held for N ms → `mouseup` |
| Click and drag | CDP `mousedown` → interpolated `mousemove` sequence → `mouseup` |
| Scroll | CDP `Input.dispatchMouseEvent` type `mouseWheel` |
| Upload any file from disk | CDP `DOM.setFileInputFiles` — bypasses OS file dialog entirely, cross-platform |
| Navigate inside iframes | CDP iframe target switching — agent navigates inside embedded pages |
| Open new tabs | CDP new target creation |
| Switch between tabs | CDP target activation |
| Close tabs | CDP target closing |
| Read images | Vision model describes image content |
| Read video thumbnails | Vision model describes video thumbnail |
| Understand visual UI state | Screenshot → vision model → text description injected into prompt |

### 3.2 Task Types (What You Can Ask It To Do)

**Anything on the web:**
- "Message John on WhatsApp saying I'll be 10 minutes late"
- "Find me 50 LinkedIn profiles of startup founders in India and export them"
- "Go build a logo on Canva for my brand [describes brand]"
- "Book the cheapest flight from Mumbai to London in March"
- "Reply to all my unread emails with a friendly holding message"
- "Post this image on Instagram with this caption"
- "Make money" → agent plans: finds a freelancing approach using your existing accounts, researches opportunities, pitches

**Research:**
- "What's the best CRM for a 3-person sales team under $50/month?"
- "Find me the top 10 competitors of Notion and summarize their pricing"
- Agent searches its knowledge database first. If not found, uses headless browser to research. Cites sources.

**File operations:**
- "Upload the PDF in my Downloads folder to this form"
- "Find the latest invoice in my Desktop and send it to this email"

**Open-ended autonomous tasks:**
- "Grow my LinkedIn following" → agent plans a multi-week approach, executes daily
- "Find sales leads for my SaaS targeting e-commerce stores"

### 3.3 What It CANNOT Do (Hard Limits)
- No camera access
- No microphone access  
- No purchases unless explicitly authorized per-session
- No access to domains in `never_domains` list (user-configured)
- No fabricating credentials — if it doesn't have the password, it asks
- No OS-level operations outside of file access in configured directories

---

## 4. The Architecture (9 Layers)

Built bottom-up. Each layer has one responsibility. No layer is modified after the layer above it depends on it.

### Layer 1: Momentum Engine
- **What:** Local LLM inference. Returns structured JSON. Nothing else.
- **How:** `llama-cpp-rs` (Rust binding) — direct in-process llama.cpp. No Ollama, no HTTP server. Fastest possible inference.
- **Model:** `momentum-engine-3b.gguf` — Llama-3.2-3B-Instruct Q5_K_M renamed. ~2.2GB RAM.
- **Accuracy:** Grammar-constrained decoding (GBNF schema) — model is mathematically forced to output valid JSON on every call. Zero parsing errors.
- **Acceleration:** Metal GPU on Mac M-series. CUDA on Windows GPU. CPU fallback on all.
- **Future:** Fine-tuned version of the same model where agent behavior and JSON schema understanding is baked INTO the weights. Eliminates system prompt overhead. Called "Momentum Engine v2" when ready.

### Layer 2: Momentum Vision
- **What:** Screenshot → text description and raw click coordinates (`click_coordinates`).
- **How:** Highly-performant 800M Vision Model quantized to Q4 — ~300MB-400MB RAM. Runs as child process via llama.cpp's own built-in server (NOT Ollama). Called "Momentum Vision" everywhere.
- **When it runs:** Lazy-loaded — only spawned when vision is needed (CAPTCHA, opaque Canvas fallback, ambiguous DOM)
- **When it's called:**
  - After every navigate (confirm page actually loaded what we expected)
  - When DOM extraction finds ambiguous/empty elements
  - When CAPTCHA or QR suspected
  - When task involves understanding image/video content
  - When agent needs to verify an action was executed correctly

### Layer 3: Stealth Browser Engine
- **What:** Execute actions on Chrome via CDP. Return raw DOM + screenshots.
- **Chrome:** User's real Chrome with their own profile (cookies, sessions, login states)
- **Connection:** CDP remote debugging port 9222
- **Stealth:** Injected via `Page.addScriptToEvaluateOnNewDocument` before every page load:
  - Remove `navigator.webdriver`
  - Randomize `navigator.hardwareConcurrency`, `navigator.deviceMemory`
  - Override canvas/WebGL fingerprinting signatures
  - Mask CDP-specific JS properties
- **DOM Stabilization:** Poll DOM hash every 400ms. Wait for 2 identical consecutive hashes AND ≥1 interactive element. Max 15 seconds. Zero `sleep()` calls anywhere else. Ever.

### Layer 3.5: Terminal & File Engine (Desktop Scope)
- **What:** Controlled axis for limited local desktop execution. Gives Momentum real-world file manipulation.
- **Why:** Full Headless visual desktop control is error-prone. We isolate macOS/OS interaction purely to:
  - Running validated CLI commands (via zsh)
  - Reading/Writing specific files, folders, and code
- **Safety:** Sandboxed to `~/Downloads`, `~/Desktop`, and specific project repositories. Excluded from destructive system paths (`/System`, `/Library`).
- **Vision Coordinate Fallback (Canvas/Desktop Crisis):** If we must click outside the DOM (e.g. native Mac Prompts or Canvas games), we use **Set-of-Mark (SOM)**. A local YOLO/GroundingDINO detector draws numbered boxes over buttons on the screenshot. Momentum Vision 800M reads the ID number and spits out precise `click_coordinates(x, y)` without hallucinatory guesswork.

### 3.2 Dual Modality Encoding (The "Eyes + DOM" Standard)
When the active Chromium session loads a new state, Momentum does **not** rely on a lazy fallback. It executes two streams perfectly in parallel:
1. **CDP DOM Fast-Pass:** Rust stealthily strips the `aria-labels` and spatial bounding boxes into a sub-300 token JSON string mapping.
2. **Qwen-0.8B Vision Node:** Rust simultaneously captures the active viewport buffer and passes it to the lightweight 800M Set-of-Mark node, translating purely visual icons into text spatial nodes.

The 3B Cognitive LLM ingests BOTH arrays sequentially before making its singular step decision. This guarantees zero semantic loss for complex visual sliders alongside flawless precision for hidden textual anchors.

### Layer 4: DOM Intelligence
- **What:** Turn a 3000-element webpage into a 300–400 token LLM-usable representation. Nothing else.

**The 2-Pass approach:**

Pass 1 — Structural Skeleton (~80 tokens):
```json
{
  "url": "https://linkedin.com/feed",
  "title": "LinkedIn Feed",
  "sections": [
    {"id": "nav", "label": "Top navigation bar", "element_count": 12},
    {"id": "main", "label": "News feed with 24 posts", "element_count": 847},
    {"id": "sidebar", "label": "Profile summary sidebar", "element_count": 8}
  ]
}
```

Brain reads skeleton → says which section matters → Pass 2 runs ONLY on that section:
```json
{
  "elements": [
    {"id": 1, "tag": "input", "type": "text", "placeholder": "Search", "visible": true},
    {"id": 2, "tag": "button", "text": "Post", "visible": true},
    {"id": 3, "tag": "img", "alt": "Sarah's profile photo", "visible": true},
    {"id": 4, "tag": "video", "title": "Product demo video", "duration": "2:34"},
    {"id": 5, "tag": "iframe", "src": "https://...", "title": "Embedded survey"}
  ],
  "text_content": "Latest from Sarah: 'Announcing our Series A...' · 3h · 289 reactions"
}
```

- Elements assigned stable integer IDs per extraction (not CSS selectors — those break)
- Images described by alt text + vision description
- Videos described by title + duration + thumbnail description from vision
- iframes extracted recursively
- Repeated similar elements compressed ("32 similar job cards, showing first 5")

### Layer 5: Multi-Tab Manager
- Tracks all open Chrome tabs with SQLite-persisted state
- Brain can: `open_tab`, `switch_tab`, `close_tab`
- When switching: current tab state serialized → next tab context loaded → brain continues
- Tab state: URL, title, last DOM hash, screenshot, task context (compressed)
- Enables cross-tab workflows: "Open Gmail in Tab 1, copy a message, switch to LinkedIn Tab 2, paste it in a message"

### Layer 5.5: Agent Self-Model
The agent's persistent knowledge of itself. Consulted before EVERY navigation and EVERY task.

```toml
# ~/.momentum/self_model.toml — user-editable, agent auto-updates

[identity]
name = "Momentum"
owner = "Sanjeev"

[accounts]
"linkedin.com" = { has_account = true, username = "sanjeev@..." }
"whatsapp.com" = { has_account = true, phone = "+91..." }
"canva.com" = { has_account = true }
"twitter.com" = { has_account = false }  # won't try to log in
"bankofamerica.com" = { has_account = false, note = "Never access" }

[capabilities]
browser_control = true
file_access = true
camera = false
microphone = false

[files]
downloads_dir = "/Users/sanjeev/Downloads"
desktop_dir = "/Users/sanjeev/Desktop"
documents_dir = "/Users/sanjeev/Documents"

[constraints]
never_domains = []        # user adds no-go sites here
max_spend = 0             # 0 = cannot make purchases unless overridden per-session
require_confirm_actions = ["delete", "send_email", "post_publicly"]
```

Injected into every brain prompt as:
```
SELF AWARENESS:
I have accounts on: LinkedIn (sanjeev@...), WhatsApp (+91...), Canva, Gmail
I do NOT have accounts on: Twitter, any banking sites
I can access files at: /Users/sanjeev/Downloads, /Users/sanjeev/Desktop
I cannot: use camera, microphone, make purchases, access bankofamerica.com
```

When agent navigates to a site where `has_account = false` → it immediately outputs `ask_user`: *"I don't have an account on Twitter. Should I create one, or is there another way to do this?"* Never attempts login.

Agent auto-updates self_model.toml when it successfully logs into a new site.

### Layer 5.7: System & Browser Awareness
The agent knows the machine it's running on.

```json
{
  "os": "macOS 14.4",
  "cpu": "Apple M2, 8 cores",
  "ram_total_gb": 8,
  "ram_available_gb": 3.1,
  "gpu": "Apple Metal (integrated)",
  "running_processes": ["Chrome", "Slack", "Spotify"],
  "can_run_metal": true,
  "can_run_cuda": false
}
```

- Agent knows if RAM is low → warns user before starting heavy tasks
- Agent knows what's running → can reference running apps if relevant
- Agent knows OS → uses correct Chrome binary path, correct file system structure
- Injected into browser context: what tabs are open, what actions are possible, what the current tab is

### Layer 5.9: Knowledge Database
The agent's offline general knowledge bank. Searched BEFORE going to the internet.

- **Storage:** SQLite with FTS5 full-text search — no vector database, no extra RAM
- **Size:** Pre-loaded with 200+ curated reliable sources
- **Research:** Separate invisible headless browser (clean chromiumoxide instance) fetches and ingests new pages when needed — never interferes with user's Chrome

**Pre-loaded Source Categories:**
| Category | Examples |
|---|---|
| General knowledge | Wikipedia top 10k articles, WikiHow top 500 |
| Tech / Dev | MDN Web Docs, Stack Overflow top Q&As |
| Productivity | Google Workspace help, MS Office docs |
| Social / Business | LinkedIn help, Gmail help, WhatsApp help |
| Design | Canva tutorials, Figma docs |
| E-commerce | Amazon seller, Shopify, Stripe docs |
| Marketing | HubSpot blog, Neil Patel guides |
| Finance basics | Investopedia key concepts |

**Query flow:**
1. Task: "How do I schedule a LinkedIn post?"
2. Agent searches: `SELECT content FROM knowledge_chunks WHERE knowledge_chunks MATCH 'schedule post linkedin'`
3. Match found → inject as `BACKGROUND KNOWLEDGE:` in prompt → agent answers from it
4. No match → headless browser searches Google → reads top 3 results → saves to DB → retries
5. Agent also explicitly adds sources: "I found a useful guide, adding it to my knowledge base"

### Layer 6: Memory System
Full persistent memory across sessions. All stored in SQLite locally on device.

**What's persisted:**

| Data | Description | Accessible via |
|---|---|---|
| Chat messages | Every message in every session | UI sidebar + agent context |
| Action log | Every browser action (type, target, result, screenshot, thought) | UI history panel |
| Compressed steps | One-line summaries of each cognitive step (used in context window) | Agent brain |
| Autonomous plans | Multi-step plans for vague goals (step array, current step, status) | Agent brain |
| Learned knowledge | Domain-specific knowledge observed during tasks | Agent brain |
| System snapshots | RAM + processes at time of task execution | Debug/analytics |

**Context window management:**
- After each action: compress `(what I saw, what I did, result)` into 1-line summary
- Full raw DOM dropped immediately after compression
- Brain prompt contains: Self Awareness + System Context + Goal + last 10 compressed steps + fresh DOM
- Context window stays ~2k tokens regardless of task length

**History UI (Next.js sidebar):**
- All sessions listed with date, initial goal, duration
- Click any session → full chat replay + action list with screenshots
- Learning log: what the agent has learned, user can edit/delete entries

### Layer 7: Skills System — Two-Speed Execution
The key insight: a verified known task should NOT require LLM planning. Rust executes the steps; LLM only identifies which element on screen matches each step description.

**Skill File Format v2:**
```
# whatsapp.com/send_message
DESCRIPTION: Send a text message to a WhatsApp contact
VARIABLES: contact_name, message_content
STATUS: verified

STEPS:
  1. navigate: https://web.whatsapp.com
  2. click: search input (placeholder "Search or start new chat")
  3. type: {contact_name}
  4. click: first matching contact result
  5. click: message input area
  6. type: {message_content}
  7. key_combo: Enter
```

**Two execution modes:**

| Mode | Trigger | Speed | How |
|---|---|---|---|
| Fast Path | Task matches a `verified` skill + required variables known | ~5-10x faster | Rust reads steps, LLM only does element ID per step |
| Exploration | No matching skill, or task is new/ambiguous | Normal | Full LLM cognitive loop every step |

**Promotion pipeline:**
1. Exploration mode completes successfully → auto-saves as `unverified` skill
2. User tests it a few times, it works reliably → user marks `verified`
3. Now runs on fast path permanently
4. If a verified skill fails 3 times → demoted to `unverified`, agent reinvestigates

**Remote hosting:**
- Skills hosted at `/api/skills/[...slug]` on our Next.js app (Vercel)
- Agent authenticates: `Authorization: Bearer MOMENTUM_SKILLS_TOKEN`
- Local cache: `~/.momentum/skills_cache/` — refreshed every 24h
- Startup pre-fetch: WhatsApp, LinkedIn, Gmail, Google, Canva

### Layer 8: Single Brain — Continuous Runtime
The heart of the system. Always alive. Always observing. Always deciding.

**Runtime Lifecycle:**
```
STARTUP
  └─ Load self_model.toml
  └─ Connect to Chrome (CDP port 9222)
  └─ Start Momentum Engine (in-process)
  └─ Pre-fetch skills cache
  └─ Enter IDLE mode
  └─ Tell user: "I'm ready. What do you want to do?"

IDLE MODE (no task running)
  └─ Listen for user input
  └─ Can do periodic background reflection (optional)
  └─ WebSocket stays connected

USER SENDS MESSAGE
  └─ Evaluated by unified continuous brain.
  └─ Capable of spawning DeerFlow 2.0 Sub-Agents for long-horizon background tasks.
  └─ Instantly begins action loop without gating if intent is clear.

PLANNING MODE (for goals like "make money", "grow my business")
  └─ Agent thinks from self-model and memory.
  └─ Outputs plan array or spans `spawn_background_task` to run headless Chromes.
  └─ Streams plan to user.
  └─ User approves / edits / says "go".
  └─ Agent executes step by step or supervises Sub-Agents.
  └─ Between steps: outputs `reflect` or `update_intent` → reviews progress.

BACKGROUND SUB-AGENT SPAWNING (DeerFlow 2.0 Pattern)
  └─ Agent: "This requires deep research. `spawn_background_task('scrape_leads', 'AI startups')`."
  └─ Rust spins up a headless Chromium thread isolated from the main chat.
  └─ Main chat remains 100% active and unblocked.
  └─ Agent uses `check_sub_agent` to poll results later.

ACTIVE LOOP (runs indefinitely until idle action)
  A. Load self-model summary
  B. Load active tab context (tabs.rs)
  C. Fetch DOM → 2-pass semantic extraction (dom.rs)
  D. Capture screenshot → If Canvas/Native OS → Pass to SOM Detector (Layer 3.5) → Momentum Vision 800M.
  E. Load domain skill file (skills.rs)
  F. Build prompt:
       SELF AWARENESS: [accounts, capabilities, constraints]
       SYSTEM: [OS, RAM available, running processes]
       GOAL: [Original Goal + Active Intent]
       SUB-AGENTS: [List of running background threads]
       MEMORY: [last 10 compressed steps]
       DOMAIN KNOWLEDGE: [matched skill file for current domain]
       BROWSER STATE: [open tabs, current tab, available actions]
       WHAT I SEE (DOM or SOM JSON): [semantic elements or numbered button logic]
  G. Call Momentum Engine → JSON action
  H. Stream action.speech to UI immediately
  I. If known skill fast-path match → execute via skills.rs
     Else → execute action via browser.rs, or OS via terminal.rs
  J. Record to memory.rs (action + compressed step)
  K. If action.type == 'idle' → back to IDLE
     If action.type == 'ask_user' | 'need_auth' → pause, wait for input, resume
     If stuck (same action 3x with no progress) → output `reflect` → replan
     Else → go to A

REFLECT (anti-stuck logic)
  └─ Agent reviews: "I've tried clicking X 3 times and nothing changed."
  └─ Outputs `reflect` action with new approach
  └─ May output `ask_user` if genuinely stuck
  └─ NEVER infinitely loops. Always either progresses or asks.
```

**The Frozen Action Schema (everything flows through this):**
```json
{
  "thought": "internal reasoning, never shown to user",
  "speech": "what to say to user right now",
  "action": {
    "type": "chat | navigate | click | type | scroll | key_combo | hold_click | drag | upload_file | ask_user | need_auth | open_tab | switch_tab | close_tab | plan | reflect | idle | task_complete | error | spawn_background_task | update_intent | check_sub_agent | click_coordinates",
    "target_id": 42,
    "value": "text to type",
    "url": "https://...",
    "message": "question or status for user",
    "file_path": "/absolute/path/to/file",
    "tab_id": "uuid",
    "key": "ctrl+c",
    "from_id": 10,
    "to_id": 20,
    "plan_steps": ["Step 1: ...", "Step 2: ..."]
  }
}
```

This schema is defined in `src/models.rs`. **It never changes without a version bump.**

**Auth / CAPTCHA / Stuck Detection:**
Baked into every prompt:
> "If you see a login form: check your self-model. If you have an account → continue. If you don't → output `ask_user` and explain. Never fabricate credentials."
> "If you see a CAPTCHA, QR code, phone verification → output `need_auth` immediately."
> "If you've taken the same action 3 times with no visible change → output `reflect` with a new approach."

### Layer 9: Web UI
- **Framework:** Next.js (App Router, TypeScript)
- **Backend:** Axum (Rust) WebSocket server on port 3000
- **UI port:** 3001
- **Design:** Premium dark mode — Momentum purple (#6d56fa) accent, Inter font, glassmorphism elements

**Features:**
- Chat interface with message bubbles (user right, agent left)
- Real-time streaming as agent speaks
- Typing indicator while agent is thinking
- Connection status (green/amber/red dot)
- History sidebar (all past sessions, searchable)
- Session detail view (full message replay + action log with screenshots)
- Settings panel (self_model.toml editor, skill management)
- [Future] Auth + subscription management

**WebSocket message protocol:**
```json
// UI → Agent
{"type": "user_message", "content": "..."}
{"type": "user_reply", "content": "..."}     // response to agent question
{"type": "cancel_task"}

// Agent → UI (real-time stream)
{"type": "ready"}
{"type": "agent_speech", "content": "..."}
{"type": "agent_question", "content": "..."}
{"type": "need_auth", "content": "..."}
{"type": "tab_opened", "tab_id": "...", "url": "..."}
{"type": "task_done", "content": "..."}
{"type": "error", "content": "..."}
{"type": "echo", "content": "..."}           // M1 testing only
```

---

## 5. RAM Budget

| Component | RAM | When |
|---|---|---|
| Momentum Engine (3B Q5_K_M) | ~2.2 GB | Always |
| Momentum Vision (800M Q4) | ~400 MB | Lazy — only when vision needed |
| Chrome (user profile) | ~400 MB | Always |
| Rust agent process | ~150 MB | Always |
| SQLite + knowledge + cache | ~100 MB | Always |
| OS overhead | ~1.0 GB | Always |
| **Normal operation (no vision)** | **~3.85 GB** | — |
| **Peak (vision active)** | **~4.25 GB** | — |

✅ Under 5GB peak. 8GB machine has 3GB headroom.  
✅ Metal (Mac M-series) and CUDA (Windows GPU) offload model computation — effective RAM usage lower.  
✅ CPU fallback: same RAM figures, just slower inference.

---

## 6. File Structure

```
Momentum Agents/
├── momentum-agent/                   # Rust backend
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                   # Axum server, WebSocket, startup lifecycle
│   │   ├── brain.rs                  # Cognitive loop (continuous runtime)
│   │   ├── engine.rs                 # Momentum Engine (llama-cpp-rs)
│   │   ├── vision.rs                 # Momentum Vision (Qwen/moondream sidecar)
│   │   ├── browser.rs                # CDP Chrome control + stealth patches
│   │   ├── dom.rs                    # 2-pass semantic DOM extraction
│   │   ├── tabs.rs                   # Multi-tab state manager
│   │   ├── self_model.rs             # Agent self-awareness (accounts, caps)
│   │   ├── system.rs                 # PC specs + browser state awareness
│   │   ├── memory.rs                 # SQLite: chat, actions, plans, knowledge
│   │   ├── knowledge.rs              # Knowledge DB + headless research browser
│   │   ├── skills.rs                 # Two-speed skill execution
│   │   └── models.rs                 # ALL shared types (frozen action schema)
│   ├── models/
│   │   └── momentum-engine-3b.gguf  # Main LLM (renamed)
│   └── ui/                           # Next.js frontend
│       └── src/app/
│           ├── page.tsx              # Chat interface
│           ├── layout.tsx
│           ├── globals.css
│           └── api/skills/[...slug]/
│               └── route.ts          # Skills proxy (Vercel)
├── skills/                           # Skill files (pushed to Vercel repo)
│   └── platforms/
│       ├── whatsapp.txt
│       ├── linkedin.txt
│       ├── gmail.txt
│       ├── google.txt
│       └── canva.txt
├── docs/
│   ├── MOMENTUM.md                   # This document
│   └── phases/                       # Phase docs (M1, M2, ... M12)
│       └── M1_foundation.md
└── .agents/
    └── workflows/
        └── phase_documentation.md    # Phase doc workflow rules
```

---

## 7. Tech Stack

| Layer | Technology | Why |
|---|---|---|
| LLM inference | `llama-cpp-rs` (Rust binding) | Fastest possible — in-process, no HTTP overhead, grammar constraints at C++ level |
| Vision model | Qwen2-VL-2B or moondream2 Q4 | ~1GB RAM, fast inference, CPU-capable |
| Browser control | `chromiumoxide` (CDP) | Best async CDP Rust library, Mac + Windows |
| Web server | `axum` (Rust) | Async, lightweight, native WebSocket |
| Async runtime | `tokio` | Standard Rust async |
| Database | `rusqlite` (SQLite, bundled) | Zero infra, embedded, cross-platform |
| Full-text search | SQLite FTS5 (built-in) | No extra RAM, fast, good enough for knowledge search |
| System info | `sysinfo` crate | CPU, RAM, processes, OS |
| Config | `toml` crate | self_model.toml parsing |
| UI | Next.js (TypeScript) | Subscription-ready, extensible, great UX |
| UI deployment | Vercel | Free tier, global CDN, skills API hosting |
| Serialization | `serde` + `serde_json` | Standard |
| HTTP client | `reqwest` | Skills fetch from Vercel |

---

## 8. Branding Rules (Zero Exceptions)

| What | Correct Name |
|---|---|
| The product | Momentum |
| The LLM | Momentum Engine |
| The vision model | Momentum Vision |
| The GGUF file | `momentum-engine-3b.gguf` |
| Rust variables | `momentum_engine`, `engine_call()`, `vision_describe()` |
| Logs | "Momentum Engine initialized", "Momentum Vision active" |
| UI status | "Momentum online" |
| `llama-cpp-rs` crate | Internal dep only, never user-facing |
| Qwen / moondream | Internal dep only, never user-facing |

---

## 9. Learning & Growth (How the Agent Gets Smarter)

### Session-Level Learning
Every action taken → logged with: action type, what was targeted, result (success/fail), screenshot. This is the raw training data.

### Domain-Level Learning  
Every time the agent navigates a site successfully → the successful navigation pattern is saved as `learned_knowledge` in SQLite. Next time it visits: faster, more accurate.

### Skill Promotion
Successful exploration runs → auto-saved as unverified skills → user verifies → fast-path execution. The agent literally teaches itself new skills from real usage.

### Fine-Tuning (Phase 2+)
After enough real sessions, the action logs become training data for a QLoRA fine-tune of `momentum-engine-3b.gguf`. Result: `momentum-engine-v2.gguf` — a model that natively understands the JSON action schema, the agent's persona, and common task patterns without needing a system prompt. This is the "Momentum Engine" that behaves like it was purpose-built, not derived from Llama.

---

## 10. Development Process Rules

1. **No phase starts without a phase doc** at `docs/phases/M{N}_{name}.md`
2. **Every problem encountered is logged immediately** in the phase doc — what broke, what fixed it
3. **No phase is closed** without user testing and explicit sign-off
4. **No layer is modified** after the layer above it has been built on top of it
5. **The action schema in `models.rs` is sacred** — only additive changes, never removals, requires version bump
6. **No hardcoded sleeps** anywhere — DOM stabilization handles all timing
7. **No string heuristics** (`is_chat()`, `is_greeting()`, `detect_platform()`) — LLM decides all behavioral questions, Rust decides all factual/deterministic questions

---

## 11. Build Milestones

| Milestone | What Works |
|---|---|
| M1: Foundation | Rust WS server + Next.js chat UI — echo round-trip ✅ |
| M2: Momentum Engine | LLM responds in chat, grammar-constrained JSON |
| M3: Browser Live | Agent navigates + reads DOM, DOM stabilization working |
| M4: Full Loop | Think → Act end-to-end on a simple task |
| M5: Self-Model | Agent knows its accounts, doesn't try to log into unknown sites |
| M6: Vision Online | Screenshots described, CAPTCHA detected |
| M7: Memory | History persists, context compression works, history UI live |
| M8: Knowledge DB | 200+ sources loaded, FTS search working, headless research fallback |
| M9: Skills Fast-Path | Known skills execute 5-10x faster than exploration |
| M10: Planning Mode | Vague goals get planned, stepped, reflected on |
| M11: Multi-Tab | Cross-tab tasks work, context preserved across switches |
| M12: Polish + Cross-Platform | Full test suite, Windows validation |
