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

pub fn get_frontmost_app() -> String {
    if cfg!(target_os = "macos") {
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
    } else if cfg!(target_os = "windows") {
        if let Ok(output) = std::process::Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-Command",
                "$w = Add-Type -MemberDefinition '[DllImport(\"user32.dll\")] public static extern IntPtr GetForegroundWindow();' -Name Win32 -Namespace Win32Functions -PassThru; $h = $w::GetForegroundWindow(); (Get-Process | Where-Object { $_.MainWindowHandle -eq $h }).ProcessName"
            ])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                return text;
            }
        }
    }
    "Desktop".to_string()
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
                        let clean_history: Vec<&crate::types::ChatMessage> = history.iter().filter(|msg| !msg.text.trim().is_empty()).collect();
                        let mut history_str = String::new();
                        let keep_count = 6;
                        let skip = if clean_history.len() > keep_count { clean_history.len() - keep_count } else { 0 };
                        for msg in clean_history.iter().skip(skip) {
                            history_str.push_str(&format!("<|im_start|>{role}\n{text}<|im_end|>\n", role=msg.role, text=msg.text));
                        }
                        
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
                
                // Do not filter out short messages like "hi" — it breaks conversational context
                let clean_history: Vec<&crate::types::ChatMessage> = history.iter().filter(|msg| {
                    !msg.text.trim().is_empty()
                }).collect();

                let mut history_str = String::new();
                let keep_count = 6; // Reduced from 20 to speed up TTFT for faster replies
                let skip = if clean_history.len() > keep_count { clean_history.len() - keep_count } else { 0 };
                // Don't inject omitted warnings into the history prompt, it confuses the model
                for msg in clean_history.iter().skip(skip) {
                    history_str.push_str(&format!("<|im_start|>{role}\n{text}<|im_end|>\n", role=msg.role, text=msg.text));
                }

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

                            // 1.5. Fast-Path Local Planning: Check deterministic OS/App sequences FIRST (0ms, no network delay)
                            let local_sequence = match brain_agent.extract_sequence_args(
                                &refined_goal, 
                                &history_str_agent, 
                                &active_app, 
                                &driver_clone.get_installed_app_names()
                            ).await {
                                Ok(val) => val.as_array().cloned().unwrap_or_default(),
                                Err(_) => Vec::new(),
                            };

                            let steps: Vec<serde_json::Value> = if !local_sequence.is_empty() {
                                println!("⚡ [0ms Fast Plan] Local cognitive sequence selected ({} steps)", local_sequence.len());
                                local_sequence
                            } else {
                                // 2. External Web Research & Discovery: Only for prospecting or informational research goals
                                let domain_opt = {
                                    let mut kg = kg_operator.lock().await;
                                    kg.find_domain(&refined_goal)
                                };

                                let domain = match domain_opt {
                                    Some(d) => {
                                        println!("⚡ [KG HIT: 0ms] Recalled stored domain \"{}\" for: \"{}\"", d.domain_name, refined_goal);
                                        let recall_speech = format!("I know the platforms for {}. Moving on that now.", d.domain_name);
                                        let _ = tx_op_resp.send(WsResponse {
                                            msg_type: "action".into(),
                                            t: None,
                                            speech: Some(recall_speech.clone()),
                                            action: Some(Action::Talk { speech: recall_speech }),
                                            thought: None,
                                            agent_mode: Some(AgentMode::Operator),
                                            macro_state: Some(MacroState::Planning),
                                            tool_name: None,
                                        });
                                        d
                                    }
                                    None => {
                                        println!("🔍 [KG MISS] Novel goal. Autonomous Web Researcher initiated for: \"{}\"", refined_goal);
                                        let research_speech = "Hold on, let me research platforms for that real quick...".to_string();
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

                                        let researched_entry = researcher_operator.research_domain(&refined_goal, Some(&brain_agent)).await;
                                        let top_names = researched_entry.platforms.iter().take(3).map(|p| p.name.clone()).collect::<Vec<_>>().join(", ");
                                        if !top_names.is_empty() {
                                            let update_speech = format!("Found {}. Setting up navigation now.", top_names);
                                            let _ = tx_op_resp.send(WsResponse {
                                                msg_type: "action".into(),
                                                t: None,
                                                speech: Some(update_speech.clone()),
                                                action: Some(Action::Talk { speech: update_speech }),
                                                thought: None,
                                                agent_mode: Some(AgentMode::Operator),
                                                macro_state: Some(MacroState::ActOnSurface),
                                                tool_name: None,
                                            });
                                        }
                                        researched_entry
                                    }
                                };

                                if !domain.platforms.is_empty() {
                                    let top_platform = domain.platforms.first().map(|p| p.name.clone()).unwrap_or_else(|| "Platform".to_string());
                                    println!("⚡ [0ms Fast Plan] Using researched domain milestones for \"{}\" (Platform: {})", domain.domain_name, top_platform);
                                    let nav_speech = format!("Opening {} for {}.", top_platform, domain.domain_name);
                                    let _ = tx_op_resp.send(WsResponse {
                                        msg_type: "action".into(),
                                        t: None,
                                        speech: Some(nav_speech.clone()),
                                        action: Some(Action::Talk { speech: nav_speech }),
                                        thought: None,
                                        agent_mode: Some(AgentMode::Operator),
                                        macro_state: Some(MacroState::ActOnSurface),
                                        tool_name: None,
                                    });
                                }
                                domain.to_milestones(None, &refined_goal).into_iter()
                                    .map(|s| serde_json::to_value(s).unwrap_or_default())
                                    .collect()
                            };

                            let mut execution_log = Vec::new();

                            if !steps.is_empty() {
                                println!("🗺️ Strategic Milestones executing: {} steps", steps.len());
                                
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
                                                    if target_app.eq_ignore_ascii_case("browser") || target_app.eq_ignore_ascii_case("chrome") {
                                                        let mut b_guard = b_mu_op.lock().await;
                                                        let launch_res = b_guard.ensure_launched().await;
                                                        drop(b_guard);
                                                        if launch_res.is_ok() {
                                                            milestone_outcome = "Browser launched and ready".to_string();
                                                        } else {
                                                            match driver_clone.open_app(target_app) {
                                                                Ok(opened) => milestone_outcome = format!("Opened {}", opened),
                                                                Err(e) => milestone_outcome = format!("Could not open {}: {}", target_app, e),
                                                            }
                                                        }
                                                    } else {
                                                        match driver_clone.open_app(target_app) {
                                                            Ok(opened) => {
                                                                milestone_outcome = format!("Opened {}", opened);
                                                            }
                                                            Err(e) => {
                                                                milestone_outcome = format!("Could not open {}: {}", target_app, e);
                                                            }
                                                        }
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                                                },
                                                "Navigate" => {
                                                    let target = input_val.unwrap_or(&refined_goal);
                                                    let resolved_url = driver_clone.resolve_target_url(target);
                                                    println!("🧭 Navigating directly to: {}", resolved_url);

                                                    let mut navigated = false;
                                                    let mut b_guard = b_mu_op.lock().await;
                                                    if let Ok(_) = b_guard.ensure_launched().await {
                                                        if let Some(page) = b_guard.page.as_ref() {
                                                            if let Ok(_) = page.goto(&resolved_url).await {
                                                                 let _ = page.wait_for_navigation().await;
                                                                navigated = true;
                                                            }
                                                        }
                                                    }
                                                    drop(b_guard);

                                                    if !navigated {
                                                        driver_clone.open_url(&resolved_url);
                                                    }
                                                    milestone_outcome = format!("Navigated to {}", resolved_url);
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                                                },
                                                "Scroll" => {
                                                    let direction = input_val.unwrap_or("down");
                                                    let mut scrolled_browser = false;
                                                    let b_guard = b_mu_op.lock().await;
                                                    if let Some(page) = b_guard.page.as_ref() {
                                                        let scroll_js = if direction.eq_ignore_ascii_case("up") { "window.scrollBy(0, -500)" } else { "window.scrollBy(0, 500)" };
                                                        let _ = page.evaluate(scroll_js).await;
                                                        scrolled_browser = true;
                                                    }
                                                    drop(b_guard);
                                                    if !scrolled_browser {
                                                        let _ = driver_clone.scroll(direction);
                                                    }
                                                    milestone_outcome = format!("Scrolled {} to reveal more surface content", direction);
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
                                                },
                                                "UI_Intent" => {
                                                    // Human UI Closed Loop: Perceive -> Act -> If Offscreen, Scroll to Discover -> Re-perceive -> Complete
                                                    let mut attempts = 0;
                                                    loop {
                                                        attempts += 1;
                                                        let mut ui_list = Vec::new();
                                                        let b_guard = b_mu_op.lock().await;
                                                        if let Some(page) = b_guard.page.as_ref() {
                                                            if let Ok(els) = crate::perception::perceive(page).await {
                                                                ui_list = els;
                                                            }
                                                        }
                                                        drop(b_guard);

                                                        // Fallback to native OS accessibility tree if browser has no elements
                                                        if ui_list.is_empty() {
                                                            if let Ok(native_els) = crate::perception::perceive_native().await {
                                                                ui_list = native_els;
                                                            }
                                                        }

                                                        // Universal Algorithmic UI interaction
                                                        match driver_clone.execute_ui_intent(intent, input_val, &ui_list) {
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
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                                                },
                                                "Search" => {
                                                    let query = input_val.unwrap_or(&refined_goal);
                                                    driver_clone.search_web(query);
                                                    milestone_outcome = format!("Searched web for {}", query);
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                                                },
                                                _ => {
                                                    println!("  ⚠️ Unknown action_type: {}", action_type);
                                                    milestone_outcome = format!("Completed {}", action_type);
                                                }
                                            }

                                            execution_log.push(milestone_outcome.clone());

                                            // Non-blocking step reaction: runs in background so physical execution continues at full speed
                                            let brain_react = brain_agent.clone();
                                            let tx_react = tx_op_resp.clone();
                                            let goal_react = refined_goal.clone();
                                            let desc_react = milestone_desc.clone();
                                            let out_react = milestone_outcome.clone();
                                            tokio::spawn(async move {
                                                if let Ok(reaction) = brain_react.step_reaction(&goal_react, &desc_react, &out_react).await {
                                                    if let StepReaction::Speak { text } = reaction {
                                                        println!("🗣️ Step reaction: {}", text);
                                                        let _ = tx_react.send(WsResponse {
                                                            msg_type: "action".into(),
                                                            t: None,
                                                            speech: Some(text.clone()),
                                                            action: Some(Action::Talk { speech: text }),
                                                            thought: None,
                                                            agent_mode: Some(AgentMode::Operator),
                                                            macro_state: Some(MacroState::ActOnSurface),
                                                            tool_name: None,
                                                        });
                                                    }
                                                }
                                            });
                                }
                            } else {
                                    let resolved = driver_clone.resolve_target_url(&refined_goal);
                                    driver_clone.open_url(&resolved);
                                    execution_log.push(format!("Navigated to {}", resolved));
                                }

                            // 4. Algorithm notifies LLM of outcome for final conversational narration
                            let summary_str = execution_log.join(". ");
                            let final_speech = match brain_agent.evaluate_and_narrate(&refined_goal, &summary_str).await {
                                Ok(narration) if !narration.is_empty() => narration,
                                _ => {
                                    if !execution_log.is_empty() {
                                        format!("All set! {}.", execution_log[0])
                                    } else {
                                        format!("All set! Completed: {}.", refined_goal)
                                    }
                                }
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
                        });
                        *active_guard = Some(handle);
                    }
                }
            }
        }
    }
}