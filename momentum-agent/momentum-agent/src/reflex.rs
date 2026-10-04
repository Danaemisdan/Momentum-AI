use crate::brain::MomentumBrain;
use crate::perception::perceive_native;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use std::process::Command;

/// The Spinal Cord: A continuous, zero-JSON background loop that bypasses the OODA overhead.
/// It rips the native UI tree every second, feeds it to the Tiny LLM, and instantly executes physical clicks.
pub async fn start_spinal_cord(micro_goal: String, brain: Arc<MomentumBrain>) {
    println!("SPINAL CORD ONLINE. MICRO-GOAL: {}", micro_goal);
    
    // We limit to 5 iterations to prevent infinite loops and spam clicking
    for iteration in 0..5 {
        println!("Reflex Loop iteration {}...", iteration);
        
        // 1. Perceive Native OS State
        let ui_elements = match perceive_native().await {
            Ok(elements) => elements,
            Err(e) => {
                println!("Spinal Cord Perception Error: {}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        // 2. Compress UI for Tiny LLM Reflexes, grouped by spatial zones
        use std::collections::HashMap;
        let mut zones: HashMap<String, Vec<&crate::types::UIElement>> = HashMap::new();
        for el in ui_elements.iter().take(50) { // Native UI can be large, take top 50
            let z = el.spatial_zone.clone().unwrap_or_else(|| "Main Content".to_string());
            zones.entry(z).or_default().push(el);
        }
        
        let mut grouped_lines = Vec::new();
        let order = ["Top Navigation", "Left Sidebar", "Right Sidebar", "Main Content", "Bottom Bar"];
        for z in order {
            if let Some(els) = zones.get(z) {
                grouped_lines.push(format!("\n[{}]", z));
                for el in els {
                    let intent_str = match &el.semantic_intent {
                        Some(intent) => format!(" | {{{}}}", intent),
                        None => "".to_string(),
                    };
                    grouped_lines.push(format!("{} | {} | {}{}", el.id, el.role, el.text, intent_str));
                }
            }
        }
        let ui_prompt = grouped_lines.join("\n").trim().to_string();

        let prompt = format!(
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\
            You are the spinal cord reflex engine. You execute instant physical actions.\n\
            GOAL: {goal}\n\
            UI:\n{ui}\n\
            OUTPUT STRICTLY ONE OF:\n\
            c_ID (to click)\n\
            t_ID_text (to type)\n\
            DONE (if goal is met)\n\
            NO JSON. NO EXPLANATION. JUST THE COMMAND. Example: c_native:el_8\n\
            <|eot_id|><|start_header_id|>assistant<|end_header_id|>\n",
            goal = micro_goal,
            ui = ui_prompt
        );

        // 3. Generate reflex token (Fastest possible TTFT)
        let result = brain.generate(&prompt, None).await;

        match result {
            Ok(res) => {
                let action = res.trim();
                println!("Reflex Output: {}", action);

                // 4. Physical Execution Mapping
                if action.starts_with("c_") {
                    let id = action.strip_prefix("c_").unwrap_or("");
                    if let Some(el) = ui_elements.iter().find(|e| e.id == id || e.selector == id) {
                        println!("Physical Reflex: Clicking {} at ({}, {})", el.id, el.x, el.y);
                        // Fire native physical click via mac_click script
                        let _ = Command::new("./scripts/mac_click")
                            .arg(el.x.to_string())
                            .arg(el.y.to_string())
                            .output();
                        println!("Spinal cord micro-goal executed.");
                        break;
                    } else {
                        println!("Reflex Element {} not found in UI tree.", id);
                        break;
                    }
                } else if action.starts_with("t_") {
                    // TODO: implement typing script if needed, for now just break
                    println!("Reflex Type triggered, breaking loop.");
                    break;
                } else if action == "DONE" {
                    println!("Spinal cord micro-goal met.");
                    break;
                } else {
                    println!("Unrecognized reflex output, breaking loop.");
                    break;
                }
            },
            Err(e) => {
                println!("Reflex Generate Error: {}", e);
            }
        }

        // Wait a tiny bit for the UI to settle after a physical click before looping
        sleep(Duration::from_millis(1500)).await;
    }
}
