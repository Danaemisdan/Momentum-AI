use llama_cpp_2::{
    llama_backend::LlamaBackend,
    context::params::LlamaContextParams,
    model::{params::LlamaModelParams, LlamaModel, AddBos},
    llama_batch::LlamaBatch,
    sampling::LlamaSampler,
};
use crate::types::{
    Action, AgentDecision, AgentMode, AgentPlan, ClarificationResult, MacroState, ToolCall,
    ToolName, ToolSpec, UIElement, WsResponse,
};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

// LlamaContext is not Send by default, but we can safely send it between threads 
// as long as we don't use it concurrently. Axum/Tokio require Send for state.
pub struct SendContext(llama_cpp_2::context::LlamaContext<'static>);
unsafe impl Send for SendContext {}

pub struct MomentumBrain {
    model: &'static LlamaModel,
    context: Arc<Mutex<SendContext>>,
}

impl MomentumBrain {
    fn default_search_engine_name() -> &'static str {
        crate::browser::MomentumBrowser::default_search_engine_name()
    }

    fn tool_name_wire(tool: &ToolName) -> String {
        serde_json::to_string(tool)
            .unwrap_or_else(|_| "\"wait\"".into())
            .trim_matches('"')
            .to_string()
    }

    fn macro_state_wire(state: &MacroState) -> String {
        serde_json::to_string(state)
            .unwrap_or_else(|_| "\"inspect_surface\"".into())
            .trim_matches('"')
            .to_string()
    }

    fn heuristic_mode(message: &str) -> Option<AgentMode> {
        let normalized = message.trim().to_lowercase();
        } else {
            1
        }
    });

    // Build current plan context
    let current_step = plan.get(current_step_idx)
        .cloned()
        .unwrap_or_else(|| format!("Step {}: Continue toward goal.", current_step_idx + 1));
    
    let remaining_steps: Vec<&String> = plan.iter().skip(current_step_idx + 1).collect();
    let remaining_str = if remaining_steps.is_empty() {
        "This is the final step. Complete it and report to user.".to_string()
    } else {
        remaining_steps.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" → ")
    };
    let legal_tools = Self::legal_tools_for_state(macro_state);

    let ui_prompt = ui_elements.iter()
        .take(60)
        .map(|el| {
            format!(
                "ID: {} | TAG: {} | ROLE: {} | TEXT: {} | HREF: {} | INPUT: {} | CLICKABLE: {}",
                el.id,
                el.tag.as_deref().unwrap_or(""),
                el.role,
                el.text,
                el.href.as_deref().unwrap_or(""),
                el.is_input_like,
                el.is_clickable
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // DYNAMIC SKILL INJECTION
    let plat_skill = crate::skills::SkillLoader::load_platform_skill(current_url);
    let role_skill = crate::skills::SkillLoader::load_role_skill(goal);
    
    let mut extra_prompt = String::new();
    if let Some(ps) = &plat_skill {
        extra_prompt.push_str(&crate::skills::SkillLoader::get_platform_prompt(ps));
    }
    if let Some(rs) = &role_skill {
        extra_prompt.push_str(&crate::skills::SkillLoader::get_role_prompt(rs));
    }

    let skill_id = plat_skill.map(|s| s.platform);
    let role_id = role_skill.map(|r| r.name);

    let tool_spec_lines = Self::tool_specs()
        .into_iter()
        .filter(|spec| legal_tools.iter().any(|tool| tool == &spec.name))
        .map(|spec| {
            format!(
                "- {}: {} Allowed in {:?}. Recovery: {}",
                Self::tool_name_wire(&spec.name), spec.description, spec.allowed_states, spec.recovery_guidance
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let legal_tool_lines = legal_tools
        .iter()
        .map(Self::tool_name_wire)
        .collect::<Vec<_>>()
        .join(", ");

    let prompt = format!(
        concat!(
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n",
            "You are Momentum operating in OPERATOR mode. Keep your sarcastic voice, but obey the contract exactly.\n",
            "Think in OODA, but select exactly one legal tool from the current macro state.\n",
            "Do not invent tools. Do not claim success unless verification supports it. Do not narrate outcomes that have not happened.\n",
            "\n",
            "MODE: operator\n",
            "MACRO STATE: {macro_state:?}\n",
            "LEGAL TOOLS IN THIS STATE: {legal_tools}\n",
            "TOOL RULES:\n{tool_rules}\n",
            "\n",
            "{extra}\n",
            "TASK GOAL: {goal}\n",
            "SEARCH ENGINE DEFAULT: {search_engine}\n",
            "CURRENT PLAN STEP: {current_step}\n",
            "REMAINING STEPS: {remaining}\n",
            "CURRENT URL: {url}\n",
            "PAGE SUMMARY: {summary}\n",
            "LAST ACTION RESULT: {last}\n",
            "EXPECTED TRANSITION: choose the most likely next macro state after this action.\n",
            "\n",
            "VISIBLE UI ELEMENTS:\n{ui}\n",
            "\n",
            "OUTPUT ONLY ONE JSON OBJECT. The model MUST output real values based on the task, NOT copy the examples below:\n",
            "{{\n",
            "  \"internal\": {{\n",
            "    \"intent\": \"<EXECUTE|ASSESS|RECOVER|COMPLETE based on situation>\",\n\n",
            "    \"environment\": \"<BROWSER|DESKTOP|PLATFORM based on current URL>\",\n\n",
            "    \"mode\": \"<SCAN|SEEK|ACT|NAVIGATE|RECOVER|VERIFY based on task>\",\n\n",
            "    \"macro_state\": \"{macro_state_snake}\",\n",
            "    \"step\": \"{step}\",\n",
            "    \"tool\": \"<CHOOSE ONE from legal_tools list above>\",\n",
            "    \"speech\": \"<Brief natural narration of what you're about to do>\",\n",
            "    \"arguments\": {{<see TOOL RULES section for required fields for your chosen tool>}}\n",
            "    \"expected_transition\": \"<next_state_after_this_action>\",\n",
            "    \"perceive\": \"<What UI elements are visible and relevant?>\",\n",
            "  }}\n",
            "}}\n",
            current_step = current_step,
            remaining = remaining_str,
            macro_state = macro_state,
            macro_state_snake = Self::macro_state_wire(macro_state),
            legal_tools = legal_tool_lines,
            tool_rules = tool_spec_lines,
            url = current_url,
            summary = page_summary,
            last = last_action_res,
            ui = ui_prompt,
            step = current_step_idx + 1,
            search_engine = Self::default_search_engine_name(),
            extra = extra_prompt,
        )
    );

    let raw = self.generate(&prompt, None).await?;
    // The assistant was primed with `{` so reconstruct the full JSON
    let with_brace = format!("{{{}", raw.trim());
    let candidates = [with_brace.as_str(), raw.trim()];
    let mut decision_result: Option<crate::types::AgentDecision> = None;
    for candidate in &candidates {
        let clean = candidate.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        if let Some(end) = clean.rfind('}') {
            let sliced = &clean[..=end];
            if let Ok(d) = serde_json::from_str::<crate::types::AgentDecision>(sliced) {
                decision_result = Some(d);
                break;
            }
        }
    }

    #[test]
    fn validate_action_rejects_non_absolute_url() {
        let err = MomentumBrain::validate_action(
            &Action::Navigate {
                url: "/amazon".into(),
            },
            &sample_ui(),
        )
        .unwrap_err();
        assert!(err.contains("absolute URL"));
    }

    #[test]
    fn heuristic_classifier_routes_youtube_watch_request_to_task() {
        let intent = MomentumBrain::heuristic_mode(
            "Can you watch a video with me on youtube then? Find me a biryani tutorial let's watch",
        );
        assert_eq!(intent, Some(AgentMode::Operator));
    }

    #[test]
    fn heuristic_classifier_routes_amazon_find_request_to_task() {
        let intent = MomentumBrain::heuristic_mode("Can you find me some water bottles on amazon?");
        assert_eq!(intent, Some(AgentMode::Operator));
    }

    #[test]
    fn heuristic_classifier_keeps_simple_greeting_as_chat() {
        let intent = MomentumBrain::heuristic_mode("hi");
        assert_eq!(intent, Some(AgentMode::Chat));
    }
}
