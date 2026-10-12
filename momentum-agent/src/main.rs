mod types;
mod browser;
mod perception;
mod brain;
mod executor;
mod server;
mod skills;
mod scraper;
mod vision_stream;
mod reflex;
mod intent;
mod services;
mod driver;
mod knowledge;
mod researcher;
mod authority;

use crate::browser::MomentumBrowser;
use crate::brain::{MomentumBrain, StepReaction};
use crate::executor::{ActionExecutor, ActionResult};
use crate::perception::{perceive, perceive_summary};
use crate::server::{start_server, ServerState};
use crate::types::{
    Action, AgentMode, ClientMessage, ClarificationResult, ControlCommand, MacroState, ToolCall,
    ToolName, TransitionResult, WsResponse,
};
use std::{env, sync::Arc};
use dotenv::dotenv;
use tokio::sync::{broadcast, mpsc, Mutex};

async fn cancel_active_task(
    active_task: &Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    ask_notifier: &Arc<Mutex<Option<mpsc::Sender<String>>>>,
    tx_agent: &broadcast::Sender<WsResponse>,
) -> bool {
    let handle = {
        let mut active_guard = active_task.lock().await;
        active_guard.take()
    };

    if let Some(handle) = handle {
        handle.abort();
        let mut notifier_guard = ask_notifier.lock().await;
        *notifier_guard = None;
        let _ = tx_agent.send(WsResponse {
            msg_type: "action".into(),
            t: None,
            speech: Some("Stopped.".into()),
            action: Some(Action::Cancelled {
                reason: "Stopped by user".into(),
            }),
            thought: None,
            agent_mode: Some(AgentMode::Operator),
            macro_state: Some(MacroState::Cancelled),
            tool_name: None,
        });
        true
    } else {
        false
    }
}

#[cfg(target_os = "windows")]
mod win_util {
    use std::ffi::c_void;

    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> *mut c_void;
        fn GetWindowThreadProcessId(hwnd: *mut c_void, lpdw_process_id: *mut u32) -> u32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(dw_desired_access: u32, b_inherit_handle: i32, dw_process_id: u32) -> *mut c_void;
        fn CloseHandle(h_object: *mut c_void) -> i32;
        fn QueryFullProcessImageNameA(h_process: *mut c_void, dw_flags: u32, lp_exe_name: *mut u8, lpdw_size: *mut u32) -> i32;
    }

    pub fn get_foreground_process_name() -> String {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return "Desktop".to_string();
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 {
                return "Desktop".to_string();
            }
            let handle = OpenProcess(0x1000, 0, pid);
            if handle.is_null() {
                return "Desktop".to_string();
            }
            let mut buf = [0u8; 512];
            let mut size: u32 = buf.len() as u32;
            let success = QueryFullProcessImageNameA(handle, 0, buf.as_mut_ptr(), &mut size);
            CloseHandle(handle);
            if success != 0 && size > 0 {
                let full_path = String::from_utf8_lossy(&buf[..size as usize]).to_string();
                if let Some(file_name) = std::path::Path::new(&full_path).file_stem() {
                    return file_name.to_string_lossy().to_string();
                }
            }
            "Desktop".to_string()
        }
    }
}

pub fn get_frontmost_app() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("sh")
            .arg("-c")
            .arg("lsappinfo info -only name $(lsappinfo front)")
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(pos) = text.find("\"LSDisplayName\"=\"") {
                let rest = &text[pos + 17..];
                if let Some(end) = rest.find('"') {
                    let app = rest[..end].trim();
                    if !app.is_empty() {
                        return app.to_string();
                    }
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        let name = win_util::get_foreground_process_name();
        if !name.is_empty() {
            return name;
        }
    }
    "Desktop".to_string()
}


fn format_clean_chat_history(history: &[crate::types::ChatMessage], current_message: &str) -> String {
    let mut msgs: Vec<(String, String)> = Vec::new();
    for msg in history.iter().filter(|m| !m.text.trim().is_empty()) {
        let role = if msg.role == "agent" || msg.role == "assistant" {
            "assistant".to_string()
        } else {
            "user".to_string()
        };
        msgs.push((role, msg.text.trim().to_string()));
    }

    // Drop trailing message if it is an exact duplicate of current user input (UI sends it inside history)
    if let Some(last) = msgs.last() {
        if last.0 == "user" && last.1 == current_message.trim() {
            msgs.pop();
        }
    }

    // Consolidate consecutive messages from the same role (e.g. streaming sentence chunks)
    let mut consolidated: Vec<(String, String)> = Vec::new();
    for (role, text) in msgs {
        if let Some(prev) = consolidated.last_mut() {
            if prev.0 == role {
                prev.1.push(' ');
                prev.1.push_str(&text);
                continue;
            }
        }
        consolidated.push((role, text));
    }

    // Keep the last 10 real dialogue turns
    let keep_turns = 10;
    let skip = if consolidated.len() > keep_turns { consolidated.len() - keep_turns } else { 0 };

    let mut out = String::new();
    for (role, text) in consolidated.into_iter().skip(skip) {
        out.push_str(&format!("<|im_start|>{}\n{}<|im_end|>\n", role, text));
    }
    out
}

fn infer_initial_macro_state(goal: &str) -> MacroState {
    let _ = goal;
    MacroState::AcquireSurface
}

fn default_search_engine_name() -> &'static str {
    MomentumBrowser::default_search_engine_name()
}

fn evaluate_transition(
    current_state: &MacroState,
    action: &Action,
    result: &ActionResult,
) -> TransitionResult {
    if matches!(action, Action::Ask { .. }) {
        return TransitionResult {
            next_state: MacroState::WaitingForUser,
            reason: "Waiting for user reply".into(),
        };
    }
    if matches!(action, Action::Achievement { .. }) {
        return TransitionResult {
            next_state: MacroState::Done,
            reason: "Goal completed".into(),
        };
    }
    if !result.verification_passed {
        return TransitionResult {
            next_state: MacroState::Recover,
            reason: format!("Verification failed: {:?}", result.failure_class),
        };
    }

    let next_state = match current_state {
        MacroState::Planning => MacroState::AcquireSurface,
        MacroState::AcquireSurface => MacroState::InspectSurface,
        MacroState::InspectSurface => match action {
            Action::Click { .. } => MacroState::ActOnSurface,
            Action::Scroll { .. } | Action::Wait { .. } => MacroState::InspectSurface,
            _ => MacroState::ActOnSurface,
        },
        MacroState::ActOnSurface => MacroState::VerifyOutcome,
        MacroState::VerifyOutcome => MacroState::InspectSurface,
        MacroState::Recover => MacroState::InspectSurface,
        MacroState::WaitingForUser => MacroState::ActOnSurface,
        MacroState::Done => MacroState::Done,
        MacroState::Cancelled => MacroState::Cancelled,
    };

    TransitionResult {
        next_state,
        reason: result.summary.clone(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    println!("🚀 Momentum Agent starting...");

    let model_path = env::var("MOMENTUM_MODEL_PATH")
        .ok()
        .filter(|p| std::path::Path::new(p).exists())
        .unwrap_or_else(|| {
            let candidates = [
                "/Users/sanjeevn/Downloads/Momentum AI/momentum-engine-tiny-0.5b.gguf",
                "../momentum-engine-tiny-0.5b.gguf",
                "momentum-engine-tiny-0.5b.gguf",
                "models/momentum-engine-tiny-0.5b.gguf",
                "/Users/sanjeevn/Downloads/Momentum AI/momentum-engine-3b.gguf",
                "../momentum-engine-3b.gguf",
                "momentum-engine-3b.gguf",
            ];
            for c in candidates {
                if std::path::Path::new(c).exists() {
                    return c.to_string();
                }
            }
            "momentum-engine-tiny-0.5b.gguf".to_string()
        });
    println!("🧠 Loaded Model: {}", model_path);
    
    // Channels & Shared State
    // 512-capacity broadcast: prevents dropping clarification/plan messages when prior chat tokens fill the buffer
    let (tx_chat, mut rx_chat) = broadcast::channel::<ClientMessage>(64);
    let (tx_agent, _) = broadcast::channel(512);
    let (tx_screencast, _) = broadcast::channel(10);
    
    let server_state = Arc::new(ServerState {
        tx_chat: tx_chat.clone(),
        tx_screencast: tx_screencast.clone(),
        tx_agent: tx_agent.clone(),
    });

    // 2. Spawn Server IMMEDIATELY
    let server_state_clone = server_state.clone();
    tokio::spawn(async move {
        start_server(server_state_clone).await;
    });

    // 3. Init Core Modules (Lazy)
    let browser_mu = Arc::new(Mutex::new(MomentumBrowser::init()));
    let brain = Arc::new(MomentumBrain::init(&model_path)?);
    let os_name = std::env::consts::OS;
    let driver = Arc::new(crate::driver::ActionDriver::new(os_name));
    
    // Init Vision Stream Optic Nerve (Shared Memory)
    // We must keep the stream instance alive so the memory map doesn't get dropped!
    let _vision_stream = crate::vision_stream::VisionStream::new().ok();
    if let Some(stream) = &_vision_stream {
        stream.start();
    } else {
        println!("⚠️ Failed to initialize Vision Stream (SHM). Agent will run blind.");
    }

    // 4. Spawn Screencast Loop (Only if browser launched)
    let tx_sc = tx_screencast.clone();
    let b_mu_sc = browser_mu.clone();
    tokio::spawn(async move {
        loop {
            let b = b_mu_sc.lock().await;
            if b.page.is_some() {
                if let Ok(frame) = b.capture_screenshot().await {
                    let wrapped_frame = format!("{{\"type\": \"frame\", \"image\": \"{}\"}}", frame);
                    let _ = tx_sc.send(wrapped_frame);
                }
            }
            drop(b);
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    });

    // 5. Main Loop: Reactive & Autonomous
    let active_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    let ask_notifier: Arc<Mutex<Option<tokio::sync::mpsc::Sender<String>>>> = Arc::new(Mutex::new(None));
    let service_manager = Arc::new(Mutex::new(crate::services::ServiceManager::new()));
    let kg_store = Arc::new(Mutex::new(crate::knowledge::LocalKnowledgeGraph::load()));
    let researcher = Arc::new(crate::researcher::AutonomousWebResearcher::new(kg_store.clone()));

    println!("✨ Momentum is ready! (Chrome only launches for web tasks)");

    loop {
        tokio::select! {
            result = rx_chat.recv() => {
                let input = match result {
                    Ok(i) => i,
                    Err(_) => continue,
                };

                let (history, message) = match input {
                    ClientMessage::Control { command: ControlCommand::Stop, .. } => {
                        if cancel_active_task(&active_task, &ask_notifier, &tx_agent).await {
                            println!("🛑 Active agent task cancelled by user.");
                        }
                        continue;
                    }
                    ClientMessage::InterimInput { history, text } => {
                        let history_str = format_clean_chat_history(&history, &text);
                        let brain_prefill = brain.clone();
                        let text_clone = text.clone();
                        tokio::spawn(async move {
                            let active_app = get_frontmost_app();
                            let prefill_prompt = brain_prefill.build_chat_prompt(&text_clone, &history_str, &active_app, false).await;
                            if let Err(e) = brain_prefill.prefill_context(&prefill_prompt).await {
                                eprintln!("⚠️ Prefill error: {}", e);
                            }
                        });
                        continue; // Do not proceed to intent classification or generation
                    }
                    ClientMessage::UserInput { history, message } => (history, message),
                };

                // FIRST: Check if the agent is waiting for an answer from the user
                let mut notifier_guard = ask_notifier.lock().await;
                if let Some(tx) = notifier_guard.take() {
                    let _ = tx.send(message.clone()).await;
                    continue;
                }
                drop(notifier_guard);

                println!("💬 New input: {}", message);
                
                let history_str = format_clean_chat_history(&history, &message);

                // --- INTENT CLASSIFICATION ---
                let agent_mode = match MomentumBrain::heuristic_mode(&message) {
                    Some(m) => m,
                    None => brain.classify_mode(&message).await.unwrap_or(AgentMode::Chat)
                };

                match agent_mode {
                    AgentMode::Chat => {
                        println!("🤖 Personality Mode (Chat)");
                        let tx_chat_resp = tx_agent.clone();
                        let brain_chat = brain.clone();
                        let driver_chat = driver.clone();
                        let msg_lower = message.to_lowercase();
                        let b_mu_chat = browser_mu.clone();
                        
                        tokio::spawn(async move {
                            let active_app = get_frontmost_app();
                            let needs_vision = msg_lower.contains("screen") || msg_lower.contains("see") || msg_lower.contains("look");
                            match brain_chat.decide_chat(&message, &history_str, &active_app, needs_vision, tx_chat_resp.clone()).await {
                                Ok(mut speech) => {
                                    // Strip any bracketed control tags or simulated actions from speech
                                    while let Some(start_idx) = speech.find("[ACTION:") {
                                        if let Some(end_idx) = speech[start_idx..].find(']') {
                                            speech = format!("{} {}", speech[..start_idx].trim(), speech[start_idx + end_idx + 1..].trim()).trim().to_string();
                                        } else {
                                            break;
                                        }
                                    }
                                    while let Some(start_idx) = speech.find("[ACTIVE:") {
                                        if let Some(end_idx) = speech[start_idx..].find(']') {
                                            speech = format!("{} {}", speech[..start_idx].trim(), speech[start_idx + end_idx + 1..].trim()).trim().to_string();
                                        } else {
                                            break;
                                        }
                                    }
                                    if let Some(idx) = speech.find("<|") {
                                        speech = speech[..idx].trim().to_string();
                                    }

                                    let mut action_obj = Action::Talk { speech: speech.clone() };
                                    
                                    // Extract and run [ACTION: START_REFLEX(...)]
                                    if let Some(start_idx) = speech.find("[ACTION: START_REFLEX(") {
                                        if let Some(end_idx) = speech[start_idx..].find(")]") {
                                            let micro_goal = speech[start_idx + 22..start_idx + end_idx].to_string();
                                            println!("⚡ Spinal Cord Triggered: {}", micro_goal);
                                            action_obj = Action::StartReflex { micro_goal: micro_goal.clone() };
                                            
                                            let reflex_brain = brain_chat.clone();
                                            tokio::spawn(async move {
                                                crate::reflex::start_spinal_cord(micro_goal, reflex_brain).await;
                                            });
                                            // Strip it from speech
                                            let mut clean = speech[..start_idx].to_string();
                                            clean.push_str(&speech[start_idx + end_idx + 2..]);
                                            speech = clean.trim().to_string();
                                        }
                                    }

                                    println!("🗣️ Momentum: {}", speech);
                                    let _ = tx_chat_resp.send(WsResponse {
                                        msg_type: "action".into(),
                                        t: None,
                                        speech: Some(speech.clone()),
                                        action: Some(action_obj),
                                        thought: None,
                                        agent_mode: Some(AgentMode::Chat),
                                        macro_state: None,
                                        tool_name: None,
                                    });

                                    // Run the vision engine sequentially if requested, avoiding GPU memory contention
                                    if needs_vision {
                                        println!("👁️ Chat implies visual context. Fetching background Vision Engine task...");
                                        let vision_res = crate::perception::perceive_vision(None).await;
                                        
                                        let mut b_guard = b_mu_chat.lock().await;
                                        let ui_context = if let Some(page) = b_guard.page.as_ref() {
                                            let ui_list = crate::perception::perceive(page).await.unwrap_or_default();
                                            let url = page.url().await.unwrap_or_default().unwrap_or_default();
                                            let mut raw_summary = format!("URL: {}\nUI ELEMENTS:\n", url);
                                            for el in ui_list.iter().take(40) {
                                                raw_summary.push_str(&format!("- [{}] {}\n", el.role, el.text));
                                            }
                                            raw_summary
                                        } else {
                                            "No browser window open.".into()
                                        };
                                        
                                        let follow_up_prompt = format!("[SYSTEM NOTE: The physical vision engine (Llava) just finished processing the screen for the user's previous request ('{}'). It sees: {}\n\nDOM tree sees: {}\n\nGive a quick follow up response providing this new visual information to the user.]", message, vision_res, ui_context);
                                        
                                        let active_app = get_frontmost_app();
                                        if let Ok(mut follow_up_speech) = brain_chat.decide_chat(&follow_up_prompt, &history_str, &active_app, false, tx_chat_resp.clone()).await {
                                            // Strip IGNORE if present
                                            if let Some(start_idx) = follow_up_speech.find("[ACTION: IGNORE]") {
                                                let mut clean = follow_up_speech[..start_idx].to_string();
                                                clean.push_str(&follow_up_speech[start_idx + 16..]);
                                                follow_up_speech = clean.trim().to_string();
                                            }
                                            if !follow_up_speech.is_empty() {
                                                let _ = tx_chat_resp.send(crate::types::WsResponse {
                                                    msg_type: "action".into(),
                                                    speech: Some(follow_up_speech),
                                                    t: None,
                                                    action: None,
                                                    thought: None,
                                                    agent_mode: Some(AgentMode::Chat),
                                                    macro_state: None,
                                                    tool_name: None,
                                                });
                                            }
                                        }
                                    }
                                }
                                Err(e) => { println!("! Chat brain failed: {}", e); }
                            }
                        });
                    }
                    AgentMode::Operator => {
                        println!("🦾 Operator Mode (Hierarchical Cognitive Planner + Algorithmic Execution)");
                        println!("🧠 Processing Goal: {}", message);

                        let brain_agent = brain.clone();
                        let driver_clone = driver.clone();
                        let msg_clone = message.clone();
                        let history_str_agent = history_str.clone();
                        let tx_op_resp = tx_agent.clone();
                        let b_mu_op = browser_mu.clone();
                        let ask_notifier_agent = ask_notifier.clone();
                        let kg_operator = kg_store.clone();
                        let researcher_operator = researcher.clone();

                        let mut active_guard = active_task.lock().await;
                        if let Some(handle) = active_guard.take() {
                            handle.abort();
                        }

                        let handle = tokio::spawn(async move {
                            // 1. Clarify Intent ONLY if genuinely missing critical non-inferrable parameters (skip for clear 3+ word goals or concrete app commands to save 10s CPU inference)
                            let word_count = msg_clone.split_whitespace().count();
                            let msg_lower = msg_clone.to_lowercase();
                            let refined_goal = if word_count >= 3 
                                || msg_lower.starts_with("open ") 
                                || msg_lower.starts_with("launch ") 
                                || msg_lower.starts_with("calc") 
                                || msg_lower.starts_with("play ") 
                                || msg_lower.starts_with("close ") {
                                msg_clone.clone()
                            } else {
                                match brain_agent.clarify_intent(&msg_clone, &history_str_agent).await {
                                    Ok(ClarificationResult::Ask { question }) => {
                                        println!("❓ Clarification genuinely needed: {}", question);
                                        let _ = tx_op_resp.send(WsResponse {
                                            msg_type: "action".into(),
                                            t: None,
                                            speech: Some(question.clone()),
                                            action: Some(Action::Ask { question: question.clone() }),
                                            thought: None,
                                            agent_mode: Some(AgentMode::Operator),
                                            macro_state: Some(MacroState::WaitingForUser),
                                            tool_name: Some(ToolName::Ask),
                                        });
                                        tokio::task::yield_now().await;
                                        let (tx_ask, mut rx_ask) = tokio::sync::mpsc::channel(1);
                                        {
                                            let mut ng = ask_notifier_agent.lock().await;
                                            *ng = Some(tx_ask);
                                        }
                                        match rx_ask.recv().await {
                                            Some(reply) => format!("{} (Additional detail: {})", msg_clone, reply),
                                            None => msg_clone.clone(),
                                        }
                                    }
                                    Ok(ClarificationResult::Proceed { refined_goal }) => refined_goal,
                                    Err(_) => msg_clone.clone(),
                                }
                            };

                            println!("🎯 Proceeding with goal: {}", refined_goal);
                            let active_app = get_frontmost_app();

                            // Fetch graph context — pure in-memory HashMap, ~0ms
                            let kg_context_for_llm = {
                                let kg_lock = kg_operator.lock().await;
                                kg_lock.inject_context_for(&refined_goal)
                            };
                            if !kg_context_for_llm.is_empty() {
                                println!("🧠 [Graph] Injecting {} chars of context into planner", kg_context_for_llm.len());
                            }

                            // Slot-filling: dynamically query the Knowledge Graph tactic nodes!
                            let slot_question = {
                                let kg_lock = kg_operator.lock().await;
                                kg_lock.check_tactic_missing_slots(&refined_goal).map(|(_tactic, q)| q)
                            };
                            if let Some(ref question) = slot_question {
                                println!("🔶 [Graph Slots] Missing required info — asking: {}", question);
                                let _ = tx_op_resp.send(WsResponse {
                                    msg_type: "action".into(),
                                    t: None,
                                    speech: Some(question.clone()),
                                    action: Some(Action::Talk { speech: question.clone() }),
                                    thought: None,
                                    agent_mode: Some(AgentMode::Operator),
                                    macro_state: Some(MacroState::Planning),
                                    tool_name: None,
                                });
                            }
                            if slot_question.is_none() {
                            // 1.5. Fast-Path Local Planning: Check deterministic OS/App sequences FIRST (0ms)
                            let local_sequence = match brain_agent.extract_sequence_args(
                                &refined_goal,
                                &history_str_agent,
                                &active_app,
                                &driver_clone.get_installed_app_names(),
                                &kg_context_for_llm,
                            ).await {
                                Ok(val) => val.as_array().cloned().unwrap_or_default(),
                                Err(_) => Vec::new(),
                            };


                            let steps: Vec<serde_json::Value> = if !local_sequence.is_empty() {
                                println!("⚡ [0ms Fast Plan] Local cognitive sequence selected ({} steps)", local_sequence.len());
                                local_sequence
                            } else {
                                // Fallback: check Knowledge Graph or conduct research ONLY if planner couldn't formulate steps
                                let is_explicit_research = refined_goal.to_lowercase().contains("research")
                                    || refined_goal.to_lowercase().contains("investigate")
                                    || refined_goal.to_lowercase().contains("find out");

                                // Look up known platform nodes for this goal via context injection
                                let kg_context = {
                                    let kg_lock = kg_operator.lock().await;
                                    kg_lock.inject_context_for(&refined_goal)
                                };

                                // If research explicitly requested, discover platforms live
                                let steps: Vec<serde_json::Value> = if is_explicit_research && kg_context.is_empty() {
                                    let research_speech = "Hold on, let me research that for you real quick...".to_string();
                                    let _ = tx_op_resp.send(WsResponse {
                                        msg_type: "action".into(),
                                        t: None,
                                        speech: Some(research_speech.clone()),
                                        action: Some(Action::Talk { speech: research_speech }),
                                        thought: None,
                                        agent_mode: Some(AgentMode::Operator),
                                        macro_state: Some(MacroState::Planning),
                                        tool_name: None,
                                    });
                                    let domain_entry = researcher_operator.research_domain(&refined_goal, Some(&brain_agent)).await;
                                    // Build milestones from first discovered platform
                                    if let Some(p) = domain_entry.platforms.first() {
                                        if p.platform_type == "desktop_app" {
                                            vec![
                                                serde_json::json!({"action_type":"OpenApp","intent":"LaunchApp","input": p.target}),
                                                serde_json::json!({"action_type":"UI_Intent","intent":"Search","input": refined_goal}),
                                            ]
                                        } else {
                                            vec![
                                                serde_json::json!({"action_type":"OpenApp","intent":"LaunchSurface","input":"browser"}),
                                                serde_json::json!({"action_type":"Navigate","intent":"VisitPlatform","input": p.target}),
                                                serde_json::json!({"action_type":"UI_Intent","intent":"Search","input": refined_goal}),
                                            ]
                                        }
                                    } else {
                                        vec![
                                            serde_json::json!({"action_type":"OpenApp","intent":"LaunchSurface","input":"browser"}),
                                            serde_json::json!({"action_type":"Navigate","intent":"SearchWeb","input": format!("https://duckduckgo.com/?q={}", refined_goal.replace(' ', "+"))}),
                                        ]
                                    }
                                } else {
                                    // Direct browser fallback
                                     let target_url = driver_clone.resolve_target_url(&refined_goal);
                                    vec![
                                        serde_json::json!({"action_type":"OpenApp","intent":"LaunchSurface","input":"browser"}),
                                        serde_json::json!({"action_type":"Navigate","intent":"OpenTarget","input": target_url}),
                                        serde_json::json!({"action_type":"UI_Intent","intent":"Search","input": refined_goal}),
                                    ]
                                };
                                steps
                            };

                            let mut execution_log = Vec::new();

                            if !steps.is_empty() {

                                println!("🗺️ Strategic Milestones executing: {} steps", steps.len());
                                let ack_speech = "On it.".to_string();
                                let _ = tx_op_resp.send(WsResponse {
                                    msg_type: "action".into(),
                                    t: None,
                                    speech: Some(ack_speech.clone()),
                                    action: Some(Action::Talk { speech: ack_speech }),
                                    thought: None,
                                    agent_mode: Some(AgentMode::Operator),
                                    macro_state: Some(MacroState::ActOnSurface),
                                    tool_name: None,
                                });
                                let mut current_target_app: Option<String> = None;
                                for (step_idx, step) in steps.iter().enumerate() {
                                            let action_type = step.get("action_type").and_then(|v| v.as_str()).unwrap_or("");
                                            let intent = step.get("intent").and_then(|v| v.as_str()).unwrap_or("");
                                            let input_val = step.get("input").and_then(|v| v.as_str());

                                            println!("  [Milestone {}/{}] {} (Intent: {}, Target: {:?})", step_idx + 1, steps.len(), action_type, intent, input_val);

                                            let milestone_desc = format!("{}: {} {:?}", action_type, intent, input_val);
                                            let mut milestone_outcome = String::new();

                                            // 3. Algorithmic Execution: Deterministic algorithm uses UI map and app catalog
                                            match action_type {
                                                "InspectEnvironment" => {
                                                    let count = driver_clone.installed_apps.len();
                                                    milestone_outcome = format!("Discovered {} system applications; active environment: {}", count, active_app);
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                                },
                                                "GenerateAsset" => {
                                                    let spec = input_val.unwrap_or(&refined_goal);
                                                    match driver_clone.generate_3d_asset(spec) {
                                                        Ok(res) => milestone_outcome = res,
                                                        Err(e) => milestone_outcome = format!("Asset generation note: {}", e),
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                                                },
                                                "OpenApp" => {
                                                    let target_app = input_val.unwrap_or("browser");
                                                    current_target_app = Some(target_app.to_string());
                                                    match driver_clone.open_app(target_app) {
                                                        Ok(opened) => {
                                                            milestone_outcome = format!("Opened {}", opened);
                                                        }
                                                        Err(e) => {
                                                            println!("⚠️ App '{}' failed to open locally: {}. Checking web fallback...", target_app, e);
                                                            let web_url = match target_app.to_lowercase().as_str() {
                                                                "spotify" => Some("https://open.spotify.com"),
                                                                "youtube" => Some("https://youtube.com"),
                                                                "whatsapp" => Some("https://web.whatsapp.com"),
                                                                "telegram" => Some("https://web.telegram.org"),
                                                                "slack" => Some("https://app.slack.com"),
                                                                "discord" => Some("https://discord.com/app"),
                                                                "twitter" | "x" => Some("https://x.com"),
                                                                "reddit" => Some("https://reddit.com"),
                                                                _ => None,
                                                             };
                                                            if let Some(url) = web_url {
                                                                driver_clone.open_url(url);
                                                                milestone_outcome = format!("'{}' opened in native browser ({})", target_app, url);
                                                            } else {
                                                                milestone_outcome = format!("Could not open {}: {}", target_app, e);
                                                            }
                                                        }
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
                                                },
                                                "Navigate" => {
                                                    let target = input_val.unwrap_or(&refined_goal);
                                                    let resolved_url = driver_clone.resolve_target_url(target);
                                                    println!("🧭 Navigating directly in native browser: {}", resolved_url);

                                                    // Use user's real browser in the foreground
                                                    current_target_app = Some("browser".to_string());
                                                    driver_clone.open_url(&resolved_url);
                                                    milestone_outcome = format!("Navigated to {}", resolved_url);
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(2200)).await;
                                                },
                                                "Scroll" => {
                                                    let direction = input_val.unwrap_or("down");
                                                    let _ = driver_clone.scroll(direction);
                                                    milestone_outcome = format!("Scrolled {}", direction);
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
                                                },
                                                "UI_Intent" => {
                                                    // Ensure target app window is strictly frontmost and focused
                                                    if let Some(ref target) = current_target_app {
                                                        driver_clone.activate_app(target);
                                                    }

                                                    // If this is a search/play intent with conversational query, refine it with LLM
                                                    let refined_input_storage;
                                                    let active_input_val = if (intent == "Search" || intent == "Play") && input_val.map_or(false, |s| s.split_whitespace().count() > 2) {
                                                        let raw_q = input_val.unwrap();
                                                        let platform_name = current_target_app.as_deref().unwrap_or("web");
                                                        refined_input_storage = brain_agent.refine_search_query(raw_q, platform_name).await;
                                                        println!("🧠 LLM Search Optimizer: Refined '{}' -> '{}'", raw_q, refined_input_storage);
                                                        Some(refined_input_storage.as_str())
                                                    } else {
                                                        input_val
                                                    };

                                                    // Human UI Closed Loop: Perceive Live Native OS Screen -> Act -> Verify
                                                    let mut attempts = 0;
                                                    loop {
                                                        attempts += 1;
                                                        // Always perceive the real user screen via native accessibility APIs targeted to current app
                                                        let ui_list = crate::perception::perceive_native_target(current_target_app.as_deref()).await.unwrap_or_default();

                                                        // Universal Algorithmic UI interaction
                                                        match driver_clone.execute_ui_intent(intent, active_input_val, &ui_list) {
                                                            Ok(detail) => {
                                                                milestone_outcome = detail.clone();
                                                                // Human Scroll-to-Discover: if element was offscreen and we scrolled down, re-perceive!
                                                                if detail.contains("Scrolled down to discover") && attempts < 3 {
                                                                    println!("  📜 [Scroll-to-Discover] Scrolled viewport. Re-perceiving in 400ms (Attempt {}/3)...", attempts);
                                                                    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
                                                                    continue;
                                                                }
                                                                break;
                                                            }
                                                            Err(e) => {
                                                                milestone_outcome = format!("UI intent note: {}", e);
                                                                break;
                                                            }
                                                        }
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                                                },
                                                "Search" => {
                                                    if let Some(ref target) = current_target_app {
                                                        driver_clone.activate_app(target);
                                                    }
                                                    let raw_query = input_val.unwrap_or(&refined_goal);
                                                    let platform_name = current_target_app.as_deref().unwrap_or("web");
                                                    let query_to_use = if raw_query.split_whitespace().count() > 2 {
                                                        let opt = brain_agent.refine_search_query(raw_query, platform_name).await;
                                                        println!("🧠 LLM Search Optimizer: Refined '{}' -> '{}'", raw_query, opt);
                                                        opt
                                                    } else {
                                                        raw_query.to_string()
                                                    };
                                                    let ui_list = crate::perception::perceive_native_target(current_target_app.as_deref()).await.unwrap_or_default();
                                                    match driver_clone.execute_ui_intent("Search", Some(&query_to_use), &ui_list) {
                                                        Ok(detail) => {
                                                            milestone_outcome = detail;
                                                        }
                                                        Err(_) => {
                                                            driver_clone.search_web(&query_to_use);
                                                            milestone_outcome = format!("Searched web for {}", query_to_use);
                                                        }
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                                                },
                                                _ => {
                                                    println!("  ⚠️ Unknown action_type: {}", action_type);
                                                    milestone_outcome = format!("Completed {}", action_type);
                                                }
                                            }

                                            execution_log.push(milestone_outcome.clone());
                                }
                            } else {
                                    let resolved = driver_clone.resolve_target_url(&refined_goal);
                                    driver_clone.open_url(&resolved);
                                    execution_log.push(format!("Navigated to {}", resolved));
                                }

                            // 4. Algorithm reports verified outcome to user (No open-ended hallucination)
                            let last_success = execution_log.iter().rev().find(|l| !l.is_empty()).cloned();
                            let final_speech = match last_success {
                                Some(detail) if detail.contains("Entered ") => {
                                    format!("Done! {}.", detail)
                                }
                                Some(detail) if detail.contains("Triggered playback") => {
                                    format!("Done! {}.", detail)
                                }
                                Some(detail) if detail.contains("Searched for") => {
                                    format!("Done! {}.", detail)
                                }
                                Some(detail) if detail.contains("Navigated to") => {
                                    format!("Done! {}.", detail)
                                }
                                Some(detail) => {
                                    format!("Done! {}.", detail)
                                }
                                None => format!("Completed {}.", refined_goal)
                            };

                            println!("🗣️ Momentum Final Response: {}", final_speech);
                            let _ = tx_op_resp.send(WsResponse {
                                msg_type: "action".into(),
                                t: None,
                                speech: Some(final_speech.clone()),
                                action: Some(Action::Talk { speech: final_speech }),
                                thought: None,
                                agent_mode: Some(AgentMode::Operator),
                                macro_state: Some(MacroState::Done),
                                tool_name: None,
                            });
                            } // end slot_question.is_none()
                        });

                        *active_guard = Some(handle);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_clean_chat_history_drops_duplicate_trailing_user_message() {
        let history = vec![
            crate::types::ChatMessage { role: "user".into(), text: "hey".into() },
            crate::types::ChatMessage { role: "agent".into(), text: "Hey!".into() },
            crate::types::ChatMessage { role: "agent".into(), text: "How's it going?".into() },
            crate::types::ChatMessage { role: "user".into(), text: "Not bad".into() },
        ];
        let current_msg = "Not bad";
        let out = format_clean_chat_history(&history, current_msg);
        assert!(out.contains("<|im_start|>user\nhey<|im_end|>"));
        assert!(out.contains("<|im_start|>assistant\nHey! How's it going?<|im_end|>"));
        // The duplicate trailing "Not bad" in history must be dropped because current_msg is "Not bad"
        assert!(!out.contains("<|im_start|>user\nNot bad<|im_end|>"));
    }

    #[test]
    fn test_format_clean_chat_history_consolidates_streaming_assistant_chunks() {
        let history = vec![
            crate::types::ChatMessage { role: "agent".into(), text: "Chunk 1.".into() },
            crate::types::ChatMessage { role: "agent".into(), text: "Chunk 2.".into() },
        ];
        let out = format_clean_chat_history(&history, "next");
        assert_eq!(out, "<|im_start|>assistant\nChunk 1. Chunk 2.<|im_end|>\n");
    }
}