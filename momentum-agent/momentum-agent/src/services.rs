use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use crate::brain::MomentumBrain;
use crate::browser::MomentumBrowser;
use crate::types::WsResponse;
use tokio::sync::broadcast;

pub struct ServiceManager {
    active_services: HashMap<String, JoinHandle<()>>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            active_services: HashMap::new(),
        }
    }

    pub async fn spawn_service(&mut self, name: String, handle: JoinHandle<()>) {
        // If a service with the same name exists, kill it first
        self.kill_service(&name).await;
        self.active_services.insert(name, handle);
    }

    pub async fn kill_service(&mut self, name: &str) -> bool {
        if let Some(handle) = self.active_services.remove(name) {
            handle.abort();
            true
        } else {
            false
        }
    }

    pub async fn list_services(&self) -> Vec<String> {
        self.active_services.keys().cloned().collect()
    }
}

pub async fn run_service(
    brain: Arc<MomentumBrain>,
    _browser: Arc<Mutex<MomentumBrowser>>,
    tx_agent: broadcast::Sender<WsResponse>,
    objective: String,
    _headless: bool
) {
    println!("🚀 Started Background Service: {}", objective);
    loop {
        // Evaluate every 30 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        
        let prompt = format!(
            "You are a background service agent for Momentum. Your objective is: '{}'.\n\
             Based on your internal knowledge and background capability, evaluate if you have completed the objective or found the information.\n\
             If you have an update or result for the user, output exactly: NOTIFY: <message>\n\
             If you need more time or found nothing yet, output exactly: WAIT",
            objective
        );
        
        if let Ok(response) = brain.generate(&prompt, None).await {
            if response.contains("NOTIFY:") {
                let msg = response.replace("NOTIFY:", "").trim().to_string();
                let _ = tx_agent.send(WsResponse {
                    msg_type: "action".into(),
                    t: None,
                    speech: Some(format!("🔔 [Service]: {}", msg)),
                    action: None,
                    thought: None,
                    agent_mode: None,
                    macro_state: None,
                    tool_name: None,
                });
                break; // End service once objective is reported
            }
        }
    }
    println!("🛑 Background Service Ended: {}", objective);
}
