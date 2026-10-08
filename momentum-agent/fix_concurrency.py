import re
with open('src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Add active_chat_task
content = content.replace('let active_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));', 
                          'let active_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));\n    let active_chat_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));')

# Modify the AgentMode::Chat block to use active_chat_task
chat_block_old = '''                        tokio::spawn(async move {
                            let mut msg_with_vision = message.clone();'''

chat_block_new = '''                        let mut active_chat_guard = active_chat_task.lock().await;
                        if let Some(handle) = active_chat_guard.take() {
                            println!("Cancelling previous chat task due to new message.");
                            handle.abort();
                        }
                        
                        let chat_handle = tokio::spawn(async move {
                            let mut msg_with_vision = message.clone();'''

content = content.replace(chat_block_old, chat_block_new)

# Add the active_chat_task clone inside the receive loop
loop_old = '''                let mut history_str = String::new();
                for msg in &ws_msg.history {'''
loop_new = '''                let active_chat_task = active_chat_task.clone();
                let mut history_str = String::new();
                for msg in &ws_msg.history {'''

content = content.replace(loop_old, loop_new)

# Save the chat_handle
chat_end_old = '''                                });
                            }

                            match brain_chat.decide_chat(&msg_with_vision, &history_str, tx_chat_resp.clone()).await {
                                Ok(mut speech) => {
                                    // Extract and run [ACTION: OPEN_APP(...)]
                                    if let Some(start_idx) = speech.find("[ACTION: OPEN_APP(") {
                                        if let Some(end_idx) = speech[start_idx..].find(")]") {
                                            let app_name = &speech[start_idx + 18..start_idx + end_idx];
                                            println!("🚀 Opening application: {}", app_name);
                                            let _ = std::process::Command::new("open")
                                                .arg("-a")
                                                .arg(app_name)
                                                .spawn();
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
                                        action: Some(Action::Talk { speech }),
                                        thought: None,
                                        agent_mode: Some(AgentMode::Chat),
                                        macro_state: None,
                                        tool_name: None,
                                    });
                                }
                                Err(e) => eprintln!("❌ Chat Generation Error: {}", e),
                            }
                        });
                    }
                    AgentMode::Action => {'''

chat_end_new = '''                                });
                            }

                            match brain_chat.decide_chat(&msg_with_vision, &history_str, tx_chat_resp.clone()).await {
                                Ok(mut speech) => {
                                    // Extract and run [ACTION: OPEN_APP(...)]
                                    if let Some(start_idx) = speech.find("[ACTION: OPEN_APP(") {
                                        if let Some(end_idx) = speech[start_idx..].find(")]") {
                                            let app_name = &speech[start_idx + 18..start_idx + end_idx];
                                            println!("🚀 Opening application: {}", app_name);
                                            let _ = std::process::Command::new("open")
                                                .arg("-a")
                                                .arg(app_name)
                                                .spawn();
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
                                        action: Some(Action::Talk { speech }),
                                        thought: None,
                                        agent_mode: Some(AgentMode::Chat),
                                        macro_state: None,
                                        tool_name: None,
                                    });
                                }
                                Err(e) => eprintln!("❌ Chat Generation Error: {}", e),
                            }
                        });
                        *active_chat_guard = Some(chat_handle);
                    }
                    AgentMode::Action => {'''

content = content.replace(chat_end_old, chat_end_new)

with open('src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)
