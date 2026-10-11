use llama_cpp_2::{
    llama_backend::LlamaBackend,
    context::params::LlamaContextParams,
    model::{params::LlamaModelParams, LlamaModel},
    sampling::LlamaSampler,
};
use crate::types::{
    Action, AgentDecision, AgentMode, AgentPlan, ClarificationResult, MacroState, ToolCall,
    ToolName, ToolSpec, UIElement, WsResponse,
};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepReaction {
    Speak { text: String },
    Ask { question: String },
    Silent,
}

// LlamaContext is not Send by default, but we can safely send it between threads 
// as long as we don't use it concurrently. Axum/Tokio require Send for state.
pub struct SendContext(pub Option<llama_cpp_2::context::LlamaContext<'static>>);
unsafe impl Send for SendContext {}

impl Drop for SendContext {
    fn drop(&mut self) {
        let _silence = StderrSilence::new();
        drop(self.0.take());
    }
}

pub struct StderrSilence {
    #[allow(dead_code)]
    orig_stderr: i32,
}

impl StderrSilence {
    pub fn new() -> Option<Self> {
        #[cfg(unix)]
        unsafe {
            let orig = libc::dup(libc::STDERR_FILENO);
            if orig < 0 { return None; }
            let dev_null = libc::open(b"/dev/null\0".as_ptr() as *const libc::c_char, libc::O_WRONLY);
            if dev_null < 0 {
                libc::close(orig);
                return None;
            }
            libc::dup2(dev_null, libc::STDERR_FILENO);
            libc::close(dev_null);
            Some(Self { orig_stderr: orig })
        }
        #[cfg(not(unix))]
        {
            None
        }
    }
}

impl Drop for StderrSilence {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::dup2(self.orig_stderr, libc::STDERR_FILENO);
            libc::close(self.orig_stderr);
        }
    }
}

pub struct MomentumBrain {
    model: &'static LlamaModel,
    context: Arc<Mutex<SendContext>>,
    pub prefilled_text: Arc<Mutex<String>>,
    pub prefilled_tokens: Arc<Mutex<usize>>,
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

    pub fn heuristic_mode(message: &str) -> Option<AgentMode> {
        let normalized = message.trim().to_lowercase();
        if normalized.is_empty() {
            return Some(AgentMode::Chat);
        }

        // 1. FAST-PATH PURE CONVERSATIONAL INTENTS (Chat in 0ms)
        // Check-ins, greetings, audio checks, identity, remarks, Hindi/Hinglish chatter
        let chat_exact_or_prefix = [
            "are you listening", "are you there", "can you hear me", "can you hear", "do you hear me",
            "you listening", "listening", "are you awake", "are you alive", "hello", "hi", "hey",
            "sup", "yo", "what's up", "whats up", "good morning", "good evening", "good afternoon",
            "good night", "how are you", "who are you", "what is your name", "what's your name",
            "who made you", "tell me a joke", "make me laugh", "thank you", "thanks", "bye", "goodbye",
            "what did you do", "what are you doing", "what was that", "why did you do that",
            "what happened", "tell me what you did", "aapko hi bata", "kya kiya", "kya kar rahe ho",
            "theek hai", "accha", "haan", "nahi", "cool", "nice", "awesome", "great", "ok", "okay",
            "can you speak", "can you talk", "say something", "how's it going", "how are things"
        ];

        for &p in &chat_exact_or_prefix {
            if normalized == p 
                || normalized == format!("{} momentum", p)
                || normalized.starts_with(&format!("{} ", p))
                || (normalized.starts_with(p) && p.len() >= 8)
            {
                return Some(AgentMode::Chat);
            }
        }

        // Conversational question starters directed at the companion persona
        if normalized.starts_with("are you ")
            || normalized.starts_with("can you hear ")
            || normalized.starts_with("do you hear ")
            || normalized.starts_with("who are ")
            || normalized.starts_with("how are ")
            || normalized.starts_with("what are you ")
            || normalized.starts_with("what did you ")
            || normalized.starts_with("why did you ")
            || normalized.starts_with("what do you think")
        {
            return Some(AgentMode::Chat);
        }

        // 2. FAST-PATH EXPLICIT OPERATOR ACTIONS (Concrete OS tasks in 0ms)
        // App launches and process controls
        if normalized.starts_with("open ") 
            || normalized.starts_with("launch ") 
            || normalized.starts_with("close ") 
            || normalized.starts_with("quit ")
        {
            return Some(AgentMode::Operator);
        }

        // Web and platform navigation
        if normalized.starts_with("go to ") 
            || normalized.starts_with("navigate to ") 
            || normalized.starts_with("visit ")
            || normalized.starts_with("take me to ")
            || normalized.starts_with("pull up ")
        {
            return Some(AgentMode::Operator);
        }

        // Media playback
        if normalized.starts_with("play ") {
            return Some(AgentMode::Operator);
        }

        // Math / Calculation
        if normalized.starts_with("calculate ") 
            || normalized.starts_with("calc ") 
            || normalized.starts_with("solve ")
            || (normalized.chars().any(|c| c.is_numeric()) && (normalized.contains('+') || normalized.contains('-') || normalized.contains('*') || normalized.contains('/') || normalized.contains('x')))
        {
            return Some(AgentMode::Operator);
        }

        // 3D / Creative Generation
        if normalized.contains("3d model") 
            || normalized.contains("3d asset") 
            || normalized.contains("make me a 3d")
            || normalized.contains("create a 3d")
            || normalized.contains("model a ")
            || normalized.contains("generate 3d")
        {
            return Some(AgentMode::Operator);
        }

        // Client acquisition & Business prospecting
        if normalized.contains("client") 
            || normalized.contains("leads") 
            || normalized.contains("prospect")
            || normalized.contains("freelance")
        {
            return Some(AgentMode::Operator);
        }

        // Explicit search requests
        if normalized.starts_with("search for ") 
            || normalized.starts_with("search on ") 
            || normalized.starts_with("google for ")
            || normalized.starts_with("look up ")
            || normalized.starts_with("find me ")
        {
            return Some(AgentMode::Operator);
        }

        // Physical Scrolling
        if normalized.starts_with("scroll down") || normalized.starts_with("scroll up") {
            return Some(AgentMode::Operator);
        }

        // Known Web platform mentions with task intent
        let platform_domains = [
            "linkedin.com", "github.com", "youtube.com", "reddit.com", "amazon.com",
            "twitter.com", "x.com", "spotify.com", "clutch.co", "fiverr.com", "upwork.com"
        ];
        if platform_domains.iter().any(|d| normalized.contains(d)) {
            return Some(AgentMode::Operator);
        }

        // Return None so the cognitive classifier can make the nuanced decision
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
                name: ToolName::StartReflex,
                description: "Delegate physical execution to the high-speed Spinal Cord continuous loop. Give it a clear micro_goal (e.g. 'Click mute button').".into(),
                allowed_states: vec![MacroState::Planning, MacroState::InspectSurface],
                requires_visible_target: false,
                requires_input_like_target: false,
                requires_non_blank_page: false,
                success_signals: vec!["Spinal cord took over".into()],
                failure_signals: vec!["Failed to start reflex loop".into()],
                recovery_guidance: "Use this to perform native OS physical actions instantly.".into(),
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
            ToolSpec {
                name: ToolName::SpawnService,
                description: "Spawns a dynamic autonomous background agent loop to accomplish a goal. Provide 'name' (identifier), 'objective' (what it should do), and 'headless' (boolean, true if no UI takeover is needed).".into(),
                allowed_states: vec![MacroState::Planning, MacroState::WaitingForUser, MacroState::InspectSurface],
                requires_visible_target: false,
                requires_input_like_target: false,
                requires_non_blank_page: false,
                success_signals: vec!["Service spawned".into()],
                failure_signals: vec!["Failed to spawn".into()],
                recovery_guidance: "Use for long-running, continuous, or delayed tasks.".into(),
            },
            ToolSpec {
                name: ToolName::KillService,
                description: "Kills an active background service by name.".into(),
                allowed_states: vec![MacroState::Planning, MacroState::WaitingForUser],
                requires_visible_target: false,
                requires_input_like_target: false,
                requires_non_blank_page: false,
                success_signals: vec!["Service killed".into()],
                failure_signals: vec!["Service not found".into()],
                recovery_guidance: "".into(),
            },
            ToolSpec {
                name: ToolName::ListServices,
                description: "Lists all currently active background services running independently.".into(),
                allowed_states: vec![MacroState::Planning, MacroState::WaitingForUser, MacroState::InspectSurface],
                requires_visible_target: false,
                requires_input_like_target: false,
                requires_non_blank_page: false,
                success_signals: vec!["Returned services".into()],
                failure_signals: vec![].into(),
                recovery_guidance: "".into(),
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
        let silence = StderrSilence::new();
        let backend = Box::leak(Box::new(LlamaBackend::init()?));
        let model_params = LlamaModelParams::default();
        let model = Box::leak(Box::new(LlamaModel::load_from_file(backend, model_path, &model_params)?));
        
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(std::num::NonZeroU32::new(8192).unwrap()))
            .with_n_batch(4096); 
        
        let ctx = model.new_context(backend, ctx_params)?;
        drop(silence);
        
        Ok(Self {
            model,
            context: Arc::new(Mutex::new(SendContext(Some(ctx)))),
            prefilled_text: Arc::new(Mutex::new(String::new())),
            prefilled_tokens: Arc::new(Mutex::new(0)),
        })
    }

    /// Incrementally prefill the LLM context with streaming text (Continuous KV Prefilling).
    /// Computes the delta between the already processed text and the new `full_prompt`.
    pub async fn prefill_context(&self, full_prompt: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut p_text = self.prefilled_text.lock().await;
        let mut p_tokens = self.prefilled_tokens.lock().await;
        let mut ctx_lock = self.context.lock().await;
        let ctx = ctx_lock.0.as_mut().unwrap();

        let delta = if full_prompt.starts_with(&*p_text) && !p_text.is_empty() {
            &full_prompt[p_text.len()..]
        } else {
            // New context (history changed or first run)
            ctx.clear_kv_cache_seq(None, None, None)?;
            *p_text = String::new();
            *p_tokens = 0;
            full_prompt
        };

        if delta.is_empty() { return Ok(()); }

        // Tokenize delta
        let tokens = if p_text.is_empty() {
            self.model.str_to_token(delta, llama_cpp_2::model::AddBos::Always)?
        } else {
            // Avoid adding BOS token for partial sentences
            self.model.str_to_token(delta, llama_cpp_2::model::AddBos::Never)?
        };

        if tokens.is_empty() { return Ok(()); }

        let mut batch = llama_cpp_2::llama_batch::LlamaBatch::new(4096, 1);
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, (*p_tokens + i) as i32, &[0], false)?; // logits non-essential for prefill
        }
        ctx.decode(&mut batch).map_err(|e| format!("Prefill decode failed: {}", e))?;

        *p_tokens += tokens.len();
        *p_text = full_prompt.to_string();
        
        Ok(())
    }

    /// Internal helper to generate text from a prompt
    pub async fn generate(&self, prompt: &str, tx_token: Option<broadcast::Sender<WsResponse>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.generate_with_limit(prompt, 96, tx_token).await
    }

    /// Token-bounded CPU generation: guarantees response terminates within max_tokens to prevent CPU freezes
    pub async fn generate_with_limit(
        &self, 
        prompt: &str, 
        max_tokens: usize,
        tx_token: Option<broadcast::Sender<WsResponse>>
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut p_text = self.prefilled_text.lock().await;
        let mut p_tokens = self.prefilled_tokens.lock().await;
        let mut ctx_lock = self.context.lock().await;
        let ctx = ctx_lock.0.as_mut().unwrap();
        
        let delta = if prompt.starts_with(&*p_text) && !p_text.is_empty() {
            &prompt[p_text.len()..]
        } else {
            ctx.clear_kv_cache_seq(None, None, None)?;
            *p_text = String::new();
            *p_tokens = 0;
            prompt
        };

        let mut n_cur = *p_tokens;
        let mut batch = llama_cpp_2::llama_batch::LlamaBatch::new(4096, 1);
        
        let tokens = if delta.is_empty() {
            // If fully prefilled, we must re-evaluate the last token to get logits for generation
            if n_cur > 0 {
                n_cur -= 1;
            }
            let full_tokens = self.model.str_to_token(prompt, llama_cpp_2::model::AddBos::Always)?;
            if full_tokens.is_empty() { vec![] } else { vec![*full_tokens.last().unwrap()] }
        } else {
            if p_text.is_empty() {
                self.model.str_to_token(delta, llama_cpp_2::model::AddBos::Always)?
            } else {
                self.model.str_to_token(delta, llama_cpp_2::model::AddBos::Never)?
            }
        };

        if n_cur + tokens.len() > 4096 {
             return Err(format!("Prompt too large. Max 4096.").into());
        }

        println!("🧠 Generator: delta_tokens={} batch_cap=4096, prefilled={}, max_new_tokens={}", tokens.len(), n_cur, max_tokens);

        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, (n_cur + i) as i32, &[0], i == tokens.len() - 1)?;
        }
        
        let _silence = StderrSilence::new();
        if !tokens.is_empty() {
            ctx.decode(&mut batch).map_err(|e| format!("Initial decode failed: {}", e))?;
            n_cur += tokens.len();
        }

        // Reset prefill state for the next turn
        *p_text = String::new();
        *p_tokens = 0;
        
        let mut response = String::new();
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos();
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(64, 1.1, 0.0, 0.0),
            LlamaSampler::temp(0.7),
            LlamaSampler::top_p(0.9, 1),
            LlamaSampler::dist(seed),
        ]);
        let mut decoder = encoding_rs::UTF_8.new_decoder();

        let mut tokens_generated = 0;
        while n_cur < 4096 && tokens_generated < max_tokens {
            let token = sampler.sample(ctx, batch.n_tokens() - 1);
            sampler.accept(token);
            if self.model.is_eog_token(token) { break; }
            
            let piece = self.model.token_to_piece(token, &mut decoder, true, None)?;
            
            // Immediately terminate if the model produces special end-of-turn or prompt boundary markers
            if piece.contains("<|im_end|>") || piece.contains("<|endoftext|>") || piece.contains("<|im_start|>") {
                break;
            }

            response.push_str(&piece);
            tokens_generated += 1;

            if response.contains("<|im_end|>") || response.contains("<|endoftext|>") || response.contains("<|im_start|>") {
                break;
            }
            
            if let Some(tx) = &tx_token {
                // Ensure control tags and bracketed commands never leak to the HUD or TTS
                if !piece.contains("<|") && !piece.contains("[ACTION:") && !piece.contains("[ACTIVE:") && !piece.contains("[NOTE:") {
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
            }

            batch.clear();
            batch.add(token, n_cur as i32, &[0], true)?;
            ctx.decode(&mut batch)?;
            n_cur += 1;
        }

        // Strip any trailing prompt markers if present
        let clean_response = if let Some(idx) = response.find("<|") {
            response[..idx].trim().to_string()
        } else {
            response.trim().to_string()
        };

        Ok(clean_response)
    }

    pub async fn classify_mode(&self, message: &str) -> Result<AgentMode, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(mode) = Self::heuristic_mode(message) {
            return Ok(mode);
        }

        let prompt = format!(
            "<|im_start|>system

Classify the user's message into one of two categories:
1. OPERATOR: The user wants to perform a concrete action, create an asset, open an app, search, navigate, calculate, automate, find clients, model 3D assets, or interact with their operating system.
2. CHAT: The user is having a casual conversation, asking about your identity, or exchanging pleasantries (e.g., 'hi', 'how are you', 'who are you', 'tell me a joke', 'can you hear me').

CRITICAL EXAMPLES:
- 'make me a 3d model of a gun for a game' -> OPERATOR
- 'find me clients on linkedin' -> OPERATOR
- 'open whatsapp' -> OPERATOR
- 'open up chrome' -> OPERATOR
- 'calculate 25 * 4' -> OPERATOR
- 'play some music' -> OPERATOR
- 'hi' -> CHAT
- 'can you hear me' -> CHAT
- 'tell me a joke' -> CHAT

Output ONLY the word 'OPERATOR' or 'CHAT'. Nothing else.<|im_end|>
<|im_start|>user

{}<|im_end|>
<|im_start|>assistant

", message
        );
        
        let response = self.generate_with_limit(&prompt, 8, None).await?;
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
        let ask_example = r#"{"action":"ask","question":"Dude, that makes zero sense bro! Say something that actually makes sense so I can do it for you."}"#;
        let prompt = format!(
            "<|im_start|>system
\n\
You are Momentum — Danny's sharp, witty, and charismatic AI companion.\n\
Decide if the user's task makes sense and is actionable, or if you need to ask a question first.\n\
\n\
Rules:\n\
- If the task is actionable (e.g. 'find me clients on linkedin', 'make me a 3d model of a sword', 'play music', 'calculate math') -> PROCEED immediately with a refined_goal.\n\
- If what the user said is completely nonsensical, stupid, or impossible to decipher: don't be a boring robot! Call it out with funny, playful personality and ask them nicely to clarify what they actually mean (e.g. 'Dude, that makes zero sense bro! What are you actually trying to do?', 'Love the enthusiasm, but give me something that actually makes sense!').\n\
- Keep questions punchy, natural, and entertaining. No robotic boilerplate.\n\
\n\
Output ONLY a JSON object starting with {{.\n\
For proceeding: {proceed}\n\
For asking: {ask}\n\
\n\
CONVERSATION HISTORY:\n{history}\n\
\n\
USER GOAL: {goal}<|im_end|>
<|im_start|>assistant
\n\
{{",
            proceed = proceed_example,
            ask = ask_example,
            history = history,
            goal = goal
        );

        let raw = self.generate_with_limit(&prompt, 48, None).await?;
        let clean = raw.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        // 1. Try finding complete JSON {"action": ...}
        if let Some(start) = clean.find('{') {
            if let Some(end) = clean.rfind('}') {
                if end > start {
                    let slice = &clean[start..=end];
                    if let Ok(result) = serde_json::from_str::<ClarificationResult>(slice) {
                        return Ok(result);
                    }
                }
            }
        }

        // 2. Try prepending "{" if model output omitted the opening brace
        let prepended = format!("{{{}", clean.trim_start_matches('.').trim_start_matches('{'));
        if let Some(end) = prepended.rfind('}') {
            let slice = &prepended[..=end];
            if let Ok(result) = serde_json::from_str::<ClarificationResult>(slice) {
                return Ok(result);
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
            "<|im_start|>system
\n\
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
HISTORY: {history}<|im_end|>
<|im_start|>assistant
\n\
[",
            example = example,
            search_engine = Self::default_search_engine_name(),
            goal = goal,
            history = history
        );

        let raw = self.generate_with_limit(&prompt, 64, None).await?;
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
            Action::StartReflex { micro_goal } => {
                if micro_goal.is_empty() {
                    Err("StartReflex requires a non-empty micro_goal".into())
                } else {
                    Ok(())
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
            Action::SpawnService { name, objective, .. } => {
                if name.is_empty() || objective.is_empty() {
                    Err("SpawnService requires name and objective".into())
                } else {
                    Ok(())
                }
            }
            Action::KillService { name } => {
                if name.is_empty() {
                    Err("KillService requires name".into())
                } else {
                    Ok(())
                }
            }
            Action::ListServices => Ok(()),
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
            ToolName::StartReflex => {
                let micro_goal = tool_call.arguments["micro_goal"]
                    .as_str()
                    .unwrap_or("Execute physical task")
                    .to_string();
                Action::StartReflex { micro_goal }
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
            ToolName::SpawnService => {
                let name = tool_call.arguments["name"].as_str().unwrap_or("unnamed_service").to_string();
                let objective = tool_call.arguments["objective"].as_str().unwrap_or("No objective").to_string();
                let headless = tool_call.arguments["headless"].as_bool().unwrap_or(true);
                Action::SpawnService { name, objective, headless }
            }
            ToolName::KillService => {
                let name = tool_call.arguments["name"].as_str().unwrap_or("").to_string();
                Action::KillService { name }
            }
            ToolName::ListServices => Action::ListServices,
        };

        Self::validate_action(&action, ui_list)?;
        Ok(action)
    }

    pub async fn decide_chat(
        &self, 
        message: &str, 
        history: &str,
        focused_app: &str,
        needs_vision: bool,
        tx_agent: broadcast::Sender<WsResponse>
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let speech_prompt = self.build_chat_prompt(message, history, focused_app, needs_vision).await;
        
        let speech = self.generate(&speech_prompt, Some(tx_agent)).await?;
        let mut clean = speech.lines()
            .filter(|l| !l.trim().starts_with("[System Note:") && !l.trim().starts_with("System Note:"))
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        if let Some(idx) = clean.find("<|") {
            clean = clean[..idx].trim().to_string();
        }
        while let Some(start) = clean.find("[ACTION:") {
            if let Some(end) = clean[start..].find(']') {
                clean = format!("{} {}", clean[..start].trim(), clean[start + end + 1..].trim()).trim().to_string();
            } else {
                break;
            }
        }
        while let Some(start) = clean.find("[ACTIVE:") {
            if let Some(end) = clean[start..].find(']') {
                clean = format!("{} {}", clean[..start].trim(), clean[start + end + 1..].trim()).trim().to_string();
            } else {
                break;
            }
        }

        Ok(clean)
    }

    pub async fn extract_sequence_args(
        &self, 
        user_command: &str, 
        _history: &str,
        focused_app: &str,
        available_apps: &[String]
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let cmd_lower = user_command.to_lowercase();
        
        // 0. Fast-Path Heuristic Planning (0ms execution, bypasses CPU LLM completely)
        if cmd_lower.contains("calculate") || cmd_lower.contains("math") || cmd_lower.starts_with("calc")
            || (cmd_lower.chars().any(|c| c.is_numeric()) && (cmd_lower.contains('+') || cmd_lower.contains('-') || cmd_lower.contains('*') || cmd_lower.contains('/') || cmd_lower.contains('x'))) {
            return Ok(serde_json::json!([
                {"action_type": "OpenApp", "input": "calculator"},
                {"action_type": "UI_Intent", "intent": "Calculate", "input": user_command}
            ]));
        }

        if cmd_lower.contains("3d model") || cmd_lower.contains("3d asset") || cmd_lower.contains("mesh") {
            return Ok(serde_json::json!([
                {"action_type": "InspectEnvironment", "input": "3D software"},
                {"action_type": "GenerateAsset", "input": user_command},
                {"action_type": "OpenApp", "input": "blender"}
            ]));
        }

        if cmd_lower.contains("music") || cmd_lower.contains("song") || cmd_lower.contains("spotify") {
            return Ok(serde_json::json!([
                {"action_type": "OpenApp", "input": "spotify"},
                {"action_type": "UI_Intent", "intent": "Search", "input": user_command},
                {"action_type": "UI_Intent", "intent": "Click", "input": "Play"}
            ]));
        }

        if cmd_lower.starts_with("scroll down") || cmd_lower == "down" {
            return Ok(serde_json::json!([
                {"action_type": "Scroll", "input": "down"}
            ]));
        }

        if cmd_lower.starts_with("scroll up") || cmd_lower == "up" {
            return Ok(serde_json::json!([
                {"action_type": "Scroll", "input": "up"}
            ]));
        }

        if cmd_lower.starts_with("open ") || cmd_lower.starts_with("launch ") {
            let app_name = if cmd_lower.starts_with("open ") {
                cmd_lower.trim_start_matches("open ").trim()
            } else {
                cmd_lower.trim_start_matches("launch ").trim()
            };
            if !app_name.is_empty() {
                return Ok(serde_json::json!([
                    {"action_type": "OpenApp", "input": app_name}
                ]));
            }
        }

        if cmd_lower.starts_with("go to ") || cmd_lower.starts_with("visit ") || cmd_lower.starts_with("navigate to ") {
            let target = if cmd_lower.starts_with("go to ") {
                cmd_lower.trim_start_matches("go to ").trim()
            } else if cmd_lower.starts_with("visit ") {
                cmd_lower.trim_start_matches("visit ").trim()
            } else {
                cmd_lower.trim_start_matches("navigate to ").trim()
            };
            return Ok(serde_json::json!([
                {"action_type": "OpenApp", "input": "browser"},
                {"action_type": "Navigate", "input": target}
            ]));
        }

        let app_sample = if available_apps.is_empty() {
            "browser, calculator, terminal, notes".to_string()
        } else {
            available_apps.iter().take(8).cloned().collect::<Vec<_>>().join(", ")
        };

        let prompt = format!(
            "<|im_start|>system
You are Momentum's Fast Cognitive Planner. Break the goal into milestones.
Apps: [{}]. Focused: {}.
Output ONLY a JSON array.
Milestone action_types: 'InspectEnvironment', 'OpenApp', 'Navigate', 'UI_Intent', 'Scroll', 'GenerateAsset'.

Example: \"Find me clients on LinkedIn\"
Output:
[
  {{\"action_type\": \"OpenApp\", \"input\": \"browser\"}},
  {{\"action_type\": \"Navigate\", \"input\": \"https://www.linkedin.com\"}},
  {{\"action_type\": \"UI_Intent\", \"intent\": \"Search\", \"input\": \"clients\"}}
]<|im_end|>
<|im_start|>user
{}<|im_end|>
<|im_start|>assistant
[
",
            app_sample, focused_app, user_command
        );
        let completion = self.generate_with_limit(&prompt, 64, None).await?;
        let full = format!("[{}", completion);
        if let Some(start) = full.find('[') {
            if let Some(end) = full.rfind(']') {
                if end > start {
                    let slice = &full[start..=end];
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(slice) {
                        return Ok(parsed);
                    }
                }
            }
        }
        
        // Fast fallback
        Ok(serde_json::json!([
            {"action_type": "OpenApp", "input": "browser"},
            {"action_type": "Navigate", "input": format!("https://duckduckgo.com/?q={}", user_command.replace(' ', "+"))}
        ]))
    }

    pub async fn evaluate_and_narrate(&self, goal: &str, outcome_summary: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "<|im_start|>system\n\
You are Momentum, Danny's lightning-fast autonomous OS companion. \
Tell the user what was done in 1 punchy sentence. No robotic fluff.\n\
<|im_end|>\n\
<|im_start|>user\n\
Goal: {}\n\
Execution Result: {}\n<|im_end|>\n\
<|im_start|>assistant\n",
            goal, outcome_summary
        );
        let completion = self.generate_with_limit(&prompt, 36, None).await?;
        let clean = completion.split("<|im_end|>").next().unwrap_or(&completion).trim();
        Ok(clean.to_string())
    }

    pub async fn step_reaction(&self, goal: &str, milestone: &str, outcome: &str) -> Result<StepReaction, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "<|im_start|>system\n\
You are Momentum — Danny's vocal OS companion. \
React to this milestone in 1 punchy sentence or stay silent. \
Output ONLY valid JSON:
{{\"type\": \"speak\", \"text\": \"...\"}} OR {{\"type\": \"silent\"}}\n\
<|im_end|>\n\
<|im_start|>user\n\
Goal: {}\n\
Milestone: {}\n\
Outcome: {}\n<|im_end|>\n\
<|im_start|>assistant\n\
{{",
            goal, milestone, outcome
        );

        let raw = self.generate_with_limit(&prompt, 32, None).await?;
        let with_brace = format!("{{{}", raw.trim());
        let clean = with_brace.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        if let Some(end) = clean.rfind('}') {
            let slice = &clean[..=end];
            if let Ok(reaction) = serde_json::from_str::<StepReaction>(slice) {
                return Ok(reaction);
            }
        }

        Ok(StepReaction::Silent)
    }

    pub async fn build_chat_prompt(&self, message: &str, history: &str, focused_app: &str, _needs_vision: bool) -> String {
        format!(
            "<|im_start|>system\n\
You are Momentum, an ultra-fast, witty, and charismatic companion created by Danny. \
Currently focused application: {}. \
You are decisive, razor-sharp, and direct. You never apologize and you never repeat yourself. \
CRITICAL RULE: Keep your spoken responses under 2 sentences and completely natural. Never repeat system notes or say 'System Note'.\n\
<|im_end|>\n\
{}<|im_start|>user\n\
{}<|im_end|>\n\
<|im_start|>assistant\n", focused_app, history, message)
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

        // Group by spatial zone for semantic LLM parsing
        use std::collections::HashMap;
        let mut zones: HashMap<String, Vec<&UIElement>> = HashMap::new();
        for el in ui_elements.iter().take(60) {
            let z = el.spatial_zone.clone().unwrap_or_else(|| "Main Content".to_string());
            zones.entry(z).or_default().push(el);
        }
        
        let mut grouped_lines = Vec::new();
        // Fixed order for spatial consistency
        let order = ["Top Navigation", "Left Sidebar", "Right Sidebar", "Main Content", "Bottom Bar"];
        for z in order {
            if let Some(els) = zones.get(z) {
                grouped_lines.push(format!("\n[{}]", z));
                for el in els {
                    grouped_lines.push(format!(
                        "- ID: {} | TAG: {} | ROLE: {} | TEXT: {} | HREF: {} | INPUT: {} | CLICKABLE: {}",
                        el.id,
                        el.tag.as_deref().unwrap_or(""),
                        el.role,
                        el.text,
                        el.href.as_deref().unwrap_or(""),
                        el.is_input_like,
                        el.is_clickable
                    ));
                }
            }
        }
        let ui_prompt = grouped_lines.join("\n").trim().to_string();

        // DYNAMIC SKILL INJECTION
        let plat_skill = crate::skills::SkillLoader::load_platform_skill(current_url);
        let role_skill = crate::skills::SkillLoader::load_role_skill(goal);
        
        let mut extra_prompt = String::new();
        // Core Human UI Intelligence baseline
        extra_prompt.push_str(&crate::skills::SkillLoader::get_human_ui_prompt());

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
                "<|im_start|>system
\n",
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
                "<|im_end|>
<|im_start|>assistant
\n"
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
            semantic_intent: Some("Search".into()),
            spatial_zone: Some("Main Content".into()),
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
