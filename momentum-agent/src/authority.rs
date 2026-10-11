use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthorityEntry {
    pub name: String,
    pub domain: String,
    pub category: String, // "creative_3d" | "business_clients" | "engineering_api" | "research_science" | "media_entertainment"
    pub description: String,
    pub default_target: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityRegistryData {
    pub version: String,
    pub entries: HashMap<String, AuthorityEntry>,
}

pub struct AuthorityRegistry {
    storage_path: PathBuf,
    pub data: AuthorityRegistryData,
}

impl AuthorityRegistry {
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".momentum")
            .join("authority_registry.json")
    }

    pub fn load_or_default() -> Self {
        let p = Self::default_path();
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                if let Ok(parsed) = serde_json::from_str::<AuthorityRegistryData>(&content) {
                    return Self { storage_path: p, data: parsed };
                }
            }
        }

        let mut reg = Self {
            storage_path: p,
            data: AuthorityRegistryData {
                version: "1.0.0".to_string(),
                entries: HashMap::new(),
            },
        };
        reg.seed_defaults();
        let _ = reg.save();
        reg
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(&self.data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let temp_path = self.storage_path.with_extension("tmp");
        std::fs::write(&temp_path, serialized)?;
        std::fs::rename(temp_path, &self.storage_path)?;
        Ok(())
    }

    pub fn add_entry(&mut self, entry: AuthorityEntry) {
        let key = entry.name.to_lowercase();
        self.data.entries.insert(key, entry);
        let _ = self.save();
    }

    /// Match authoritative entries against goal query tokens
    pub fn find_matches_for_goal(&self, goal: &str) -> Vec<AuthorityEntry> {
        let goal_lower = goal.to_lowercase();
        let mut matches = Vec::new();

        for entry in self.data.entries.values() {
            let mut score = 0;
            if goal_lower.contains(&entry.name.to_lowercase()) {
                score += 5;
            }
            for kw in &entry.keywords {
                if goal_lower.contains(kw) {
                    score += 2;
                }
            }
            if score >= 2 {
                matches.push((score, entry.clone()));
            }
        }

        matches.sort_by(|a, b| b.0.cmp(&a.0));
        matches.into_iter().map(|(_, e)| e).collect()
    }

    /// Check if search snippets or URLs reference any of our trusted authority platforms
    pub fn match_snippets(&self, snippets_text: &str, urls: &[String]) -> Vec<AuthorityEntry> {
        let text_lower = snippets_text.to_lowercase();
        let mut matched = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for entry in self.data.entries.values() {
            let domain_lower = entry.domain.to_lowercase();
            let name_lower = entry.name.to_lowercase();

            let url_hit = urls.iter().any(|u| u.to_lowercase().contains(&domain_lower));
            let text_hit = text_lower.contains(&name_lower);

            if (url_hit || text_hit) && !seen.contains(&entry.name) {
                seen.insert(entry.name.clone());
                matched.push(entry.clone());
            }
        }

        matched
    }

    fn seed_defaults(&mut self) {
        let seeds = vec![
            // ─── Creative & 3D Modeling ──────────────────────────────────────
            AuthorityEntry {
                name: "Blender".into(),
                domain: "blender.org".into(),
                category: "creative_3d".into(),
                description: "Free and open source 3D creation suite supporting modeling, rigging, animation, and rendering.".into(),
                default_target: "https://www.blender.org/".into(),
                keywords: vec!["3d model".into(), "3d modeling".into(), "render".into(), "mesh".into(), "cgi".into(), "sculpt".into()],
            },
            AuthorityEntry {
                name: "Autodesk Maya".into(),
                domain: "autodesk.com".into(),
                category: "creative_3d".into(),
                description: "Professional 3D animation, modeling, simulation, and rendering software.".into(),
                default_target: "https://www.autodesk.com/products/maya/overview".into(),
                keywords: vec!["3d model".into(), "maya".into(), "autodesk".into(), "animation".into()],
            },
            AuthorityEntry {
                name: "Autodesk 3ds Max".into(),
                domain: "autodesk.com".into(),
                category: "creative_3d".into(),
                description: "Professional 3D modeling and rendering software for design visualization and games.".into(),
                default_target: "https://www.autodesk.com/products/3ds-max/overview".into(),
                keywords: vec!["3d model".into(), "3ds max".into(), "cad".into(), "architecture".into()],
            },
            AuthorityEntry {
                name: "ZBrush".into(),
                domain: "maxon.net".into(),
                category: "creative_3d".into(),
                description: "Industry-standard digital sculpting tool for high-resolution 3D models.".into(),
                default_target: "https://www.maxon.net/zbrush".into(),
                keywords: vec!["sculpt".into(), "zbrush".into(), "character design".into(), "3d model".into()],
            },
            AuthorityEntry {
                name: "Tinkercad".into(),
                domain: "tinkercad.com".into(),
                category: "creative_3d".into(),
                description: "Free web-based 3D modeling app for beginners and rapid 3D printing design.".into(),
                default_target: "https://www.tinkercad.com/".into(),
                keywords: vec!["3d model".into(), "3d print".into(), "beginner 3d".into(), "cad".into()],
            },
            AuthorityEntry {
                name: "SketchUp".into(),
                domain: "sketchup.com".into(),
                category: "creative_3d".into(),
                description: "Premier 3D design software for architectural, interior, and civil modeling.".into(),
                default_target: "https://www.sketchup.com/".into(),
                keywords: vec!["3d model".into(), "architecture".into(), "interior design".into(), "sketchup".into()],
            },
            AuthorityEntry {
                name: "Sketchfab".into(),
                domain: "sketchfab.com".into(),
                category: "creative_3d".into(),
                description: "Leading platform to publish, share, buy and sell 3D content in real-time 3D.".into(),
                default_target: "https://sketchfab.com/".into(),
                keywords: vec!["3d model".into(), "3d assets".into(), "download 3d".into(), "sketchfab".into()],
            },
            AuthorityEntry {
                name: "Figma".into(),
                domain: "figma.com".into(),
                category: "creative_3d".into(),
                description: "Collaborative interface design, wireframing, and UI/UX prototyping platform.".into(),
                default_target: "https://www.figma.com/".into(),
                keywords: vec!["ui".into(), "ux".into(), "interface design".into(), "prototype".into(), "wireframe".into()],
            },

            // ─── Client Acquisition & Business ──────────────────────────────
            AuthorityEntry {
                name: "Upwork".into(),
                domain: "upwork.com".into(),
                category: "business_clients".into(),
                description: "Global freelance platform for contracting clients, agencies, and project bids.".into(),
                default_target: "https://www.upwork.com/nx/search/jobs/".into(),
                keywords: vec!["clients".into(), "agency".into(), "freelance".into(), "contracts".into(), "web development".into(), "jobs".into()],
            },
            AuthorityEntry {
                name: "LinkedIn".into(),
                domain: "linkedin.com".into(),
                category: "business_clients".into(),
                description: "World's largest professional network for B2B client prospecting, leads, and talent.".into(),
                default_target: "https://www.linkedin.com/search/results/people/".into(),
                keywords: vec!["clients".into(), "b2b".into(), "prospecting".into(), "agency".into(), "outreach".into(), "leads".into()],
            },
            AuthorityEntry {
                name: "Clutch".into(),
                domain: "clutch.co".into(),
                category: "business_clients".into(),
                description: "Leading ratings and reviews platform for B2B service providers, agencies, and developers.".into(),
                default_target: "https://clutch.co/".into(),
                keywords: vec!["clients".into(), "agency".into(), "web development".into(), "b2b directory".into(), "clutch".into()],
            },
            AuthorityEntry {
                name: "Fiverr Pro".into(),
                domain: "fiverr.com".into(),
                category: "business_clients".into(),
                description: "Freelance marketplace connecting businesses with vetted agency talent and projects.".into(),
                default_target: "https://www.fiverr.com/pro".into(),
                keywords: vec!["clients".into(), "freelance".into(), "gigs".into(), "agency".into()],
            },
            AuthorityEntry {
                name: "Apollo".into(),
                domain: "apollo.io".into(),
                category: "business_clients".into(),
                description: "B2B sales intelligence and lead discovery platform with contact database and outreach.".into(),
                default_target: "https://www.apollo.io/".into(),
                keywords: vec!["leads".into(), "prospects".into(), "cold email".into(), "sales".into(), "clients".into()],
            },
            AuthorityEntry {
                name: "Wellfound".into(),
                domain: "wellfound.com".into(),
                category: "business_clients".into(),
                description: "Startup job board and contracting platform connecting agencies with high-growth startups.".into(),
                default_target: "https://wellfound.com/".into(),
                keywords: vec!["startup clients".into(), "hiring".into(), "contract".into(), "developer jobs".into()],
            },

            // ─── APIs, Engineering & Code ───────────────────────────────────
            AuthorityEntry {
                name: "GitHub".into(),
                domain: "github.com".into(),
                category: "engineering_api".into(),
                description: "World's leading developer platform for code repositories, open source tools, and actions.".into(),
                default_target: "https://github.com/".into(),
                keywords: vec!["code".into(), "repository".into(), "open source".into(), "git".into(), "software".into()],
            },
            AuthorityEntry {
                name: "MDN Web Docs".into(),
                domain: "developer.mozilla.org".into(),
                category: "engineering_api".into(),
                description: "Definitive documentation and learning resource for web standards, JavaScript, CSS, and HTML.".into(),
                default_target: "https://developer.mozilla.org/".into(),
                keywords: vec!["javascript".into(), "html".into(), "css".into(), "web docs".into(), "api".into()],
            },
            AuthorityEntry {
                name: "DevDocs".into(),
                domain: "devdocs.io".into(),
                category: "engineering_api".into(),
                description: "Fast, offline-capable unified documentation browser for 100+ programming languages and APIs.".into(),
                default_target: "https://devdocs.io/".into(),
                keywords: vec!["api docs".into(), "documentation".into(), "devdocs".into(), "reference".into()],
            },
            AuthorityEntry {
                name: "Stack Overflow".into(),
                domain: "stackoverflow.com".into(),
                category: "engineering_api".into(),
                description: "The largest knowledge-sharing community for software engineers to solve programming issues.".into(),
                default_target: "https://stackoverflow.com/".into(),
                keywords: vec!["debug".into(), "code error".into(), "programming questions".into(), "solve".into()],
            },
            AuthorityEntry {
                name: "AlternativeTo".into(),
                domain: "alternativeto.net".into(),
                category: "engineering_api".into(),
                description: "Crowd-sourced software directory and recommendation platform for finding tool alternatives.".into(),
                default_target: "https://alternativeto.net/".into(),
                keywords: vec!["software alternative".into(), "tools".into(), "free alternative".into(), "programs".into()],
            },
            AuthorityEntry {
                name: "Hugging Face".into(),
                domain: "huggingface.co".into(),
                category: "engineering_api".into(),
                description: "Hub for machine learning models, open-source AI weights, datasets, and LLM spaces.".into(),
                default_target: "https://huggingface.co/".into(),
                keywords: vec!["ai model".into(), "llm".into(), "weights".into(), "machine learning".into(), "transformers".into()],
            },

            // ─── Research, Science & Academia ───────────────────────────────
            AuthorityEntry {
                name: "arXiv".into(),
                domain: "arxiv.org".into(),
                category: "research_science".into(),
                description: "Open-access archive for scholarly articles in physics, mathematics, computer science, and AI.".into(),
                default_target: "https://arxiv.org/".into(),
                keywords: vec!["research paper".into(), "arxiv".into(), "study".into(), "physics".into(), "computer science".into()],
            },
            AuthorityEntry {
                name: "PubMed".into(),
                domain: "pubmed.ncbi.nlm.nih.gov".into(),
                category: "research_science".into(),
                description: "Free biomedical and life sciences research citation database from the NIH National Library.".into(),
                default_target: "https://pubmed.ncbi.nlm.nih.gov/".into(),
                keywords: vec!["medicine".into(), "biology".into(), "clinical study".into(), "biomedical".into(), "health research".into()],
            },
            AuthorityEntry {
                name: "Google Scholar".into(),
                domain: "scholar.google.com".into(),
                category: "research_science".into(),
                description: "Broad academic paper search engine across peer-reviewed papers, theses, books, and abstracts.".into(),
                default_target: "https://scholar.google.com/".into(),
                keywords: vec!["academic paper".into(), "citations".into(), "peer reviewed".into(), "journal".into(), "scholar".into()],
            },
            AuthorityEntry {
                name: "Semantic Scholar".into(),
                domain: "semanticscholar.org".into(),
                category: "research_science".into(),
                description: "AI-powered academic research tool from the Allen Institute for AI extracting key paper findings.".into(),
                default_target: "https://www.semanticscholar.org/".into(),
                keywords: vec!["paper search".into(), "literature review".into(), "scientific studies".into()],
            },
            AuthorityEntry {
                name: "Wikipedia".into(),
                domain: "wikipedia.org".into(),
                category: "research_science".into(),
                description: "Global open encyclopedia with comprehensive historical, scientific, and factual knowledge graphs.".into(),
                default_target: "https://www.wikipedia.org/".into(),
                keywords: vec!["wiki".into(), "history".into(), "overview".into(), "encyclopedia".into(), "biography".into()],
            },

            // ─── Film, Entertainment & Media ─────────────────────────────────
            AuthorityEntry {
                name: "IMDb".into(),
                domain: "imdb.com".into(),
                category: "media_entertainment".into(),
                description: "The authoritative world database for movies, TV series, directors, cast, and crew reviews.".into(),
                default_target: "https://www.imdb.com/".into(),
                keywords: vec!["movie".into(), "film".into(), "director".into(), "cinema".into(), "actor".into(), "tv show".into()],
            },
            AuthorityEntry {
                name: "Letterboxd".into(),
                domain: "letterboxd.com".into(),
                category: "media_entertainment".into(),
                description: "Social film discovery and diary platform tracking cinema reviews, ratings, and curated lists.".into(),
                default_target: "https://letterboxd.com/".into(),
                keywords: vec!["movie".into(), "film list".into(), "cinema".into(), "letterboxd".into()],
            },
            AuthorityEntry {
                name: "TMDB".into(),
                domain: "themoviedb.org".into(),
                category: "media_entertainment".into(),
                description: "The Movie Database — community-built film and TV metadata repository and REST API.".into(),
                default_target: "https://www.themoviedb.org/".into(),
                keywords: vec!["movie database".into(), "tmdb".into(), "film details".into(), "streaming".into()],
            },
        ];

        for s in seeds {
            self.data.entries.insert(s.name.to_lowercase(), s);
        }
    }
}
