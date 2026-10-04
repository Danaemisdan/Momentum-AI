
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

pub async fn start_agent() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    println!("🚀 Momentum Agent starting...");

    let model_path = env::var("MOMENTUM_MODEL_PATH")
        .unwrap_or_else(|_| "/Users/sanjeevn/Downloads/Momentum AI/momentum-engine-3b.gguf".to_string());
    
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
    let _executor = ActionExecutor::new();
    let tx_agent_clone = tx_agent.clone();
    
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
                        if cancel_active_task(&active_task, &ask_notifier, &tx_agent_clone).await {
                            println!("🛑 Active agent task cancelled by user.");
                        }
                        continue;
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
                let keep_count = 20;
                let skip = if clean_history.len() > keep_count { clean_history.len() - keep_count } else { 0 };
                // Don't inject omitted warnings into the history prompt, it confuses the model
                for msg in clean_history.iter().skip(skip) {
                    history_str.push_str(&format!("<|start_header_id|>{role}<|end_header_id|>\n{text}<|eot_id|>\n", role=msg.role, text=msg.text));
                }

                // --- INTENT CLASSIFICATION ---
                let agent_mode = match brain.classify_mode(&message).await {
                    Ok(i) => i,
                    Err(e) => {
                        println!("! Classifier failed: {}", e);
                        AgentMode::Chat
                    }
                };

                match agent_mode {
                    AgentMode::Chat => {
                        println!("🤖 Personality Mode (Chat)");
                        let tx_chat_resp = tx_agent_clone.clone();
                        let brain_chat = brain.clone();
                        let msg_lower = message.to_lowercase();
                        let b_mu_chat = browser_mu.clone();
                        
                        tokio::spawn(async move {
                            let mut msg_with_vision = message.clone();
                            
                            // If the user seems to be asking about the screen/sight, dynamically inject the optic feed!
                            if msg_lower.contains("screen") || msg_lower.contains("see") || msg_lower.contains("look") || msg_lower.contains("what") {
                                println!("👁️ Chat implies visual context. Polling DOM Tree (Retrospect)...");
                                
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
                                msg_with_vision = format!("{}\n[SYSTEM NOTE: The agent's UI tree sees: {}]", message, ui_context);
                            }

                            match brain_chat.decide_chat(&msg_with_vision, &history_str, tx_chat_resp.clone()).await {
                                Ok(speech) => {
                                    println!("🗣️ Momentum: {}", speech);
                                    let _ = tx_chat_resp.send(WsResponse {
                                        msg_type: "action".into(),
                                        t: None,
                                        speech: Some(speech.clone()),
                                        action: Some(Action::Talk { speech }),
                                        thought: None,
                                        agent_mode: Some(AgentMode::Chat),
                                        macro_state: None,
                                        tool_name: None,
                                    });
                                }
                                Err(e) => { println!("! Chat brain failed: {}", e); }
                            }
                        });
                    }
                    AgentMode::Operator => {
                        println!("🦾 Operator Mode (Phase 1: Clarification)");
                        let goal = message.clone();
                        let b_mu_agent = browser_mu.clone();
                        let brain_agent = brain.clone();
                        let tx_agent_loop = tx_agent_clone.clone();
                        let mut executor_agent = ActionExecutor::new();
                        let ask_notifier_agent = ask_notifier.clone();
                        let history_str_agent = history_str.clone();

                        let mut active_guard = active_task.lock().await;
                        if let Some(handle) = active_guard.take() {
                            println!("🛑 Cancelling previous active agent task due to new command.");
                            handle.abort();
                        }

                        let handle = tokio::spawn(async move {
                            // ====================================================
                            // PHASE 1: INTENT CLARIFICATION (before browser opens)
                            // ====================================================
                            println!("🔍 Phase 1: Clarifying intent for: {}", goal);
                            let refined_goal = match brain_agent.clarify_intent(&goal, &history_str_agent).await {
                                Ok(ClarificationResult::Ask { question }) => {
                                    println!("❓ Clarification needed: {}", question);
                                    // Send question to user and wait for their answer
                                    let _ = tx_agent_loop.send(WsResponse {
                                        msg_type: "action".into(),
                                        t: None,
                                        speech: Some(question.clone()),
                                        action: Some(Action::Ask { question: question.clone() }),
                                        thought: None,
                                        agent_mode: Some(AgentMode::Operator),
                                        macro_state: Some(MacroState::WaitingForUser),
                                        tool_name: Some(ToolName::Ask),
                                    });
                                    // Yield to the async runtime so the WS forward task can flush
                                    // this message to the frontend BEFORE we block on rx_ask.recv().
                                    tokio::task::yield_now().await;
                                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                                    // Wait for user reply via the ask_notifier mechanism
                                    let (tx_ask, mut rx_ask) = tokio::sync::mpsc::channel(1);
                                    {
                                        let mut ng = ask_notifier_agent.lock().await;
                                        *ng = Some(tx_ask);
                                    }
                                    match rx_ask.recv().await {
                                        Some(reply) => {
                                            println!("▶️ User clarified: {}", reply);
                                            format!("{} — specifically: {}", goal, reply)
                                        }
                                        None => goal.clone()
                                    }
                                }
                                Ok(ClarificationResult::Proceed { refined_goal }) => {
                                    println!("✅ Goal clear: {}", refined_goal);
                                    refined_goal
                                }
                                Err(e) => {
                                    println!("⚠️ Clarification failed ({}), using original goal.", e);
                                    goal.clone()
                                }
                            };

                            // ====================================================
                            // PHASE 2: PLANNING
                            // ====================================================
                            println!("📋 Phase 2: Planning steps for: {}", refined_goal);
                            
                            // === LIVE WEB CONTEXT FETCH ===
                            // Fetch DDG results BEFORE planning so the model plans with real
                            // current data instead of hallucinating from stale training.
                            println!("🌐 Fetching live web context for plan...");
                            let ddg_results = crate::scraper::search_duckduckgo(&refined_goal, 5).await;
                            let now_str = {
                                use std::time::{SystemTime, UNIX_EPOCH};
                                let secs = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                // Simple IST display: UTC+5:30 = +19800 seconds
                                let ist_secs = secs + 19800;
                                let days = ist_secs / 86400;
                                let h = (ist_secs % 86400) / 3600;
                                let m = (ist_secs % 3600) / 60;
                                // Day of week (epoch was Thursday=4)
                                let days_of_week = ["Thu","Fri","Sat","Sun","Mon","Tue","Wed"];
                                let dow = days_of_week[(days % 7) as usize];
                                format!("{} {:02}:{:02} IST, April 2026", dow, h, m)
                            };
                            let web_context = if ddg_results.is_empty() {
                                println!("⚠️ DDG returned no results (offline or rate limited).");
                                String::new()
                            } else {
                                println!("✅ DDG: {} results fetched for planning.", ddg_results.len());
                                crate::scraper::format_context_block(&refined_goal, &ddg_results, &now_str)
                            };
                            
                            // Build the enriched goal string: date + live context
                            let temporal_prefix = format!("Today is {}.", now_str);
                            let plan_context = if web_context.is_empty() {
                                format!("{}\n\nGOAL: {}", temporal_prefix, refined_goal)
                            } else {
                                format!("{}\n\n{}\n\nGOAL: {}", temporal_prefix, web_context, refined_goal)
                            };
                            
                            // Don't pass full history to plan_steps — prior unrelated tasks confuse the 3B model
                            // and produce plans for the WRONG goal. The refined_goal already has all context needed.
                            let mut plan = match brain_agent.plan_steps(&refined_goal, &plan_context).await {
                                Ok(p) => {
                                    println!("📋 Raw plan ({} steps):", p.len());
                                    for (i, step) in p.iter().enumerate() {
                                        println!("   {}: {}", i + 1, step);
                                    }
                                    p
                                }
                                Err(e) => {
                                    println!("⚠️ Planning failed ({}), using fallback plan.", e);
                                    vec![]
                                }
                            };

                            // HARD OVERRIDE: The 3B model hallucinates specific sites (CNET, Wikipedia, etc.)
                            // from training data. We never trust it to pick the right site. Unless the user
                            // explicitly named a site in their request, ALWAYS force step 1 to be a Bing search.
                            let user_named_site = ["bing", "google", "youtube", "linkedin", "github", "reddit", "amazon", "twitter", "x.com", "duckduckgo"]
                                .iter()
                                .any(|s| refined_goal.to_lowercase().contains(s));

                            if !user_named_site {
                                let search_step = format!(
                                    "1. Go to {} and search for: {}",
                                    default_search_engine_name(),
                                    refined_goal
                                );
                                if plan.is_empty() {
                                    plan = vec![
                                        search_step,
                                        "2. Read the top results and click the most relevant one.".into(),
                                        "3. Extract and summarize the key information for the user.".into(),
                                    ];
                                } else {
                                    plan[0] = search_step;
                                }
                            }

                            println!("📋 Final plan ({} steps):", plan.len());
                            for (i, step) in plan.iter().enumerate() {
                                println!("   {}: {}", i + 1, step);
                            }


                            // Don't announce the raw plan steps out loud — the OODA speech already narrates actions naturally
                            let plan_speech = "On it.".to_string();
                            let mut macro_state = infer_initial_macro_state(&refined_goal);
                            let _ = tx_agent_loop.send(WsResponse {
                                msg_type: "action".into(),
                                t: None,
                                speech: Some(plan_speech),
                                action: Some(Action::Wait { ms: 1 }),
                                thought: None,
                                agent_mode: Some(AgentMode::Operator),
                                macro_state: Some(macro_state.clone()),
                                tool_name: Some(ToolName::Wait),
                            });

                            // ====================================================
                            // PHASE 3: AWARENESS-DRIVEN EXECUTION (OODA loop)
                            // ====================================================
                            println!("🚀 Phase 3: Starting execution loop for: {}", refined_goal);
                            let mut last_action_res = "Initial state. No actions taken yet.".to_string();
                            let mut last_action_key = String::new();
                            let mut consecutive_repeat_count = 0usize;
                            let mut invalid_decision_count = 0usize;
                            let mut current_plan_step: usize = 0;

                            for s in 1..=20 {
                                println!("\n--- STEP {} (OODA) [{:?} | Plan step {}/{}] ---", s, macro_state, current_plan_step + 1, plan.len());
                                let mut b = b_mu_agent.lock().await;

                                // --- PERCEIVE ---
                                let mut ui_list = vec![];
                                let mut current_url = "about:blank".to_string();
                                let mut page_summary = "Browser not open yet.".to_string();

                                if b.page.is_none() {
                                    println!("🌐 Launching Google Chrome...");
                                    if let Err(e) = b.ensure_launched().await {
                                        println!("! Failed to launch: {}", e);
                                        break;
                                    }
                                    drop(b);
                                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                    continue;
                                }

                                if let Some(page) = &b.page {
                                    current_url = page.url().await.ok().flatten()
                                        .unwrap_or_else(|| "about:blank".to_string());
                                    
                                    // Get skill-aware page summary (includes block detection + platform context)
                                    page_summary = perceive_summary(page, &current_url).await;
                                    
                                    // Log block signals immediately if detected
                                    if page_summary.contains("[BLOCKED:") {
                                        println!("🚨 BLOCK SIGNAL DETECTED at: {}", current_url);
                                    } else if page_summary.contains("[PLATFORM:") {
                                        println!("📍 Platform identified for: {}", current_url);
                                    }
                                    
                                    println!("👁️ Page: {} | {}", current_url, &page_summary[..page_summary.len().min(120)]);
                                    
                                    // Try to get UI elements with retries
                                    for i in 0..3 {
                                        match perceive(page).await {
                                            Ok(list) if !list.is_empty() => {
                                                ui_list = list;
                                                break;
                                            }
                                            Ok(_) => {
                                                println!("⚠️ UI Empty. Waiting 1s (Attempt {}/3)...", i + 1);
                                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                            }
                                            Err(e) => {
                                                println!("! Perception error: {}", e);
                                                break;
                                            }
                                        }
                                    }
                                }

                                if matches!(macro_state, MacroState::Planning) {
                                    macro_state = MacroState::AcquireSurface;
                                }

                                if matches!(macro_state, MacroState::AcquireSurface)
                                    && current_url != "about:blank"
                                    && !current_url.is_empty()
                                    && !ui_list.is_empty()
                                {
                                    macro_state = MacroState::InspectSurface;
                                    last_action_res = format!(
                                        "Page surface acquired at {}. Visible UI is available. Inspect the page and choose the next interaction.",
                                        current_url
                                    );
                                }

                                // Hard reload recovery if UI still empty on a real page
                                if ui_list.is_empty() && current_url != "about:blank" && !current_url.is_empty() {
                                    println!("! UI empty on non-blank page. Attempting HARD RELOAD...");
                                    if let Some(p) = &b.page {
                                        let _ = p.reload().await;
                                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                        if let Ok(list) = perceive(p).await {
                                            ui_list = list;
                                        }
                                        page_summary = perceive_summary(p, &current_url).await;
                                    }
                                }

                                if ui_list.is_empty() && current_url != "about:blank" {
                                    println!("! UI STILL empty. Asking user.");
                                    let question = "I'm having trouble seeing the page. Is it showing a CAPTCHA, blank screen, or normal homepage?".to_string();
                                    let _ = tx_agent_loop.send(WsResponse {
                                        msg_type: "action".into(),
                                        t: None,
                                        speech: Some("I can't see anything on this page. It might be a bot check or still loading. Can you help?".into()),
                                        action: Some(Action::Ask { question: question.clone() }),
                                        thought: None,
                                        agent_mode: Some(AgentMode::Operator),
                                        macro_state: Some(MacroState::WaitingForUser),
                                        tool_name: Some(ToolName::Ask),
                                    });
                                    let (tx_ask, mut rx_ask) = tokio::sync::mpsc::channel(1);
                                    {
                                        let mut ng = ask_notifier_agent.lock().await;
                                        *ng = Some(tx_ask);
                                    }
                                    drop(b);
                                    match rx_ask.recv().await {
                                        Some(reply) => {
                                            println!("▶️ AGENT RESUMED AFTER PERCEPTION HELP: {}", reply);
                                            last_action_res = format!(
                                                "User says the page state is: {}. Re-perceive the current page and continue from there.",
                                                reply
                                            );
                                            macro_state = MacroState::InspectSurface;
                                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                            continue;
                                        }
                                        None => break,
                                    }
                                }

                                // --- DECIDE (Cognition) ---
                                executor_agent.set_ui(ui_list.clone());
                                let decision = match brain_agent.decide_agent(
                                    &refined_goal,
                                    &plan,
                                    current_plan_step,
                                    &history_str_agent,
                                    &macro_state,
                                    &current_url,
                                    &page_summary,
                                    &ui_list,
                                    &last_action_res,
                                    tx_agent_loop.clone()
                                ).await {
                                    Ok(d) => {
                                        invalid_decision_count = 0;
                                        d
                                    }
                                    Err(e) => {
                                        invalid_decision_count += 1;
                                        println!("! Agent Brain failed: {}. Injecting recovery hint and retrying...", e);
                                        if invalid_decision_count >= 2 {
                                            let question = "I'm stuck because my next action keeps coming back malformed. Can you rephrase the request or tell me the next move?".to_string();
                                            let _ = tx_agent_loop.send(WsResponse {
                                                msg_type: "action".into(),
                                                t: None,
                                                speech: Some(question.clone()),
                                                action: Some(Action::Ask { question }),
                                                thought: None,
                                                agent_mode: Some(AgentMode::Operator),
                                                macro_state: Some(MacroState::WaitingForUser),
                                                tool_name: Some(ToolName::Ask),
                                            });
                                            break;
                                        }
                                        last_action_res = format!(
                                            "⚠️ {} \
                                            Output exactly one supported JSON action with no extra text.",
                                            e
                                        );
                                        drop(b);
                                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                        continue; // retry this OODA step, don't kill the loop
                                    }
                                };

                                // --- COMMUNICATE (speech + thought to UI) ---
                                let tool_call = ToolCall {
                                    tool: decision.tool.clone(),
                                    arguments: decision.arguments.clone(),
                                };
                                let action = match MomentumBrain::validate_tool_call(&tool_call, &ui_list, &macro_state) {
                                    Ok(action) => action,
                                    Err(e) => {
                                        invalid_decision_count += 1;
                                        last_action_res = format!("Contract violation: {}. Choose a legal tool for {:?}.", e, macro_state);
                                        println!("! Tool contract failed: {}", e);
                                        drop(b);
                                        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                                        continue;
                                    }
                                };

                                println!("🗣️ Agent: {}", decision.external);
                                let _ = tx_agent_loop.send(WsResponse {
                                    msg_type: "action".into(),
                                    t: None,
                                    speech: Some(decision.external.clone()),
                                    action: Some(action.clone()),
                                    thought: Some(decision.internal.clone()),
                                    agent_mode: Some(AgentMode::Operator),
                                    macro_state: Some(macro_state.clone()),
                                    tool_name: Some(decision.tool.clone()),
                                });

                                // --- ACT & VERIFY ---
                                match action.clone() {
                                    Action::Achievement { reason } => {
                                        println!("🏁 TASK COMPLETE: {}", reason);
                                        macro_state = MacroState::Done;
                                        break;
                                    }
                                    Action::Ask { question } => {
                                        println!("⏸️ AGENT PAUSED: Waiting for reply to: {}", question);
                                        macro_state = MacroState::WaitingForUser;
                                        let (tx_ask, mut rx_ask) = tokio::sync::mpsc::channel(1);
                                        {
                                            let mut ng = ask_notifier_agent.lock().await;
                                            *ng = Some(tx_ask);
                                        }
                                        if let Some(reply) = rx_ask.recv().await {
                                            println!("▶️ AGENT RESUMED: User replied: {}", reply);
                                            last_action_res = format!("User answered: {}", reply);
                                            macro_state = MacroState::ActOnSurface;
                                        }
                                    }
                                    _ => {
                                        if b.page.is_none() {
                                            if let Err(e) = b.ensure_launched().await {
                                                println!("! Failed to launch: {}", e);
                                                break;
                                            }
                                        }

                                        match executor_agent.execute(&b, &action).await {
                                            Ok(res) => {
                                                println!("✅ EXEC: {}", res.summary);
                                                let action_key = format!("{:?}", action);
                                                let transition = evaluate_transition(&macro_state, &action, &res);
                                                macro_state = transition.next_state.clone();

                                                if matches!(action, Action::Wait { .. }) {
                                                    consecutive_repeat_count = 0;
                                                    last_action_key = action_key;
                                                    last_action_res = res.summary.clone();
                                                } else if res.verification_passed {
                                                    consecutive_repeat_count = 0;
                                                    last_action_key = action_key;
                                                    last_action_res = res.summary.clone();
                                                    if matches!(transition.next_state, MacroState::InspectSurface | MacroState::VerifyOutcome)
                                                        && current_plan_step + 1 < plan.len()
                                                    {
                                                        current_plan_step += 1;
                                                        println!("📋 Advancing to plan step {}: {}", 
                                                            current_plan_step + 1,
                                                            plan.get(current_plan_step).cloned().unwrap_or_default()
                                                        );
                                                    }
                                                } else {
                                                    if action_key == last_action_key {
                                                        consecutive_repeat_count += 1;
                                                    } else {
                                                        consecutive_repeat_count = 0;
                                                        last_action_key = action_key.clone();
                                                    }

                                                    if consecutive_repeat_count >= 2 {
                                                        let question = "I keep hitting the same thing and the page isn't changing. Can you take a quick look or rephrase the task?".to_string();
                                                        let _ = tx_agent_loop.send(WsResponse {
                                                            msg_type: "action".into(),
                                                            t: None,
                                                            speech: Some(question.clone()),
                                                            action: Some(Action::Ask { question }),
                                                            thought: None,
                                                            agent_mode: Some(AgentMode::Operator),
                                                            macro_state: Some(MacroState::WaitingForUser),
                                                            tool_name: Some(ToolName::Ask),
                                                        });
                                                        break;
                                                    }

                                                    if consecutive_repeat_count >= 1 {
                                                        println!("🔁 LOOP DETECTED: same action repeated with no visible progress.");
                                                        last_action_res = format!(
                                                            "{} RECOVERY: {:?}. Choose a different legal tool for {:?} now.",
                                                            res.summary,
                                                            res.failure_class,
                                                            MacroState::Recover
                                                        );
                                                        macro_state = MacroState::Recover;
                                                    } else {
                                                        last_action_res = format!(
                                                            "{} Failure class: {:?}. No visible progress happened on the page.",
                                                            res.summary,
                                                            res.failure_class
                                                        );
                                                        macro_state = MacroState::VerifyOutcome;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!("⚠️ EXEC FAILED: {}. Retrying after 2s...", e);
                                                last_action_res = format!("Action failed: {}. Try a different approach.", e);
                                                consecutive_repeat_count = 0;
                                                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                            }
                                        }
                                    }
                                }
                                drop(b);
                                tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                            }
                            println!("🏁 Agent Loop Finished.");
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
    use super::cancel_active_task;
    use crate::types::Action;
    use tokio::sync::{broadcast, mpsc, Mutex};
    use std::sync::Arc;

    #[tokio::test]
    async fn cancel_active_task_clears_state_and_emits_cancelled() {
        let active_task = Arc::new(Mutex::new(Some(tokio::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }))));
        let ask_notifier = Arc::new(Mutex::new(Some(mpsc::channel::<String>(1).0)));
        let (tx_agent, mut rx_agent) = broadcast::channel(8);

        let cancelled = cancel_active_task(&active_task, &ask_notifier, &tx_agent).await;

        assert!(cancelled);
        assert!(active_task.lock().await.is_none());
        assert!(ask_notifier.lock().await.is_none());

        let response = rx_agent.recv().await.unwrap();
        match response.action.unwrap() {
            Action::Cancelled { reason } => assert_eq!(reason, "Stopped by user"),
            other => panic!("expected cancelled action, got {:?}", other),
        }
    }
}
