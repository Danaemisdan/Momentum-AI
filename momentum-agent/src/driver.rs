use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    pub action: String, // click, type, press_key, scroll, shell
    pub ui_role: Option<String>,
    pub ui_name: Option<String>,
    pub input: Option<String>,
    pub key: Option<String>,
    pub command: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub actions: Vec<ActionStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateGraph {
    pub states: Vec<String>,
    pub transitions: Vec<Transition>,
}

pub struct ActionDriver {
    pub graph: StateGraph,
    pub current_os: String,
}

impl ActionDriver {
    pub fn new(os: &str) -> Self {
        let rules_str = fs::read_to_string("memory/rules.json")
            .expect("Failed to load procedural memory (rules.json)");
        let graph: StateGraph = serde_json::from_str(&rules_str)
            .expect("Invalid rules.json format");
        
        Self {
            graph,
            current_os: os.to_string(),
        }
    }

    pub fn get_all_states(&self) -> Vec<String> {
        self.graph.states.clone()
    }

    pub fn find_shortest_path(&self, start: &str, target: &str) -> Option<Vec<Transition>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut came_from = HashMap::new();

        queue.push_back(start.to_string());
        visited.insert(start.to_string());

        while let Some(current) = queue.pop_front() {
            if current == target { break; }

            for trans in &self.graph.transitions {
                if trans.from == current && !visited.contains(&trans.to) {
                    visited.insert(trans.to.clone());
                    came_from.insert(trans.to.clone(), (current.clone(), trans.clone()));
                    queue.push_back(trans.to.clone());
                }
            }
        }

        if !came_from.contains_key(target) { return None; }

        let mut path = Vec::new();
        let mut curr = target.to_string();
        while curr != start {
            let (prev, trans) = came_from.remove(&curr).unwrap();
            path.push(trans);
            curr = prev;
        }
        path.reverse();
        Some(path)
    }

    pub fn execute_step(&self, step: &ActionStep, target_x: Option<f64>, target_y: Option<f64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use enigo::{Direction::{Click, Press, Release}, Enigo, Key, Keyboard, Mouse, Settings};
        let mut enigo = Enigo::new(&Settings::default()).unwrap();

        match step.action.as_str() {
            "shell" => {
                if let Some(cmd) = &step.command {
                    println!("    💻 Running shell: {}", cmd);
                    let mut parts = cmd.split_whitespace();
                    if let Some(prog) = parts.next() {
                        let _ = std::process::Command::new(prog)
                            .args(parts)
                            .spawn();
                    }
                }
            },
            "click" => {
                if let (Some(x), Some(y)) = (target_x, target_y) {
                    println!("    🖱️ Clicking {}, {}", x, y);
                    let _ = enigo.move_mouse(x as i32, y as i32, enigo::Coordinate::Abs);
                    let _ = enigo.button(enigo::Button::Left, Click);
                } else {
                    println!("    🖱️ Blind click (no target)");
                    let _ = enigo.button(enigo::Button::Left, Click);
                }
            },
            "type" => {
                if let Some(text) = &step.input {
                    println!("    ⌨️ Typing: {}", text);
                    let _ = enigo.text(text);
                }
            },
            "press_key" => {
                if let Some(k) = &step.key {
                    println!("    🎹 Pressing: {}", k);
                    let enigo_key = match k.to_lowercase().as_str() {
                        "enter" => Key::Return,
                        "tab" => Key::Tab,
                        "escape" => Key::Escape,
                        "space" => Key::Space,
                        _ => Key::Unicode(k.chars().next().unwrap_or(' ')),
                    };
                    let _ = enigo.key(enigo_key, Click);
                }
            },
            _ => { println!("    ⚠️ Unknown action: {}", step.action); }
        }

        if let Some(ms) = step.timeout_ms {
            thread::sleep(Duration::from_millis(ms));
        } else {
            thread::sleep(Duration::from_millis(100)); // Default debounce
        }

        Ok(())
    }
}
