use crate::knowledge::{LocalKnowledgeGraph, KnowledgeGraph, Node, NodeType, Edge, EdgeRel};
use crate::scraper::{search_duckduckgo, SearchResult};
use crate::brain::MomentumBrain;
use crate::authority::AuthorityRegistry;
use std::sync::Arc;
use chrono::Utc;

/// Lightweight platform record used during discovery before committing to the graph.
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub name: String,
    pub platform_type: String,
    pub target: String,
    pub best_for: String,
}

/// Backward-compat return type so callers in brain.rs / main.rs don't break.
#[derive(Debug, Clone)]
pub struct DomainEntry {
    pub domain_name: String,
    pub query_pattern: String,
    pub platforms: Vec<PlatformInfo>,
    pub clarification_question: String,
}

pub struct AutonomousWebResearcher {
    kg: Arc<tokio::sync::Mutex<LocalKnowledgeGraph>>,
}

impl AutonomousWebResearcher {
    pub fn new(kg: Arc<tokio::sync::Mutex<LocalKnowledgeGraph>>) -> Self {
        Self { kg }
    }

    /// Primary entry point: Researches a novel goal using live search + 0.5B cognitive distillation,
    /// verifies local OS apps (macOS & Windows), generates a natural clarification question,
    /// and saves discovered platforms as graph nodes permanently.
    pub async fn research_domain(
        &self,
        goal: &str,
        brain: Option<&MomentumBrain>,
    ) -> DomainEntry {
        println!("🌐 [AutonomousWebResearcher] Initiating live web discovery for: \"{}\"", goal);

        let query = Self::formulate_query(goal);
        println!("🔍 [AutonomousWebResearcher] Formulated query: \"{}\"", query);
        let results = search_duckduckgo(&query, 5).await;
        println!("📄 [AutonomousWebResearcher] Retrieved {} search snippets", results.len());

        let (mut platforms, question) = if let Some(b) = brain {
            self.distill_with_brain(b, goal, &results).await
        } else {
            let p = Self::extract_platforms(&results, goal);
            let q = Self::synthesize_default_question(goal, &p);
            (p, q)
        };

        if platforms.is_empty() {
            let target_url = format!("https://duckduckgo.com/?q={}", goal.replace(' ', "+"));
            platforms.push(PlatformInfo {
                name: "Web Search".to_string(),
                platform_type: "web".to_string(),
                target: target_url,
                best_for: format!("Live web discovery and direct search for {}", goal),
            });
        }

        let now = Utc::now().to_rfc3339();
        let domain_title = Self::format_domain_title(goal);
        let concept_id = format!("concept:{}", KnowledgeGraph::normalize_label(goal));

        // Write discovered platforms as graph nodes + edges
        {
            let mut kg_lock = self.kg.lock().await;
            for p in &platforms {
                let node_id = format!("platform:{}", p.name.to_lowercase().replace(' ', "_"));
                let mut meta = std::collections::HashMap::new();
                meta.insert("platform_type".to_string(), serde_json::json!(p.platform_type.clone()));
                meta.insert("target".to_string(), serde_json::json!(p.target.clone()));
                meta.insert("nature".to_string(), serde_json::json!(p.best_for.clone()));
                meta.insert("discovered_via".to_string(), serde_json::json!(format!("research:{}", goal)));
                let node = Node {
                    id: node_id.clone(),
                    node_type: NodeType::Platform,
                    label: p.name.clone(),
                    meta,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };
                kg_lock.upsert_node(node);
                kg_lock.add_edge(Edge::new(&node_id, &concept_id, EdgeRel::BestFor));
            }
            println!("💾 [AutonomousWebResearcher] Wrote {} platform nodes to graph", platforms.len());
        }

        DomainEntry {
            domain_name: domain_title,
            query_pattern: goal.to_string(),
            platforms,
            clarification_question: question,
        }
    }

    /// Natural intent query formulation — strips conversational filler without hardcoding
    pub fn formulate_query(goal: &str) -> String {
        let clean = goal.trim();
        let lower = clean.to_lowercase();

        let prefixes = [
            "can you please ", "could you please ", "please ",
            "can you ", "could you ", "help me ", "i want to ",
            "i need to ", "how to ", "how do i ", "tell me how to ",
        ];

        let mut stripped = clean;
        for prefix in &prefixes {
            if lower.starts_with(prefix) {
                stripped = &clean[prefix.len()..];
                break;
            }
        }

        stripped.trim().to_string()
    }

    /// Distill search snippets using the 0.5B model in ~100ms
    async fn distill_with_brain(
        &self,
        brain: &MomentumBrain,
        goal: &str,
        results: &[SearchResult],
    ) -> (Vec<PlatformInfo>, String) {
        let registry = AuthorityRegistry::load_or_default();
        let goal_matches = registry.find_matches_for_goal(goal);
        let urls: Vec<String> = results.iter().map(|r| r.url.clone()).collect();
        let snippet_text_full = results.iter().map(|r| format!("{} {}", r.title, r.snippet)).collect::<Vec<_>>().join(" ");
        let snippet_matches = registry.match_snippets(&snippet_text_full, &urls);

        // Build candidate pool strictly vetted against numbers, dates, and filler
        let mut candidates = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // 1. Authoritative registry matches (highest quality)
        for m in &goal_matches {
            let key = m.name.to_lowercase();
            if !seen.contains(&key) {
                seen.insert(key);
                candidates.push(m.name.clone());
            }
        }
        for m in &snippet_matches {
            let key = m.name.to_lowercase();
            if !seen.contains(&key) {
                seen.insert(key);
                candidates.push(m.name.clone());
            }
        }

        // 2. High-quality domain roots from search results
        for r in results {
            let domain = Self::extract_domain(&r.url);
            let root = domain.split('.').next().unwrap_or(&domain);
            let root_clean = root.trim().to_lowercase();
            if root_clean != "duckduckgo" && root_clean != "bing" && root_clean != "yahoo"
                && root_clean != "google" && root_clean != "youtube" && root_clean != "wikipedia"
                && root_clean.len() >= 3 && root_clean.len() <= 25
                && !root_clean.chars().all(|c| c.is_numeric())
            {
                let mut c = root.chars();
                let capitalized = match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                };
                if !seen.contains(&root_clean) && !capitalized.is_empty() {
                    seen.insert(root_clean);
                    candidates.push(capitalized);
                }
            }
        }

        // 3. Proper nouns from titles (strict filter: alphabetic, capitalized, no dates/numbers/stopwords)
        let banned_words = [
            "best", "top", "free", "how", "ways", "methods", "guide", "tools",
            "software", "online", "proven", "tactics", "agencies", "freelancers",
            "find", "make", "create", "build", "clients", "year", "years", "tips",
            "solutions", "platform", "platforms", "services", "results", "updated",
            "what", "when", "where", "with", "from", "into", "over", "more", "help",
            "are", "you", "your", "yours", "listening", "listen", "can", "could", "will",
            "would", "shall", "should", "the", "this", "that", "these", "those", "is",
            "am", "was", "were", "been", "being", "have", "has", "had", "does", "did",
            "done", "about", "above", "after", "again", "all", "and", "any", "because",
            "before", "both", "but", "down", "each", "few", "here", "there", "then",
            "once", "only", "other", "some", "such", "than", "too", "very", "who", "whom",
            "why", "just", "now", "not", "out", "off", "our", "ours", "get", "got", "know",
            "see", "think", "come", "look", "tell", "podcast", "episode", "video", "article",
            "open", "opening", "play", "playing", "player", "support", "music", "song", "songs",
            "audio", "click", "login", "sign", "home", "page", "using", "uses", "site", "web"
        ];

        for r in results {
            for word in r.title.split_whitespace() {
                let clean_word: String = word.chars().filter(|c| c.is_alphabetic()).collect();
                let lower = clean_word.to_lowercase();
                if clean_word.len() >= 4 && clean_word.len() <= 20
                    && clean_word.chars().next().map_or(false, |c| c.is_uppercase())
                    && !clean_word.chars().all(|c| c.is_numeric())
                    && !banned_words.contains(&lower.as_str())
                    && !seen.contains(&lower)
                {
                    seen.insert(lower);
                    candidates.push(clean_word);
                }
            }
        }

        let candidates: Vec<String> = candidates.into_iter().take(6).collect();

        if candidates.is_empty() && results.is_empty() {
            return (vec![], format!("Ready to search for {}. Want me to proceed?", goal));
        }

        // Fast-path: If we already have 2+ vetted candidate options from authoritative registry or snippets, pick them in 0ms!
        if candidates.len() >= 2 {
            let top_entities = candidates.iter().take(3).cloned().collect::<Vec<_>>();
            let p = Self::build_platforms_from_entities(&top_entities, goal, results, &registry);
            let q = Self::synthesize_default_question(goal, &p);
            println!("⚡ [AutonomousWebResearcher] Fast-path distillation (0ms) selected: {}", top_entities.join(", "));
            return (p, q);
        }

        // Format dense snippet context for logging
        println!("📄 [AutonomousWebResearcher] Search Snippets:");
        for (i, r) in results.iter().take(5).enumerate() {
            println!("   [{}] {} ({})\n       {}", i + 1, r.title, r.url, r.snippet);
        }

        let candidate_list = candidates.iter().enumerate()
            .map(|(i, c)| format!("{}. {}", i + 1, c))
            .collect::<Vec<_>>()
            .join("\n");

        println!("📋 [AutonomousWebResearcher] Distillation Candidate Pool ({} options):\n{}", candidates.len(), candidate_list);

        let prompt = format!(
            "<|im_start|>system\n\
You are an expert cognitive distillation engine. \
Given the user goal and candidate options, select the 2 or 3 best candidate tools or platforms, then write one concise conversational question asking Danny which option he wants.\n\
<|im_end|>\n\
<|im_start|>user\n\
Goal: {}\n\n\
Candidate Options:\n\
{}\n\n\
Task:\n\
Select the best entities and write the question in this exact format:\n\
Entities: Option 1, Option 2, Option 3\n\
Question: [one concise conversational question asking Danny which option he wants to use]\n\
<|im_end|>\n\
<|im_start|>assistant\n\
Entities: ",
            goal, candidate_list
        );

        let completion = match brain.generate(&prompt, None).await {
            Ok(c) => c,
            Err(e) => {
                println!("⚠️ [AutonomousWebResearcher] Brain generation failed: {}, using candidate fallback", e);
                let fallback_entities = candidates.iter().take(3).cloned().collect::<Vec<_>>();
                let p = Self::build_platforms_from_entities(&fallback_entities, goal, results, &registry);
                let q = Self::synthesize_default_question(goal, &p);
                return (p, q);
            }
        };

        let raw_full = format!("Entities: {}", completion);
        let clean_completion = raw_full.split("<|im_end|>").next().unwrap_or(&raw_full);
        println!("🧠 [AutonomousWebResearcher] 0.5B Cognitive Distillation:\n{}", clean_completion.trim());

        // Parse extracted entities and question
        let mut entity_names: Vec<String> = Vec::new();
        let mut question = String::new();

        for line in clean_completion.lines() {
            let trimmed = line.trim();
            if trimmed.to_lowercase().starts_with("entities:") {
                let rest = trimmed[9..].trim().trim_matches(|c| c == '[' || c == ']');
                for part in rest.split(',') {
                    let mut name = part.trim().trim_matches(|c| c == '"' || c == '\'' || c == '*' || c == '-').trim();
                    // Strip leading numbers like "1. Clutch" -> "Clutch"
                    name = name.trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ' ');
                    // Invariant: MUST NOT be purely numeric (reject 2025/2026), must be valid length
                    if !name.is_empty() && name.len() >= 2 && name.len() <= 35 
                        && !name.chars().all(|c| c.is_numeric() || c == '/' || c == '-')
                        && !name.to_lowercase().contains("question") 
                    {
                        entity_names.push(name.to_string());
                    }
                }
            } else if trimmed.to_lowercase().starts_with("question:") {
                let q = trimmed[9..].trim().trim_matches(|c| c == '[' || c == ']' || c == '"').trim();
                if !q.is_empty() {
                    question = q.to_string();
                }
            } else if question.is_empty() && trimmed.ends_with('?') {
                question = trimmed.to_string();
            }
        }

        // If the model was empty or invalid, fallback to top candidates from candidate pool
        if entity_names.is_empty() {
            for cand in candidates.iter().take(3) {
                entity_names.push(cand.clone());
            }
        }

        let platforms = Self::build_platforms_from_entities(&entity_names, goal, results, &registry);

        if question.is_empty() {
            question = Self::synthesize_default_question(goal, &platforms);
        }

        (platforms, question)
    }

    /// Build PlatformInfo entries for each discovered entity, checking local apps, registry, and search results
    fn build_platforms_from_entities(
        entities: &[String],
        goal: &str,
        results: &[SearchResult],
        registry: &AuthorityRegistry,
    ) -> Vec<PlatformInfo> {
        let mut platforms = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for name in entities {
            if platforms.len() >= 3 { break; }
            let key = name.to_lowercase();
            if seen.contains(&key) { continue; }
            seen.insert(key.clone());

            // 1. Check if the app is already installed locally on the system
            if let Some(local_path) = Self::find_local_app(name) {
                platforms.push(PlatformInfo {
                    name: name.clone(),
                    platform_type: "desktop_app".to_string(),
                    target: local_path,
                    best_for: format!("Installed local application for {}", goal),
                });
                continue;
            }

            // 2. Check if present in Authority Registry (the curated Bible)
            if let Some(auth) = registry.data.entries.get(&key) {
                platforms.push(PlatformInfo {
                    name: auth.name.clone(),
                    platform_type: "web".to_string(),
                    target: auth.default_target.clone(),
                    best_for: auth.description.clone(),
                });
                continue;
            }

            // 3. Check search results for matching official URL
            let mut target_url = format!("https://duckduckgo.com/?q={}", name.replace(' ', "+"));
            let mut best_for = format!("Online platform and workflow for {}", goal);

            for r in results {
                let domain = Self::extract_domain(&r.url).to_lowercase();
                let title_lower = r.title.to_lowercase();
                if domain.contains(&key) || key.contains(&domain.split('.').next().unwrap_or(&domain)) || title_lower.contains(&key) {
                    target_url = r.url.clone();
                    if !r.snippet.is_empty() {
                        best_for = r.snippet.chars().take(120).collect();
                    }
                    break;
                }
            }

            platforms.push(PlatformInfo {
                name: name.clone(),
                platform_type: "web".to_string(),
                target: target_url,
                best_for,
            });
        }

        platforms
    }

    /// Dynamic local application discovery on macOS & Windows without hardcoded lists
    pub fn find_local_app(name: &str) -> Option<String> {
        let clean = name.to_lowercase();
        let is_app_match = |app_name: &str| -> bool {
            if app_name == clean { return true; }
            if clean.len() >= 4 {
                app_name.split(|c: char| !c.is_alphanumeric()).any(|w| w == clean)
            } else {
                false
            }
        };

        #[cfg(target_os = "macos")]
        {
            let mut search_dirs = vec![
                std::path::PathBuf::from("/Applications"),
                std::path::PathBuf::from("/System/Applications"),
            ];
            if let Some(home) = dirs::home_dir() {
                search_dirs.push(home.join("Applications"));
            }

            for dir in search_dirs {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |ext| ext == "app") {
                            let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
                            if is_app_match(&stem) {
                                return Some(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let mut search_dirs = vec![
                std::path::PathBuf::from("C:\\Program Files"),
                std::path::PathBuf::from("C:\\Program Files (x86)"),
            ];
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                search_dirs.push(std::path::PathBuf::from(local_app_data).join("Programs"));
            }

            for dir in search_dirs {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if is_app_match(&file_name) {
                            return Some(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        let _ = clean;
        None
    }

    /// Fallback extractor when brain is offline
    pub fn extract_platforms(results: &[SearchResult], goal: &str) -> Vec<PlatformInfo> {
        let registry = AuthorityRegistry::load_or_default();
        let goal_matches = registry.find_matches_for_goal(goal);
        let urls: Vec<String> = results.iter().map(|r| r.url.clone()).collect();
        let snippet_text_full = results.iter().map(|r| format!("{} {}", r.title, r.snippet)).collect::<Vec<_>>().join(" ");
        let snippet_matches = registry.match_snippets(&snippet_text_full, &urls);

        let mut entities = Vec::new();
        for m in goal_matches.iter().chain(snippet_matches.iter()) {
            if !entities.contains(&m.name) {
                entities.push(m.name.clone());
            }
        }

        if entities.is_empty() {
            for r in results {
                let name = Self::extract_brand_name(&r.title, &r.url);
                if !name.is_empty() && name != "Resource" && !entities.contains(&name) {
                    entities.push(name);
                }
            }
        }

        Self::build_platforms_from_entities(&entities, goal, results, &registry)
    }

    fn extract_brand_name(title: &str, url: &str) -> String {
        let domain = Self::extract_domain(url);
        let root = domain.split('.').next().unwrap_or(&domain);
        if !root.is_empty() && root != "duckduckgo" && root != "bing" && root != "yahoo" && root != "google" {
            let mut c = root.chars();
            return match c.next() {
                None => "Resource".to_string(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            };
        }

        for sep in [": ", " - ", " | ", " — "] {
            if let Some(pos) = title.rfind(sep) {
                let cand = title[pos + sep.len()..].trim();
                if cand.len() >= 2 && cand.len() <= 25 && !cand.contains("202") {
                    return cand.to_string();
                }
            }
        }

        "Resource".to_string()
    }

    pub fn extract_domain(url: &str) -> String {
        let clean = url.trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("www.");
        clean.split('/').next().unwrap_or(clean).to_string()
    }

    fn format_domain_title(goal: &str) -> String {
        goal.split_whitespace()
            .take(4)
            .map(|word| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn synthesize_default_question(goal: &str, platforms: &[PlatformInfo]) -> String {
        if platforms.len() >= 2 {
            format!(
                "I mapped out the top platforms for {}: {} and {}. Which one do you want to hit?",
                goal, platforms[0].name, platforms[1].name
            )
        } else if let Some(p) = platforms.first() {
            format!("Found {} for this. Want me to open that up?", p.name)
        } else {
            format!("Ready to search for {}. Want me to proceed?", goal)
        }
    }
}
