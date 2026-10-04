fn main() {
    let actual_message = "I wanna find some clients for my AI business".to_string();
    let is_browser_task = {
        let u = actual_message.to_lowercase();
        false || // session_memory.active_goal.is_some()
        u.contains("[auto_step]") ||
        u.contains("open ") || u.contains("navigate") || u.contains("go to") ||
        u.contains("search ") || u.contains("click ") ||
        u.contains("scroll") || u.contains("buy ") ||
        u.contains("watch ") || u.contains("play ") || u.contains("book a") ||
        u.contains("post to") || u.contains("tweet") ||
        u.contains("dm ") || u.contains("github") ||
        u.contains("youtube") || u.contains("google ") || u.contains("reddit") ||
        u.contains("amazon") || u.contains("linkedin") ||
        u.contains("browser") || u.starts_with("http") ||
        u.contains("client") || u.contains("lead") ||
        u.contains("prospect") || u.contains("job") ||
        u.contains("apply")
    };
    println!("is_browser_task: {}", is_browser_task);
}
