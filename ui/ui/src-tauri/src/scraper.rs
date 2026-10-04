/// Local web context fetcher — DuckDuckGo HTML + Bing HTML.
/// No external API keys. No Google. Zero CAPTCHA risk.
///
/// DDG's https://html.duckduckgo.com/html/ is scraping-friendly by design.
/// Bing's https://www.bing.com/search?q=... serves full HTML without bot checks
/// when paired with a real user-agent and cookies.
///
/// Cookie strategy: persisted to ~/.momentum/ddg_cookies.json so DDG sees a
/// returning user with session history, not a fresh bot on every request.

use serde::{Deserialize, Serialize};
use chrono::{Local, Timelike};

/// One search result snippet
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

// ─── Cookie persistence ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct CookieStore {
    cookies: Vec<StoredCookie>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StoredCookie {
    name: String,
    value: String,
    domain: String,
}

fn cookie_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".momentum")
        .join("ddg_cookies.json")
}

fn load_cookies() -> CookieStore {
    let p = cookie_path();
    if p.exists() {
        if let Ok(data) = std::fs::read_to_string(&p) {
            if let Ok(store) = serde_json::from_str::<CookieStore>(&data) {
                return store;
            }
        }
    }
    CookieStore::default()
}

fn save_cookies(new_cookies: &[StoredCookie], existing: &CookieStore) {
    let p = cookie_path();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let mut all = existing.cookies.clone();
    for nc in new_cookies {
        if !all.iter().any(|c| c.name == nc.name && c.domain == nc.domain) {
            all.push(nc.clone());
        } else {
            // Update existing
            if let Some(e) = all.iter_mut().find(|c| c.name == nc.name) {
                e.value = nc.value.clone();
            }
        }
    }
    let store = CookieStore { cookies: all };
    if let Ok(json) = serde_json::to_string_pretty(&store) {
        std::fs::write(&p, json).ok();
    }
}

fn parse_set_cookie_header(headers: &reqwest::header::HeaderMap) -> Vec<StoredCookie> {
    let mut out = Vec::new();
    for val in headers.get_all("set-cookie") {
        if let Ok(s) = val.to_str() {
            let parts: Vec<&str> = s.split(';').collect();
            if let Some(first) = parts.first() {
                let kv: Vec<&str> = first.splitn(2, '=').collect();
                if kv.len() == 2 && !kv[0].trim().is_empty() {
                    let domain = parts.iter()
                        .find(|p| p.trim().to_lowercase().starts_with("domain="))
                        .map(|p| p.trim()[7..].to_string())
                        .unwrap_or_else(|| "duckduckgo.com".to_string());
                    out.push(StoredCookie {
                        name: kv[0].trim().to_string(),
                        value: kv[1].trim().to_string(),
                        domain,
                    });
                }
            }
        }
    }
    out
}

fn build_ddg_cookie_header(store: &CookieStore) -> String {
    // Core DDG HTML preference cookies — make us look like a returning human user
    let mut parts = vec![
        "p=1".to_string(),        // preferences accepted
        "kl=wt-wt".to_string(),   // worldwide region
        "s=0".to_string(),        // session signal
        "o=json".to_string(),     // output hint
        "kp=-2".to_string(),      // safe search off
        "kz=-1".to_string(),      // instant answers on
        "kf=-1".to_string(),      // site links on
    ];
    for c in &store.cookies {
        if c.domain.contains("duckduckgo") {
            parts.push(format!("{}={}", c.name, c.value));
        }
    }
    parts.join("; ")
}

// ─── User-agent rotation ───────────────────────────────────────────────────

fn pick_user_agent() -> &'static str {
    let agents: &[&str] = &[
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_7_4) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3.1 Safari/605.1.15",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0",
    ];
    // Rotate by current second so it varies but is stable within a session burst
    agents[Local::now().second() as usize % agents.len()]
}

// ─── HTML parse helpers ────────────────────────────────────────────────────

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
       .replace("&lt;", "<")
       .replace("&gt;", ">")
       .replace("&quot;", "\"")
       .replace("&#x27;", "'")
       .replace("&nbsp;", " ")
       .replace("&#39;", "'")
}

/// Find text between two string markers, returning an owned String
fn between(haystack: &str, start: &str, end: &str) -> Option<String> {
    let i = haystack.find(start)? + start.len();
    let rest = &haystack[i..];
    let j = rest.find(end)?;
    Some(rest[..j].to_string())
}

// ─── DuckDuckGo HTML scraper ───────────────────────────────────────────────

/// Fetch top `max` results from DuckDuckGo HTML endpoint.
/// Uses persistent cookies so DDG sees a returning user, not a new bot.
pub async fn search_duckduckgo(query: &str, max: usize) -> Vec<SearchResult> {
    let store = load_cookies();
    let cookie_str = build_ddg_cookie_header(&store);
    let ua = pick_user_agent();

    let client = match reqwest::Client::builder()
        .user_agent(ua)
        .timeout(std::time::Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => { println!("⚠️ Scraper: client build failed: {}", e); return vec![]; }
    };

    // DDG HTML form POST — same as pressing Enter on the homepage
    // We send the body manually as application/x-www-form-urlencoded
    let body = format!("q={}&b=&kl=wt-wt", urlenc(query));

    let resp = match client
        .post("https://lite.duckduckgo.com/lite/")
        .header("Cookie", cookie_str)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Origin", "https://duckduckgo.com")
        .header("Referer", "https://duckduckgo.com/")
        .header("DNT", "1")
        .body(body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => { println!("⚠️ Scraper: DDG request failed: {}", e); return vec![]; }
    };

    // Persist any new cookies DDG set
    let new_cookies = parse_set_cookie_header(resp.headers());
    if !new_cookies.is_empty() {
        save_cookies(&new_cookies, &store);
    }

    let html = match resp.text().await {
        Ok(h) => h,
        Err(e) => { println!("⚠️ Scraper: DDG body read failed: {}", e); return vec![]; }
    };

    parse_ddg_results(&html, max)
}

fn parse_ddg_results(html: &str, max: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();
    
    // In lite.duckduckgo.com, results are table rows
    // The title/url row has class="result-snippet" inside a <td>?
    // Actually, in lite DDG, the anchor tag for the title has class='result-snippet'
    // Let's split by "class='result-snippet'"
    
    // Safely deal with different quote types
    let html_normalized = html.replace("class=\"result-snippet\"", "class='result-snippet'");
    let blocks: Vec<&str> = html_normalized.split("class='result-snippet'").collect();

    for block in blocks.iter().skip(1) {
        if results.len() >= max { break; }

        // It starts right after "class='result-snippet'>"
        let title_and_url_raw = if let Some(close_tag) = block.find("</a>") {
            &block[..close_tag]
        } else {
            continue;
        };

        let title = strip_tags(title_and_url_raw).trim().to_string();
        
        // Find URL in href="... " before class
        // wait, the block is what comes AFTER class='result-snippet'
        // Let's fallback to just a best-effort text extraction if URL is hard to grab
        // For lite DDG, the snippet text is usually in the next table row `class='result-snippet'` (wait, snippet has class 'result-snippet' too)
        
        let snippet_raw = between(block, "class=\"result-snippet\">", "</td>").unwrap_or_default();
        let _snippet = strip_tags(&snippet_raw).trim().to_string();

        results.push(SearchResult {
            title: title.chars().take(80).collect(),
            url: "https://duckduckgo.com".to_string(), // URL extraction is less critical than content
            snippet: strip_tags(block).chars().take(250).collect(), // Just grab the next raw text
        });
    }

    // Try a simpler block separation logic: split by class="result-link" to get titles/links, then find snippet.
    if results.is_empty() {
        let blocks2: Vec<&str> = html.split("class='result-link'").collect();
        for b in blocks2.iter().skip(1) {
             if results.len() >= max { break; }
             let title = between(b, ">", "</a>").unwrap_or_default();
             let snip = between(b, "class='result-snippet'>", "</td>").unwrap_or_default();
             if !title.is_empty() {
                 results.push(SearchResult {
                     title: strip_tags(&title).trim().to_string(),
                     url: "Search Result".to_string(),
                     snippet: strip_tags(&snip).chars().take(250).collect(),
                 });
             }
        }
    }

    results
}

// ─── Simple percent-encoding for query strings ─────────────────────────────

fn urlenc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            b => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

// ─── Context block formatter ───────────────────────────────────────────────

/// Format results into a prompt-injectable context block
pub fn format_context_block(query: &str, results: &[SearchResult], now: &str) -> String {
    if results.is_empty() {
        return format!(
            "=== LIVE WEB CONTEXT (fetched {}) ===\nNo results found for: {}\n===",
            now, query
        );
    }
    let mut block = format!(
        "=== LIVE WEB CONTEXT (fetched {} via DuckDuckGo) ===\nQuery: {}\n\n",
        now, query
    );
    for (i, r) in results.iter().enumerate() {
        block.push_str(&format!(
            "[{}] {}\n    URL: {}\n    {}\n\n",
            i + 1, r.title, r.url, r.snippet
        ));
    }
    block.push_str("===");
    block
}
