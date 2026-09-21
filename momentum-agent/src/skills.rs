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
}
