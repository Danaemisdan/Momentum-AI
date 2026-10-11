use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformInfo {
    pub name: String,
    pub platform_type: String, // "web" | "desktop_app"
    pub target: String,        // URL or app name
    pub best_for: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MilestoneStep {
    pub action_type: String,
    pub intent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEntry {
    pub domain_name: String,
    pub query_pattern: String,
    pub summary: String,
    pub platforms: Vec<PlatformInfo>,
    pub clarification_question: String,
    pub learned_at: String,
    pub last_used: String,
    pub use_count: u64,
}

impl DomainEntry {
    /// Convert the chosen (or first) platform into executable driver milestones
    pub fn to_milestones(&self, preferred_platform: Option<&str>, goal: &str) -> Vec<MilestoneStep> {
        let platform = if let Some(pref) = preferred_platform {
            self.platforms.iter().find(|p| p.name.eq_ignore_ascii_case(pref))
                .or_else(|| self.platforms.first())
        } else {
            self.platforms.first()
        };

        if let Some(p) = platform {
            if p.platform_type == "desktop_app" {
                vec![
                    MilestoneStep {
                        action_type: "OpenApp".to_string(),
                        intent: format!("LaunchApp ({})", p.name),
                        input: Some(p.target.clone()),
                    },
                    MilestoneStep {
                        action_type: "InspectEnvironment".to_string(),
                        intent: format!("Ready {}", p.name),
                        input: Some(goal.to_string()),
                    },
                ]
            } else {
                vec![
                    MilestoneStep {
                        action_type: "OpenApp".to_string(),
                        intent: "LaunchSurface".to_string(),
                        input: Some("browser".to_string()),
                    },
                    MilestoneStep {
                        action_type: "Navigate".to_string(),
                        intent: format!("VisitPlatform ({})", p.name),
                        input: Some(p.target.clone()),
                    },
                    MilestoneStep {
                        action_type: "UI_Intent".to_string(),
                        intent: "Search".to_string(),
                        input: Some(goal.to_string()),
                    },
                ]
            }
        } else {
            vec![
                MilestoneStep {
                    action_type: "OpenApp".to_string(),
                    intent: "LaunchSurface".to_string(),
                    input: Some("browser".to_string()),
                },
                MilestoneStep {
                    action_type: "Navigate".to_string(),
                    intent: "SearchWeb".to_string(),
                    input: Some(format!("https://duckduckgo.com/?q={}", goal.replace(' ', "+"))),
                },
            ]
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraphData {
    pub version: String,
    pub domains: HashMap<String, DomainEntry>,
}

impl Default for KnowledgeGraphData {
    fn default() -> Self {
        Self {
            version: "2.0.0".to_string(),
            domains: HashMap::new(),
        }
    }
}

pub struct LocalKnowledgeGraph {
    storage_path: PathBuf,
    pub data: KnowledgeGraphData,
}

impl LocalKnowledgeGraph {
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".momentum")
            .join("knowledge_graph.json")
    }

    pub fn load() -> Self {
        Self::load_from(Self::default_path())
    }

    pub fn load_from(storage_path: PathBuf) -> Self {
        let mut data = KnowledgeGraphData::default();
        if storage_path.exists() {
            if let Ok(content) = fs::read_to_string(&storage_path) {
                if let Ok(parsed) = serde_json::from_str::<KnowledgeGraphData>(&content) {
                    data = parsed;
                }
            }
        }
        Self { storage_path, data }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(&self.data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        
        let temp_path = self.storage_path.with_extension("tmp");
        fs::write(&temp_path, serialized)?;
        fs::rename(temp_path, &self.storage_path)?;
        Ok(())
    }

    /// Normalize an intent into a clean token key
    pub fn normalize_intent(goal: &str) -> String {
        let stop_words = [
            "a", "an", "the", "me", "for", "on", "to", "please", "some", "my",
            "how", "can", "you", "i", "want", "need", "like", "do", "we"
        ];
        
        let cleaned: String = goal.chars()
            .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
            .collect();
            
        let tokens: Vec<String> = cleaned
            .to_lowercase()
            .split_whitespace()
            .filter(|t| !stop_words.contains(t) && t.len() > 1)
            .map(|t| t.to_string())
            .collect();
            
        tokens.join(" ")
    }

    fn token_similarity(a: &str, b: &str) -> f32 {
        let tokens_a: std::collections::HashSet<&str> = a.split_whitespace().collect();
        let tokens_b: std::collections::HashSet<&str> = b.split_whitespace().collect();
        
        if tokens_a.is_empty() || tokens_b.is_empty() {
            return 0.0;
        }

        let intersection_count = tokens_a.intersection(&tokens_b).count();
        let union_count = tokens_a.union(&tokens_b).count();

        if union_count == 0 {
            0.0
        } else {
            intersection_count as f32 / union_count as f32
        }
    }

    /// Find an existing domain entry for the given user goal (0ms cache lookup)
    pub fn find_domain(&mut self, goal: &str) -> Option<DomainEntry> {
        let normalized = Self::normalize_intent(goal);
        if normalized.is_empty() {
            return None;
        }

        let now = Utc::now().to_rfc3339();

        // 1. Exact key match
        if let Some(entry) = self.data.domains.get_mut(&normalized) {
            entry.use_count += 1;
            entry.last_used = now;
            let result = entry.clone();
            let _ = self.save();
            return Some(result);
        }

        // 2. Token overlap or substring match across keys and platforms
        let mut best_match_key: Option<String> = None;
        let mut best_score = 0.0f32;

        for (key, entry) in self.data.domains.iter() {
            // Direct substring
            if normalized.contains(key) || key.contains(&normalized) {
                best_match_key = Some(key.clone());
                break;
            }

            // Check if goal mentions any of the known platforms
            for p in &entry.platforms {
                let p_norm = p.name.to_lowercase();
                if normalized.contains(&p_norm) {
                    best_match_key = Some(key.clone());
                    break;
                }
            }
            if best_match_key.is_some() {
                break;
            }

            let sim = Self::token_similarity(&normalized, key);
            if sim > best_score && sim >= 0.40 {
                best_score = sim;
                best_match_key = Some(key.clone());
            }
        }

        if let Some(key) = best_match_key {
            if let Some(entry) = self.data.domains.get_mut(&key) {
                entry.use_count += 1;
                entry.last_used = now;
                let result = entry.clone();
                let _ = self.save();
                return Some(result);
            }
        }

        None
    }

    /// Save a newly discovered domain with its platforms and clarification question
    pub fn save_domain(&mut self, entry: DomainEntry) {
        let normalized = Self::normalize_intent(&entry.query_pattern);
        let key = if normalized.is_empty() {
            entry.domain_name.trim().to_lowercase()
        } else {
            normalized
        };

        self.data.domains.insert(key, entry);
        let _ = self.save();
    }

    /// Remove a domain by goal/query (useful for resets or testing)
    pub fn remove_domain(&mut self, goal: &str) -> bool {
        let normalized = Self::normalize_intent(goal);
        let key = if normalized.is_empty() {
            goal.trim().to_lowercase()
        } else {
            normalized
        };

        let keys_to_remove: Vec<String> = self.data.domains.keys()
            .filter(|k| *k == &key || k.contains(&key) || key.contains(*k))
            .cloned()
            .collect();

        let mut any_removed = false;
        for k in keys_to_remove {
            if self.data.domains.remove(&k).is_some() {
                any_removed = true;
            }
        }

        if any_removed {
            let _ = self.save();
        }
        any_removed
    }

    pub fn len(&self) -> usize {
        self.data.domains.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.domains.is_empty()
    }
}
