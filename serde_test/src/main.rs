use serde::{Deserialize, Serialize, Deserializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub thought: String,
    pub speech: String,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Chat,
    Idle,
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        if let Some(s) = v.as_str() {
            if s == "idle" { return Ok(Action::Idle); }
            return Ok(Action::Chat);
        }
        Ok(Action::Chat)
    }
}

fn main() {
    let json = r#"{
      "thought": "Thinking...",
      "speech": "Hey",
      "action": "idle"
    }"#;
    match serde_json::from_str::<AgentResponse>(json) {
        Ok(r) => println!("SUCCESS: {:?}", r),
        Err(e) => println!("ERROR: {}", e),
    }
}
