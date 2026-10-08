import re

with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# We need to insert the missing functions before `pub async fn decide_chat`
missing_functions = """
    pub async fn clarify_intent(&self, goal: &str, history: &str) -> Result<ClarificationResult, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "<|im_start|>system\\nYou are Momentum's intent clarifier. Decide if the goal is specific enough to act on, or ask one clarifying question.\\n\\nUSER GOAL: {}\\nHISTORY: {}<|im_end|>\\n<|im_start|>assistant\\n{{",
            goal, history
        );
        let raw = self.generate(&prompt, None).await?;
        let with_brace = format!("{{{}", raw.trim());
        if let Some(end) = with_brace.rfind('}') {
            let sliced = &with_brace[..=end];
            if let Ok(res) = serde_json::from_str::<ClarificationResult>(sliced) {
                return Ok(res);
            }
        }
        Ok(ClarificationResult::Proceed { refined_goal: goal.to_string() })
    }

    pub async fn plan_steps(&self, goal: &str, history: &str) -> Result<AgentPlan, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "<|im_start|>system\\nYou are Momentum's planner. Generate a JSON array of strings representing the step-by-step action plan.\\n\\nUSER GOAL: {}\\nHISTORY: {}<|im_end|>\\n<|im_start|>assistant\\n[",
            goal, history
        );
        let raw = self.generate(&prompt, None).await?;
        let with_bracket = format!("[{}", raw.trim());
        if let Some(end) = with_bracket.rfind(']') {
            let sliced = &with_bracket[..=end];
            if let Ok(plan) = serde_json::from_str::<AgentPlan>(sliced) {
                return Ok(plan);
            }
        }
        Ok(vec![format!("Proceed with goal: {}", goal)])
    }

    pub(crate) fn validate_action(action: &Action, ui_list: &[UIElement]) -> Result<(), String> {
        let get_el = |selector: &str| ui_list.iter().find(|el| el.id == selector);
        match action {
            Action::Navigate { url } => {
                let parsed = reqwest::Url::parse(url).map_err(|_| format!("Navigate action requires an absolute URL, got '{}'", url))?;
                match parsed.scheme() {
                    "http" | "https" => Ok(()),
                    _ => Err(format!("Navigate URL must be http or https, got '{}'", url)),
                }
            }
            Action::Click { selector, .. } | Action::Type { selector, .. } => {
                if get_el(selector).is_none() {
                    return Err(format!("Selector '{}' does not exist in current UI", selector));
                }
                Ok(())
            }
            Action::Scroll { direction } => {
                if !["up", "down", "left", "right"].contains(&direction.as_str()) {
                    return Err(format!("Scroll direction must be up, down, left, or right, got '{}'", direction));
                }
                Ok(())
            }
            Action::Wait { .. } | Action::Ask { .. } | Action::Achievement { .. } | Action::Cancelled { .. } | Action::Talk { .. } => Ok(()),
            Action::Perceive { selector } => {
                if let Some(sel) = selector {
                    if get_el(sel).is_none() {
                        return Err(format!("Perceive selector '{}' does not exist in current UI", sel));
                    }
                }
                Ok(())
            }
        }
    }

    pub(crate) fn validate_tool_call(tool_call: &ToolCall, ui_list: &[UIElement], current_state: &MacroState) -> Result<Action, String> {
        let legal_tools = Self::legal_tools_for_state(current_state);
        if !legal_tools.iter().any(|tool| tool == &tool_call.tool) {
            return Err(format!("Tool '{:?}' is not legal in state {:?}", tool_call.tool, current_state));
        }
        let action = match tool_call.tool {
            ToolName::Navigate => {
                let url = tool_call.arguments["url"].as_str().unwrap_or_default();
                Action::Navigate { url: url.to_string() }
            }
            ToolName::Click => {
                let selector = tool_call.arguments["selector"].as_str().unwrap_or_default();
                Action::Click { selector: selector.to_string(), frame_id: None }
            }
            ToolName::Type => {
                let selector = tool_call.arguments["selector"].as_str().unwrap_or_default();
                let text = tool_call.arguments["text"].as_str().unwrap_or_default();
                Action::Type { selector: selector.to_string(), text: text.to_string(), frame_id: None }
            }
            ToolName::Scroll => {
                let direction = tool_call.arguments["direction"].as_str().unwrap_or_default();
                Action::Scroll { direction: direction.to_string() }
            }
            ToolName::Wait => {
                let ms = tool_call.arguments["ms"].as_u64().unwrap_or(1000);
                Action::Wait { ms }
            }
            ToolName::Ask => {
                let question = tool_call.arguments["question"].as_str().unwrap_or_default();
                Action::Ask { question: question.to_string() }
            }
            ToolName::Achievement => {
                let reason = tool_call.arguments["reason"].as_str().unwrap_or_default();
                Action::Achievement { reason: reason.to_string() }
            }
            ToolName::Perceive => {
                let selector = tool_call.arguments["selector"].as_str().map(|s| s.to_string());
                Action::Perceive { selector }
            }
        };
        Self::validate_action(&action, ui_list)?;
        Ok(action)
    }

"""

content = content.replace("pub async fn decide_chat", missing_functions + "\n    pub async fn decide_chat")

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
