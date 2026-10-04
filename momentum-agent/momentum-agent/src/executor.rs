use crate::browser::{MomentumBrowser, PageSnapshot};
use crate::types::{Action, ToolFailureClass, UIElement};

#[derive(Debug, Clone)]
pub struct ActionResult {
    pub summary: String,
    pub url_before: String,
    pub url_after: String,
    pub page_changed: bool,
    pub verification_passed: bool,
    pub failure_class: Option<ToolFailureClass>,
    pub expected_outcome: Option<String>,
    pub observed_outcome: Option<String>,
    pub title_after: Option<String>,
    pub target_text: Option<String>,
}

pub struct ActionExecutor {
    pub last_ui: Vec<UIElement>,
}

impl ActionExecutor {
    pub fn new() -> Self {
        Self { last_ui: vec![] }
    }

    pub fn set_ui(&mut self, ui_list: Vec<UIElement>) {
        self.last_ui = ui_list;
    }

    fn changed(before: &PageSnapshot, after: &PageSnapshot) -> bool {
        before.url != after.url
            || before.fingerprint != after.fingerprint
            || (before.scroll_y - after.scroll_y).abs() > f64::EPSILON
    }

    fn action_result(
        summary: String,
        before: &PageSnapshot,
        after: &PageSnapshot,
        target_text: Option<String>,
    ) -> ActionResult {
        let changed = Self::changed(before, after);
        ActionResult {
            summary,
            url_before: before.url.clone(),
            url_after: after.url.clone(),
            page_changed: changed,
            verification_passed: changed,
            failure_class: if changed { None } else { Some(ToolFailureClass::NoEffect) },
            expected_outcome: None,
            observed_outcome: None,
            title_after: after.title.clone(),
            target_text,
        }
    }

    pub async fn execute(&self, browser: &MomentumBrowser, action: &Action) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        match action {
            Action::Navigate { url } => {
                let before = browser.snapshot().await?;
                browser.navigate(url).await?;
                // Crucial fix: Let SPA sites finish rendering after the navigation event
                tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                let after = browser.snapshot().await?;
                let changed = before.url != after.url;
                Ok(ActionResult {
                    summary: if changed {
                        format!("Navigated to {}", after.url)
                    } else {
                        format!("Navigate to {} had no visible effect", url)
                    },
                    url_before: before.url,
                    url_after: after.url.clone(),
                    page_changed: changed,
                    verification_passed: changed,
                    failure_class: if changed { None } else { Some(ToolFailureClass::NoEffect) },
                    expected_outcome: Some(format!("Land on {}", url)),
                    observed_outcome: Some(after.url.clone()),
                    title_after: after.title,
                    target_text: Some(url.clone()),
                })
            }
            Action::Click { selector, frame_id: _ } => {
                let el = self.last_ui.iter().find(|e| e.id == *selector)
                    .ok_or_else(|| format!("Selector '{}' not found in current UI", selector))?;
                
                let before = browser.snapshot().await?;
                // Physical click at coordinates
                browser.click_at(el.x, el.y).await?;
                tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
                let after = browser.snapshot().await?;
                let result = Self::action_result(
                    format!("Clicked '{}' ({})", el.text, el.role),
                    &before,
                    &after,
                    Some(el.text.clone()),
                );
                Ok(ActionResult {
                    summary: if result.page_changed {
                        result.summary.clone()
                    } else {
                        format!("Click on '{}' had no visible effect", el.text)
                    },
                    verification_passed: result.page_changed,
                    failure_class: if result.page_changed { None } else { Some(ToolFailureClass::NoEffect) },
                    expected_outcome: Some(format!("Click target '{}'", el.text)),
                    observed_outcome: Some(after.url.clone()),
                    ..result
                })
            }
            Action::Type { selector, text, frame_id: _ } => {
                let el = self.last_ui.iter().find(|e| e.id == *selector)
                    .ok_or_else(|| format!("Selector '{}' not found in current UI", selector))?;
                
                let before = browser.snapshot().await?;
                // Focus and then type humanly
                browser.click_at(el.x, el.y).await?;
                browser.type_human(text).await?;
                // Wait slightly after typing in case an auto-dropdown/AJAX triggers
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                let after = browser.snapshot().await?;
                let result = Self::action_result(
                    format!("Typed '{}' into '{}'", text, el.text),
                    &before,
                    &after,
                    Some(el.text.clone()),
                );
                Ok(ActionResult {
                    summary: if result.page_changed {
                        result.summary.clone()
                    } else {
                        format!("Type into '{}' had no visible effect", el.text)
                    },
                    verification_passed: result.page_changed,
                    failure_class: if result.page_changed { None } else { Some(ToolFailureClass::NoEffect) },
                    expected_outcome: Some(format!("Submit text into '{}'", el.text)),
                    observed_outcome: Some(after.url.clone()),
                    ..result
                })
            }
            Action::Scroll { direction } => {
                let before = browser.snapshot().await?;
                browser.scroll(direction).await?;
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let after = browser.snapshot().await?;
                let result = Self::action_result(
                    format!("Scrolled {}", direction),
                    &before,
                    &after,
                    Some(direction.clone()),
                );
                Ok(ActionResult {
                    summary: if result.page_changed {
                        result.summary.clone()
                    } else {
                        format!("Scroll {} had no visible effect", direction)
                    },
                    verification_passed: result.page_changed,
                    failure_class: if result.page_changed { None } else { Some(ToolFailureClass::NoEffect) },
                    expected_outcome: Some(format!("Reveal more content by scrolling {}", direction)),
                    observed_outcome: Some(after.scroll_y.to_string()),
                    ..result
                })
            }
            Action::Wait { ms } => {
                let before = browser.snapshot().await?;
                tokio::time::sleep(tokio::time::Duration::from_millis(*ms)).await;
                let mut after = browser.snapshot().await?;
                // Waiting is intentional; don't treat unchanged pages as a failure.
                after.fingerprint.push_str("|wait");
                Ok(Self::action_result(
                    format!("Waited {}ms", ms),
                    &before,
                    &after,
                    Some(ms.to_string()),
                ))
            }
            Action::Achievement { reason } => {
                let snapshot = browser.snapshot().await?;
                Ok(ActionResult {
                    summary: format!("Goal achieved: {}", reason),
                    url_before: snapshot.url.clone(),
                    url_after: snapshot.url,
                    page_changed: true,
                    verification_passed: true,
                    failure_class: None,
                    expected_outcome: Some(reason.clone()),
                    observed_outcome: Some("completed".into()),
                    title_after: snapshot.title,
                    target_text: Some(reason.clone()),
                })
            }
            Action::Cancelled { reason } => {
                let snapshot = browser.snapshot().await?;
                Ok(ActionResult {
                    summary: format!("Goal cancelled: {}", reason),
                    url_before: snapshot.url.clone(),
                    url_after: snapshot.url,
                    page_changed: true,
                    verification_passed: true,
                    failure_class: None,
                    expected_outcome: Some(reason.clone()),
                    observed_outcome: Some("cancelled".into()),
                    title_after: snapshot.title,
                    target_text: Some(reason.clone()),
                })
            }
            Action::Talk { speech } => {
                let snapshot = browser.snapshot().await?;
                Ok(ActionResult {
                    summary: format!("SPOKE: {}", speech),
                    url_before: snapshot.url.clone(),
                    url_after: snapshot.url,
                    page_changed: true,
                    verification_passed: true,
                    failure_class: None,
                    expected_outcome: Some("Spoke reply".into()),
                    observed_outcome: Some("spoken".into()),
                    title_after: snapshot.title,
                    target_text: Some(speech.clone()),
                })
            }
            Action::Ask { question } => {
                let snapshot = browser.snapshot().await?;
                Ok(ActionResult {
                    summary: format!("ASKED: {}", question),
                    url_before: snapshot.url.clone(),
                    url_after: snapshot.url,
                    page_changed: true,
                    verification_passed: true,
                    failure_class: None,
                    expected_outcome: Some(question.clone()),
                    observed_outcome: Some("asked".into()),
                    title_after: snapshot.title,
                    target_text: Some(question.clone()),
                })
            }
            Action::Perceive { selector } => {
                let snapshot = browser.snapshot().await?;
                
                let crop_box = if let Some(sel) = selector {
                    let el = self.last_ui.iter().find(|e| e.id == *sel)
                        .ok_or_else(|| format!("Selector '{}' not found in current UI", sel))?;
                    // Create a 400x400 crop centered on the element (Foveated Vision)
                    let half = 200.0;
                    let x = (el.x - half).max(0.0) as i32;
                    let y = (el.y - half).max(0.0) as i32;
                    Some((x, y, 400, 400))
                } else {
                    None
                };
                
                let vision_output = crate::perception::perceive_vision(crop_box).await;
                let mut after = snapshot.clone();
                after.fingerprint.push_str("|perceived");
                
                Ok(ActionResult {
                    summary: format!("Vision System Sees: {}", vision_output),
                    url_before: snapshot.url.clone(),
                    url_after: after.url.clone(),
                    page_changed: true,
                    verification_passed: true,
                    failure_class: None,
                    expected_outcome: Some("Inspect UI visually".into()),
                    observed_outcome: Some(vision_output.clone()),
                    title_after: after.title,
                    target_text: Some(vision_output),
                })
            }
            Action::StartReflex { .. } => unreachable!("StartReflex is intercepted in main.rs"),
            Action::SpawnService { .. } | Action::KillService { .. } | Action::ListServices => unreachable!("Service tools are intercepted in main.rs"),
        }
    }
}
