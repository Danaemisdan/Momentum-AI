# M2 Cognitive Loop & Agentic Behavior

**Status:** Planning
**Phase Goal:** Establish true agentic behavior by decoupling "Core Human Skills" from "Platform Skills." The system must balance extreme speed for known daily tasks with robust, self-healing exploration and learning for entirely new workflows and platforms.

## 1. The Separation of Intelligence: Core vs. Platform

To achieve both high-speed task execution and true human-like adaptability, Momentum AI strictly separates its knowledge base into two distinct domains. Mixing these is strictly prohibited, as it leads to brittle, unscalable architectures.

### Tier 1: Core Human Skills (The Executive Engine)
These skills define **how to be a human being**. They govern the agent's underlying logic, motives, physical interactions, and reasoning capabilities. They are completely decoupled from any specific website or tool.

- **Physical OS/Browser Interaction:** How to mimic a human using a computer (mouse movements, drag-and-drop, scroll behavior, key combinations like Cmd+C/Cmd+V).
- **Reasoning & Deduction:** Planning the immediate next steps aligned with a broader, abstract goal.
- **Subject-Matter & Domain Expertise:**
  - *Design Rules:* Discerning good design from bad design, recognizing spacing, formatting, and premium aesthetic requirements.
  - *Sales & Persuasion:* Understanding human psychological motives, pitching, copy-writing, and empathy.
  - *Abstract Motives:* Acting with a specific meta-intent like "focus on monetization", "prioritize speed," or "do this for passion."
- **DOM & Intent Interpretation:** The fundamental ability to analyze a completely novel UI layout, read text and shape heuristics, and deduce the logical intent of an interactive element. 

### Tier 2: Platform Skills (The Reflexes)
These are hardcoded, optimized execution paths for specific, well-mapped applications (e.g., WhatsApp, LinkedIn, Notion).

- **How it works:** Instead of actively deducing the layout of WhatsApp every time it wants to send a message, the agent relies on pre-mapped DOM targets and task sequences.
- **Benefit:** Lightning-fast daily tasks. The agent acts on pure "muscle memory," bypassing the slow cognitive planning latency, ensuring responses remain instantaneous.

---

## 2. The Cognitive Agentic Loop

When presented with an open-ended request ("Make money") or dropped into a completely novel platform where no "Platform Skill" exists, the agent defaults to True Exploration Mode.

### The Autonomous Protocol:
1. **Goal Assessment:** 
   - Analyze the goal against available resources. 
   - *Crucial Agentic Trait:* If the goal is impossible or highly ambiguous, the agent must inherently **ask for user clarification** rather than making blind assumptions or hallucinating.
2. **Context Gathering:** Navigate to the platform and capture the raw DOM state.
3. **Intent Visualization (via Core Skills):** The agent applies its human-like DOM interpretation skill to figure out the interactive landscape of the page.
4. **Action Planning:** Plan the immediate first physical step needed to progress toward the objective. 
5. **Execution:** Act (click, type, navigate, combo).
6. **Observation & Retry:** Observe the new DOM. 
   - Did we progress toward the goal? If yes, continue.
   - If the action failed or the state is stuck, the agent must **course-correct natively, re-evaluate the DOM, and try a different approach**. 

---

## 3. Implementation Steps for This Phase

1. **Skill System Refactoring:** Physically isolate `skills/core/` from `skills/platforms/` on the file structure.
2. **Schema Hardening:** Implement strict JSON actions in `models.rs` (`ask_user`, `reflect`, `plan`, `act`). 
3. **Loop Construction:** Write `brain.rs` to loop: Goal -> Read DOM -> Try Platform Task -> Fallback to Core Deductive Loop -> Act or Ask for Clarification.
4. **Robust Retries:** Bake recursive retries into the Rust execution engine when DOM states do not match expected outcomes.

## Issues Encountered & Fixes

- *To be populated during active coding...*
