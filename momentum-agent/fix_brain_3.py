import os

with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# We need to completely rewrite brain.rs to remove llama_cpp_2 and use reqwest
import urllib.request
# Just use a robust string replacement to extract the skeleton of brain.rs and inject the new generate/new functions.
# The simplest way is to manually extract the blocks.

# 1. Remove Llama related imports
lines = content.split('\n')
new_lines = []
skip = False
for line in lines:
    if "use llama_cpp_2::" in line:
        skip = True
    if skip and "};" in line:
        skip = False
        continue
    if not skip:
        new_lines.append(line)

content = '\n'.join(new_lines)

# 2. Re-write the MomentumBrain struct
import re
content = re.sub(r'pub struct MomentumBrain \{.*?\}', '''pub struct MomentumBrain {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}''', content, flags=re.DOTALL)

# 3. Replace MomentumBrain::new
new_fn_pattern = r'pub fn new\(.*?\) -> Result<Self, Box<dyn std::error::Error \+ Send \+ Sync>> \{.*?Ok\(Self \{.*?\}\)\n    \}'
new_fn_replacement = '''pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-dummy".to_string());
        let base_url = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
            model,
        }
    }'''
content = re.sub(new_fn_pattern, new_fn_replacement, content, flags=re.DOTALL)

# 4. Replace generate function
generate_fn_pattern = r'async fn generate\(&self, prompt: &str, tx_token: Option<broadcast::Sender<WsResponse>>\) -> Result<String, Box<dyn std::error::Error \+ Send \+ Sync>> \{.*?Ok\(response\)\n    \}'
generate_fn_replacement = '''async fn generate(&self, prompt: &str, tx_token: Option<broadcast::Sender<WsResponse>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    }'''
content = re.sub(generate_fn_pattern, generate_fn_replacement, content, flags=re.DOTALL)

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
