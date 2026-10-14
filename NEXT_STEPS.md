# Momentum — Next Steps & Architecture Vision
**Last updated: October 12, 2026 — 5:05 PM**

---

## The Core Philosophy

The agent is not a command executor. It's a **thinking partner that acts autonomously**.

Single actions are the least interesting thing it can do. Anyone can build a button that opens Spotify. What makes Momentum different is everything that runs **without you asking** — projects that execute themselves, schedules that fire at the right time, triggers that react to the world.

The agent should be so well-configured to your life that you could leave for a week and come back to find your outreach done, your inbox summarised, your leads followed up with, and your project a few phases further along.

---

## The Four Agent Modes

### Mode 1 — Instant Action (Least Important, Already Exists)
Single, immediate tasks. "Calculate 25*4", "open Blender", "play music".  
Fast path, minimal planning, done in seconds.  
**Polish needed:** slot-filling, post-execution follow-up, ambiguity resolution. But don't over-invest here.

---

### Mode 2 — Project Mode (Most Important)
Long-running, multi-phase work. "Make a game from scratch", "build me a website", "run a full outreach campaign to 200 people".

**How it works:**
1. Agent understands scope — asks the minimum questions to understand what's being built
2. Generates a **project plan** — phases, milestones, tools needed, estimated time
3. Shows the plan to the user for approval before starting
4. Executes phase by phase, checking in at each milestone
5. Stores the entire project as a **Task node tree in the graph** — survives restarts
6. Adapts if something fails — replans, asks for input, continues

**Why the graph is critical here:**  
The graph stores project state. If you close Momentum and reopen it, it knows it's in phase 3 of the game dev project, Blender is done, now it needs to set up Unity. It never forgets where it left off.

**Examples:**
- "Make a game" → Concept → Assets (Blender/SD) → Engine (Unity/Godot/Unreal) → Code → Test → Export
- "Run outreach to 200 leads in fintech" → Find leads → Qualify → Draft messages → Send wave 1 → Wait → Followup wave 2 → Report results
- "Build me a portfolio website" → Design (Figma/Canva) → Code (VS Code) → Deploy (Vercel/Netlify) → Done

---

### Mode 3 — Scheduled / Repeat Mode
Tasks that run on a timer. Set once, run forever until stopped.

**Examples:**
- "Every morning at 9, check my email and summarise the important stuff"
- "Every Monday, generate and send the weekly status report to my team"
- "Follow up with these leads every 3 days until they respond"
- "Every hour, check my LinkedIn messages and notify me if anything urgent"

**How it works:**
- Agent creates a **recurring Task node** in the graph: `{type: scheduled, interval: "3 days", tactic: followup_email, targets: [...]}`
- ServiceManager picks it up, runs on schedule
- Each run: execute the task, write the outcome to graph, update tactic success rates
- Gets smarter over time — if followup message style X gets more replies, it uses that more

**What's needed to build this:**
- Proper cron-like scheduler in ServiceManager (currently a skeleton)
- Task node schema with interval, last_run, next_run, targets, tactic
- Execution loop that picks up due tasks on startup and on timer

---

### Mode 4 — Reactive / Trigger Mode (The Hard One)
Agent listens to the world and fires when conditions are met. No human prompt needed.

**Examples:**
- "When I get a message from jagadeesh, notify me immediately"
- "When a LinkedIn job post matches [criteria], apply automatically"
- "When bitcoin hits 100k, send me a notification"
- "When my inbox has more than 20 unread, summarise them"
- "When [specific person] posts on LinkedIn, like and comment"

**How it works:**
- Agent creates a **condition node** in the graph: `{type: trigger, condition: "message_from:jagadeesh", action: "notify"}`
- Background watcher checks conditions continuously
- On match: fire the action

**The hard part — how to actually listen:**
- **macOS**: Hook into notification center API (`NSUserNotificationCenter` / `UNUserNotificationCenter`) — this is the right way. Notification comes in from WhatsApp → agent intercepts it → checks if it matches any condition node → fires action.
- **Windows**: Windows toast notification API + accessibility events
- **Polling fallback**: For things that don't generate notifications (LinkedIn posts, price checks) — periodic checks every N minutes. Not elegant but works.

**This is genuinely novel.** Very few AI agents do reactive mode properly. This + project mode is what makes Momentum something people can't live without.

---

## Cross-Cutting Features (All Modes Need These)

### Slot-Filling (Generalised — Not Hardcoded)
Before executing ANY task, check what's missing from required_inputs of the relevant tactic node.  
Ask ONE question. Get the answer. Continue.  
Never ask multiple questions. Never ask stupid questions. Never ask things you already know.

### Ambiguity Resolution
Fires when there are multiple matches:
- Multiple apps with same name → "Which Blender? 3.6 or 4.1?"
- Multiple contacts → "Jagadeesh who? I see Jagadeesh R and Jagadeesh K"
- Multiple platforms relevant → "WhatsApp, LinkedIn, or both?"
- Nothing installed → "Not installed. Want me to find the download?"

### Multi-Platform Fan-Out
If user specifies 2+ platforms → execute on all of them, sequentially.  
Report per-platform: "Sent on WhatsApp ✓. Sending on LinkedIn now..."

### Post-Execution Follow-Up ("What Next?")
Never just say "Done." and stop. Always close the loop:
- "Sent! Want me to set a followup reminder?"
- "Blender's open. What are we building?"
- "Found 47 leads. Filter by role, company size, or location?"

### Research → Output Routing
Information needs to go somewhere. Always ask (or use stored preference):
- Google Doc, Notion, Slides, email, or just tell me?
- If user made this choice before → use it automatically

### User Preference Memory
Write every choice to the graph on `person:user`:
- Preferred music app, notes app, messaging platform, tone, outreach order
- Once known → never ask again. Confirm only: "Using Spotify as usual?"

---

## The Full Execution Loop (Target Architecture)

```
Trigger (user input / schedule fires / condition met)
    ↓
Which Mode?
  → Instant: fast path execution
  → Project: load project state from graph, advance next phase
  → Scheduled: execute the recurring task, write outcome
  → Reactive: evaluate condition, fire action if matched
    ↓
Slot Check — what's missing?
  → Ask ONE question if needed
    ↓
Ambiguity Check — multiple matches?
  → Ask which one if needed
    ↓
Graph Context Injection → inject_context_for(goal) [0ms]
    ↓
Plan → execute → AX perception → verify
    ↓
Write outcome to graph (SUCCEEDED_IN / FAILED_IN / tactic weights)
    ↓
Post-Execution
  → What next? / Where does output go?
  → Update project phase if in project mode
  → Schedule next run if in repeat mode
```

---

## Priority Order

| # | Feature | Mode | Complexity | Impact |
|---|---------|------|------------|--------|
| 🔴 1 | Platform disambiguation (which platform for messaging) | Instant | Low | Huge |
| 🔴 2 | Post-execution follow-up ("what next?") | All | Medium | Huge |
| 🔴 3 | Project mode — phase planning + graph state | Project | High | Massive |
| 🔴 4 | Scheduled/repeat task execution | Scheduled | Medium | Big |
| 🟡 5 | Multi-platform fan-out dispatch | Instant/Project | Medium | Big |
| 🟡 6 | Research → output routing | All | Medium | Big |
| 🟡 7 | Ambiguity resolution (multiple app/contact matches) | All | Medium | Big |
| 🟡 8 | User preference memory in graph | All | Medium | Big |
| 🟢 9 | Reactive/trigger mode (notification hooks) | Reactive | High | Massive |
| 🟢 10 | Message generation from relationship context | Instant | Medium | Medium |
| 🟢 11 | "Keep messaging" — recurring per-contact outreach | Scheduled | High | Big |

---

## Hard Rules

- **No hardcoding of platform flows** — graph tells what's possible, AX tells how
- **Never execute without understanding intent** — slot-fill first
- **Never stop after executing** — always close the loop
- **Never ask more than one question at a time**
- **Never hallucinate steps** — only act on what AX can actually see
- **User preferences once known → never ask again**
- **Project state always in graph** — agent survives restarts, knows where it left off
