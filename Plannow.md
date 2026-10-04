You are a senior systems architect and Rust engineer.

Your task is to REBUILD and SIMPLIFY an existing AI browser agent system called "Momentum Agent".

The current system is overengineered, fragmented, and does not exhibit real agentic behavior.

Your goal is to design and implement a CLEAN, MINIMAL, WORKING architecture that behaves like a real browser automation agent.

---

## CORE OBJECTIVE

Build a deterministic agent loop that can:

* Understand a user goal
* Observe the browser (DOM + structured UI)
* Decide the next best action
* Execute it
* Repeat until goal is achieved

---

## CRITICAL REQUIREMENTS

1. The system MUST be SIMPLE
2. The system MUST be FAST (works on 8GB RAM)
3. The system MUST MINIMIZE hallucinations
4. The system MUST be modular but NOT fragmented
5. The system MUST actually complete tasks (not just output actions)

---

## REMOVE / AVOID

* Over-abstraction
* Dead modules
* Unused patterns
* Overly complex planners
* YAML-based skill systems (for now)
* Anything that is not directly used in the agent loop

---

## REQUIRED ARCHITECTURE (STRICT)

Design the system using ONLY these core modules:

1. main.rs → entry point and loop controller
2. brain.rs → LLM interaction (Gemma GGUF)
3. perception.rs → DOM → structured UI conversion
4. browser.rs → Chrome/CDP control
5. executor.rs → executes actions
6. types.rs → shared structs

---

## AGENT LOOP (MANDATORY)

Implement EXACTLY this loop:

loop {
1. Capture DOM
2. Convert DOM → structured UI (roles like input, button, link)
3. Send goal + UI → LLM
4. Get EXACTLY ONE action (JSON)
5. Validate action (selector must exist)
6. Execute action
7. Store result in history
8. Check if goal is achieved
}

---

## ACTION SYSTEM (STRICT)

Only support these actions:

* navigate(url)
* click(selector)
* type(selector, text)
* scroll(direction)
* wait(ms)

Do NOT add more actions.

---

## LLM CONTRACT (VERY IMPORTANT)

The LLM must:

* Output ONLY JSON
* Output EXACTLY ONE action
* NEVER explain anything
* NEVER hallucinate selectors

Example:

{"action":"click","selector":"#search_btn"}

---

## PERCEPTION SYSTEM

Convert raw DOM into structured UI:

Example output:

[
{ "role": "search_input", "selector": "#search_box" },
{ "role": "button", "text": "Search", "selector": "#search_btn" }
]

DO NOT pass raw DOM to the LLM.

---

## VALIDATION LAYER

Before executing:

* Ensure selector exists in UI
* Prevent repeated actions
* Prevent infinite loops

---

## OUTPUT REQUIREMENTS

You must:

1. Rewrite the architecture cleanly
2. Provide full Rust code for all modules
3. Ensure it compiles and runs
4. Keep code minimal and readable
5. Add comments explaining logic
6. Remove all unnecessary complexity

---

## PERFORMANCE

* Must run locally using GGUF (llama.cpp or similar)
* Must work on 8GB RAM
* Avoid heavy allocations
* Keep inference tight

---

## FINAL GOAL

The system should successfully complete:

"Go to Google and search for dog images"

WITHOUT breaking, looping, or hallucinating.

---

Think like an engineer building a reliable system — not an experimental AI researcher.

Focus on correctness, simplicity, and execution.
