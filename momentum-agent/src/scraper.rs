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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
    // Core DDG HTML preference cookies
    let mut parts = vec![
        "kl=wt-wt".to_string(),   // worldwide region
        "kp=-2".to_string(),      // safe search off
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

// ─── DuckDuckGo Instant Knowledge API ──────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
struct DdgApiResponse {
    #[serde(default, rename = "Heading")]
    heading: String,
    #[serde(default, rename = "AbstractText")]
    abstract_text: String,
    #[serde(default, rename = "AbstractURL")]
    abstract_url: String,
    #[serde(default, rename = "RelatedTopics")]
    related_topics: Vec<DdgApiTopic>,
    #[serde(default, rename = "Results")]
    results: Vec<DdgApiResult>,
}

#[derive(Debug, Deserialize, Default)]
struct DdgApiTopic {
    #[serde(default, rename = "Text")]
    text: String,
    #[serde(default, rename = "FirstURL")]
    first_url: String,
    #[serde(default, rename = "Topics")]
    topics: Vec<DdgApiTopic>,
}

#[derive(Debug, Deserialize, Default)]
struct DdgApiResult {
    #[serde(default, rename = "Text")]
    text: String,
    #[serde(default, rename = "FirstURL")]
    first_url: String,
}

pub async fn query_ddg_knowledge_api(query: &str, max: usize) -> Vec<SearchResult> {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()
    {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=0", urlenc(query));
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let data: DdgApiResponse = match resp.json().await {
        Ok(d) => d,
        Err(_) => return vec![],
    };

    let mut results = Vec::new();

    // Direct Abstract Definition
    if !data.heading.is_empty() && !data.abstract_text.is_empty() {
        results.push(SearchResult {
            title: data.heading.clone(),
            url: if data.abstract_url.is_empty() { format!("https://duckduckgo.com/?q={}", urlenc(&data.heading)) } else { data.abstract_url },
            snippet: data.abstract_text,
        });
    }

    // Direct Results
    for r in data.results {
        if results.len() >= max { break; }
        if !r.text.is_empty() {
            let title = r.text.split(" - ").next().unwrap_or(&r.text).trim().to_string();
            results.push(SearchResult {
                title,
                url: r.first_url,
                snippet: r.text,
            });
        }
    }

    // Related Topics
    fn extract_topics(topics: Vec<DdgApiTopic>, out: &mut Vec<SearchResult>, max: usize) {
        for t in topics {
            if out.len() >= max { break; }
            if !t.text.is_empty() && !t.first_url.is_empty() {
                let title = t.text.split(" - ").next().unwrap_or(&t.text).trim().to_string();
                let snippet = if let Some(pos) = t.text.find(" - ") {
                    t.text[pos + 3..].trim().to_string()
                } else {
                    t.text.clone()
                };
                out.push(SearchResult {
                    title,
                    url: t.first_url,
                    snippet,
                });
            }
            if !t.topics.is_empty() {
                extract_topics(t.topics, out, max);
            }
        }
    }
    extract_topics(data.related_topics, &mut results, max);

    results
}

// ─── DuckDuckGo HTML scraper ───────────────────────────────────────────────

/// Fetch top `max` results from DuckDuckGo HTML endpoint.
/// Uses persistent cookies so DDG sees a returning user, not a new bot.
pub async fn search_duckduckgo(query: &str, max: usize) -> Vec<SearchResult> {
    // 1. First try DuckDuckGo Instant Knowledge API (pure structured JSON, zero blog spam)
    let knowledge_hits = query_ddg_knowledge_api(query, max).await;
    if !knowledge_hits.is_empty() {
        return knowledge_hits;
    }

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

    // 2. Try DuckDuckGo HTML GET directly
    let ddg_get_url = format!("https://html.duckduckgo.com/html/?q={}", urlenc(query));
    if let Ok(resp) = client.get(&ddg_get_url)
        .header("Cookie", &cookie_str)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Referer", "https://html.duckduckgo.com/")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "same-origin")
        .header("Sec-Fetch-User", "?1")
        .header("Upgrade-Insecure-Requests", "1")
        .send()
        .await
    {
        if let Ok(html) = resp.text().await {
            let res = parse_ddg_results(&html, max);
            if !res.is_empty() {
                return res;
            }
        }
    }

    // 2. Try DuckDuckGo Lite POST
    let body = format!("q={}&b=&kl=wt-wt", urlenc(query));
    if let Ok(resp) = client.post("https://lite.duckduckgo.com/lite/")
        .header("Cookie", &cookie_str)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Origin", "https://duckduckgo.com")
        .header("Referer", "https://duckduckgo.com/")
        .body(body)
        .send()
        .await
    {
        if let Ok(html) = resp.text().await {
            let res = parse_ddg_results(&html, max);
            if !res.is_empty() {
                return res;
            }
        }
    }

    // 3. Try Yahoo Search (extremely reliable, zero bot traps)
    println!("ℹ️ Scraper: Trying Yahoo Search engine...");
    let yahoo_res = search_yahoo(query, max).await;
    if !yahoo_res.is_empty() {
        return yahoo_res;
    }

    // 4. Try Bing Search as final fallback
    println!("ℹ️ Scraper: Trying Bing Search engine...");
    search_bing(query, max).await
}

/// Fetch top `max` results from Yahoo Search
pub async fn search_yahoo(query: &str, max: usize) -> Vec<SearchResult> {
    let ua = pick_user_agent();
    let client = match reqwest::Client::builder()
        .user_agent(ua)
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let url = format!("https://search.yahoo.com/search?p={}&ei=UTF-8", urlenc(query));
    let resp = match client.get(&url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let html = match resp.text().await {
        Ok(h) => h,
        Err(_) => return vec![],
    };

    parse_yahoo_results(&html, max)
}

fn parse_yahoo_results(html: &str, max: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let split_token = if html.contains("class=\"algo") {
        "class=\"algo"
    } else {
        "class=\"compTitle"
    };

    let blocks: Vec<&str> = html.split(split_token).collect();

    for block in blocks.iter().skip(1) {
        if results.len() >= max { break; }

        let (url, title) = if let Some(h3_idx) = block.find("<h3") {
            let h3_part = &block[h3_idx..];
            let h3_end = h3_part.find("</h3>").unwrap_or(h3_part.len());
            let h3_content = &h3_part[..h3_end];

            let u = if let Some(href_idx) = h3_content.find("href=\"") {
                let rest = &h3_content[href_idx + 6..];
                let end = rest.find('"').unwrap_or(rest.len());
                let raw = &rest[..end];
                if let Some(ru_idx) = raw.find("/RU=") {
                    let after_ru = &raw[ru_idx + 4..];
                    let ru_end = after_ru.find('/').unwrap_or(after_ru.len());
                    urldecode(&after_ru[..ru_end])
                } else {
                    raw.to_string()
                }
            } else {
                continue;
            };

            let t = if let Some(open) = h3_content.find('>') {
                strip_tags(&h3_content[open + 1..]).trim().to_string()
            } else {
                String::new()
            };

            (u, t)
        } else {
            continue;
        };

        let snippet = if let Some(p_idx) = block.find("<p class=\"lh-16").or_else(|| block.find("<div class=\"compText")).or_else(|| block.find("<span class=\" fz-ms")) {
            let rest = &block[p_idx..];
            if let Some(open) = rest.find('>') {
                let after = &rest[open + 1..];
                let end = after.find("</p>")
                    .or_else(|| after.find("</div>"))
                    .or_else(|| after.find("</span>"))
                    .unwrap_or(after.len().min(300));
                strip_tags(&after[..end]).trim().to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if !title.is_empty() && !url.is_empty() {
            results.push(SearchResult {
                title: title.chars().take(120).collect(),
                url,
                snippet: snippet.chars().take(300).collect(),
            });
        }
    }

    results
}

/// Fetch top `max` results from Bing search as an alternate live engine
pub async fn search_bing(query: &str, max: usize) -> Vec<SearchResult> {
    let ua = pick_user_agent();
    let client = match reqwest::Client::builder()
        .user_agent(ua)
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let url = format!("https://www.bing.com/search?q={}", urlenc(query));
    let resp = match client.get(&url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let html = match resp.text().await {
        Ok(h) => h,
        Err(_) => return vec![],
    };

    parse_bing_results(&html, max)
}

fn parse_bing_results(html: &str, max: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let blocks: Vec<&str> = html.split("<li class=\"b_algo\"").collect();

    for block in blocks.iter().skip(1) {
        if results.len() >= max { break; }

        let url = if let Some(h2_idx) = block.find("<h2>") {
            let h2_block = &block[h2_idx..];
            if let Some(href_idx) = h2_block.find("href=\"") {
                let rest = &h2_block[href_idx + 6..];
                let end = rest.find('"').unwrap_or(rest.len());
                rest[..end].to_string()
            } else {
                continue;
            }
        } else {
            continue;
        };

        let title = if let Some(h2_idx) = block.find("<h2>") {
            let h2_block = &block[h2_idx..];
            if let Some(open) = h2_block.find('>') {
                let after_open = &h2_block[open + 1..];
                if let Some(close) = after_open.find("</h2>") {
                    strip_tags(&after_open[..close]).trim().to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let snippet = if let Some(p_idx) = block.find("<p") {
            let p_block = &block[p_idx..];
            if let Some(open) = p_block.find('>') {
                let after_open = &p_block[open + 1..];
                if let Some(close) = after_open.find("</p>") {
                    strip_tags(&after_open[..close]).trim().to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if !title.is_empty() && !url.is_empty() {
            results.push(SearchResult {
                title: title.chars().take(100).collect(),
                url,
                snippet: snippet.chars().take(300).collect(),
            });
        }
    }

    results
}

fn urldecode(s: &str) -> String {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn extract_real_url(html_tag: &str) -> Option<String> {
    if let Some(pos) = html_tag.find("uddg=") {
        let rest = &html_tag[pos + 5..];
        let end = rest.find('&')
            .or_else(|| rest.find('"'))
            .or_else(|| rest.find('\''))
            .unwrap_or(rest.len());
        let raw = &rest[..end];
        let decoded = urldecode(raw);
        if decoded.starts_with("http") && !decoded.contains("duckduckgo.com") {
            return Some(decoded);
        }
    }

    if let Some(href_idx) = html_tag.find("href=\"").or_else(|| html_tag.find("href='")) {
        let rest = &html_tag[href_idx + 6..];
        let end = rest.find('"').or_else(|| rest.find('\'')).unwrap_or(rest.len());
        let candidate = &rest[..end];
        if candidate.starts_with("http") && !candidate.contains("duckduckgo.com") {
            return Some(candidate.to_string());
        }
    }

    None
}

fn parse_ddg_results(html: &str, max: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // 1. Modern html.duckduckgo.com format (result__body / result__a)
    if html.contains("result__body") || html.contains("result__a") {
        let chunks: Vec<&str> = html.split("result__body").collect();
        for chunk in chunks.iter().skip(1) {
            if results.len() >= max { break; }

            // Extract title and URL from <a ... class="result__a" ...>Title</a>
            let mut url = None;
            let mut title = String::new();

            if let Some(a_idx) = chunk.find("<a ") {
                let rest_a = &chunk[a_idx..];
                if let Some(tag_end) = rest_a.find('>') {
                    let tag = &rest_a[..tag_end + 1];
                    url = extract_real_url(tag);

                    let after_tag = &rest_a[tag_end + 1..];
                    if let Some(close) = after_tag.find("</a>") {
                        title = strip_tags(&after_tag[..close]).trim().to_string();
                    }
                }
            }

            // Extract snippet from class="result__snippet"
            let snippet = if let Some(snip_idx) = chunk.find("result__snippet") {
                let rest = &chunk[snip_idx..];
                if let Some(open) = rest.find('>') {
                    let after = &rest[open + 1..];
                    let end = after.find("</a>")
                        .or_else(|| after.find("</div>"))
                        .unwrap_or(after.len().min(300));
                    strip_tags(&after[..end]).trim().to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            if !title.is_empty() {
                let final_url = url.unwrap_or_else(|| "https://duckduckgo.com".to_string());
                results.push(SearchResult {
                    title: title.chars().take(120).collect(),
                    url: final_url,
                    snippet: snippet.chars().take(300).collect(),
                });
            }
        }
    }

    if !results.is_empty() {
        return results;
    }

    // 2. Fallback: lite.duckduckgo.com format (class="result-link" and class="result-snippet")
    let html_norm = html.replace("class=\"result-link\"", "class='result-link'")
                        .replace("class=\"result-snippet\"", "class='result-snippet'");

    let link_blocks: Vec<&str> = html_norm.split("class='result-link'").collect();

    for (i, block) in link_blocks.iter().enumerate().skip(1) {
        if results.len() >= max { break; }

        let mut real_url = None;
        if let Some(start_a) = block.rfind("<a ") {
            let tag_part = &block[start_a..];
            real_url = extract_real_url(tag_part);
        }
        if real_url.is_none() && i > 0 {
            let prev = link_blocks[i - 1];
            if let Some(start_a) = prev.rfind("<a ") {
                real_url = extract_real_url(&prev[start_a..]);
            }
        }

        let title = if let Some(open) = block.find('>') {
            let after_open = &block[open + 1..];
            if let Some(close) = after_open.find("</a>") {
                strip_tags(&after_open[..close]).trim().to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let snippet = if let Some(snip_pos) = block.find("class='result-snippet'") {
            let snip_rest = &block[snip_pos..];
            if let Some(snip_open) = snip_rest.find('>') {
                let snip_body = &snip_rest[snip_open + 1..];
                if let Some(snip_close) = snip_body.find("</td>") {
                    strip_tags(&snip_body[..snip_close]).trim().to_string()
                } else {
                    strip_tags(snip_body).chars().take(250).collect()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if !title.is_empty() {
            let url = real_url.unwrap_or_else(|| "https://duckduckgo.com".to_string());
            results.push(SearchResult {
                title: title.chars().take(120).collect(),
                url,
                snippet: snippet.chars().take(300).collect(),
            });
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
