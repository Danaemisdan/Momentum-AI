use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    Chat,
    Task,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentMode {
    Chat,
    Operator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MacroState {
    Planning,
    AcquireSurface,
    InspectSurface,
    ActOnSurface,
    VerifyOutcome,
    Recover,
    WaitingForUser,
    Done,
    Cancelled,
}

/// Result of Phase 1: intent clarification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClarificationResult {
    /// Goal is specific enough — proceed with this refined goal
    Proceed { refined_goal: String },
    /// Need to ask the user one question before proceeding
    Ask { question: String },
}

/// An ordered list of concrete plan steps (Phase 2 output)
pub type AgentPlan = Vec<String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    pub id: String,     // e.g. "el_42"
    pub role: String,   // e.g. "button"
    pub text: String,   // e.g. "Submit"
    pub selector: String,
    pub x: f64,
    pub y: f64,
    pub href: Option<String>,
    pub title: Option<String>,
    pub tag: Option<String>,
    pub aria_label: Option<String>,
    pub placeholder: Option<String>,
    pub is_clickable: bool,
    pub is_input_like: bool,
    pub container: Option<String>,       // UI hierarchy container (e.g. "web document", "tabs", "toolbar", "timeline")
    pub semantic_label: String,          // Unified rich label: role + title + placeholder + description + container
    pub frame_id: Option<String>, // Which frame this belongs to
    pub semantic_intent: Option<String>, // Heuristic mapping (e.g. 'Navigate', 'Search')
    pub spatial_zone: Option<String>, // Dynamic geometric clustering zone (e.g. 'Left Sidebar')
}

impl UIElement {
    /// Computes a unified, rich semantic label describing the element, what it does, and where it lives.
    pub fn compute_semantic_label(
        role: &str,
        text: &str,
        title: Option<&str>,
        aria_label: Option<&str>,
        placeholder: Option<&str>,
        container: Option<&str>,
    ) -> String {
        let role_lower = role.to_lowercase();
        let clean_role = match role_lower.as_str() {
            "axbutton" | "button" => "button",
            "axtextfield" | "axtextarea" | "edit" | "input" => "input field",
            "axsearchfield" | "searchbox" => "search field",
            "axlink" | "link" | "hyperlink" => "link",
            "axcombobox" | "combobox" => {
                if text.to_lowercase().contains("search") || placeholder.map(|p| p.to_lowercase().contains("search")).unwrap_or(false) {
                    "search field"
                } else {
                    "input field"
                }
            },
            "axpopupbutton" => "dropdown",
            "axcheckbox" | "checkbox" => "checkbox",
            "axradiobutton" | "radiobutton" => "radio button",
            "axmenuitem" | "menuitem" => "menu item",
            "axtabitem" | "tabitem" => "tab",
            "axheading" | "heading" => "heading",
            "aximage" | "image" => "image",
            other => other.strip_prefix("ax").unwrap_or(other),
        };

        let mut parts: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Prioritize primary text, title, aria_label
        for cand_opt in [Some(text), title, aria_label] {
            if let Some(cand) = cand_opt {
                let trimmed = cand.trim();
                let lower = trimmed.to_lowercase();
                if !trimmed.is_empty() && !seen.contains(&lower) {
                    seen.insert(lower);
                    parts.push(trimmed.to_string());
                }
            }
        }

        // If no primary name was provided, fallback to placeholder / help text
        if parts.is_empty() {
            if let Some(cand) = placeholder {
                let trimmed = cand.trim();
                let lower = trimmed.to_lowercase();
                if !trimmed.is_empty() && !seen.contains(&lower) {
                    seen.insert(lower);
                    parts.push(trimmed.to_string());
                }
            }
        }

        let main_desc = if parts.is_empty() {
            clean_role.to_string()
        } else {
            format!("{} {}", parts.join(" "), clean_role)
        };

        if let Some(c) = container {
            let c_trimmed = c.trim();
            if !c_trimmed.is_empty() && c_trimmed != "window" {
                return format!("{} in {}", main_desc, c_trimmed);
            }
        }

        main_desc
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    Navigate { url: String },
    Click { selector: String, #[serde(default)] frame_id: Option<String> },
    Type { selector: String, text: String, #[serde(default)] frame_id: Option<String> },
    Scroll { direction: String },
    Wait { ms: u64 },
    Achievement { reason: String }, // Goal met
    Cancelled { reason: String },   // Task was stopped
    Talk { speech: String },        // Chat persona output
    Ask { question: String },       // Chat asking user
    Perceive { #[serde(default)] selector: Option<String> }, // Foveated localized vision
    StartReflex { micro_goal: String }, // Triggers the Tiny LLM continuous spinal cord loop
    SpawnService { name: String, objective: String, headless: bool },
    KillService { name: String },
    ListServices,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolName {
    Navigate,
    Click,
    Type,
    Scroll,
    Wait,
    Ask,
    Achievement,
    Perceive,
    StartReflex,
    SpawnService,
    KillService,
    ListServices,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolFailureClass {
    NoEffect,
    WrongTarget,
    Blocked,
    PageNotReady,
    ContractViolation,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: ToolName,
    pub description: String,
    pub allowed_states: Vec<MacroState>,
    pub requires_visible_target: bool,
    pub requires_input_like_target: bool,
    pub requires_non_blank_page: bool,
    pub success_signals: Vec<String>,
    pub failure_signals: Vec<String>,
    pub recovery_guidance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionResult {
    pub next_state: MacroState,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: ToolName,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" | "agent"
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInput {
    pub history: Vec<ChatMessage>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ControlCommand {
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ControlSource {
    Text,
    Voice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClientMessage {
    UserInput {
        history: Vec<ChatMessage>,
        message: String,
    },
    InterimInput {
        history: Vec<ChatMessage>,
        text: String,
    },
    Control {
        command: ControlCommand,
        source: ControlSource,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskIntent {
    Acquire,
    Create,
    Modify,
    Communicate,
    Execute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Environment {
    None,
    Browser,
    Local,
    Platform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InteractionMode {
    Scan,
    Seek,
    ScrollToReveal,
    WaitForStability,
    Act,
    Extract,
    Navigate,
    BypassChallenge,
    Recover,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentThought {
    pub intent: String,      // ACQUIRE | CREATE | MODIFY | COMMUNICATE | EXECUTE (or anything model outputs)
    pub environment: String, // BROWSER | LOCAL | PLATFORM | NONE
    pub mode: Option<String>, // SCAN | SEEK | ACT | NAVIGATE | RECOVER etc.
    pub step: String,
    pub macro_state: MacroState,
    pub allowed_tools: Vec<ToolName>,
    pub expected_transition: MacroState,
    pub understand: String,
    pub perceive: String,
    pub orient: String,
    pub decide: String,
    pub verify: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDecision {
    #[serde(rename = "internal")]
    pub internal: AgentThought, // Silent Reasoning
    #[serde(rename = "speech")]
    pub external: String,       // What the agent says aloud
    pub tool: ToolName,
    pub arguments: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    #[serde(rename = "type")]
    pub msg_type: String, // "token", "action", "error", "thought"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>, // token char
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech: Option<String>, // UI Captions / Kokoro
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<AgentThought>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_mode: Option<AgentMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macro_state: Option<MacroState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<ToolName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPattern {
    pub name: String,
    pub strategy: Vec<String>,
    pub reference_patterns: Vec<String>,
    pub decision_logic: Vec<String>,
    pub actions: Vec<String>,
    pub verification: Vec<String>,
    pub recovery: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSkill {
    pub platform: String,
    #[serde(default)]
    pub system_behavior: String,
    pub intents: Vec<IntentPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleContext {
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformKnowledge {
    pub ui_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSkill {
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub websites: Vec<String>,
    #[serde(default)]
    pub reference_patterns: Vec<serde_json::Value>, 
    #[serde(default)]
    pub decision_logic: String,
    #[serde(default)]
    pub constraints: String,
    #[serde(default)]
    pub verification: String,
    #[serde(default)]
    pub recovery: String,
    
    // Legacy support for SkillLoader's current logic
    #[serde(default)]
    pub context: Option<RoleContext>,
    #[serde(default)]
    pub platform_knowledge: Option<PlatformKnowledge>,
    #[serde(default)]
    pub capability: String,
}

#[cfg(test)]
mod tests {
    use super::UIElement;

    #[test]
    fn test_compute_semantic_label_tab_search() {
        let label = UIElement::compute_semantic_label(
            "AXButton",
            "Tab search",
            Some("Search your open tabs"),
            None,
            None,
            Some("tabs"),
        );
        assert_eq!(label, "Tab search Search your open tabs button in tabs");
    }

    #[test]
    fn test_compute_semantic_label_web_search() {
        let label = UIElement::compute_semantic_label(
            "AXSearchField",
            "Search",
            Some("Search by title, skill, or company"),
            None,
            Some("Search"),
            Some("web document"),
        );
        assert_eq!(label, "Search Search by title, skill, or company search field in web document");
    }

    #[test]
    fn test_compute_semantic_label_creative_tool() {
        let label = UIElement::compute_semantic_label(
            "AXButton",
            "Razor Blade",
            Some("Split clip at playhead"),
            None,
            None,
            Some("timeline"),
        );
        assert_eq!(label, "Razor Blade Split clip at playhead button in timeline");
    }
}
