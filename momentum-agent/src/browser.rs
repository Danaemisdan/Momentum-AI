use chromiumoxide::{Browser, BrowserConfig, Page, layout::Point};
use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use futures::StreamExt;
use std::env;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PageSnapshot {
    pub url: String,
    pub title: Option<String>,
    pub fingerprint: String,
    pub scroll_y: f64,
}

pub struct MomentumBrowser {
    pub browser: Option<Browser>,
    pub page: Option<Page>,
}

impl MomentumBrowser {
    pub fn init() -> Self {
        Self { browser: None, page: None }
    }

    pub fn find_chrome_executable() -> Option<String> {
        if let Ok(path) = env::var("MOMENTUM_CHROME_PATH") {
            if Path::new(&path).exists() { return Some(path); }
        }

        let candidates = [
            // macOS
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            // Windows
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
            // Linux
            "/usr/bin/google-chrome",
            "/usr/bin/chromium-browser",
            "/usr/bin/chromium",
        ];

        for c in &candidates {
            if Path::new(c).exists() {
                return Some(c.to_string());
            }
        }

        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            let win_chrome = format!(r"{}\Google\Chrome\Application\chrome.exe", local_app_data);
            if Path::new(&win_chrome).exists() {
                return Some(win_chrome);
            }
            let win_edge = format!(r"{}\Microsoft\Edge\Application\msedge.exe", local_app_data);
            if Path::new(&win_edge).exists() {
                return Some(win_edge);
            }
        }

        None
    }

    pub async fn ensure_launched(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let is_dead = if let Some(b) = &self.browser {
            b.version().await.is_err()
        } else {
            true
        };

        if !is_dead {
            return Ok(());
        }

        if self.browser.is_some() {
            println!("🚨 Browser connection lost. Restarting...");
            self.browser = None;
            self.page = None;
        }

        println!("🌐 Launching Stealth Chrome...");
        let home_dir = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".into());
        let profile_path = format!("{}/.momentum/chrome-profile", home_dir);
        let chrome_bin = Self::find_chrome_executable()
            .ok_or_else(|| "Google Chrome or compatible browser not found. Please install Chrome or set MOMENTUM_CHROME_PATH.".to_string())?;
        let headless = env::var("MOMENTUM_HEADLESS")
            .map(|value| matches!(value.trim().to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
        let start_url = Self::default_search_url();

        let user_agent = if cfg!(target_os = "windows") {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
        } else {
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
        };

        std::fs::create_dir_all(&profile_path).ok();
        // Clean up stale Chrome singleton locks from crashed previous sessions.
        // Without this, Chrome refuses to start if it didn't exit cleanly.
        for lock_file in &["SingletonLock", "SingletonSocket", "SingletonCookie"] {
            let lock_path = format!("{}/{}", profile_path, lock_file);
            if std::path::Path::new(&lock_path).exists() {
                println!("🧹 Removing stale Chrome lock: {}", lock_file);
                let _ = std::fs::remove_file(&lock_path);
            }
        }

        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .chrome_executable(&chrome_bin)
                .user_data_dir(&profile_path)
                // ─── STEALTH CONFIG ─────────────────────────────────────────────────────
                // Headful mode is the most human-like. Headless can be re-enabled with
                // MOMENTUM_HEADLESS=1 for environments where a visible browser is impossible.
                .arg(if headless { "--headless=new" } else { "--start-maximized" })
                .arg("--no-sandbox")
                .arg("--disable-setuid-sandbox")
                // Core anti-bot fingerprint erasure
                .arg("--disable-blink-features=AutomationControlled")
                .arg("--exclude-switches=enable-automation")
                .arg("--disable-infobars")
                // Window size matching a real 1080p screen
                .arg("--window-size=1920,1080")
                .arg(format!("--user-agent={}", user_agent))
                // GPU / rendering flags for stability on macOS
                .arg("--disable-gpu-sandbox")
                .arg("--no-first-run")
                .arg("--no-default-browser-check")
                .arg("--disable-popup-blocking")
                .arg("--disable-notifications")
                .arg("--disable-dev-shm-usage")
                // ────────────────────────────────────────────────────────────────────────
                .build()?,
        ).await?;

        tokio::spawn(async move {
            while let Some(msg) = handler.next().await {
                match msg {
                    Err(e) => {
                        let s = e.to_string();
                        println!("! Browser handler error: {}", s);
                        if s.contains("Connection reset") || s.contains("broken pipe") || s.contains("EOF") {
                            println!("🚨 FATAL: Browser connection severed. Will restart on next action.");
                            break;
                        }
                    }
                    Ok(_) => {}
                }
            }
        });

        // Get or create a page
        let pages = browser.pages().await?;
        let page = if pages.is_empty() {
            browser.new_page("about:blank").await?
        } else {
            pages[0].clone()
        };
        
        // Inject stealth JS to erase navigator.webdriver traces immediately
        let platform_str = if cfg!(target_os = "windows") {
            "Win32"
        } else if cfg!(target_os = "macos") {
            "MacIntel"
        } else {
            "Linux x86_64"
        };
        let stealth_js = format!(r#"
            Object.defineProperty(navigator, 'webdriver', {{ get: () => undefined }});
            Object.defineProperty(navigator, 'plugins', {{ get: () => [1, 2, 3, 4, 5] }});
            Object.defineProperty(navigator, 'languages', {{ get: () => ['en-US', 'en'] }});
            window.chrome = {{ runtime: {{}} }};
            Object.defineProperty(navigator, 'platform', {{ get: () => '{}' }});
        "#, platform_str);
        let _ = page.evaluate(stealth_js.as_str()).await;
        
        // Set viewport explicitly — critical for headless mode so getBoundingClientRect works
        use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
        let _ = page.execute(
            SetDeviceMetricsOverrideParams::builder()
                .width(1920u32)
                .height(1080u32)
                .device_scale_factor(1.0)
                .mobile(false)
                .build()
                .unwrap()
        ).await;
        
        println!("🌐 Navigating to {}...", Self::default_search_engine_name());
        let _ = page.goto(&start_url).await;
        let _ = page.wait_for_navigation().await;
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        self.browser = Some(browser);
        self.page = Some(page);
        
        println!("✅ Chrome ready at {}", Self::default_search_engine_name());
        Ok(())
    }

    pub fn default_search_engine_name() -> &'static str {
        match env::var("MOMENTUM_SEARCH_ENGINE")
            .unwrap_or_else(|_| "google".to_string())
            .trim()
            .to_lowercase()
            .as_str()
        {
            "bing" => "Bing",
            "duckduckgo" | "ddg" => "DuckDuckGo",
            _ => "Google",
        }
    }

    pub fn default_search_url() -> String {
        match env::var("MOMENTUM_SEARCH_ENGINE")
            .unwrap_or_else(|_| "google".to_string())
            .trim()
            .to_lowercase()
            .as_str()
        {
            "bing" => "https://www.bing.com".to_string(),
            "duckduckgo" | "ddg" => "https://duckduckgo.com".to_string(),
            _ => "https://www.google.com/ncr".to_string(),
        }
    }

    pub async fn capture_screenshot(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(page) = &self.page {
            use base64::{Engine as _, engine::general_purpose};
            let data = page.screenshot(chromiumoxide::page::ScreenshotParams::builder().build()).await?;
            Ok(general_purpose::STANDARD.encode(data))
        } else {
            Err("Browser not launched".into())
        }
    }

    pub async fn snapshot(&self) -> Result<PageSnapshot, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(page) = &self.page {
            let url = page.url().await.ok().flatten().unwrap_or_default();
            let js = r#"
                (() => {
                    const visibleElements = Array.from(document.querySelectorAll('a,button,input,textarea,select,[role]'))
                        .filter((el) => {
                            const rect = el.getBoundingClientRect();
                            const style = window.getComputedStyle(el);
                            return rect.width > 0
                                && rect.height > 0
                                && style.display !== 'none'
                                && style.visibility !== 'hidden'
                                && style.opacity !== '0';
                        })
                        .slice(0, 10)
                        .map((el) => ({
                            role: el.getAttribute('role') || el.tagName.toLowerCase(),
                            text: (el.getAttribute('aria-label')
                                || el.getAttribute('title')
                                || el.getAttribute('placeholder')
                                || el.innerText
                                || '').trim().slice(0, 80),
                            href: (el instanceof HTMLAnchorElement && el.href) ? el.href : '',
                        }));
                    return {
                        title: document.title || '',
                        scrollY: window.scrollY || 0,
                        elements: visibleElements,
                    };
                })()
            "#;
            let value = page.evaluate(js).await?.value().cloned().unwrap_or(serde_json::Value::Null);
            let title = value["title"].as_str().unwrap_or("").trim().to_string();
            let scroll_y = value["scrollY"].as_f64().unwrap_or(0.0);
            let fingerprint = serde_json::to_string(&value)?;

            Ok(PageSnapshot {
                url,
                title: if title.is_empty() { None } else { Some(title) },
                fingerprint,
                scroll_y,
            })
        } else {
            Err("Browser not launched".into())
        }
    }

    pub async fn navigate(&self, url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(page) = &self.page {
            page.goto(url).await?;
            // Wait for navigation and then a settle period for SPA rendering
            let _ = page.wait_for_navigation().await;
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await; 
            Ok(())
        } else {
            Err("Browser not launched".into())
        }
    }

    pub async fn click_at(&self, x: f64, y: f64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(page) = &self.page {
            // Human-like move and click
            let point = Point::new(x, y);
            page.move_mouse(point).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            page.click(point).await?;
            Ok(())
        } else {
            Err("Browser not launched".into())
        }
    }

    pub async fn type_human(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(page) = &self.page {
            for c in text.chars() {
                let cmd = DispatchKeyEventParams::builder()
                    .r#type(DispatchKeyEventType::Char)
                    .text(c.to_string())
                    .build()
                    .unwrap();
                page.execute(cmd).await?;
                
                let delay = {
                    use rand::Rng;
                    rand::thread_rng().gen_range(50..150)
                };
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }
            
            let enter = DispatchKeyEventParams::builder()
                .r#type(DispatchKeyEventType::KeyDown)
                .key("Enter")
                .code("Enter")
                .windows_virtual_key_code(13)
                .build()
                .unwrap();
            page.execute(enter).await?;
            
            let enter_up = DispatchKeyEventParams::builder()
                .r#type(DispatchKeyEventType::KeyUp)
                .key("Enter")
                .code("Enter")
                .windows_virtual_key_code(13)
                .build()
                .unwrap();
            page.execute(enter_up).await?;

            Ok(())
        } else {
            Err("Browser not launched".into())
        }
    }

    #[allow(dead_code)]
    pub async fn click_el(&self, _frame_id: Option<&str>, selector: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(page) = &self.page {
            let js = format!("document.querySelector('{}')?.click()", selector);
            page.evaluate(js).await?;
            Ok(())
        } else {
            Err("Browser not launched".into())
        }
    }

    #[allow(dead_code)]
    pub async fn type_el(&self, _frame_id: Option<&str>, selector: &str, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(page) = &self.page {
            let js = format!(r#"
                const el = document.querySelector('{}');
                if (el) {{
                    el.focus();
                    el.value = '{}';
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}
            "#, selector, text);
            page.evaluate(js).await?;
            Ok(())
        } else {
            Err("Browser not launched".into())
        }
    }

    pub async fn scroll(&self, direction: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(page) = &self.page {
            let js = if direction == "down" { "window.scrollBy(0, 500)" } else { "window.scrollBy(0, -500)" };
            page.evaluate(js).await?;
            Ok(())
        } else {
            Err("Browser not launched".into())
        }
    }

}
