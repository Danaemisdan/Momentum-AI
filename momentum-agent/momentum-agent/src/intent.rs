pub fn infer_intent(role: &str, text: &str, title: Option<&str>, aria_label: Option<&str>) -> Option<String> {
    let mut haystack = String::new();
    haystack.push_str(&text.to_lowercase());
    haystack.push(' ');
    
    if let Some(t) = title {
        haystack.push_str(&t.to_lowercase());
        haystack.push(' ');
    }
    if let Some(a) = aria_label {
        haystack.push_str(&a.to_lowercase());
        haystack.push(' ');
    }
    
    let role_lower = role.to_lowercase();
    
    // Split haystack by non-alphanumeric chars to get exact words for short keywords
    let words: Vec<&str> = haystack.split(|c: char| !c.is_alphanumeric()).filter(|s| !s.is_empty()).collect();

    let contains_kw = |keywords: &[&str]| -> bool {
        for &kw in keywords {
            if kw.chars().all(char::is_alphabetic) && kw.len() <= 3 {
                if words.contains(&kw) { return true; }
            } else {
                if haystack.contains(kw) { return true; }
            }
        }
        false
    };

    // 1. Search
    let search_keywords = ["search", "find", "magnifying glass", "🔍", "axsearchfield"];
    if contains_kw(&search_keywords) || search_keywords.iter().any(|k| role_lower.contains(k)) {
        return Some("Search".to_string());
    }

    // 2. Navigation
    let nav_keywords = ["back", "forward", "home", "🔙", "menu", "sidebar", "navigation"];
    if contains_kw(&nav_keywords) {
        return Some("Navigate".to_string());
    }

    // 3. Confirmation
    let confirm_keywords = ["ok", "yes", "confirm", "submit", "save", "done", "apply", "accept", "continue"];
    if contains_kw(&confirm_keywords) {
        return Some("Confirm".to_string());
    }

    // 4. Cancellation
    let cancel_keywords = ["cancel", "no", "close", "x", "quit", "exit", "dismiss", "abort", "close button"];
    if contains_kw(&cancel_keywords) || role_lower == "axclosebutton" {
        return Some("Cancel".to_string());
    }

    // 5. Communication
    let comm_keywords = ["send", "chat", "message", "reply", "compose", "new chat", "write"];
    if contains_kw(&comm_keywords) {
        return Some("Communicate".to_string());
    }

    // 6. Media Control
    let media_keywords = ["play", "pause", "stop", "next", "volume", "mute", "rewind", "fast forward", "▶️", "⏸️"];
    if contains_kw(&media_keywords) {
        return Some("Media Control".to_string());
    }
    
    // 7. Settings / Options
    let settings_keywords = ["settings", "preferences", "options", "gear", "⚙️", "configure", "setup"];
    if contains_kw(&settings_keywords) {
        return Some("Settings".to_string());
    }
    
    // 8. Profiles / Account
    let account_keywords = ["profile", "account", "login", "sign in", "sign out", "logout", "user"];
    if contains_kw(&account_keywords) {
        return Some("Account".to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_intent() {
        assert_eq!(infer_intent("AXButton", "🔙", None, None), Some("Navigate".to_string()));
        assert_eq!(infer_intent("AXSearchField", "", None, None), Some("Search".to_string()));
        assert_eq!(infer_intent("AXButton", "Cancel", None, None), Some("Cancel".to_string()));
        assert_eq!(infer_intent("AXButton", "", Some("Close Window"), None), Some("Cancel".to_string()));
        assert_eq!(infer_intent("AXButton", "Send", None, None), Some("Communicate".to_string()));
        assert_eq!(infer_intent("AXGenericElement", "Unknown Thing", None, None), None);
    }
}
