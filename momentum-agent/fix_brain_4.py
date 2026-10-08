import re

with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Completely remove pub fn init and async fn generate, then insert pub fn new and async fn generate
# To be robust, let's just find the start of pub fn init and slice it there
init_idx = content.find("pub fn init(model_path: &str)")
if init_idx == -1:
    init_idx = content.find("pub fn new()")

head = content[:init_idx]

tail_idx = content.find("pub async fn decide_chat")
tail = content[tail_idx:]

new_functions = '''pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-dummy".to_string());
        let base_url = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
            model,
        }
    }

    async fn generate(&self, prompt: &str, tx_token: Option<broadcast::Sender<WsResponse>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut messages = Vec::new();
        let mut current_role = String::new();
        let mut current_content = String::new();
        
        for line in prompt.lines() {
            if line.starts_with("<|im_start|>") {
                if !current_role.is_empty() {
                    messages.push(serde_json::json!({
                        "role": current_role,
                        "content": current_content.trim()
                    }));
                }
                current_content.clear();
                let role = line.trim_start_matches("<|im_start|>").trim_end_matches("<|im_end|>").trim_end();
                current_role = role.to_string();
            } else if line.starts_with("<|im_end|>") {
                messages.push(serde_json::json!({
                    "role": current_role,
                    "content": current_content.trim()
                }));
                current_role.clear();
                current_content.clear();
            } else {
                current_content.push_str(line);
                current_content.push('\\n');
            }
        }
        
        if !current_role.is_empty() && !current_content.is_empty() {
            messages.push(serde_json::json!({
                "role": current_role,
                "content": current_content.trim()
            }));
        }

        let is_streaming = tx_token.is_some();

        let payload = serde_json::json!({
            "model": &self.model,
            "messages": messages,
            "stream": is_streaming,
            "temperature": 0.2
        });

        let url = format!("{}/chat/completions", self.base_url);
        
        let mut req = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload);

        if !is_streaming {
            let res = req.send().await?.json::<serde_json::Value>().await?;
            if let Some(content) = res["choices"][0]["message"]["content"].as_str() {
                return Ok(content.to_string());
            } else {
                return Err(format!("Invalid response from API: {:?}", res).into());
            }
        } else {
            let mut res = req.send().await?;
            let mut full_response = String::new();
            
            while let Some(chunk) = res.chunk().await? {
                let chunk_str = String::from_utf8_lossy(&chunk);
                for line in chunk_str.lines() {
                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if data.trim() == "[DONE]" { continue; }
                        
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                full_response.push_str(content);
                                
                                if let Some(tx) = &tx_token {
                                    let _ = tx.send(WsResponse { 
                                        msg_type: "token".into(), 
                                        t: Some(content.to_string()), 
                                        speech: None, 
                                        action: None,
                                        thought: None,
                                        agent_mode: None,
                                        macro_state: None,
                                        tool_name: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            return Ok(full_response);
        }
    }

    '''

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(head + new_functions + tail)
