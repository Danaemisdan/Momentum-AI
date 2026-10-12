use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::Utc;

// ─────────────────────────────────────────────
// NODE TYPES
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Person,       // jagadeesh, sanjeev — real humans
    Platform,     // whatsapp, linkedin, youtube, github
    AppCategory,  // messaging, browser, terminal, creative
    Industry,     // saas, manufacturing, real_estate
    Domain,       // sales, comms, coding, research
    Tactic,       // cold_outreach.personalized, followup.24hr
    Task,         // recorded task executions
    Concept,      // lead_gen, bug_finding, 3d_modeling
    Company,      // organisations
    Source,       // news/info sources with reliability meta
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,           // e.g. "person:jagadeesh", "platform:whatsapp"
    pub node_type: NodeType,
    pub label: String,        // human-readable name
    pub meta: HashMap<String, serde_json::Value>, // all extra info, open-ended
    pub created_at: String,
    pub updated_at: String,
}

impl Node {
    pub fn new(id: impl Into<String>, node_type: NodeType, label: impl Into<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: id.into(),
            node_type,
            label: label.into(),
            meta: HashMap::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }

    pub fn set_meta(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.meta.insert(key.into(), value.into());
        self.updated_at = Utc::now().to_rfc3339();
    }

    pub fn get_meta_str(&self, key: &str) -> Option<&str> {
        self.meta.get(key)?.as_str()
    }

    pub fn get_meta_f64(&self, key: &str) -> Option<f64> {
        self.meta.get(key)?.as_f64()
    }

    pub fn get_meta_u64(&self, key: &str) -> Option<u64> {
        self.meta.get(key)?.as_u64()
    }
}

// ─────────────────────────────────────────────
// EDGE TYPES
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EdgeRel {
    // People
    Knows,            // Person → Person
    ReachableOn,      // Person → Platform
    WorksAt,          // Person → Company

    // Platforms & capabilities
    HasCapability,    // Platform → Concept
    RunsOn,           // Domain/Tactic → Platform
    BestFor,          // Platform/Source → Industry
    PartOf,           // anything → parent category/domain
    Requires,         // Tactic → Tactic (prerequisite chain)

    // Tasks & outcomes
    UsedTactic,       // Task → Tactic
    Involved,         // Task → Person/Platform
    SucceededIn,      // Tactic → context node (Industry/Platform)
    FailedIn,         // Tactic → context node
    LearnedFrom,      // Node → Task (provenance)

    // Source reliability
    MoreReliableThan, // Source → Source
    PrimarySourceFor, // Source → Concept/Industry
    AggregatesFrom,   // Source → Source

    // Generic
    RelatedTo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,    // node id
    pub to: String,      // node id
    pub rel: EdgeRel,
    pub weight: f32,     // 0.0–1.0, confidence/strength
    pub meta: HashMap<String, serde_json::Value>,
    pub created_at: String,
}

impl Edge {
    pub fn new(from: impl Into<String>, to: impl Into<String>, rel: EdgeRel) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            rel,
            weight: 1.0,
            meta: HashMap::new(),
            created_at: Utc::now().to_rfc3339(),
        }
    }

    pub fn with_weight(mut self, w: f32) -> Self {
        self.weight = w;
        self
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }
}

// ─────────────────────────────────────────────
// GRAPH DATA
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub version: String,
    pub nodes: HashMap<String, Node>,
    pub edges: Vec<Edge>,
}

impl Default for GraphData {
    fn default() -> Self {
        Self {
            version: "3.0.0".to_string(),
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }
}

// ─────────────────────────────────────────────
// KNOWLEDGE GRAPH ENGINE
// ─────────────────────────────────────────────

pub struct KnowledgeGraph {
    storage_path: PathBuf,
    pub data: GraphData,
}

impl KnowledgeGraph {
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".momentum")
            .join("graph.json")
    }

    pub fn load() -> Self {
        Self::load_from(Self::default_path())
    }

    pub fn load_from(storage_path: PathBuf) -> Self {
        let mut data = GraphData::default();
        if storage_path.exists() {
            if let Ok(content) = fs::read_to_string(&storage_path) {
                if let Ok(parsed) = serde_json::from_str::<GraphData>(&content) {
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
        fs::write(&temp_path, &serialized)?;
        fs::rename(temp_path, &self.storage_path)?;
        Ok(())
    }

    // ─── Node Operations ───

    pub fn upsert_node(&mut self, node: Node) {
        let existing = self.data.nodes.get_mut(&node.id);
        if let Some(existing) = existing {
            // Merge meta, update timestamp
            for (k, v) in &node.meta {
                existing.meta.insert(k.clone(), v.clone());
            }
            existing.updated_at = Utc::now().to_rfc3339();
        } else {
            self.data.nodes.insert(node.id.clone(), node);
        }
        let _ = self.save();
    }

    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.data.nodes.get(id)
    }

    pub fn find_nodes_by_type(&self, node_type: &NodeType) -> Vec<&Node> {
        self.data.nodes.values()
            .filter(|n| &n.node_type == node_type)
            .collect()
    }

    pub fn find_node_by_label(&self, label: &str) -> Option<&Node> {
        let label_lower = label.to_lowercase();
        self.data.nodes.values().find(|n| {
            n.label.to_lowercase() == label_lower
                || n.id.to_lowercase().contains(&label_lower)
                || n.meta.get("aliases")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().any(|a| {
                        a.as_str().map(|s| s.to_lowercase().contains(&label_lower)).unwrap_or(false)
                    }))
                    .unwrap_or(false)
        })
    }

    // ─── Edge Operations ───

    pub fn add_edge(&mut self, edge: Edge) {
        // Deduplicate: same from/to/rel only once
        let exists = self.data.edges.iter().any(|e| {
            e.from == edge.from && e.to == edge.to && e.rel == edge.rel
        });
        if !exists {
            self.data.edges.push(edge);
            let _ = self.save();
        }
    }

    /// All nodes reachable from `from_id` via `rel`
    pub fn traverse(&self, from_id: &str, rel: &EdgeRel) -> Vec<&Node> {
        self.data.edges.iter()
            .filter(|e| &e.from == from_id && &e.rel == rel)
            .filter_map(|e| self.data.nodes.get(&e.to))
            .collect()
    }

    /// All nodes pointing TO `to_id` via `rel`
    pub fn traverse_reverse(&self, to_id: &str, rel: &EdgeRel) -> Vec<&Node> {
        self.data.edges.iter()
            .filter(|e| &e.to == to_id && &e.rel == rel)
            .filter_map(|e| self.data.nodes.get(&e.from))
            .collect()
    }

    /// K-hop neighborhood of a node (BFS, max depth 2 by default)
    pub fn neighborhood(&self, node_id: &str, max_depth: usize) -> Vec<&Node> {
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut queue: std::collections::VecDeque<(String, usize)> = std::collections::VecDeque::new();
        let mut result: Vec<&Node> = Vec::new();

        queue.push_back((node_id.to_string(), 0));
        visited.insert(node_id.to_string());

        while let Some((current_id, depth)) = queue.pop_front() {
            if let Some(node) = self.data.nodes.get(&current_id) {
                if current_id != node_id {
                    result.push(node);
                }
            }

            if depth < max_depth {
                for edge in &self.data.edges {
                    let neighbor = if edge.from == current_id {
                        Some(&edge.to)
                    } else if edge.to == current_id {
                        Some(&edge.from)
                    } else {
                        None
                    };

                    if let Some(n_id) = neighbor {
                        if !visited.contains(n_id) {
                            visited.insert(n_id.clone());
                            queue.push_back((n_id.clone(), depth + 1));
                        }
                    }
                }
            }
        }

        result
    }

    // ─── LLM Context Injection ───

    /// Serialize the most relevant subgraph for a given goal into a compact
    /// prompt block the LLM can use before reasoning.
    pub fn inject_context_for(&self, goal: &str) -> String {
        let goal_lower = goal.to_lowercase();
        let mut relevant_nodes: Vec<&Node> = Vec::new();
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();

        // 1. Find nodes whose label/id/aliases match any word in the goal
        for node in self.data.nodes.values() {
            let node_label = node.label.to_lowercase();
            let node_id = node.id.to_lowercase();
            let aliases_match = node.meta.get("aliases")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().any(|a| {
                    a.as_str().map(|s| goal_lower.contains(&s.to_lowercase())).unwrap_or(false)
                }))
                .unwrap_or(false);

            if goal_lower.contains(&node_label) || goal_lower.contains(&node_id) || aliases_match {
                if seen.insert(node.id.as_str()) {
                    relevant_nodes.push(node);
                }
            }
        }

        // 2. Expand 1-hop neighborhood for matched nodes
        let mut expanded: Vec<&Node> = Vec::new();
        for node in &relevant_nodes {
            for neighbor in self.neighborhood(&node.id, 1) {
                if seen.insert(neighbor.id.as_str()) {
                    expanded.push(neighbor);
                }
            }
        }
        relevant_nodes.extend(expanded);

        if relevant_nodes.is_empty() {
            return String::new();
        }

        // 3. Serialize to compact prompt text
        let mut block = String::from("\n[MOMENTUM KNOWLEDGE GRAPH CONTEXT]\n");

        // Group by type for readability
        let mut by_type: HashMap<String, Vec<&Node>> = HashMap::new();
        for node in &relevant_nodes {
            by_type.entry(format!("{:?}", node.node_type))
                .or_default()
                .push(node);
        }

        for (type_name, nodes) in &by_type {
            block.push_str(&format!("  {type_name}s:\n"));
            for node in nodes {
                block.push_str(&format!("    - {} ({})", node.label, node.id));
                // Add key meta fields inline
                for key in ["description", "nature", "reliability", "success_rate", "platform_type", "aliases"] {
                    if let Some(val) = node.meta.get(key) {
                        let val_str = match val {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        block.push_str(&format!(" | {key}: {val_str}"));
                    }
                }
                block.push('\n');
            }
        }

        // Add relevant edges
        let relevant_ids: std::collections::HashSet<&str> = relevant_nodes.iter().map(|n| n.id.as_str()).collect();
        block.push_str("  Relationships:\n");
        for edge in &self.data.edges {
            if relevant_ids.contains(edge.from.as_str()) && relevant_ids.contains(edge.to.as_str()) {
                block.push_str(&format!("    {} --[{:?}]--> {}\n", edge.from, edge.rel, edge.to));
            }
        }

        block.push_str("[END CONTEXT]\n");
        block
    }

    // ─── High-level helpers ───

    /// Record a successfully completed task and update tactic success counts
    pub fn record_success(&mut self, tactic_id: &str, context_id: &str) {
        // Upsert tactic node with incremented success count
        if let Some(node) = self.data.nodes.get_mut(tactic_id) {
            let count = node.meta.get("success_count").and_then(|v| v.as_u64()).unwrap_or(0);
            node.meta.insert("success_count".to_string(), serde_json::Value::Number((count + 1).into()));
            node.updated_at = Utc::now().to_rfc3339();
        }
        self.add_edge(Edge::new(tactic_id, context_id, EdgeRel::SucceededIn).with_weight(1.0));
        let _ = self.save();
    }

    pub fn record_failure(&mut self, tactic_id: &str, context_id: &str) {
        if let Some(node) = self.data.nodes.get_mut(tactic_id) {
            let count = node.meta.get("failure_count").and_then(|v| v.as_u64()).unwrap_or(0);
            node.meta.insert("failure_count".to_string(), serde_json::Value::Number((count + 1).into()));
            node.updated_at = Utc::now().to_rfc3339();
        }
        self.add_edge(Edge::new(tactic_id, context_id, EdgeRel::FailedIn).with_weight(1.0));
        let _ = self.save();
    }

    /// Find or create a Person node. Returns the node id.
    pub fn ensure_person(&mut self, name: &str) -> String {
        let id = format!("person:{}", name.to_lowercase().replace(' ', "_"));
        if !self.data.nodes.contains_key(&id) {
            let node = Node::new(&id, NodeType::Person, name)
                .with_meta("first_seen", Utc::now().to_rfc3339());
            self.upsert_node(node);
        }
        id
    }

    /// Link a person to a platform
    pub fn link_person_to_platform(&mut self, person_name: &str, platform_id: &str) {
        let person_id = self.ensure_person(person_name);
        self.add_edge(Edge::new(&person_id, platform_id, EdgeRel::ReachableOn));
    }

    pub fn node_count(&self) -> usize {
        self.data.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.data.edges.len()
    }

    /// Dynamically inspects tactic nodes in the graph to see if any tactic matches the user goal,
    /// and checks whether all its `required_inputs` are provided in the goal.
    /// Returns (tactic_label, first_missing_slot_description) if a required input is missing.
    pub fn check_tactic_missing_slots(&self, goal: &str) -> Option<(String, String)> {
        let goal_lower = goal.to_lowercase();
        let words: Vec<&str> = goal_lower.split_whitespace().collect();

        let tactics = self.find_nodes_by_type(&NodeType::Tactic);
        for tactic in tactics {
            let t_id = tactic.id.to_lowercase().replace("tactic:", "").replace('_', " ");
            let t_label = tactic.label.to_lowercase();

            let id_matches = t_id.split_whitespace().any(|tw| tw.len() > 3 && words.contains(&tw));
            let label_matches = t_label.split_whitespace().any(|tw| tw.len() > 3 && words.contains(&tw));

            if !id_matches && !label_matches {
                continue;
            }

            if let Some(req_inputs) = tactic.meta.get("required_inputs").and_then(|v| v.as_array()) {
                for req in req_inputs {
                    if let Some(slot_name) = req.as_str() {
                        let clean_slot = slot_name.to_lowercase().replace('_', " ");
                        let is_provided = clean_slot.split_whitespace().any(|sw| goal_lower.contains(sw))
                            || (clean_slot.contains("body") && (goal_lower.contains(" that ") || goal_lower.contains(" saying ") || goal_lower.contains('"')))
                            || (clean_slot.contains("audience") && (goal_lower.contains(" in ") || goal_lower.contains(" for ")))
                            || (clean_slot.contains("company") && (goal_lower.contains(" at ") || goal_lower.contains(" for ")))
                            || (clean_slot.contains("email") && goal_lower.contains('@'));

                        if !is_provided {
                            let question = match slot_name {
                                "message_body" | "value_proposition" => "What message would you like to convey?".into(),
                                "contact_name" | "prospect_email" | "lead_id" => "Who specifically should we reach out to?".into(),
                                "target_audience" | "company_name" => "What target audience or industry should we focus on?".into(),
                                "platform" => "Which platform would you prefer to use?".into(),
                                "delay_days" => "What follow-up timeframe should we set?".into(),
                                other => format!("Please provide details for the {}.", other.replace('_', " ")),
                            };
                            return Some((tactic.label.clone(), question));
                        }
                    }
                }
            }
        }
        None
    }

    /// Normalize an arbitrary string into a stable snake_case label for use as a node ID segment
    pub fn normalize_label(s: &str) -> String {
        let stop_words = [
            "a", "an", "the", "me", "for", "on", "to", "please", "some", "my",
            "how", "can", "you", "i", "want", "need", "like", "do", "we", "it",
        ];
        s.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c.to_ascii_lowercase() } else { '_' })
            .collect::<String>()
            .split('_')
            .filter(|t| !t.is_empty() && !stop_words.contains(t) && t.len() > 1)
            .collect::<Vec<_>>()
            .join("_")
    }
}

// ─────────────────────────────────────────────
// BACKWARD-COMPAT ALIAS (old code used LocalKnowledgeGraph)
// We keep a thin wrapper so existing callers in main.rs / brain.rs don't break
// ─────────────────────────────────────────────
pub type LocalKnowledgeGraph = KnowledgeGraph;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn in_memory_kg() -> KnowledgeGraph {
        let unique_id = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        KnowledgeGraph::load_from(PathBuf::from(format!("/tmp/momentum_test_graph_{}.json", unique_id)))
    }

    #[test]
    fn test_upsert_and_get_node() {
        let mut kg = in_memory_kg();
        let node = Node::new("platform:whatsapp", NodeType::Platform, "WhatsApp")
            .with_meta("platform_type", "desktop_app");
        kg.upsert_node(node);
        let found = kg.get_node("platform:whatsapp").unwrap();
        assert_eq!(found.label, "WhatsApp");
        assert_eq!(found.get_meta_str("platform_type"), Some("desktop_app"));
    }

    #[test]
    fn test_add_edge_deduplicates() {
        let mut kg = in_memory_kg();
        kg.upsert_node(Node::new("person:jag", NodeType::Person, "Jagadeesh"));
        kg.upsert_node(Node::new("platform:whatsapp", NodeType::Platform, "WhatsApp"));
        kg.add_edge(Edge::new("person:jag", "platform:whatsapp", EdgeRel::ReachableOn));
        kg.add_edge(Edge::new("person:jag", "platform:whatsapp", EdgeRel::ReachableOn));
        let edges: Vec<_> = kg.data.edges.iter()
            .filter(|e| e.from == "person:jag" && e.rel == EdgeRel::ReachableOn)
            .collect();
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn test_traverse() {
        let mut kg = in_memory_kg();
        kg.upsert_node(Node::new("person:jag", NodeType::Person, "Jagadeesh"));
        kg.upsert_node(Node::new("platform:whatsapp", NodeType::Platform, "WhatsApp"));
        kg.add_edge(Edge::new("person:jag", "platform:whatsapp", EdgeRel::ReachableOn));
        let platforms = kg.traverse("person:jag", &EdgeRel::ReachableOn);
        assert_eq!(platforms.len(), 1);
        assert_eq!(platforms[0].label, "WhatsApp");
    }

    #[test]
    fn test_inject_context_for_finds_relevant_nodes() {
        let mut kg = in_memory_kg();
        kg.upsert_node(Node::new("person:jagadeesh", NodeType::Person, "Jagadeesh")
            .with_meta("aliases", serde_json::json!(["jag", "jag bro"])));
        kg.upsert_node(Node::new("platform:whatsapp", NodeType::Platform, "WhatsApp"));
        kg.add_edge(Edge::new("person:jagadeesh", "platform:whatsapp", EdgeRel::ReachableOn));

        let ctx = kg.inject_context_for("message jagadeesh on whatsapp");
        assert!(ctx.contains("Jagadeesh"));
        assert!(ctx.contains("WhatsApp"));
    }

    #[test]
    fn test_ensure_person_idempotent() {
        let mut kg = in_memory_kg();
        kg.ensure_person("Sanjeev");
        kg.ensure_person("Sanjeev");
        assert_eq!(kg.node_count(), 1);
    }

    #[test]
    fn test_record_success_increments_count() {
        let mut kg = in_memory_kg();
        kg.upsert_node(Node::new("tactic:cold_outreach", NodeType::Tactic, "Cold Outreach"));
        kg.upsert_node(Node::new("industry:saas", NodeType::Industry, "SaaS"));
        kg.record_success("tactic:cold_outreach", "industry:saas");
        kg.record_success("tactic:cold_outreach", "industry:saas");
        let count = kg.get_node("tactic:cold_outreach")
            .unwrap()
            .get_meta_u64("success_count")
            .unwrap_or(0);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_check_tactic_missing_slots() {
        let mut kg = in_memory_kg();
        let tactic = Node::new("tactic:cold_outreach", NodeType::Tactic, "Cold Outreach")
            .with_meta("required_inputs", serde_json::json!(["prospect_email", "message_body"]));
        kg.upsert_node(tactic);

        // When goal only has email, it should detect missing message_body
        let missing = kg.check_tactic_missing_slots("do cold outreach to alex@example.com");
        assert!(missing.is_some());
        let (label, question) = missing.unwrap();
        assert_eq!(label, "Cold Outreach");
        assert!(question.contains("message"));

        // When goal has both email and message body, it should pass with None
        let complete = kg.check_tactic_missing_slots("cold outreach to alex@example.com saying let's meet");
        assert!(complete.is_none());
    }
}
