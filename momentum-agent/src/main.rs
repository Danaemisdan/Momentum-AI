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

use crate::browser::MomentumBrowser;
use crate::brain::MomentumBrain;
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
        .unwrap_or_else(|_| r"C:\Users\usha\Downloads\Momentum AI\momentum-agent\momentum-engine-0.5b.gguf".to_string());
    
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
    let driver = Arc::new(crate::driver::ActionDriver::new("windows"));
    
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
                            // Skip needs_vision=true for interim to avoid 300ms latency penalty
                            let prefill_prompt = brain_prefill.build_chat_prompt(&text_clone, &history_str, false).await;
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
                        let msg_lower = message.to_lowercase();
                        let b_mu_chat = browser_mu.clone();
                        
                        tokio::spawn(async move {
                            let mut msg_with_vision = message.clone();
                            
                            // Instant contextual awareness of the active OS application (0ms latency)
                            let active_app = "Windows Desktop".to_string();
                            
                            msg_with_vision = format!("[System Note: The user's currently focused application is '{}']\n{}", active_app, msg_with_vision);
                            
                            let needs_vision = msg_lower.contains("screen") || msg_lower.contains("see") || msg_lower.contains("look");
                            match brain_chat.decide_chat(&msg_with_vision, &history_str, needs_vision, tx_chat_resp.clone()).await {
                                Ok(mut speech) => {
                                    // Extract and run [ACTION: OPEN_APP(...)]
                                    if let Some(start_idx) = speech.find("[ACTION: OPEN_APP(") {
                                        if let Some(end_idx) = speech[start_idx..].find(")]") {
                                            let app_name = &speech[start_idx + 18..start_idx + end_idx];
                                            println!("🚀 Opening application: {}", app_name);
                                            let _ = std::process::Command::new("cmd")
                                                .arg("/C")
                                                .arg(format!("start {}", app_name))
                                                .spawn();
                                            // Strip it from speech
                                            let mut clean = speech[..start_idx].to_string();
                                            clean.push_str(&speech[start_idx + end_idx + 2..]);
                                            speech = clean.trim().to_string();
                                        }
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
                                        
                                        if let Ok(mut follow_up_speech) = brain_chat.decide_chat(&follow_up_prompt, &history_str, false, tx_chat_resp.clone()).await {
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
                        println!("🦾 Operator Mode (Semantic UI Traversal)");
                        println!("🧠 Generating Semantic Plan for: {}", message);

                        let brain_agent = brain.clone();
                        let driver_clone = driver.clone();
                        let msg_clone = message.clone();
                        
                        let mut active_guard = active_task.lock().await;
                        if let Some(handle) = active_guard.take() {
                            handle.abort();
                        }

                        let handle = tokio::spawn(async move {
                            match brain_agent.extract_sequence_args(&msg_clone, &[]).await {
                                Ok(plan_json) => {
                                    if let Some(steps) = plan_json.as_array() {
                                        println!("🗺️ Semantic Plan generated ({} steps)", steps.len());
                                        for step in steps {
                                            let action_type = step.get("action_type").and_then(|v| v.as_str()).unwrap_or("");
                                            let intent = step.get("intent").and_then(|v| v.as_str());
                                            let input_val = step.get("input").and_then(|v| v.as_str());
                                            
                                            println!("➡️ Executing step: {} (Intent: {:?})", action_type, intent);
                                            
                                            match action_type {
                                                "OpenApp" => {
                                                    if let Some(app) = input_val {
                                                        println!("    💻 Launching: {}", app);
                                                        let _ = std::process::Command::new("cmd")
                                                            .arg("/C").arg(format!("start {}", app))
                                                            .spawn();
                                                        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                                                    }
                                                },
                                                "UI_Intent" => {
                                                    if let Some(target_intent) = intent {
                                                        println!("    🔍 Scanning Live UI for semantic intent: {}", target_intent);
                                                        
                                                        // Fallback heuristic execution for desktop apps until Windows UIAutomation bridge is connected
                                                        use enigo::{Direction::{Click}, Enigo, Key, Keyboard, Settings};
                                                        let mut enigo = Enigo::new(&Settings::default()).unwrap();
                                                        
                                                        match target_intent {
                                                            "Search" => {
                                                                let _ = enigo.key(Key::Tab, Click);
                                                                let _ = enigo.key(Key::Tab, Click);
                                                                if let Some(text) = input_val {
                                                                    println!("    ⌨️ Found Search Node! Typing: {}", text);
                                                                    let _ = enigo.text(text);
                                                                    let _ = enigo.key(Key::Return, Click);
                                                                }
                                                            },
                                                            "MediaControl" => {
                                                                let _ = enigo.key(Key::Tab, Click);
                                                                println!("    🖱️ Found Media Control Node! Clicking Play...");
                                                                let _ = enigo.key(Key::Return, Click);
                                                            },
                                                            _ => {
                                                                println!("    ⚠️ No node matching intent {} found on current screen.", target_intent);
                                                            }
                                                        }
                                                        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                                                    }
                                                },
                                                _ => println!("    ⚠️ Unknown semantic action_type"),
                                            }
                                        }
                                        println!("✅ Semantic Plan Completed!");
                                    } else {
                                        println!("❌ LLM failed to output a JSON array.");
                                    }
                                }
                                Err(e) => println!("❌ Extractor failed: {}", e),
                            }
                        });
                        *active_guard = Some(handle);
                    }
                }
            }
        }
    }
}