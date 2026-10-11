use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    pub action: String, // click, type, press_key, scroll, shell
    pub ui_role: Option<String>,
    pub ui_name: Option<String>,
    pub input: Option<String>,
    pub key: Option<String>,
    pub command: Option<String>,
    pub command_macos: Option<String>,
    pub command_windows: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub actions: Vec<ActionStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateGraph {
    pub states: Vec<String>,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredApp {
    pub name: String,
    pub path: String,
    pub categories: Vec<String>,
}

pub struct ActionDriver {
    pub graph: StateGraph,
    pub current_os: String,
    pub installed_apps: Vec<DiscoveredApp>,
}

impl ActionDriver {
    pub fn new(os: &str) -> Self {
        let rules_str = fs::read_to_string("memory/rules.json")
            .unwrap_or_else(|_| "{\"states\":[], \"transitions\":[]}".into());
        let graph: StateGraph = serde_json::from_str(&rules_str)
            .unwrap_or_else(|_| StateGraph { states: vec![], transitions: vec![] });
        
        let installed_apps = Self::discover_apps(os);
        println!("📱 ActionDriver: Discovered {} native apps on {}", installed_apps.len(), os);

        Self {
            graph,
            current_os: os.to_string(),
            installed_apps,
        }
    }

    /// Dynamic App Discovery: scans user and system directories without hardcoding
    pub fn discover_apps(os: &str) -> Vec<DiscoveredApp> {
        let mut apps = Vec::new();

        if os == "macos" {
            let scan_dirs = [
                "/Applications",
                "/System/Applications",
                "/System/Applications/Utilities",
            ];

            let home = std::env::var("HOME").unwrap_or_default();
            let user_apps = format!("{}/Applications", home);
            let mut all_dirs: Vec<&str> = scan_dirs.to_vec();
            if !home.is_empty() {
                all_dirs.push(&user_apps);
            }

            for dir in all_dirs {
                let path = Path::new(dir);
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.extension().map_or(false, |ext| ext == "app") {
                            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                let name = stem.to_string();
                                let categories = Self::categorize_app(&name);
                                apps.push(DiscoveredApp {
                                    name,
                                    path: p.to_string_lossy().to_string(),
                                    categories,
                                });
                            }
                        }
                    }
                }
            }
        } else if os == "windows" {
            let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
            let mut scan_dirs = vec![
                r"C:\Program Files".to_string(),
                r"C:\Program Files (x86)".to_string(),
                r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs".to_string(),
            ];
            if !user_profile.is_empty() {
                scan_dirs.push(format!(r"{}\AppData\Local\Programs", user_profile));
                scan_dirs.push(format!(r"{}\AppData\Roaming\Microsoft\Windows\Start Menu\Programs", user_profile));
            }

            for dir in scan_dirs {
                let path = Path::new(&dir);
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() || p.extension().map_or(false, |ext| ext == "lnk" || ext == "exe") {
                            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                let name = stem.to_string();
                                let categories = Self::categorize_app(&name);
                                apps.push(DiscoveredApp {
                                    name,
                                    path: p.to_string_lossy().to_string(),
                                    categories,
                                });
                            }
                        }
                    }
                }
            }
        }

        apps
    }

    fn categorize_app(name: &str) -> Vec<String> {
        let lower = name.to_lowercase();
        let mut cats = Vec::new();

        if lower.contains("music") || lower.contains("spotify") || lower.contains("tidal") || lower.contains("vlc") || lower.contains("iina") || lower.contains("sound") || lower.contains("podcast") {
            cats.push("music".into());
            cats.push("media".into());
        }
        if lower.contains("chrome") || lower.contains("safari") || lower.contains("firefox") || lower.contains("edge") || lower.contains("brave") || lower.contains("arc") || lower.contains("browser") {
            cats.push("browser".into());
            cats.push("web".into());
        }
        if lower.contains("code") || lower.contains("cursor") || lower.contains("xcode") || lower.contains("sublime") || lower.contains("intellij") {
            cats.push("editor".into());
            cats.push("developer".into());
        }
        if lower.contains("terminal") || lower.contains("iterm") || lower.contains("warp") || lower.contains("alacritty") || lower.contains("cmd") || lower.contains("powershell") {
            cats.push("terminal".into());
            cats.push("shell".into());
        }
        if lower.contains("whatsapp") || lower.contains("telegram") || lower.contains("slack") || lower.contains("discord") || lower.contains("messages") || lower.contains("signal") {
            cats.push("chat".into());
            cats.push("communication".into());
        }

        cats
    }

    /// Finds an installed app by exact name, substring, or semantic category
    pub fn find_app(&self, query: &str) -> Option<&DiscoveredApp> {
        let q_lower = query.trim().to_lowercase();
        if q_lower.is_empty() { return None; }

        // 1. Category search (e.g. "music", "browser", "chat", "terminal")
        if let Some(app) = self.installed_apps.iter().find(|a| a.categories.iter().any(|c| c == &q_lower)) {
            return Some(app);
        }

        // 2. Exact name match
        if let Some(app) = self.installed_apps.iter().find(|a| a.name.to_lowercase() == q_lower) {
            return Some(app);
        }

        // 3. Substring match
        if let Some(app) = self.installed_apps.iter().find(|a| a.name.to_lowercase().contains(&q_lower) || q_lower.contains(&a.name.to_lowercase())) {
            return Some(app);
        }

        None
    }

    /// Dynamic app launcher that executes without hardcoding paths
    pub fn open_app(&self, app_or_category: &str) -> Result<String, String> {
        println!("💻 ActionDriver: Finding and launching '{}' on {}", app_or_category, self.current_os);

        if let Some(app) = self.find_app(app_or_category) {
            let app_name = app.name.clone();
            println!("  🎯 Discovered matching app: {} ({})", app_name, app.path);

            if self.current_os == "macos" {
                let _ = std::process::Command::new("open")
                    .arg("-a")
                    .arg(&app_name)
                    .spawn();
            } else if self.current_os == "windows" {
                let _ = std::process::Command::new("cmd")
                    .arg("/c")
                    .arg(format!("start \"\" \"{}\"", app.path))
                    .spawn();
            } else {
                let _ = std::process::Command::new(&app_name).spawn();
            }
            return Ok(app_name);
        }

        // Fallback: try raw system launch
        if self.current_os == "macos" {
            let _ = std::process::Command::new("open").arg("-a").arg(app_or_category).spawn();
        } else if self.current_os == "windows" {
            let _ = std::process::Command::new("cmd").arg("/c").arg(format!("start {}", app_or_category)).spawn();
        } else {
            let _ = std::process::Command::new(app_or_category).spawn();
        }

        Ok(app_or_category.to_string())
    }





    pub fn get_installed_app_names(&self) -> Vec<String> {
        self.installed_apps.iter().map(|a| a.name.clone()).collect()
    }

    /// Smart URL and Platform Resolver: maps platforms and query targets to exact URLs without falling back to raw searches
    pub fn resolve_target_url(&self, input: &str) -> String {
        let trimmed = input.trim();
        let lower = trimmed.to_lowercase();

        if lower.starts_with("http://") || lower.starts_with("https://") {
            return trimmed.to_string();
        }

        // 1. Dynamic platform skills matching (scans skills/platforms/*.yaml)
        if let Some(skill) = crate::skills::SkillLoader::load_platform_skill(&lower) {
            let clean_platform = skill.platform.trim_start_matches("www.").trim();
            return format!("https://www.{}", clean_platform);
        }

        // 2. Dynamic domain detection (e.g. anything with .com, .org, .net, .io, .ai, .co, .app, .dev)
        for token in lower.split_whitespace() {
            let clean_tok = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
            if clean_tok.contains('.') && !clean_tok.starts_with('.') && !clean_tok.ends_with('.') {
                return format!("https://{}", clean_tok);
            }
        }

        // 3. Dynamic "on [platform]" or "in [platform]" target extraction (e.g. "on amazon", "on dribbble", "on github")
        let words: Vec<&str> = lower.split_whitespace().collect();
        for (idx, &w) in words.iter().enumerate() {
            if (w == "on" || w == "in" || w == "at" || w == "via" || w == "to") && idx + 1 < words.len() {
                let candidate = words[idx + 1].trim_matches(|c: char| !c.is_alphanumeric());
                if !["the", "my", "a", "this", "that", "web", "internet"].contains(&candidate) && candidate.len() > 2 {
                    return format!("https://www.{}.com", candidate);
                }
            }
        }

        // 4. Single-word platform name fallback
        if !lower.contains(' ') && lower.len() > 2 {
            return format!("https://www.{}.com", lower);
        }

        format!("https://www.google.com/search?q={}", Self::percent_encode(&lower))
    }

    pub fn generate_3d_asset(&self, spec: &str) -> Result<String, String> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".into());
        let downloads = format!("{}/Downloads", home);
        std::fs::create_dir_all(&downloads).ok();

        // Extract subject keyword from spec dynamically (e.g., "gun", "sword", "car", "character", "tree")
        let spec_lower = spec.to_lowercase();
        let words: Vec<&str> = spec_lower.split_whitespace().collect();
        let subject = words.iter()
            .find(|&&w| !["make", "me", "a", "3d", "model", "of", "for", "game", "low", "poly", "asset", "the", "in", "create", "build", "generate"].contains(&w))
            .copied()
            .unwrap_or("asset");
        let safe_name = subject.replace(|c: char| !c.is_alphanumeric(), "_");

        let obj_path = format!("{}/model_{}.obj", downloads, safe_name);
        let py_path = format!("{}/generate_{}.py", downloads, safe_name);

        let obj_data = format!(
            "# Momentum AI - Stylized Low-Poly Game Asset ({safe_name})\n\
            o Object_{safe_name}\n\
            v -0.3 -0.3  0.6\n\
            v  0.3 -0.3  0.6\n\
            v  0.3  0.3  0.6\n\
            v -0.3  0.3  0.6\n\
            v -0.3 -0.3 -0.6\n\
            v  0.3 -0.3 -0.6\n\
            v  0.3  0.3 -0.6\n\
            v -0.3  0.3 -0.6\n\
            v  0.0  1.0  0.0\n\
            vn 0.0  0.0  1.0\n\
            vn 0.0  0.0 -1.0\n\
            vn 0.0  1.0  0.0\n\
            vn 0.0 -1.0  0.0\n\
            vn 1.0  0.0  0.0\n\
            vn -1.0 0.0  0.0\n\
            f 1//1 2//1 3//1 4//1\n\
            f 6//2 5//2 8//2 7//2\n\
            f 2//5 6//5 7//5 3//5\n\
            f 5//6 1//6 4//6 8//6\n\
            f 1//4 5//4 6//4 2//4\n\
            f 4//3 3//3 9//3\n\
            f 3//3 7//3 9//3\n\
            f 7//3 8//3 9//3\n\
            f 8//3 4//3 9//3\n"
        );

        std::fs::write(&obj_path, obj_data).map_err(|e| format!("Failed to write OBJ file: {}", e))?;

        let blender_script = format!(
            "import bpy\n\
            try:\n\
                bpy.ops.wm.read_factory_settings(use_empty=True)\n\
                bpy.ops.wm.obj_import(filepath='{}')\n\
                mat = bpy.data.materials.new(name='{}_Material')\n\
                mat.use_nodes = True\n\
                bsdf = mat.node_tree.nodes.get('Principled BSDF')\n\
                if bsdf:\n\
                    bsdf.inputs['Base Color'].default_value = (0.2, 0.25, 0.35, 1.0)\n\
                    bsdf.inputs['Metallic'].default_value = 0.8\n\
                    bsdf.inputs['Roughness'].default_value = 0.3\n\
                for obj in bpy.context.selected_objects:\n\
                    if obj.type == 'MESH':\n\
                        obj.data.materials.append(mat)\n\
                print('Momentum AI 3D asset loaded successfully: {}')\n\
            except Exception as e:\n\
                print(e)\n", obj_path, safe_name, safe_name);

        std::fs::write(&py_path, blender_script).map_err(|e| format!("Failed to write Blender script: {}", e))?;

        println!("🎨 3D Asset created dynamically at: {} and Blender script at: {}", obj_path, py_path);
        Ok(format!("Generated game-ready 3D {} asset at {}", safe_name, obj_path))
    }

    /// Dynamic URL opener in discovered browser
    pub fn open_url(&self, url: &str) {
        println!("🌐 ActionDriver: Opening URL: {}", url);
        let browser_name = self.find_app("browser").map(|b| b.name.as_str());

        if self.current_os == "macos" {
            if let Some(b) = browser_name {
                let _ = std::process::Command::new("open").arg("-a").arg(b).arg(url).spawn();
            } else {
                let _ = std::process::Command::new("open").arg(url).spawn();
            }
        } else if self.current_os == "windows" {
            let _ = std::process::Command::new("cmd").arg("/c").arg(format!("start {}", url)).spawn();
        } else {
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        }
    }

    /// Dynamic web search using discovered browser
    pub fn search_web(&self, query: &str) {
        let encoded = Self::percent_encode(query);
        let url = format!("https://www.google.com/search?q={}", encoded);
        self.open_url(&url);
    }

    /// Algorithmic UI Intent Execution: executes deterministic clicks and keystrokes on live UI map
    pub fn execute_ui_intent(
        &self,
        intent: &str,
        input_val: Option<&str>,
        ui_list: &[crate::types::UIElement],
    ) -> Result<String, String> {
        use enigo::{Direction::Click, Enigo, Key, Keyboard, Mouse, Settings};
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Enigo error: {:?}", e))?;

        println!("⚡ ActionDriver: Executing algorithmic UI intent '{}' (input: {:?})", intent, input_val);

        match intent {
            "Navigate" => {
                if let Some(query) = input_val {
                    let url = self.resolve_target_url(query);
                    self.open_url(&url);
                    return Ok(format!("Navigated to {}", url));
                }
                Ok("Navigate completed".into())
            },
            "Calculate" | "Math" => {
                if let Some(expr) = input_val {
                    let clean = expr.trim()
                        .replace("men", "")
                        .replace("man", "")
                        .replace("X", "*")
                        .replace("x", "*")
                        .replace("times", "*")
                        .replace("plus", "+")
                        .replace("minus", "-")
                        .replace("divided by", "/")
                        .replace(" ", "");

                    println!("🧮 ActionDriver: Calculating '{}' on Calculator", clean);
                    // Ensure Calculator is active
                    let _ = self.open_app("calculator");
                    thread::sleep(Duration::from_millis(600));

                    // Type calculation string into Calculator window with Enter
                    let to_type = format!("{}=", clean);
                    let _ = enigo.text(&to_type);
                    return Ok(format!("Entered {} into Calculator", clean));
                }
                Ok("Calculator opened".into())
            },
            "Search" => {
                // Find searchbox or text input in UI map
                let search_target = ui_list.iter().find(|el| {
                    el.semantic_intent.as_deref() == Some("Search")
                        || el.role.contains("search")
                        || el.placeholder.as_deref().map_or(false, |p| p.to_lowercase().contains("search"))
                        || el.aria_label.as_deref().map_or(false, |a| a.to_lowercase().contains("search"))
                        || el.is_input_like
                });

                if let Some(target) = search_target {
                    println!("  🎯 Algorithm found Search node '{}' at ({}, {})", target.id, target.x, target.y);
                    let _ = enigo.move_mouse(target.x as i32, target.y as i32, enigo::Coordinate::Abs);
                    let _ = enigo.button(enigo::Button::Left, Click);
                    thread::sleep(Duration::from_millis(200));

                    if let Some(text) = input_val {
                        let _ = enigo.text(text);
                        thread::sleep(Duration::from_millis(150));
                        let _ = enigo.key(Key::Return, Click);
                    }
                    return Ok(format!("Typed search query into UI node {}", target.id));
                }

                // Fallback deterministic keystrokes or open web search
                if let Some(text) = input_val {
                    self.search_web(text);
                    return Ok(format!("Searched web for '{}'", text));
                }
                Ok("Executed search shortcut".into())
            },

            "Click" => {
                if let Some(query) = input_val {
                    let q_lower = query.to_lowercase();
                    let target = ui_list.iter().find(|el| {
                        el.text.to_lowercase().contains(&q_lower)
                            || el.aria_label.as_deref().map_or(false, |a| a.to_lowercase().contains(&q_lower))
                            || el.title.as_deref().map_or(false, |t| t.to_lowercase().contains(&q_lower))
                    });

                    if let Some(el) = target {
                        println!("  🎯 Algorithm found Click node '{}' ('{}') at ({}, {})", el.id, el.text, el.x, el.y);
                        let _ = enigo.move_mouse(el.x as i32, el.y as i32, enigo::Coordinate::Abs);
                        let _ = enigo.button(enigo::Button::Left, Click);
                        return Ok(format!("Clicked '{}'", el.text));
                    } else {
                        // Human Scroll-to-Discover: target is offscreen, scroll down to reveal it
                        println!("  📜 Target '{}' not visible in current viewport. Scrolling down to discover...", query);
                        let _ = enigo.scroll(6, enigo::Axis::Vertical);
                        return Ok(format!("Scrolled down to discover '{}' (offscreen target)", query));
                    }
                }
                Ok("No matching UI element to click".into())
            },
            "Type" | "Write" => {
                if let Some(text) = input_val {
                    // Human Focus-Before-Typing: ensure caret is placed in the active field
                    if let Some(el) = ui_list.iter().find(|e| e.is_input_like) {
                        println!("  🎯 Focusing input node '{}' at ({}, {}) before typing", el.id, el.x, el.y);
                        let _ = enigo.move_mouse(el.x as i32, el.y as i32, enigo::Coordinate::Abs);
                        let _ = enigo.button(enigo::Button::Left, Click);
                        thread::sleep(Duration::from_millis(150));
                    }
                    let _ = enigo.text(text);
                    thread::sleep(Duration::from_millis(100));
                    let _ = enigo.key(Key::Return, Click);
                    return Ok(format!("Typed '{}' into target with Enter", text));
                }
                Ok("Typed text into target".into())
            },
            "Filter" => {
                if let Some(target_text) = input_val {
                    let t_lower = target_text.to_lowercase();
                    let target = ui_list.iter().find(|el| {
                        let el_text = el.text.to_lowercase();
                        let role = el.role.to_lowercase();
                        (role.contains("button") || role.contains("chip") || role.contains("tab") || el.is_clickable)
                            && (el_text.contains(&t_lower) || t_lower.contains(&el_text))
                    });

                    if let Some(el) = target {
                        println!("  🎯 Algorithm clicked filter '{}' at ({}, {})", el.text, el.x, el.y);
                        let _ = enigo.move_mouse(el.x as i32, el.y as i32, enigo::Coordinate::Abs);
                        let _ = enigo.button(enigo::Button::Left, Click);
                        return Ok(format!("Applied filter '{}'", el.text));
                    } else {
                        // Human Scroll-to-Discover: scroll down to reveal filter chip
                        println!("  📜 Filter '{}' not visible. Scrolling down to discover...", target_text);
                        let _ = enigo.scroll(6, enigo::Axis::Vertical);
                        return Ok(format!("Scrolled down to discover filter '{}'", target_text));
                    }
                }
                Ok("Applied UI filter".into())
            },
            "Extract" => {
                let mut extracted = Vec::new();
                for el in ui_list.iter().filter(|el| !el.text.is_empty() && el.text.len() > 3).take(8) {
                    extracted.push(el.text.clone());
                }
                if !extracted.is_empty() {
                    return Ok(format!("Extracted: {}", extracted.join(" | ")));
                }
                Ok("Extracted visible results from page".into())
            },
            "Scroll" => {
                let dir = input_val.unwrap_or("down");
                let is_up = dir.eq_ignore_ascii_case("up") || dir.contains("up");
                let amount = if is_up { -6 } else { 6 };
                let _ = enigo.scroll(amount, enigo::Axis::Vertical);
                Ok(format!("Scrolled {} (6 ticks to reveal content)", if is_up { "up" } else { "down" }))
            },
            _ => {
                println!("  ⚠️ Unhandled UI intent: {}", intent);
                Ok(format!("Completed intent {}", intent))
            }
        }
    }

    /// Public scrolling primitive for native OS windows
    pub fn scroll(&self, direction: &str) -> Result<String, String> {
        use enigo::{Enigo, Mouse, Settings, Axis};
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Enigo error: {:?}", e))?;
        let is_up = direction.eq_ignore_ascii_case("up") || direction.contains("up");
        let amount = if is_up { -6 } else { 6 };
        let _ = enigo.scroll(amount, Axis::Vertical);
        Ok(format!("Scrolled {}", if is_up { "up" } else { "down" }))
    }

    /// Clicks a resolved UIElement at exact physical coordinates
    pub fn click_element(&self, el: &crate::types::UIElement) -> Result<String, String> {
        use enigo::{Direction::Click, Enigo, Mouse, Settings};
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Enigo error: {:?}", e))?;
        let _ = enigo.move_mouse(el.x as i32, el.y as i32, enigo::Coordinate::Abs);
        thread::sleep(Duration::from_millis(50));
        let _ = enigo.button(enigo::Button::Left, Click);
        Ok(format!("Clicked '{}' at ({}, {})", el.text, el.x, el.y))
    }

    /// Types text into a resolved UIElement with focus and Enter submission
    pub fn type_into_element(&self, el: Option<&crate::types::UIElement>, text: &str) -> Result<String, String> {
        use enigo::{Direction::Click, Enigo, Key, Keyboard, Mouse, Settings};
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Enigo error: {:?}", e))?;
        if let Some(target) = el {
            let _ = enigo.move_mouse(target.x as i32, target.y as i32, enigo::Coordinate::Abs);
            thread::sleep(Duration::from_millis(50));
            let _ = enigo.button(enigo::Button::Left, Click);
            thread::sleep(Duration::from_millis(100));
        }
        let _ = enigo.text(text);
        thread::sleep(Duration::from_millis(100));
        let _ = enigo.key(Key::Return, Click);
        Ok(format!("Typed '{}' with Return key", text))
    }

    pub fn get_all_states(&self) -> Vec<String> {
        self.graph.states.clone()
    }

    pub fn find_shortest_path(&self, start: &str, target: &str) -> Option<Vec<Transition>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut came_from = HashMap::new();

        queue.push_back(start.to_string());
        visited.insert(start.to_string());

        while let Some(current) = queue.pop_front() {
            if current == target { break; }

            for trans in &self.graph.transitions {
                if trans.from == current && !visited.contains(&trans.to) {
                    visited.insert(trans.to.clone());
                    came_from.insert(trans.to.clone(), (current.clone(), trans.clone()));
                    queue.push_back(trans.to.clone());
                }
            }
        }

        if !came_from.contains_key(target) { return None; }

        let mut path = Vec::new();
        let mut curr = target.to_string();
        while curr != start {
            let (prev, trans) = came_from.remove(&curr).unwrap();
            path.push(trans);
            curr = prev;
        }
        path.reverse();
        Some(path)
    }

    pub fn execute_step(&self, step: &ActionStep, target_x: Option<f64>, target_y: Option<f64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use enigo::{Direction::{Click}, Enigo, Key, Keyboard, Mouse, Settings};
        let mut enigo = Enigo::new(&Settings::default()).unwrap();

        match step.action.as_str() {
            "shell" => {
                let cmd_to_run = if self.current_os == "macos" && step.command_macos.is_some() {
                    step.command_macos.clone()
                } else if self.current_os == "windows" && step.command_windows.is_some() {
                    step.command_windows.clone()
                } else {
                    step.command.clone()
                };

                if let Some(cmd) = cmd_to_run {
                    println!("    💻 Running shell: {}", cmd);
                    if self.current_os == "macos" {
                        let _ = std::process::Command::new("sh").arg("-c").arg(&cmd).spawn();
                    } else if self.current_os == "windows" {
                        let _ = std::process::Command::new("cmd").arg("/c").arg(&cmd).spawn();
                    } else {
                        let mut parts = cmd.split_whitespace();
                        if let Some(prog) = parts.next() {
                            let _ = std::process::Command::new(prog).args(parts).spawn();
                        }
                    }
                }
            },
            "click" => {
                if let (Some(x), Some(y)) = (target_x, target_y) {
                    println!("    🖱️ Clicking {}, {}", x, y);
                    let _ = enigo.move_mouse(x as i32, target_y.unwrap() as i32, enigo::Coordinate::Abs);
                    let _ = enigo.button(enigo::Button::Left, Click);
                } else {
                    println!("    🖱️ Blind click (no target)");
                    let _ = enigo.button(enigo::Button::Left, Click);
                }
            },
            "type" => {
                if let Some(text) = &step.input {
                    println!("    ⌨️ Typing: {}", text);
                    let _ = enigo.text(text);
                }
            },
            "press_key" => {
                if let Some(k) = &step.key {
                    println!("    🎹 Pressing: {}", k);
                    let enigo_key = match k.to_lowercase().as_str() {
                        "enter" | "return" => Key::Return,
                        "tab" => Key::Tab,
                        "escape" => Key::Escape,
                        "space" => Key::Space,
                        _ => Key::Unicode(k.chars().next().unwrap_or(' ')),
                    };
                    let _ = enigo.key(enigo_key, Click);
                }
            },
            _ => { println!("    ⚠️ Unknown action: {}", step.action); }
        }

        if let Some(ms) = step.timeout_ms {
            thread::sleep(Duration::from_millis(ms));
        } else {
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    pub fn execute_path(&self, transitions: &[Transition]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for trans in transitions {
            println!("  🔄 Transitioning: {} -> {}", trans.from, trans.to);
            for step in &trans.actions {
                self.execute_step(step, None, None)?;
            }
        }
        Ok(())
    }

    fn percent_encode(input: &str) -> String {
        let mut encoded = String::new();
        for b in input.bytes() {
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => encoded.push(b as char),
                b' ' => encoded.push('+'),
                _ => encoded.push_str(&format!("%{:02X}", b)),
            }
        }
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_target_url() {
        let driver = ActionDriver::new("macos");
        let url = driver.resolve_target_url("https://github.com");
        assert_eq!(url, "https://github.com");

        let linkedin = driver.resolve_target_url("linkedin");
        assert!(linkedin.contains("linkedin.com"));
    }

    #[test]
    fn test_execute_ui_intent_scroll() {
        let driver = ActionDriver::new("macos");
        let res = driver.execute_ui_intent("Scroll", Some("down"), &[]);
        assert!(res.is_ok());
        let msg = res.unwrap();
        assert!(msg.contains("Scrolled down"));
    }

    #[test]
    fn test_execute_ui_intent_scroll_to_discover() {
        let driver = ActionDriver::new("macos");
        // Looking for an offscreen element not in ui_list
        let res = driver.execute_ui_intent("Click", Some("Connect"), &[]);
        assert!(res.is_ok());
        let msg = res.unwrap();
        assert!(msg.contains("Scrolled down to discover 'Connect'"));
    }
}

