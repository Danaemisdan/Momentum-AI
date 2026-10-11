use crate::types::{PlatformSkill, RoleSkill};
use std::fs;
use std::path::{Path, PathBuf};

pub struct SkillLoader;

impl SkillLoader {
    fn find_yaml_files(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    files.extend(Self::find_yaml_files(&path));
                } else if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    files.push(path);
                }
            }
        }
        files
    }

    pub fn load_platform_skill(url: &str) -> Option<PlatformSkill> {
        let files = Self::find_yaml_files(Path::new("skills/platforms"));
        for path in files {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(skill) = serde_yaml::from_str::<PlatformSkill>(&content) {
                    if url.contains(&skill.platform) {
                        return Some(skill);
                    }
                }
            }
        }
        None
    }

    pub fn load_role_skill(goal: &str) -> Option<RoleSkill> {
        let files = Self::find_yaml_files(Path::new("skills/roles"));
        let goal_lower = goal.to_lowercase();
        
        for path in files {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(role) = serde_yaml::from_str::<RoleSkill>(&content) {
                    // Match if goal contains the role name (e.g., "growth hacker") or capability
                    if goal_lower.contains(&role.role.to_lowercase()) || 
                       goal_lower.contains(&role.capability.to_lowercase()) {
                        return Some(role);
                    }
                }
            }
        }
        None
    }

    pub fn get_platform_prompt(skill: &PlatformSkill) -> String {
        let mut prompt = format!("\n### 🌍 PLATFORM SKILL: {}\n{}\n", skill.platform, skill.system_behavior);
        
        prompt.push_str("\nAVAILABLE INTENTS & BEHAVIORS:\n");
        for intent in &skill.intents {
            prompt.push_str(&format!(
                "- INTENT: {}\n  STRATEGY: {}\n  DECISION LOGIC: {}\n  ACTIONS: {}\n  VERIFICATION: {}\n  RECOVERY: {}\n\n",
                intent.name,
                intent.strategy.join(" | "),
                intent.decision_logic.join(" | "),
                intent.actions.join(" | "),
                intent.verification.join(" | "),
                intent.recovery.join(" | ")
            ));
        }
        prompt
    }

    pub fn get_role_prompt(role: &RoleSkill) -> String {
        let pk = role.platform_knowledge.as_ref().map(|pk| {
            format!("\nKNOWN UI PATTERNS:\n- {}\n", pk.ui_patterns.join("\n- "))
        }).unwrap_or_default();

        format!(
            "\n### 🧩 ROLE STRATEGY: {} ({})\nDESCRIPTION: {}\nDECISION LOGIC: {}\nCONSTRAINTS: {}\nVERIFICATION: {}\nRECOVERY: {}\n{}\n",
            role.name, 
            role.role, 
            role.description,
            role.decision_logic,
            role.constraints,
            role.verification,
            role.recovery,
            pk
        )
    }

    pub fn get_human_ui_prompt() -> String {
        let paths = [
            Path::new("skills/core/human_ui_intelligence.yaml"),
            Path::new("momentum-agent/skills/core/human_ui_intelligence.yaml"),
            Path::new("../momentum-agent/skills/core/human_ui_intelligence.yaml"),
        ];

        for path in &paths {
            if let Ok(content) = fs::read_to_string(path) {
                return format!(
                    "\n### 🧠 UNIVERSAL HUMAN UI INTELLIGENCE PRINCIPLES (HOW HUMANS USE UI):\n\
                    - SURFACE ACQUISITION FIRST: If target app/page is not focused or open, acquire it first (launch or navigate). Never send clicks into the void.\n\
                    - FOCUS BEFORE TYPING: Always click into an input, text area, or search field to place caret focus before typing.\n\
                    - SEARCH-FIRST LAW: Never manually scan 1,000 items if a search bar, filter, or shortcut exists. Search directly for what you need.\n\
                    - SCROLL-TO-DISCOVER: If target element, button, or result is not in current visible viewport, SCROLL DOWN (or up) to discover it! Never guess offscreen coordinates. After scrolling, observe newly visible elements.\n\
                    - MODAL & POPUP DISMISSAL: Clear blocking dialogs, cookie consent banners, or overlays first (click 'Accept', 'X', 'Close', or press Esc).\n\
                    - CLOSED-LOOP DELTA VERIFICATION (OODA): Look at what changed after every action. If a click/keystroke had no effect, do not repeat blindly; scroll, refocus, or adapt.\n\
                    - SUBMIT ON ENTER: Press Return/Enter to submit searches and messages after typing.\n\
                    - CONFIRM ACHIEVEMENT: Look for visual confirmation (e.g. 'Playing', 'Message sent', results list) before declaring success.\n\n\
                    [Loaded from {}]\n\n{}\n",
                    path.display(),
                    content
                );
            }
        }

        "\n### 🧠 UNIVERSAL HUMAN UI INTELLIGENCE PRINCIPLES (HOW HUMANS USE UI):\n\
        - SURFACE ACQUISITION FIRST: Acquire target app/browser surface first.\n\
        - FOCUS BEFORE TYPING: Click input field before typing.\n\
        - SEARCH-FIRST LAW: Use search bar/filter instead of scanning long lists.\n\
        - SCROLL-TO-DISCOVER: Scroll viewport down/up to find offscreen targets; re-perceive after scrolling.\n\
        - MODAL DISMISSAL: Clear popups/banners first.\n\
        - CLOSED-LOOP VERIFICATION: Verify delta after every action.\n\
        - SUBMIT ON ENTER: Press Return after search queries.\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_ui_prompt_contains_core_principles() {
        let prompt = SkillLoader::get_human_ui_prompt();
        assert!(prompt.contains("SURFACE ACQUISITION FIRST"));
        assert!(prompt.contains("FOCUS BEFORE TYPING"));
        assert!(prompt.contains("SEARCH-FIRST LAW"));
        assert!(prompt.contains("SCROLL-TO-DISCOVER"));
        assert!(prompt.contains("CLOSED-LOOP DELTA VERIFICATION"));
    }
}

