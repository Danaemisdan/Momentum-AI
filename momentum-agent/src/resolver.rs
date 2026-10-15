use crate::types::UIElement;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NextActionType {
    Click,
    Drag,
    Type,
    Slide,
    Scroll,
    OpenApp,
    Clarify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextActionDecision {
    pub target_id: String,
    pub target_role: String,
    pub target_label: String,
    pub action_type: NextActionType,
    pub x: f64,
    pub y: f64,
    pub end_x: Option<f64>,
    pub end_y: Option<f64>,
    pub text_payload: Option<String>,
    pub confidence: f32,
    pub rationale: String,
}

/// The Next-Step Resolver: Evaluates perceived UI elements against user goal
/// and deterministically determines the single best next action in 0ms.
pub struct NextActionResolver;

impl NextActionResolver {
    pub fn resolve(goal: &str, ui_elements: &[UIElement]) -> Option<NextActionDecision> {
        let goal_lower = goal.to_lowercase().trim().to_string();
        if goal_lower.is_empty() || ui_elements.is_empty() {
            return None;
        }

        // 1. Resizing / Pane Divider / Splitter Intent
        if Self::is_resize_intent(&goal_lower) {
            if let Some(decision) = Self::resolve_resize(&goal_lower, ui_elements) {
                return Some(decision);
            }
        }

        // 2. Slider / Scrubber / Volume / Brightness Intent
        if Self::is_slider_intent(&goal_lower) {
            if let Some(decision) = Self::resolve_slider(&goal_lower, ui_elements) {
                return Some(decision);
            }
        }

        // 3. Search / Query Intent
        if Self::is_search_intent(&goal_lower) {
            if let Some(decision) = Self::resolve_search(&goal_lower, ui_elements) {
                return Some(decision);
            }
        }

        // 4. App Launch / Dock Intent
        if Self::is_launch_intent(&goal_lower) {
            if let Some(decision) = Self::resolve_launch(&goal_lower, ui_elements) {
                return Some(decision);
            }
        }

        // 5. General Button / Tab / Control Click Intent
        if let Some(decision) = Self::resolve_control_click(&goal_lower, ui_elements) {
            return Some(decision);
        }

        // 6. Generic Fallback: find closest matching interactive element
        Self::resolve_generic_fallback(&goal_lower, ui_elements)
    }

    fn is_resize_intent(g: &str) -> bool {
        g.contains("resize")
            || g.contains("wider")
            || g.contains("narrower")
            || g.contains("expand")
            || g.contains("collapse")
            || g.contains("drag divider")
            || g.contains("drag splitter")
            || g.contains("pane")
            || g.contains("divider")
            || g.contains("splitter")
            || (g.contains("make") && (g.contains("bigger") || g.contains("smaller") || g.contains("sidebar")))
    }

    fn resolve_resize(g: &str, elements: &[UIElement]) -> Option<NextActionDecision> {
        let is_sidebar = g.contains("sidebar") || g.contains("left");
        let is_auxiliary = g.contains("auxiliary") || g.contains("chat") || g.contains("right");
        let is_wider = g.contains("wider") || g.contains("expand") || g.contains("bigger") || g.contains("increase");
        let is_narrower = g.contains("narrower") || g.contains("collapse") || g.contains("smaller") || g.contains("decrease");

        // Find candidate splitters / pane resize handles
        let mut candidates: Vec<(&UIElement, f32)> = elements.iter()
            .filter(|e| {
                e.role == "AXPaneResizeHandle"
                    || e.role == "AXSplitter"
                    || e.action.as_deref() == Some("drag")
                    || e.role == "AXWindowResizeBorder"
                    || e.role == "AXWindowResizeCorner"
            })
            .map(|e| {
                let mut score = 0.5f32;
                let text_lower = e.text.to_lowercase();
                let cont_lower = e.container.as_deref().unwrap_or("").to_lowercase();

                if is_sidebar && (text_lower.contains("sidebar") || cont_lower.contains("sidebar")) {
                    score += 0.45;
                }
                if is_auxiliary && (text_lower.contains("auxiliary") || text_lower.contains("chat") || cont_lower.contains("auxiliary")) {
                    score += 0.45;
                }
                if e.role == "AXPaneResizeHandle" {
                    score += 0.1;
                }
                (e, score)
            })
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((best, score)) = candidates.first() {
            let start_x = best.x;
            let start_y = best.y;

            // Compute delta
            let delta_x = if is_sidebar {
                if is_narrower { -100.0 } else { 100.0 }
            } else if is_auxiliary {
                if is_wider { -100.0 } else { 100.0 }
            } else if is_wider {
                100.0
            } else if is_narrower {
                -100.0
            } else {
                80.0
            };

            let end_x = (start_x + delta_x).max(50.0);
            let end_y = start_y;

            let rationale = format!(
                "Identified pane divider '{}' (role: {}). Dragging from ({:.0}, {:.0}) to ({:.0}, {:.0}) to adjust width.",
                best.text, best.role, start_x, start_y, end_x, end_y
            );

            return Some(NextActionDecision {
                target_id: best.id.clone(),
                target_role: best.role.clone(),
                target_label: best.text.clone(),
                action_type: NextActionType::Drag,
                x: start_x,
                y: start_y,
                end_x: Some(end_x),
                end_y: Some(end_y),
                text_payload: None,
                confidence: *score,
                rationale,
            });
        }

        None
    }

    fn is_slider_intent(g: &str) -> bool {
        g.contains("volume")
            || g.contains("brightness")
            || g.contains("slider")
            || g.contains("louder")
            || g.contains("quieter")
            || g.contains("dimmer")
            || g.contains("brighter")
    }

    fn resolve_slider(g: &str, elements: &[UIElement]) -> Option<NextActionDecision> {
        let is_volume = g.contains("volume") || g.contains("louder") || g.contains("quieter");
        let is_brightness = g.contains("brightness") || g.contains("dimmer") || g.contains("brighter");

        let target = elements.iter().find(|e| {
            let text_lower = e.text.to_lowercase();
            let role_lower = e.role.to_lowercase();
            (role_lower.contains("slider") || e.action.as_deref() == Some("slide"))
                && ((is_volume && text_lower.contains("volume"))
                    || (is_brightness && text_lower.contains("brightness"))
                    || (!is_volume && !is_brightness))
        });

        if let Some(el) = target {
            return Some(NextActionDecision {
                target_id: el.id.clone(),
                target_role: el.role.clone(),
                target_label: el.text.clone(),
                action_type: NextActionType::Slide,
                x: el.x,
                y: el.y,
                end_x: Some(el.x + 30.0),
                end_y: Some(el.y),
                text_payload: None,
                confidence: 0.90,
                rationale: format!("Detected slider '{}'. Adjusting level via slide action.", el.text),
            });
        }

        None
    }

    fn is_search_intent(g: &str) -> bool {
        g.starts_with("search")
            || g.contains("search for")
            || g.starts_with("find ")
            || g.contains("look for")
            || g.contains("lookup")
            || g.starts_with("play ")
            || g.starts_with("watch ")
    }

    fn resolve_search(g: &str, elements: &[UIElement]) -> Option<NextActionDecision> {
        // Extract search query
        let query = Self::extract_search_query(g);

        // Find primary search field
        let search_field = elements.iter().find(|e| {
            e.role == "AXSearchField"
                || e.role.to_lowercase().contains("search")
                || e.semantic_intent.as_deref() == Some("Search")
                || (e.is_input_like && e.placeholder.as_deref().map_or(false, |p| p.to_lowercase().contains("search")))
                || (e.is_input_like && e.text.to_lowercase().contains("search"))
        }).or_else(|| {
            // Fallback: any input field in top navigation
            elements.iter().find(|e| e.is_input_like && e.spatial_zone.as_deref() == Some("Top Navigation"))
        }).or_else(|| {
            // Fallback: first input-like element
            elements.iter().find(|e| e.is_input_like)
        });

        if let Some(field) = search_field {
            let rationale = format!(
                "Detected search target '{}' (role: {}). Focusing input at ({:.0}, {:.0}) and typing query '{}'.",
                field.text, field.role, field.x, field.y, query
            );

            return Some(NextActionDecision {
                target_id: field.id.clone(),
                target_role: field.role.clone(),
                target_label: if !field.text.is_empty() { field.text.clone() } else { "Search Input".into() },
                action_type: NextActionType::Type,
                x: field.x,
                y: field.y,
                end_x: None,
                end_y: None,
                text_payload: Some(query),
                confidence: 0.95,
                rationale,
            });
        }

        None
    }

    fn extract_search_query(g: &str) -> String {
        let prefixes = [
            "search for ", "search ", "find me ", "find ", "look for ", "lookup ",
            "play some ", "play ", "watch "
        ];

        let mut q = g.to_string();
        for p in &prefixes {
            if q.starts_with(p) {
                q = q[p.len()..].trim().to_string();
                break;
            }
        }

        // Strip trailing platform specifications if present
        let platform_suffixes = [" on youtube", " in youtube", " on google", " on chrome", " on spotify"];
        for s in &platform_suffixes {
            if q.ends_with(s) {
                q = q[..q.len() - s.len()].trim().to_string();
            }
        }

        if q.is_empty() {
            g.to_string()
        } else {
            q
        }
    }

    fn is_launch_intent(g: &str) -> bool {
        g.starts_with("open ") || g.starts_with("launch ") || g.starts_with("switch to ")
    }

    fn resolve_launch(g: &str, elements: &[UIElement]) -> Option<NextActionDecision> {
        let target_app = if g.starts_with("open ") {
            g["open ".len()..].trim()
        } else if g.starts_with("launch ") {
            g["launch ".len()..].trim()
        } else {
            g["switch to ".len()..].trim()
        };

        let app_lower = target_app.to_lowercase();

        // 1. Look for Dock item matching app name
        let dock_match = elements.iter().find(|e| {
            e.role == "AXDockItem" && e.text.to_lowercase().contains(&app_lower)
        });

        if let Some(dock_el) = dock_match {
            return Some(NextActionDecision {
                target_id: dock_el.id.clone(),
                target_role: dock_el.role.clone(),
                target_label: dock_el.text.clone(),
                action_type: NextActionType::Click,
                x: dock_el.x,
                y: dock_el.y,
                end_x: None,
                end_y: None,
                text_payload: None,
                confidence: 0.95,
                rationale: format!("Found Dock item '{}'. Clicking at ({:.0}, {:.0}) to launch/switch application.", dock_el.text, dock_el.x, dock_el.y),
            });
        }

        None
    }

    fn resolve_control_click(g: &str, elements: &[UIElement]) -> Option<NextActionDecision> {
        // Window controls
        if g.contains("close window") || g.contains("close active") {
            if let Some(el) = elements.iter().find(|e| e.text.to_lowercase().contains("close") && e.role.contains("Button")) {
                return Some(NextActionDecision {
                    target_id: el.id.clone(),
                    target_role: el.role.clone(),
                    target_label: el.text.clone(),
                    action_type: NextActionType::Click,
                    x: el.x,
                    y: el.y,
                    end_x: None,
                    end_y: None,
                    text_payload: None,
                    confidence: 0.92,
                    rationale: format!("Clicking window close button at ({:.0}, {:.0})", el.x, el.y),
                });
            }
        }

        if g.contains("minimize") {
            if let Some(el) = elements.iter().find(|e| e.text.to_lowercase().contains("minimize") && e.role.contains("Button")) {
                return Some(NextActionDecision {
                    target_id: el.id.clone(),
                    target_role: el.role.clone(),
                    target_label: el.text.clone(),
                    action_type: NextActionType::Click,
                    x: el.x,
                    y: el.y,
                    end_x: None,
                    end_y: None,
                    text_payload: None,
                    confidence: 0.92,
                    rationale: format!("Clicking window minimize button at ({:.0}, {:.0})", el.x, el.y),
                });
            }
        }

        if g.contains("maximize") || g.contains("full screen") {
            if let Some(el) = elements.iter().find(|e| (e.text.to_lowercase().contains("maximize") || e.text.to_lowercase().contains("full screen")) && e.role.contains("Button")) {
                return Some(NextActionDecision {
                    target_id: el.id.clone(),
                    target_role: el.role.clone(),
                    target_label: el.text.clone(),
                    action_type: NextActionType::Click,
                    x: el.x,
                    y: el.y,
                    end_x: None,
                    end_y: None,
                    text_payload: None,
                    confidence: 0.92,
                    rationale: format!("Clicking window maximize button at ({:.0}, {:.0})", el.x, el.y),
                });
            }
        }

        None
    }

    fn resolve_generic_fallback(g: &str, elements: &[UIElement]) -> Option<NextActionDecision> {
        let words: Vec<&str> = g.split_whitespace()
            .filter(|&w| !["the", "a", "an", "on", "in", "at", "to", "for", "me", "please"].contains(&w))
            .collect();

        if words.is_empty() {
            return None;
        }

        let mut best_match: Option<(&UIElement, f32)> = None;

        for el in elements {
            let text_lower = el.text.to_lowercase();
            let role_lower = el.role.to_lowercase();
            let cont_lower = el.container.as_deref().unwrap_or("").to_lowercase();
            let purp_lower = el.purpose.as_deref().unwrap_or("").to_lowercase();

            let mut match_count = 0;
            for w in &words {
                if text_lower.contains(w) || purp_lower.contains(w) || cont_lower.contains(w) {
                    match_count += 1;
                }
            }

            if match_count > 0 {
                let mut score = (match_count as f32) / (words.len() as f32);
                if el.is_clickable || role_lower.contains("button") {
                    score += 0.2;
                }
                if el.is_input_like {
                    score += 0.15;
                }

                if best_match.as_ref().map_or(true, |(_, s)| score > *s) {
                    best_match = Some((el, score));
                }
            }
        }

        if let Some((el, score)) = best_match {
            if score >= 0.4 {
                let action_type = if el.is_input_like {
                    NextActionType::Type
                } else if el.role == "AXPaneResizeHandle" || el.action.as_deref() == Some("drag") {
                    NextActionType::Drag
                } else {
                    NextActionType::Click
                };

                return Some(NextActionDecision {
                    target_id: el.id.clone(),
                    target_role: el.role.clone(),
                    target_label: el.text.clone(),
                    action_type,
                    x: el.x,
                    y: el.y,
                    end_x: None,
                    end_y: None,
                    text_payload: None,
                    confidence: score.min(1.0),
                    rationale: format!("Best matching UI element '{}' (role: {}) with score {:.2}.", el.text, el.role, score),
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_element(id: &str, role: &str, text: &str, x: f64, y: f64, is_input: bool, is_click: bool, action: Option<&str>) -> UIElement {
        UIElement {
            id: id.into(),
            role: role.into(),
            text: text.into(),
            selector: id.into(),
            x,
            y,
            href: None,
            title: None,
            tag: None,
            aria_label: None,
            placeholder: None,
            is_clickable: is_click,
            is_input_like: is_input,
            container: None,
            semantic_label: text.into(),
            frame_id: None,
            semantic_intent: None,
            spatial_zone: None,
            action: action.map(|s| s.into()),
            purpose: None,
            destination: None,
            human_intent: None,
        }
    }

    #[test]
    fn test_resolve_sidebar_resize_wider() {
        let elements = vec![
            mock_element("el_1", "AXButton", "File", 50.0, 15.0, false, true, Some("click")),
            mock_element("el_191", "AXPaneResizeHandle", "Antigravity Sidebar Resize Divider", 416.0, 450.0, false, true, Some("drag")),
            mock_element("el_192", "AXPaneResizeHandle", "Antigravity Auxiliary Pane Resize Divider", 1051.0, 450.0, false, true, Some("drag")),
        ];

        let decision = NextActionResolver::resolve("make the sidebar wider", &elements).expect("Should resolve sidebar resize");
        assert_eq!(decision.target_id, "el_191");
        assert_eq!(decision.action_type, NextActionType::Drag);
        assert_eq!(decision.x, 416.0);
        assert_eq!(decision.end_x, Some(516.0));
    }

    #[test]
    fn test_resolve_search_query_extraction() {
        let elements = vec![
            mock_element("el_1", "AXButton", "Home", 50.0, 100.0, false, true, Some("click")),
            mock_element("el_42", "AXSearchField", "Search", 600.0, 40.0, true, true, Some("type")),
        ];

        let decision = NextActionResolver::resolve("search for lofi hip hop beats on youtube", &elements).expect("Should resolve search");
        assert_eq!(decision.target_id, "el_42");
        assert_eq!(decision.action_type, NextActionType::Type);
        assert_eq!(decision.text_payload.as_deref(), Some("lofi hip hop beats"));
    }

    #[test]
    fn test_resolve_dock_launch() {
        let elements = vec![
            mock_element("el_1", "AXButton", "File", 50.0, 15.0, false, true, Some("click")),
            mock_element("el_206", "AXDockItem", "Terminal", 850.0, 950.0, false, true, Some("click")),
            mock_element("el_202", "AXDockItem", "Google Chrome", 650.0, 950.0, false, true, Some("click")),
        ];

        let decision = NextActionResolver::resolve("open Terminal from dock", &elements).expect("Should resolve dock launch");
        assert_eq!(decision.target_id, "el_206");
        assert_eq!(decision.action_type, NextActionType::Click);
    }
}
