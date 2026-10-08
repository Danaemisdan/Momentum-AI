import re

with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Remove llama_cpp_2 imports
content = re.sub(r'use llama_cpp_2::\{[\s\S]*?\};\n', '', content)
content = re.sub(r'pub struct SendContext.*?unsafe impl Send for SendContext \{\}\n', '', content)

# Change MomentumBrain struct
old_struct = r'pub struct MomentumBrain \{.*?context: Arc<Mutex<SendContext>>,\n\}'
new_struct = '''pub struct MomentumBrain {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}'''
content = re.sub(old_struct, new_struct, content, flags=re.DOTALL)

# Change MomentumBrain::new
old_new = r'pub fn new\(.*?\) -> Result<Self, Box<dyn std::error::Error \+ Send \+ Sync>> \{[\s\S]*?Ok\(Self \{[\s\S]*?\}\)\n    \}'
new_new = '''pub fn new() -> Self {
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set in .env or environment");
        
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        let base_url = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
            model,
        }
    }'''
content = re.sub(old_new, new_new, content, flags=re.DOTALL)

# Rewrite generate function
old_generate = r'async fn generate\(&self, prompt: &str, tx_token: Option<broadcast::Sender<WsResponse>>\) -> Result<String,\nBox<dyn std::error::Error \+ Send \+ Sync>> \{[\s\S]*?Ok\(response\)\n    \}'
new_generate = '''async fn generate(&self, prompt: &str, tx_token: Option<broadcast::Sender<WsResponse>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use futures::stream::StreamExt;
        
        // Parse the ChatML formatted prompt into OpenAI message format
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
                current_content.push('\n');
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
            "model": self.model,
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
                        if data == "[DONE]" { continue; }
                        
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

# Note: Using re.DOTALL is important for multiline matching
import re
content = re.sub(old_generate, new_generate, content, flags=re.DOTALL)

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
