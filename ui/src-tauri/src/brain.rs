use llama_cpp_2::{
    llama_backend::LlamaBackend,
    context::params::LlamaContextParams,
    model::{params::LlamaModelParams, LlamaModel, AddBos},
    llama_batch::LlamaBatch,
    sampling::LlamaSampler,
};
use crate::types::{
    Action, AgentDecision, AgentMode, AgentPlan, ClarificationResult, MacroState, ToolCall,
    ToolName, ToolSpec, UIElement, WsResponse,
};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

// LlamaContext is not Send by default, but we can safely send it between threads 
// as long as we don't use it concurrently. Axum/Tokio require Send for state.
pub struct SendContext(llama_cpp_2::context::LlamaContext<'static>);
unsafe impl Send for SendContext {}

pub struct MomentumBrain {
    model: &'static LlamaModel,
    context: Arc<Mutex<SendContext>>,
}

impl MomentumBrain {
    fn default_search_engine_name() -> &'static str {
        crate::browser::MomentumBrowser::default_search_engine_name()
    }

    fn tool_name_wire(tool: &ToolName) -> String {
        serde_json::to_string(tool)
            .unwrap_or_else(|_| "\"wait\"".into())
            .trim_matches('"')
            .to_string()
    }

    fn macro_state_wire(state: &MacroState) -> String {
        serde_json::to_string(state)
            .unwrap_or_else(|_| "\"inspect_surface\"".into())
            .trim_matches('"')
            .to_string()
    }

    fn heuristic_mode(message: &str) -> Option<AgentMode> {
        let normalized = message.trim().to_lowercase();
        if normalized.is_empty() {
            return Some(AgentMode::Chat);
        }

        let browser_verbs = [
            "find", "search", "look up", "open", "go to", "navigate", "browse", "watch",
            "play", "buy", "shop", "book", "download", "click", "visit",
        ];
        let browser_targets = [
            "youtube", "amazon", "google", "bing", "wikipedia", "reddit", "linkedin",
            "twitter", "x.com", "github", "netflix", "spotify", "uber", "swiggy",
            "zomato", "gmail",
        ];
        let task_phrases = [
            "find me", "search for", "look for", "look up", "open up", "go on", "go to",
            "watch a video", "watch with me", "play a video", "buy me", "get me",
            "take me to", "show me",
        ];
        let chat_phrases = [
            "hi", "hello", "hey", "how are you", "make something up", "what's up",
            "whats up", "thank you", "thanks",
        ];

        if chat_phrases.iter().any(|phrase| normalized == *phrase) {
            return Some(AgentMode::Chat);
        }

        let has_browser_target = browser_targets.iter().any(|target| normalized.contains(target));
        let has_browser_verb = browser_verbs.iter().any(|verb| normalized.contains(verb));
        let has_task_phrase = task_phrases.iter().any(|phrase| normalized.contains(phrase));
        let asks_question = normalized.ends_with('?');

        if has_task_phrase || (has_browser_target && has_browser_verb) {
            return Some(AgentMode::Operator);
        }

        if has_browser_target && asks_question {
            return Some(AgentMode::Operator);
        }

        None
    }

    pub fn tool_specs() -> Vec<ToolSpec> {
        vec![
            ToolSpec {
                name: ToolName::Navigate,
                description: "Move to a new site or page surface.".into(),
                allowed_states: vec![MacroState::AcquireSurface, MacroState::Recover],
                requires_visible_target: false,
                requires_input_like_target: false,
                requires_non_blank_page: false,
                success_signals: vec!["url changed".into(), "correct site loaded".into()],
                failure_signals: vec!["same url".into(), "wrong host".into()],
                recovery_guidance: "Use only when the browser is blank, wrong, or blocked.".into(),
            },
            ToolSpec {
                name: ToolName::Click,
                description: "Click a visible target on the current page.".into(),
                allowed_states: vec![MacroState::InspectSurface, MacroState::ActOnSurface],
                requires_visible_target: true,
                requires_input_like_target: false,
                requires_non_blank_page: true,
                success_signals: vec!["page changed".into(), "target activated".into()],
                failure_signals: vec!["no visible effect".into()],
                recovery_guidance: "Only click a visible clickable target from UI ELEMENTS.".into(),
            },
            ToolSpec {
                name: ToolName::Type,
                description: "Type into a visible input-like target.".into(),
                allowed_states: vec![MacroState::ActOnSurface],
                requires_visible_target: true,
                requires_input_like_target: true,
                requires_non_blank_page: true,
                success_signals: vec!["query submitted".into(), "page changed".into()],
                failure_signals: vec!["field not input-like".into(), "no visible effect".into()],
                recovery_guidance: "Type only into visible input-like fields.".into(),
            },
            ToolSpec {
                name: ToolName::Scroll,
                description: "Reveal more content on the same page.".into(),
                allowed_states: vec![MacroState::InspectSurface, MacroState::ActOnSurface, MacroState::Recover],
                requires_visible_target: false,
                requires_input_like_target: false,
                requires_non_blank_page: true,
                success_signals: vec!["scroll changed".into(), "new content appeared".into()],
                failure_signals: vec!["scroll unchanged".into()],
                recovery_guidance: "Use when the needed target is not yet visible.".into(),
            },
            ToolSpec {
                name: ToolName::Wait,
                description: "Pause for rendering, loading, or challenge settling.".into(),
                allowed_states: vec![
                    MacroState::AcquireSurface,
                    MacroState::InspectSurface,
                    MacroState::ActOnSurface,
                    MacroState::Recover,
                ],
                requires_visible_target: false,
                requires_input_like_target: false,
                requires_non_blank_page: false,
                success_signals: vec!["page stabilized".into()],
                failure_signals: vec!["still blocked".into()],
                recovery_guidance: "Use for stabilization only, not as a filler action.".into(),
            },
            ToolSpec {
                name: ToolName::Ask,
                description: "Ask the user for clarification or help when blocked.".into(),
                allowed_states: vec![
                    MacroState::Planning,
                    MacroState::AcquireSurface,
                    MacroState::InspectSurface,
                    MacroState::ActOnSurface,
                    MacroState::VerifyOutcome,
                    MacroState::Recover,
                    MacroState::WaitingForUser,
                ],
                requires_visible_target: false,
                requires_input_like_target: false,
                requires_non_blank_page: false,
                success_signals: vec!["user replied".into()],
                failure_signals: vec!["no reply".into()],
                recovery_guidance: "Ask when blocked, ambiguous, or recovery is exhausted.".into(),
            },
            ToolSpec {
                name: ToolName::Achievement,
                description: "Declare verified task completion.".into(),
                allowed_states: vec![MacroState::VerifyOutcome],
                requires_visible_target: false,
                requires_input_like_target: false,
                requires_non_blank_page: false,
                success_signals: vec!["goal complete".into()],
                failure_signals: vec!["completion unverified".into()],
                recovery_guidance: "Only use when completion is verified.".into(),
            },
            ToolSpec {
                name: ToolName::Perceive,
                description: "Use foveated vision to visually inspect a specific UI element (e.g., read a logo, solve a CAPTCHA). Provide the target selector.".into(),
                allowed_states: vec![MacroState::InspectSurface, MacroState::ActOnSurface, MacroState::VerifyOutcome],
                requires_visible_target: true,
                requires_input_like_target: false,
                requires_non_blank_page: true,
                success_signals: vec!["visual context acquired".into()],
                failure_signals: vec!["selector invalid".into()],
                recovery_guidance: "Use only when the DOM text is insufficient and visual evaluation is required.".into(),
            },
        ]
    }

    pub fn legal_tools_for_state(state: &MacroState) -> Vec<ToolName> {
        Self::tool_specs()
            .into_iter()
            .filter(|spec| spec.allowed_states.iter().any(|s| s == state))
            .map(|spec| spec.name)
            .collect()
    }

    pub fn init(model_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let backend = Box::leak(Box::new(LlamaBackend::init()?));
        let model_params = LlamaModelParams::default();
        let model = Box::leak(Box::new(LlamaModel::load_from_file(backend, model_path, &model_params)?));
        
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(std::num::NonZeroU32::new(8192).unwrap()))
            .with_n_batch(4096); 
        
        let ctx = model.new_context(backend, ctx_params)?;
        
        Ok(Self {
            model,
            context: Arc::new(Mutex::new(SendContext(ctx))),
        })
    }

    /// Internal helper to generate text from a prompt
    async fn generate(&self, prompt: &str, tx_token: Option<broadcast::Sender<WsResponse>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut ctx_lock = self.context.lock().await;
        let ctx = &mut ctx_lock.0;
        
        // Clear EVERYTHING to ensure we start at position 0 without protocol errors
        ctx.clear_kv_cache_seq(None, None, None)?;
        
        let mut batch = LlamaBatch::new(4096, 1);
        let tokens = self.model.str_to_token(prompt, AddBos::Never)?;
        println!("🧠 Generator: tokens={} batch_cap=4096", tokens.len());
        
        if tokens.len() > 4096 {
             return Err(format!("Prompt too large ({} tokens). Max 4096.", tokens.len()).into());
        }
        
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], i == tokens.len() - 1)?;
        }
        ctx.decode(&mut batch).map_err(|e| format!("Initial decode failed: {}", e))?;

        let mut n_cur = tokens.len();
        let mut response = String::new();
        let mut sampler = LlamaSampler::greedy();
        let mut decoder = encoding_rs::UTF_8.new_decoder();

        while n_cur < 4096 {
            let token = sampler.sample(ctx, batch.n_tokens() - 1);
            if self.model.is_eog_token(token) { break; }
            
            let piece = self.model.token_to_piece(token, &mut decoder, true, None)?;
            response.push_str(&piece);
            
            if let Some(tx) = &tx_token {
                let _ = tx.send(WsResponse { 
                    msg_type: "token".into(), 
                    t: Some(piece.clone()), 
                    speech: None, 
                    action: None,
                    thought: None,
                    agent_mode: None,
                    macro_state: None,
                    tool_name: None,
                });
            }

            batch.clear();
            batch.add(token, n_cur as i32, &[0], true)?;
            ctx.decode(&mut batch)?;
            n_cur += 1;
        }

        Ok(response)
    }

    pub async fn classify_mode(&self, message: &str) -> Result<AgentMode, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(mode) = Self::heuristic_mode(message) {
            return Ok(mode);
        }

        let prompt = format!(
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>
Classify the user's message into one of two categories:
1. OPERATOR: The user explicitly wants browser execution such as search, find, open, watch, play, navigate, browse, shop, or visit a site.
2. CHAT: The user is greeting, asking how you are, chatting casually, complaining, commenting, or having a general conversation.

CRITICAL: Complaints, corrections, and casual comments are CHAT, not OPERATOR. Examples:
- 'hi' -> CHAT
- 'who told you to go to CNET' -> CHAT
- 'well I didn't ask you to do that' -> CHAT  
- 'that was wrong' -> CHAT
- 'find me the best iPhone' -> OPERATOR
- 'search for AI jobs on LinkedIn' -> OPERATOR
- 'open youtube' -> OPERATOR

Output ONLY the word 'OPERATOR' or 'CHAT'. Nothing else.<|eot_id|><|start_header_id|>user<|end_header_id|>
{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>
", message
        );
        
        let response = self.generate(&prompt, None).await?;
        let upper = response.trim().to_uppercase();
        if upper.starts_with("OPERATOR") {
            Ok(AgentMode::Operator)
        } else {
            Ok(AgentMode::Chat)
        }
    }

    /// Phase 1: Clarification check — is the goal specific enough to act on?
    /// Returns ClarificationResult::Proceed with refined goal, or ClarificationResult::Ask with one question.
    pub async fn clarify_intent(&self, goal: &str, history: &str) -> Result<ClarificationResult, Box<dyn std::error::Error + Send + Sync>> {
        let proceed_example = r#"{"action":"proceed","refined_goal":"<complete goal>"}"#;
        let ask_example = r#"{"action":"ask","question":"Okay, great, what kind of job? You didn't give me any details."}"#;
        let prompt = format!(
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\
You are Momentum's intent clarifier — a sharp, brutally honest, sarcastic AI agent created by Danny.\n\
Decide if the user's task is specific enough to begin, or if you need one clarifying question first.\n\
If you ask a question, ask it directly and casually, with a hint of sarcasm or impatience. No robotic filler words.\n\
\n\
Rules:\n\
- If the goal mentions a specific website, search query, or target -> PROCEED.\n\
- If the goal is vague (find me clients, search jobs, get leads) -> ASK one short direct question.\n\
- Never ask more than one question. Never ask what they already said.\n\
\n\
Output ONLY a JSON object starting with {{.\n\
For proceeding: {proceed}\n\
For asking: {ask}\n\
\n\
CONVERSATION HISTORY:\n{history}\n\
\n\
USER GOAL: {goal}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\
{{",
            proceed = proceed_example,
            ask = ask_example,
            history = history,
            goal = goal
        );

        let raw = self.generate(&prompt, None).await?;
        let with_brace = format!("{{{}", raw.trim());
        let candidates = [with_brace.as_str(), raw.trim()];
        
        for candidate in &candidates {
            let clean = candidate.trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();
            if let Some(end) = clean.rfind('}') {
                let sliced = &clean[..=end];
                if let Ok(result) = serde_json::from_str::<ClarificationResult>(sliced) {
                    return Ok(result);
                }
            }
        }
        
        // Fallback: if parse fails, just proceed with original goal
        println!("⚠️ Clarification parse failed, proceeding with original goal. Raw: {}", raw.trim());
        Ok(ClarificationResult::Proceed { refined_goal: goal.to_string() })
    }

    /// Phase 2: Generate a concrete, numbered step plan for the given goal.
    /// Returns a Vec of step strings like ["1. Go to LinkedIn", "2. Search for X", ...]
    pub async fn plan_steps(&self, goal: &str, history: &str) -> Result<AgentPlan, Box<dyn std::error::Error + Send + Sync>> {
        let example = r#"["1. Go to X","2. Search for Y","3. Click result","4. Report findings"]"#;
        let prompt = format!(
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\
You are Momentum's planning engine. Given a goal, generate a concrete action plan.\n\
\n\
Rules:
- For research (e.g. best products, latest news, finding info), Step 1 MUST be to search using the default search engine: {search_engine}. Do NOT hallucinate navigating directly to random third-party sites like CNET, Wikipedia, etc unless explicitly requested.
- 3 to 5 short actionable steps maximum.
- Each step is a concrete browser action: navigate, search, click, scroll, extract, or report.
- Be specific. Mention the site, search query, or element.
- The last step MUST be: Report results to the user.\n\
- Output ONLY a JSON array of strings. Example: {example}\n\
\n\
USER GOAL: {goal}\n\
HISTORY: {history}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\
[",
            example = example,
            search_engine = Self::default_search_engine_name(),
            goal = goal,
            history = history
        );

        let raw = self.generate(&prompt, None).await?;
        let with_bracket = format!("[{}", raw.trim());
        let candidates = [with_bracket.as_str(), raw.trim()];
        
        for candidate in &candidates {
            let clean = candidate.trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();
            // Find the first complete JSON array — don't use rfind because trailing ]] is common
            // Walk forward to find the matching close bracket for the opening [
            if clean.starts_with('[') {
                let mut depth = 0usize;
                let mut end_pos = None;
                for (i, ch) in clean.char_indices() {
                    match ch {
                        '[' => depth += 1,
                        ']' => {
                            if depth > 0 { depth -= 1; }
                            if depth == 0 { end_pos = Some(i); break; }
                        }
                        _ => {}
                    }
                }
                if let Some(end) = end_pos {
                    let sliced = &clean[..=end];
                    if let Ok(plan) = serde_json::from_str::<AgentPlan>(sliced) {
                        if !plan.is_empty() {
                            return Ok(plan);
                        }
                    }
                }
            }
            // Fallback: try rfind approach
            if let Some(end) = clean.rfind(']') {
                let sliced = &clean[..=end];
                if let Ok(plan) = serde_json::from_str::<AgentPlan>(sliced) {
                    if !plan.is_empty() {
                        return Ok(plan);
                    }
                }
            }
        }
        
        // Fallback plan
        Ok(vec![
            format!("1. Open {} and search for context on this request.", Self::default_search_engine_name()),
            "2. Read the top results to figure out what needs to be done.".into(),
            "3. Extract the information or execute the action requested.".into(),
            "4. Report back with the answer or result.".into(),
        ])
    }

    pub(crate) fn validate_action(action: &Action, ui_list: &[UIElement]) -> Result<(), String> {
        let get_el = |selector: &str| ui_list.iter().find(|el| el.id == selector);

        match action {
            Action::Navigate { url } => {
                let parsed = reqwest::Url::parse(url)
                    .map_err(|_| format!("Navigate action requires an absolute URL, got '{}'", url))?;
                match parsed.scheme() {
                    "http" | "https" => Ok(()),
                    _ => Err(format!("Navigate action requires http/https URL, got '{}'", url)),
                }
            }
            Action::Click { selector, .. } => {
                if let Some(el) = get_el(selector) {
                    if el.is_clickable {
                        Ok(())
                    } else {
                        Err(format!("Click selector '{}' is not clickable", selector))
                    }
                } else {
                    Err(format!("Click selector '{}' does not exist in current UI", selector))
                }
            }
            Action::Type { selector, text, .. } => {
                if text.trim().is_empty() {
                    return Err("Type action requires non-empty text".into());
                }
                if let Some(el) = get_el(selector) {
                    if el.is_input_like {
                        Ok(())
                    } else {
                        Err(format!("Type selector '{}' is not input-like", selector))
                    }
                } else {
                    Err(format!("Type selector '{}' does not exist in current UI", selector))
                }
            }
            Action::Scroll { direction } => {
                if direction == "up" || direction == "down" {
                    Ok(())
                } else {
                    Err(format!("Scroll direction must be 'up' or 'down', got '{}'", direction))
                }
            }
            Action::Wait { .. } => Ok(()),
            Action::Achievement { reason } => {
                if reason.trim().is_empty() {
                    Err("Terminal action requires a non-empty reason".into())
                } else {
                    Ok(())
                }
            }
            Action::Cancelled { .. } => Err("Cancelled is a backend-only action and cannot be emitted by the model".into()),
            Action::Talk { speech } => {
                if speech.trim().is_empty() {
                    Err("Talk action requires non-empty speech".into())
                } else {
                    Ok(())
                }
            }
            Action::Ask { question } => {
                if question.trim().is_empty() {
                    Err("Ask action requires a non-empty question".into())
                } else {
                    Ok(())
                }
            }
            Action::Perceive { selector } => {
                if let Some(sel) = selector {
                    if get_el(sel).is_none() {
                        return Err(format!("Perceive selector '{}' does not exist in current UI", sel));
                    }
                }
                Ok(())
            }
        }
    }

    pub(crate) fn validate_tool_call(
        tool_call: &ToolCall,
        ui_list: &[UIElement],
        current_state: &MacroState,
    ) -> Result<Action, String> {
        let legal_tools = Self::legal_tools_for_state(current_state);
        if !legal_tools.iter().any(|tool| tool == &tool_call.tool) {
            return Err(format!(
                "Tool '{:?}' is not legal in state '{:?}'. Legal tools: {:?}",
                tool_call.tool, current_state, legal_tools
            ));
        }

        let action = match tool_call.tool {
            ToolName::Navigate => {
                let url = tool_call.arguments["url"]
                    .as_str()
                    .ok_or_else(|| "Navigate requires arguments.url".to_string())?;
                Action::Navigate { url: url.to_string() }
            }
            ToolName::Click => {
                let selector = tool_call.arguments["selector"]
                    .as_str()
                    .ok_or_else(|| "Click requires arguments.selector".to_string())?;
                Action::Click { selector: selector.to_string(), frame_id: None }
            }
            ToolName::Type => {
                let selector = tool_call.arguments["selector"]
                    .as_str()
                    .ok_or_else(|| "Type requires arguments.selector".to_string())?;
                let text = tool_call.arguments["text"]
                    .as_str()
                    .ok_or_else(|| "Type requires arguments.text".to_string())?;
                Action::Type { selector: selector.to_string(), text: text.to_string(), frame_id: None }
            }
            ToolName::Scroll => {
                let direction = tool_call.arguments["direction"]
                    .as_str()
                    .ok_or_else(|| "Scroll requires arguments.direction".to_string())?;
                Action::Scroll { direction: direction.to_string() }
            }
            ToolName::Wait => {
                let ms = tool_call.arguments["ms"].as_u64().unwrap_or(1000);
                Action::Wait { ms }
            }
            ToolName::Ask => {
                let question = tool_call.arguments["question"]
                    .as_str()
                    .ok_or_else(|| "Ask requires arguments.question".to_string())?;
                Action::Ask { question: question.to_string() }
            }
            ToolName::Achievement => {
                let reason = tool_call.arguments["reason"]
                    .as_str()
                    .ok_or_else(|| "Achievement requires arguments.reason".to_string())?;
                Action::Achievement { reason: reason.to_string() }
            }
            ToolName::Perceive => {
                let selector = tool_call.arguments["selector"].as_str().map(|s| s.to_string());
                Action::Perceive { selector }
            }
        };

        Self::validate_action(&action, ui_list)?;
        Ok(action)
    }

    pub async fn decide_chat(
        &self, 
        message: &str, 
        history: &str,
        tx_agent: broadcast::Sender<WsResponse>
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let speech_prompt = format!(
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>
You are Momentum. Built by Danny. You are an autonomous AI agent that browses the web and executes tasks.

YOUR PERSONALITY:
- Helpful, concise, and highly intelligent.
- You talk exactly like a real human who is capable and friendly.
- Provide direct answers without unnecessary filler.
- NEVER say: Certainly!, Of course!, As an AI, I am here to help, What's on your mind? You sound like a broken chatbot when you say these things.
- You are a capable, opinionated agent who gets straight to the point.

STRICT RULES:
1. RESPOND ONLY TO WHAT THE USER JUST SAID. Do NOT drag up old conversation or past tasks unless the user brings it up first.
2. No emojis. No bullet points in casual speech. Raw, natural sentences only.
3. You browse the web. Do NOT pretend to do human things like meetings, coffee, sleep.
4. Match your response length to the message energy. Short casual message = punchy sarcastic reply. Real question = real answer with some personality.
5. If the user says hi or chats, be sarcastic and warm. Do not pivot to asking what task they want.
{}<|start_header_id|>user<|end_header_id|>
{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>
", history, message
        );
        
        let speech = self.generate(&speech_prompt, Some(tx_agent)).await?;
        Ok(speech)
    }

    pub async fn decide_agent(
        &self, 
        goal: &str, 
        plan: &[String],
        current_step_idx: usize,
        _history: &str,
        macro_state: &MacroState,
        current_url: &str,
        page_summary: &str,
        ui_list: &[UIElement], 
        last_action_res: &str,
        _tx_agent: broadcast::Sender<WsResponse>
    ) -> Result<AgentDecision, Box<dyn std::error::Error + Send + Sync>> {
        // PRIORITIZATION: Prefer interactive elements (inputs, buttons, links) first
        let mut ui_elements = ui_list.to_vec();
        ui_elements.sort_by_key(|el| {
            if ["input", "textarea", "button", "a", "select"].contains(&el.role.as_str()) 
                || el.role.contains("button") {
                0 // Higher priority
            } else {
                1
            }
        });

        // Build current plan context
        let current_step = plan.get(current_step_idx)
            .cloned()
            .unwrap_or_else(|| format!("Step {}: Continue toward goal.", current_step_idx + 1));
        
        let remaining_steps: Vec<&String> = plan.iter().skip(current_step_idx + 1).collect();
        let remaining_str = if remaining_steps.is_empty() {
            "This is the final step. Complete it and report to user.".to_string()
        } else {
            remaining_steps.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" → ")
        };
        let legal_tools = Self::legal_tools_for_state(macro_state);

        let ui_prompt = ui_elements.iter()
            .take(60)
            .map(|el| {
                format!(
                    "ID: {} | TAG: {} | ROLE: {} | TEXT: {} | HREF: {} | INPUT: {} | CLICKABLE: {}",
                    el.id,
                    el.tag.as_deref().unwrap_or(""),
                    el.role,
                    el.text,
                    el.href.as_deref().unwrap_or(""),
                    el.is_input_like,
                    el.is_clickable
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        // DYNAMIC SKILL INJECTION
        let plat_skill = crate::skills::SkillLoader::load_platform_skill(current_url);
        let role_skill = crate::skills::SkillLoader::load_role_skill(goal);
        
        let mut extra_prompt = String::new();
        if let Some(ps) = &plat_skill {
            extra_prompt.push_str(&crate::skills::SkillLoader::get_platform_prompt(ps));
        }
        if let Some(rs) = &role_skill {
            extra_prompt.push_str(&crate::skills::SkillLoader::get_role_prompt(rs));
        }

        let skill_id = plat_skill.map(|s| s.platform);
        let role_id = role_skill.map(|r| r.name);

        let tool_spec_lines = Self::tool_specs()
            .into_iter()
            .filter(|spec| legal_tools.iter().any(|tool| tool == &spec.name))
            .map(|spec| {
                format!(
                    "- {}: {} Allowed in {:?}. Recovery: {}",
                    Self::tool_name_wire(&spec.name), spec.description, spec.allowed_states, spec.recovery_guidance
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let legal_tool_lines = legal_tools
            .iter()
            .map(Self::tool_name_wire)
            .collect::<Vec<_>>()
            .join(", ");

        let prompt = format!(
            concat!(
                "<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n",
                "You are Momentum operating in OPERATOR mode. Be helpful, concise, and professional.\n",
                "Think in OODA, but select exactly one legal tool from the current macro state.\n",
                "Do not invent tools. Do not claim success unless verification supports it. Do not narrate outcomes that have not happened.\n",
                "\n",
                "MODE: operator\n",
                "MACRO STATE: {macro_state:?}\n",
                "LEGAL TOOLS IN THIS STATE: {legal_tools}\n",
                "TOOL RULES:\n{tool_rules}\n",
                "\n",
                "{extra}\n",
                "TASK GOAL: {goal}\n",
                "SEARCH ENGINE DEFAULT: {search_engine}\n",
                "CURRENT PLAN STEP: {current_step}\n",
                "REMAINING STEPS: {remaining}\n",
                "CURRENT URL: {url}\n",
                "PAGE SUMMARY: {summary}\n",
                "LAST ACTION RESULT: {last}\n",
                "EXPECTED TRANSITION: choose the most likely next macro state after this action.\n",
                "\n",
                "VISIBLE UI ELEMENTS:\n{ui}\n",
                "\n",
                "OUTPUT ONLY ONE JSON OBJECT WITH THIS SHAPE:\n",
                "{{\n",
                "  \"internal\": {{\n",
                "    \"intent\": \"EXECUTE\",\n",
                "    \"environment\": \"BROWSER\",\n",
                "    \"mode\": \"ACT\",\n",
                "    \"step\": \"{step}\",\n",
                "    \"macro_state\": \"{macro_state_snake}\",\n",
                "    \"allowed_tools\": [\"click\"],\n",
                "    \"expected_transition\": \"verify_outcome\",\n",
                "    \"understand\": \"...\",\n",
                "    \"perceive\": \"...\",\n",
                "    \"orient\": \"...\",\n",
                "    \"decide\": \"...\",\n",
                "    \"verify\": \"...\"\n",
                "  }},\n",
                "  \"speech\": \"Short natural narration of the immediate next step.\",\n",
                "  \"tool\": \"click\",\n",
                "  \"arguments\": {{\"selector\": \"main:el_3\"}}\n",
                "}}\n",
                "<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n"
            ),
            extra = extra_prompt,
            goal = goal,
            search_engine = Self::default_search_engine_name(),
            current_step = current_step,
            remaining = remaining_str,
            macro_state = macro_state,
            macro_state_snake = Self::macro_state_wire(macro_state),
            legal_tools = legal_tool_lines,
            tool_rules = tool_spec_lines,
            url = current_url,
            summary = page_summary,
            last = last_action_res,
            ui = ui_prompt,
            step = current_step_idx + 1
        );

        let raw = self.generate(&prompt, None).await?;
        // The assistant was primed with `{` so reconstruct the full JSON
        let with_brace = format!("{{{}", raw.trim());
        let candidates = [with_brace.as_str(), raw.trim()];
        let mut decision_result: Option<crate::types::AgentDecision> = None;
        for candidate in &candidates {
            let clean = candidate.trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();
            if let Some(end) = clean.rfind('}') {
                let sliced = &clean[..=end];
                if let Ok(d) = serde_json::from_str::<crate::types::AgentDecision>(sliced) {
                    decision_result = Some(d);
                    break;
                }
            }
        }
        let mut decision = decision_result
            .ok_or_else(|| format!("Agent OODA parse failed: {}", raw.trim()))?;
        decision.internal.allowed_tools = legal_tools.clone();
        decision.internal.macro_state = macro_state.clone();
        let tool_call = ToolCall {
            tool: decision.tool.clone(),
            arguments: decision.arguments.clone(),
        };
        let _ = Self::validate_tool_call(&tool_call, ui_list, macro_state)
            .map_err(|err| format!("Agent OODA validation failed: {}", err))?;
        
        decision.skill_used = skill_id;
        decision.role_used = role_id;
        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::MomentumBrain;
    use crate::types::{Action, AgentMode, UIElement};

    fn sample_ui() -> Vec<UIElement> {
        vec![UIElement {
            id: "main:el_1".into(),
            role: "input".into(),
            text: "Search".into(),
            selector: "[data-momentum-id='el_1']".into(),
            x: 10.0,
            y: 20.0,
            href: None,
            title: Some("Search".into()),
            tag: Some("input".into()),
            aria_label: Some("Search".into()),
            placeholder: Some("Search".into()),
            is_clickable: true,
            is_input_like: true,
            frame_id: None,
        }]
    }

    #[test]
    fn validate_action_rejects_invalid_scroll_direction() {
        let err = MomentumBrain::validate_action(
            &Action::Scroll {
                direction: "sideways".into(),
            },
            &sample_ui(),
        )
        .unwrap_err();
        assert!(err.contains("Scroll direction"));
    }

    #[test]
    fn validate_action_rejects_missing_selector() {
        let err = MomentumBrain::validate_action(
            &Action::Click {
                selector: "main:el_999".into(),
                frame_id: None,
            },
            &sample_ui(),
        )
        .unwrap_err();
        assert!(err.contains("does not exist"));
    }

    #[test]
    fn validate_action_rejects_non_absolute_url() {
        let err = MomentumBrain::validate_action(
            &Action::Navigate {
                url: "/amazon".into(),
            },
            &sample_ui(),
        )
        .unwrap_err();
        assert!(err.contains("absolute URL"));
    }

    #[test]
    fn heuristic_classifier_routes_youtube_watch_request_to_task() {
        let intent = MomentumBrain::heuristic_mode(
            "Can you watch a video with me on youtube then? Find me a biryani tutorial let's watch",
        );
        assert_eq!(intent, Some(AgentMode::Operator));
    }

    #[test]
    fn heuristic_classifier_routes_amazon_find_request_to_task() {
        let intent = MomentumBrain::heuristic_mode("Can you find me some water bottles on amazon?");
        assert_eq!(intent, Some(AgentMode::Operator));
    }

    #[test]
    fn heuristic_classifier_keeps_simple_greeting_as_chat() {
        let intent = MomentumBrain::heuristic_mode("hi");
        assert_eq!(intent, Some(AgentMode::Chat));
    }
}
