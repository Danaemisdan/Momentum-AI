use crate::types::UIElement;
use chromiumoxide::Page;
use serde_json::Value;

/// Robust perception engine that extracts interactive elements from the main page.
pub async fn perceive(page: &Page) -> Result<Vec<UIElement>, Box<dyn std::error::Error + Send + Sync>> {
    let mut all_elements = Vec::new();
    
    let js = r#"
    (() => {
        const TAGS = new Set(['input','textarea','button','a','select','option','details','summary']);
        const vh = window.innerHeight > 0 ? window.innerHeight : 1080;
        const elements = Array.from(document.querySelectorAll('*'))
            .filter(el => {
                const rect = el.getBoundingClientRect();
                const style = window.getComputedStyle(el);
                const isInteractive = TAGS.has(el.tagName.toLowerCase()) || 
                                     style.cursor === 'pointer' || 
                                     el.getAttribute('role') === 'button' ||
                                     el.getAttribute('role') ||
                                     el.tagName.toLowerCase() === 'iframe';
                return isInteractive
                    && rect.width > 0 && rect.height > 0
                    && rect.bottom > -200 && rect.top < vh + 500
                    && style.display !== 'none'
                    && style.visibility !== 'hidden'
                    && style.opacity !== '0';
            })
            .map((el, idx) => {
                const rect = el.getBoundingClientRect();
                const style = window.getComputedStyle(el);
                const href = el instanceof HTMLAnchorElement && el.href ? el.href : '';
                const title = el.getAttribute('title') || '';
                const aria = el.getAttribute('aria-label') || '';
                const placeholder = el.getAttribute('placeholder') || '';
                const innerText = (el.innerText || '').trim();
                const shortenedHref = href ? href.replace(/^https?:\/\//, '').slice(0, 100) : '';
                const bestText = (aria || title || placeholder || innerText || shortenedHref).trim().slice(0, 100);
                // CRITICAL: Deterministic IDs with NO random suffix.
                // The model must output exactly this ID in its JSON.
                const localId = `el_${idx}`;
                el.setAttribute('data-momentum-id', localId); 
                return {
                    id: localId,
                    tag: el.tagName.toLowerCase(),
                    role: el.getAttribute('role') || el.tagName.toLowerCase(),
                    text: bestText,
                    aria_label: aria,
                    placeholder,
                    href,
                    title,
                    is_clickable: href.length > 0 || style.cursor === 'pointer' || ['button','a','summary','option'].includes(el.tagName.toLowerCase()) || el.getAttribute('role') === 'button',
                    is_input_like: ['input','textarea','select'].includes(el.tagName.toLowerCase()) || ['textbox','searchbox','combobox'].includes(el.getAttribute('role') || ''),
                    x: rect.x + rect.width / 2,
                    y: rect.y + rect.height / 2
                };
            });
        return elements;
    })()
    "#;

    let eval_res = page.evaluate(js).await?;
    let nodes = eval_res.value().cloned().unwrap_or(Value::Null);
            
    if let Some(nodes_array) = nodes.as_array() {
        for v in nodes_array {
            let local_id = v["id"].as_str().unwrap_or("error").to_string();
            // ID is "main:el_0" — exactly what the model will see and must output as the selector
            let full_id = format!("main:{}", local_id);
            all_elements.push(UIElement {
                id: full_id,
                role: v["role"].as_str().unwrap_or("").to_string(),
                text: v["text"].as_str().unwrap_or("").to_string(),
                selector: format!("[data-momentum-id='{}']", local_id),
                x: v["x"].as_f64().unwrap_or(0.0),
                y: v["y"].as_f64().unwrap_or(0.0),
                href: v["href"].as_str().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                title: v["title"].as_str().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                tag: v["tag"].as_str().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                aria_label: v["aria_label"].as_str().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                placeholder: v["placeholder"].as_str().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                is_clickable: v["is_clickable"].as_bool().unwrap_or(false),
                is_input_like: v["is_input_like"].as_bool().unwrap_or(false),
                frame_id: None,
            });
        }
    }
    
    // Sort top-to-bottom (reading order) and cap at 80
    all_elements.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
    Ok(all_elements.into_iter().take(80).collect())
}

/// Skill-aware page summary.
///
/// Instead of blindly dumping DOM text, we:
///   1. Extract raw page signals (title, headings, visible text, page state)
///   2. Cross-reference the URL against the skills DB
///   3. If a platform skill matches, prepend a "[PLATFORM: ...]" block telling
///      the model exactly what kind of page this is and what state it's in
///   4. Run wall/block detection (CAPTCHA, login wall, Cloudflare, etc.)
///      and prepend a hard "[BLOCKED: ...]" signal if triggered
///
/// The model reads this verbatim in every OODA step — it IS the agent's eyes.
pub async fn perceive_summary(page: &Page, current_url: &str) -> String {
    // ── 1. Raw DOM signals ─────────────────────────────────────────────────
    let js = r#"
    (() => {
        const title = document.title || '';
        const h1 = Array.from(document.querySelectorAll('h1,h2'))
            .map(e => e.innerText.trim()).filter(Boolean).slice(0, 2).join(' | ');
        const meta = document.querySelector('meta[name="description"]');
        const metaDesc = meta ? meta.getAttribute('content').slice(0, 150) : '';
        const bodyText = Array.from(document.querySelectorAll('p,span,li'))
            .filter(e => {
                const s = window.getComputedStyle(e);
                return s.display !== 'none' && s.visibility !== 'hidden' 
                    && e.innerText && e.innerText.trim().length > 15;
            })
            .map(e => e.innerText.trim().slice(0, 80))
            .slice(0, 4)
            .join(' • ');
        // Iframe signals (reCAPTCHA, Cloudflare, etc. live in iframes)
        const iframes = Array.from(document.querySelectorAll('iframe'))
            .map(f => f.src || f.title || '').filter(Boolean).join(', ');
        const parts = [title, h1, metaDesc, bodyText, iframes ? `[IFRAMES: ${iframes}]` : ''].filter(Boolean);
        return parts.join(' — ').slice(0, 800);
    })()
    "#;

    let raw_summary = match page.evaluate(js).await {
        Ok(res) => res.value()
            .and_then(|v| v.as_str().map(String::from))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Empty or loading page.".into()),
        Err(_) => "Page summary unavailable.".into(),
    };

    // ── 2. Wall/block detection ────────────────────────────────────────────
    // These are hard-coded signal strings that indicate the agent is BLOCKED.
    // We detect at both the URL level and the content level so we catch
    // both Google /sorry redirects AND content-level blocks (Cloudflare spinner,
    // LinkedIn login wall, reCAPTCHA iframe, etc.)
    let url_lower = current_url.to_lowercase();
    let summary_lower = raw_summary.to_lowercase();

    let block_signal: Option<(&str, &str)> = if url_lower.contains("/sorry") || url_lower.contains("google.com/sorry") {
        Some(("CAPTCHA", "Google has redirected to a bot-check page (/sorry). You cannot search here. Use action=ask to tell the user, or navigate to bing.com instead."))
    } else if url_lower.contains("captcha") || url_lower.contains("recaptcha") {
        Some(("CAPTCHA", "This page is a CAPTCHA challenge. You cannot proceed automatically. Use action=ask to request user help."))
    } else if summary_lower.contains("just a moment") || summary_lower.contains("checking your browser") || summary_lower.contains("enable javascript and cookies") {
        Some(("CLOUDFLARE", "Cloudflare browser check is active. Wait 3-5 seconds and scroll — it usually auto-clears. If still stuck after 2 retries, use action=ask."))
    } else if summary_lower.contains("i'm not a robot") || summary_lower.contains("i am not a robot") || summary_lower.contains("verify you are human") {
        Some(("RECAPTCHA", "reCAPTCHA checkbox detected. Look for an iframe or checkbox element in UI ELEMENTS and click it. If not visible, use action=ask."))
    } else if summary_lower.contains("unusual traffic") || summary_lower.contains("detected unusual") {
        Some(("RATE_LIMITED", "Search engine has flagged unusual traffic. Navigate to bing.com and retry the search there instead."))
    } else if summary_lower.contains("sign in to continue") || summary_lower.contains("log in to see") || summary_lower.contains("create an account") || (summary_lower.contains("sign in") && summary_lower.contains("continue")) {
        Some(("LOGIN_WALL", "This page requires login to proceed. Use action=ask to inform the user they need to sign in, or look for a 'Skip' or 'Continue as guest' option in UI ELEMENTS."))
    } else if summary_lower.contains("access denied") || summary_lower.contains("403 forbidden") || summary_lower.contains("you don't have permission") {
        Some(("ACCESS_DENIED", "Access denied. This page is blocked. Navigate away and try a different approach or site."))
    } else {
        None
    };

    // ── 3. Platform skill lookup ───────────────────────────────────────────
    // Check skills DB for a matching platform. If found, prepend a context
    // block so the model knows exactly what kind of platform it's on.
    let platform_context = skill_context_for_url(current_url);

    // ── 4. Build final enriched summary ───────────────────────────────────
    let mut parts: Vec<String> = Vec::new();

    // Block signal is HIGHEST priority — always first, always uppercase
    if let Some((kind, guidance)) = block_signal {
        parts.push(format!("[BLOCKED: {}] {}", kind, guidance));
    }

    // Platform context next — tells model what kind of site/state this is
    if let Some(ctx) = platform_context {
        parts.push(ctx);
    }

    // Raw DOM summary last — actual visible content
    parts.push(raw_summary);

    parts.join("\n")
}

/// Ask the Python Vision Engine what it currently sees, optionally cropped to a specific element.
pub async fn perceive_vision(crop_box: Option<(i32, i32, i32, i32)>) -> String {
    let client = reqwest::Client::new();
    let mut payload = serde_json::json!({
        "use_shm": true,
        "width": 2560,
        "height": 1600,
        "prompt": "Describe this visual element concisely."
    });
    
    if let Some((x, y, w, h)) = crop_box {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("crop_x".to_string(), serde_json::json!(x));
            obj.insert("crop_y".to_string(), serde_json::json!(y));
            obj.insert("crop_w".to_string(), serde_json::json!(w));
            obj.insert("crop_h".to_string(), serde_json::json!(h));
        }
    }

    match client.post("http://127.0.0.1:8001/perceive")
        .json(&payload)
        .send()
        .await 
    {
        Ok(res) => {
            if let Ok(json) = res.json::<serde_json::Value>().await {
                json["text"].as_str().unwrap_or("Vision stream returned empty text.").to_string()
            } else {
                "Vision stream returned invalid JSON.".into()
            }
        },
        Err(e) => format!("Vision stream disconnected: {}", e)
    }
}

/// Cross-reference the current URL against the skills DB.
/// Returns a compact "[PLATFORM: X | STATE: Y]" string if a skill matches,
/// so the model knows the platform context before deciding its next action.
fn skill_context_for_url(url: &str) -> Option<String> {
    let url_lower = url.to_lowercase();
    
    // Walk the platforms skills directory
    let skill_dir = std::path::Path::new("skills/platforms");
    let files = find_yaml_files(skill_dir);
    
    for path in files {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(skill) = serde_yaml::from_str::<crate::types::PlatformSkill>(&content) {
                if url_lower.contains(&skill.platform.to_lowercase()) {
                    // Determine what state we're likely in based on URL path
                    let state = infer_platform_state(url, &skill.platform);
                    return Some(format!(
                        "[PLATFORM: {} | STATE: {} | BEHAVIOR: {}]",
                        skill.platform,
                        state,
                        // First 200 chars of system_behavior as a hint
                        skill.system_behavior.chars().take(200).collect::<String>()
                    ));
                }
            }
        }
    }

    // No skill file found — still give a generic platform label from URL
    generic_platform_label(&url_lower)
}

/// Infer what "state" we're in on a platform based on the URL path.
/// e.g. linkedin.com/jobs → "Job Search", linkedin.com/in/ → "Profile View"
fn infer_platform_state(url: &str, platform: &str) -> &'static str {
    let u = url.to_lowercase();
    let p = platform.to_lowercase();

    if p.contains("bing") {
        if u.contains("/search") { return "Search Results"; }
        if u.contains("news.bing") { return "News Results"; }
        return "Bing Homepage";
    }
    if p.contains("google") {
        if u.contains("/search") { return "Search Results"; }
        if u.contains("/sorry") { return "BOT CHECK - BLOCKED"; }
        if u.contains("maps") { return "Maps"; }
        return "Google Homepage";
    }
    if p.contains("linkedin") {
        if u.contains("/jobs") { return "Job Search"; }
        if u.contains("/in/") { return "Profile View"; }
        if u.contains("/feed") { return "Feed"; }
        if u.contains("/messaging") { return "Messages"; }
        if u.contains("/company") { return "Company Page"; }
        return "LinkedIn";
    }
    if p.contains("youtube") {
        if u.contains("/watch") { return "Video Player"; }
        if u.contains("/results") { return "Search Results"; }
        if u.contains("/@") || u.contains("/channel") { return "Channel Page"; }
        return "YouTube Homepage";
    }
    if p.contains("amazon") {
        if u.contains("/s?") || u.contains("/s/") { return "Search Results"; }
        if u.contains("/dp/") { return "Product Page"; }
        if u.contains("/cart") { return "Cart"; }
        if u.contains("/checkout") { return "Checkout"; }
        return "Amazon";
    }
    if p.contains("github") {
        if u.contains("/pull/") { return "Pull Request"; }
        if u.contains("/issues/") { return "Issue"; }
        if u.contains("/blob/") { return "File View"; }
        if u.contains("/tree/") { return "Directory View"; }
        return "Repository";
    }
    if p.contains("notion") {
        return "Notion Document";
    }
    if p.contains("upwork") {
        if u.contains("/jobs/") { return "Job Listing"; }
        if u.contains("/search/") { return "Job Search"; }
        return "Upwork";
    }

    "Active Page"
}

/// For URLs with no matching skill file, still give a basic platform label
fn generic_platform_label(url: &str) -> Option<String> {
    // Known platforms we don't have skills for yet
    let known = [
        ("reddit.com", "Reddit", "Community discussion site. Results appear as post cards. Click post titles to open threads."),
        ("twitter.com", "Twitter/X", "Social feed. Timeline shows tweet cards. Search bar at top."),
        ("x.com", "Twitter/X", "Social feed. Timeline shows tweet cards. Search bar at top."),
        ("indeed.com", "Indeed", "Job board. Search bar at top. Results are job cards with title, company, location."),
        ("glassdoor.com", "Glassdoor", "Job and salary review site. Login walls are common — look for Skip option."),
        ("bing.com", "Bing", "Search engine. Search bar at top. Results are blue link cards. No bot blocks for headless Chrome."),
        ("wikipedia.org", "Wikipedia", "Encyclopedia. Article text is the main content. Infobox on right."),
        ("stackoverflow.com", "Stack Overflow", "Programming Q&A. Questions have score, answers, accepted checkmark."),
        ("cloudflare.com", "Cloudflare", "If you see a spinner or 'Just a moment', this is a Cloudflare JS challenge — wait 3-5s."),
    ];

    for (domain, name, hint) in &known {
        if url.contains(domain) {
            return Some(format!("[PLATFORM: {} | STATE: Active Page | BEHAVIOR: {}]", name, hint));
        }
    }

    None
}

fn find_yaml_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(find_yaml_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                files.push(path);
            }
        }
    }
    files
}
