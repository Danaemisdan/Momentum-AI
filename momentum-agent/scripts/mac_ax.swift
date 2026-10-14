import Cocoa
import ApplicationServices
import CoreGraphics
import Foundation

let mainScreen = NSScreen.main?.frame ?? CGRect(x: 0, y: 0, width: 2560, height: 1600)
let screenW = Double(mainScreen.width)
let screenH = Double(mainScreen.height)

func getAXUIElement(pid: pid_t) -> AXUIElement {
    return AXUIElementCreateApplication(pid)
}

func getChildren(of element: AXUIElement) -> [AXUIElement]? {
    var value: CFTypeRef?
    let result = AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &value)
    if result == .success, let children = value as? [AXUIElement] {
        return children
    }
    return nil
}

func getAttribute(_ element: AXUIElement, _ attribute: String) -> Any? {
    var value: CFTypeRef?
    let result = AXUIElementCopyAttributeValue(element, attribute as CFString, &value)
    if result == .success {
        return value
    }
    return nil
}

func getBounds(_ element: AXUIElement) -> (x: Double, y: Double, w: Double, h: Double)? {
    var posValue: CFTypeRef?
    var sizeValue: CFTypeRef?
    
    if AXUIElementCopyAttributeValue(element, kAXPositionAttribute as CFString, &posValue) == .success,
       AXUIElementCopyAttributeValue(element, kAXSizeAttribute as CFString, &sizeValue) == .success {
        var point = CGPoint.zero
        var size = CGSize.zero
        if let pVal = posValue { AXValueGetValue(pVal as! AXValue, .cgPoint, &point) }
        if let sVal = sizeValue { AXValueGetValue(sVal as! AXValue, .cgSize, &size) }
        return (Double(point.x), Double(point.y), Double(size.width), Double(size.height))
    }
    return nil
}

func getActionNames(_ element: AXUIElement) -> [String] {
    var actions: CFArray?
    if AXUIElementCopyActionNames(element, &actions) == .success, let list = actions as? [String] {
        return list
    }
    return []
}

struct Node: Codable {
    var id: String
    var app: String
    var role: String
    var title: String
    var description: String?
    var placeholder: String?
    var value: String?
    var container: String?
    var action: String
    var purpose: String
    var destination: String
    var human_intent: String
    var x: Double
    var y: Double
    var w: Double
    var h: Double
}

func inferCognitiveModel(
    role: String,
    subrole: String = "",
    title: String,
    desc: String?,
    placeholder: String?,
    val: String?,
    container: String,
    appName: String,
    bounds: (x: Double, y: Double, w: Double, h: Double)? = nil
) -> (action: String, purpose: String, destination: String, human_intent: String) {
    let lowerRole = role.lowercased()
    let lowerSubrole = subrole.lowercased()
    let rawText = (!title.isEmpty ? title : (desc ?? (val ?? (placeholder ?? "")))).trimmingCharacters(in: .whitespacesAndNewlines)
    let lowerText = rawText.lowercased()
    let lowerContainer = container.lowercased()

    // Tokenize text into words for boundary-accurate matching (avoids "display" matching "play")
    let tokenArray = lowerText.components(separatedBy: CharacterSet.alphanumerics.inverted).filter { !$0.isEmpty }
    let tokens = Set(tokenArray)

    // Helper for clean display subject
    let displaySubject = rawText.isEmpty ? (desc ?? "control") : rawText

    // 1. macOS Window Traffic Lights (Close / Minimize / Zoom-Fullscreen)
    let isWindowTrafficLight = lowerRole == "axbutton" && (
        lowerSubrole.contains("close") || lowerSubrole.contains("min") || lowerSubrole.contains("zoom") || lowerSubrole.contains("fullscreen") ||
        (bounds != nil && bounds!.y <= 85 && bounds!.w <= 24 && bounds!.h <= 24 && bounds!.x <= 85)
    )

    if isWindowTrafficLight {
        let bx = bounds?.x ?? 0
        if lowerSubrole == "axclosebutton" || lowerText.contains("close window") || lowerText.contains("close the window") || bx <= 35 {
            return (
                action: "click",
                purpose: "Close active window",
                destination: "Closed window",
                human_intent: "Dismiss current window to tidy desktop workspace"
            )
        }
        if lowerSubrole == "axminimizebutton" || lowerText.contains("minimize window") || lowerText.contains("minimize the window") || (bx > 35 && bx <= 55) {
            return (
                action: "click",
                purpose: "Minimize window to Dock",
                destination: "macOS Dock",
                human_intent: "Temporarily stash window in Dock while preserving task state in memory"
            )
        }
        if lowerSubrole == "axfullscreenbutton" || lowerSubrole == "axzoombutton" || lowerText.contains("zoom") || lowerText.contains("full screen") || bx > 55 {
            return (
                action: "click",
                purpose: "Toggle window full screen / zoom",
                destination: "Expanded / restored workspace",
                human_intent: "Maximize work area to full display or restore standard window dimensions"
            )
        }
    }

    // 2. macOS Dock items
    if lowerContainer.contains("dock") || lowerRole.contains("dockitem") {
        return (
            action: "click",
            purpose: "Launch or bring '\(displaySubject)' into foreground",
            destination: "Application surface: \(displaySubject)",
            human_intent: "Switch to or launch \(displaySubject) to perform tasks in that application"
        )
    }

    // 3. System / App MenuBar items
    if lowerContainer.contains("menu bar") || lowerRole.contains("menubaritem") {
        if tokens.contains("apple") || lowerText.contains("") {
            return (
                action: "click",
                purpose: "Open macOS system menu",
                destination: "System settings, restart, shutdown",
                human_intent: "Access macOS system-level configurations, updates, and power options"
            )
        }
        if tokens.contains("file") {
            return (
                action: "click",
                purpose: "Open File operations menu",
                destination: "New, Open, Save, Export commands",
                human_intent: "Manage documents, project files, and system IO operations"
            )
        }
        if tokens.contains("edit") {
            return (
                action: "click",
                purpose: "Open Edit operations menu",
                destination: "Undo, Redo, Cut, Copy, Paste commands",
                human_intent: "Perform content manipulation, clipboard actions, and find/replace"
            )
        }
        if tokens.contains("view") {
            return (
                action: "click",
                purpose: "Open View customization menu",
                destination: "Layout, Panels, Zoom, Fullscreen commands",
                human_intent: "Adjust workspace display, toggle sidebars, and configure zoom scale"
            )
        }
        if tokens.contains("window") {
            return (
                action: "click",
                purpose: "Open Window arrangement menu",
                destination: "Minimize, Tile, Move to Display commands",
                human_intent: "Organize open windows across screens and desktop spaces"
            )
        }
        if tokens.contains("help") {
            return (
                action: "click",
                purpose: "Open Help & documentation menu",
                destination: "Search help topics and guides",
                human_intent: "Lookup keyboard shortcuts, documentation, or report an issue"
            )
        }
        return (
            action: "click",
            purpose: "Open \(displaySubject) application menu",
            destination: "\(displaySubject) dropdown menu",
            human_intent: "Access \(displaySubject) commands and workflows in \(appName)"
        )
    }

    // 4. Browser Tabs and Window Controls
    if lowerContainer.contains("tabs") || lowerText.contains("tab") {
        if lowerText.contains("new tab") || lowerText == "+" {
            return (
                action: "click / keypress (Cmd+T)",
                purpose: "Open new empty browser tab",
                destination: "New blank tab",
                human_intent: "Start a fresh web browsing session or open a new webpage"
            )
        }
        if lowerText == "close" || (tokens.contains("close") && lowerContainer.contains("tabs")) {
            return (
                action: "click / keypress (Cmd+W)",
                purpose: "Close specific browser tab",
                destination: "Remaining tabs",
                human_intent: "Free up system memory and declutter the open tab bar"
            )
        }
        if lowerRole.contains("radio") {
            let cleanTitle = rawText.components(separatedBy: " - ").first ?? rawText
            return (
                action: "click",
                purpose: "Switch active browser tab to '\(cleanTitle)'",
                destination: "Active tab: \(cleanTitle)",
                human_intent: "Bring '\(cleanTitle)' into view to interact with that webpage"
            )
        }
        if lowerText.contains("tab search") || lowerText.contains("search tabs") {
            return (
                action: "click / keypress (Cmd+Shift+A)",
                purpose: "Search across open and recently closed tabs",
                destination: "Tab search popup",
                human_intent: "Quickly locate an open tab by typing keywords"
            )
        }
        if lowerText.contains("tab group") || lowerText.contains("tab groups") {
            return (
                action: "click",
                purpose: "Manage browser tab groups",
                destination: "Tab groups menu",
                human_intent: "Group related tabs together to organize research or projects"
            )
        }
    }

    // 5. Browser Navigation & Toolbar (Back, Forward, Reload, Extensions, Bookmarks)
    if lowerText == "back" || (tokens.contains("back") && lowerContainer.contains("toolbar")) {
        return (
            action: "click / keypress (Cmd+[)",
            purpose: "Navigate back in browser history",
            destination: "Previous webpage",
            human_intent: "Return to the previously visited web page"
        )
    }
    if lowerText == "forward" || (tokens.contains("forward") && lowerContainer.contains("toolbar")) {
        return (
            action: "click / keypress (Cmd+])",
            purpose: "Navigate forward in browser history",
            destination: "Next webpage",
            human_intent: "Advance to the forward page in navigation history"
        )
    }
    if lowerText == "reload" || lowerText == "refresh" || (tokens.contains("reload") && lowerContainer.contains("toolbar")) {
        return (
            action: "click / keypress (Cmd+R)",
            purpose: "Reload current webpage",
            destination: "Refreshed webpage content",
            human_intent: "Fetch latest content and refresh webpage state"
        )
    }
    if lowerText.contains("site information") || lowerText.contains("view site info") {
        return (
            action: "click",
            purpose: "Inspect site security & permissions",
            destination: "Site information popup",
            human_intent: "Check HTTPS connection security, cookies, and site permissions"
        )
    }
    if lowerText.contains("address and search bar") || lowerText.contains("omnibox") || (lowerRole.contains("textfield") && lowerContainer.contains("toolbar")) {
        return (
            action: "type / submit",
            purpose: "Enter website URL or search Google",
            destination: "Target web address or Google search",
            human_intent: "Navigate directly to a website domain or search Google"
        )
    }
    if lowerText.contains("bookmark") {
        return (
            action: "click / keypress (Cmd+D)",
            purpose: "Save tab to bookmarks",
            destination: "Bookmarks library",
            human_intent: "Add current webpage to bookmarks for rapid future access"
        )
    }
    if lowerText.contains("extension") || lowerText == "extensions" {
        return (
            action: "click",
            purpose: "Open browser extensions menu",
            destination: "Extensions popup panel",
            human_intent: "Manage or activate installed browser extensions"
        )
    }
    if lowerText.contains("control your music") || lowerText.contains("media control") {
        return (
            action: "click",
            purpose: "Open global media playback controls",
            destination: "Media control overlay",
            human_intent: "Control background audio or video playback across tabs"
        )
    }

    // 6. AI Assistant / Gemini / Copilot / Agents
    if tokens.contains("gemini") || lowerText.contains("ask gemini") {
        return (
            action: "click",
            purpose: "Activate Gemini AI assistant",
            destination: "Gemini side panel",
            human_intent: "Query Gemini AI for page summarization and intelligent assistance"
        )
    }
    if tokens.contains("copilot") || tokens.contains("chatgpt") || tokens.contains("claude") {
        return (
            action: "click",
            purpose: "Open AI assistant panel",
            destination: "AI assistant interface",
            human_intent: "Query AI model for code assistance, answers, or text generation"
        )
    }

    // 7. Execution Control / Stop / Cancel / Abort
    if tokens.contains("stop") || tokens.contains("halt") || tokens.contains("abort") {
        return (
            action: "click / keypress (Esc)",
            purpose: "Interrupt running agent execution",
            destination: "Idle agent state",
            human_intent: "Immediately abort ongoing autonomous execution or LLM generation"
        )
    }
    if tokens.contains("cancel") {
        return (
            action: "click / keypress (Esc)",
            purpose: "Cancel pending operation",
            destination: "Previous state",
            human_intent: "Abort current dialog or form without saving changes"
        )
    }

    // 8. Creation / New Item (Conversation, Tab, Note, Project, Content)
    if tokens.contains("new") || tokens.contains("create") || tokens.contains("compose") {
        if lowerText.contains("conversation") || lowerText.contains("chat") {
            return (
                action: "click",
                purpose: "Start fresh conversation",
                destination: "New conversation workspace",
                human_intent: "Begin a clean AI session without previous context"
            )
        }
        if lowerText.contains("note") {
            return (
                action: "click",
                purpose: "Create new note",
                destination: "Blank note editor",
                human_intent: "Write down thoughts or notes in a fresh document"
            )
        }
        if lowerText.contains("project") {
            return (
                action: "click",
                purpose: "Initialize new project",
                destination: "Project setup dialog",
                human_intent: "Create a new project workspace directory"
            )
        }
        if lowerText.contains("video") || lowerText == "create" {
            return (
                action: "click",
                purpose: "Upload video or create post",
                destination: "Creator studio upload modal",
                human_intent: "Upload a new video, go live, or create community post"
            )
        }
        return (
            action: "click",
            purpose: "Create new \(displaySubject)",
            destination: "\(displaySubject) editor / modal",
            human_intent: "Initialize a new item without affecting current work"
        )
    }

    // 9. Config, Settings, Automations, Customizations
    if tokens.contains("customizations") || tokens.contains("customization") {
        return (
            action: "click",
            purpose: "Configure agent skills, rules, and plugins",
            destination: "Customizations manager",
            human_intent: "Tailor agent behavior, active skills, and custom rules"
        )
    }
    if tokens.contains("automations") || tokens.contains("automation") {
        return (
            action: "click",
            purpose: "Open background automations manager",
            destination: "Automations dashboard",
            human_intent: "Inspect, create, or modify scheduled recurring background jobs"
        )
    }
    if tokens.contains("projects") || tokens.contains("project") {
        return (
            action: "click",
            purpose: "Open project workspace selector",
            destination: "Projects list",
            human_intent: "Switch between project repositories or configure workspaces"
        )
    }
    if tokens.contains("settings") || tokens.contains("preferences") || (tokens.contains("options") && !tokens.contains("display")) || lowerText == "config" {
        return (
            action: "click",
            purpose: "Open application settings & preferences",
            destination: "Settings configuration panel",
            human_intent: "Configure system preferences, API credentials, and interface options"
        )
    }
    if lowerText.contains("display options") || lowerText.contains("view options") {
        return (
            action: "click",
            purpose: "Configure layout and visual display options",
            destination: "Display settings popup",
            human_intent: "Adjust visual zoom, theme, and column arrangement"
        )
    }

    // 10. Pinning & Priority
    if tokens.contains("pin") || tokens.contains("unpin") || tokens.contains("star") || tokens.contains("favorite") {
        return (
            action: "toggle",
            purpose: "Toggle pinned status for '\(displaySubject)'",
            destination: "Pinned items collection",
            human_intent: "Keep high-priority item easily accessible at the top of the interface"
        )
    }

    // 11. Destructive & Archive
    if tokens.contains("delete") || tokens.contains("trash") || tokens.contains("remove") || tokens.contains("archive") || tokens.contains("discard") || tokens.contains("bin") {
        return (
            action: "click",
            purpose: "Delete or archive '\(displaySubject)'",
            destination: "Trash / archive storage",
            human_intent: "Permanently remove or archive unwanted item to clean up workspace"
        )
    }

    // 12. Clipboard Operations
    if tokens.contains("copy") {
        return (
            action: "click / keypress (Cmd+C)",
            purpose: "Copy '\(displaySubject)' to clipboard",
            destination: "System clipboard",
            human_intent: "Copy text or content buffer into clipboard for pasting"
        )
    }
    if tokens.contains("paste") {
        return (
            action: "click / keypress (Cmd+V)",
            purpose: "Paste from clipboard",
            destination: "Active input buffer",
            human_intent: "Insert copied content at cursor position"
        )
    }
    if tokens.contains("duplicate") || tokens.contains("clone") {
        return (
            action: "click",
            purpose: "Duplicate '\(displaySubject)'",
            destination: "Duplicated copy",
            human_intent: "Make an exact copy of selected item to edit safely"
        )
    }

    // 13. Voice & Audio Input
    if tokens.contains("voice") || tokens.contains("mic") || tokens.contains("microphone") || tokens.contains("dictate") {
        return (
            action: "click",
            purpose: "Activate voice input / microphone",
            destination: "Audio speech-to-text listener",
            human_intent: "Dictate query or message using voice instead of typing"
        )
    }

    // 14. Media & Video Playback Controls (EXACT token matching, never naive substring)
    if tokens.contains("play") || tokens.contains("pause") {
        return (
            action: "click / keypress (k)",
            purpose: "Toggle media playback",
            destination: "Playback state: playing/paused",
            human_intent: "Resume or temporarily pause media stream"
        )
    }
    if tokens.contains("mute") || tokens.contains("unmute") || tokens.contains("volume") {
        return (
            action: "click / keypress (m)",
            purpose: "Toggle audio output mute",
            destination: "Audio state: muted/unmuted",
            human_intent: "Silence or restore audio without disrupting playback"
        )
    }
    if tokens.contains("seek") || lowerRole.contains("slider") {
        return (
            action: "drag / click",
            purpose: "Scrub media playback timeline",
            destination: "Target media timestamp",
            human_intent: "Jump forward or backward to a specific point in the video"
        )
    }
    if tokens.contains("subtitles") || tokens.contains("caption") || tokens.contains("captions") {
        return (
            action: "toggle / keypress (c)",
            purpose: "Toggle closed captions overlay",
            destination: "Captions state: on/off",
            human_intent: "Display spoken dialogue as synchronized subtitle text"
        )
    }
    if lowerText.contains("full screen") || lowerText.contains("theater mode") || lowerText.contains("cinema mode") {
        return (
            action: "click / keypress (f)",
            purpose: "Expand viewing dimensions",
            destination: "Full-screen viewport",
            human_intent: "Maximize video to fill display without UI distractions"
        )
    }
    if tokens.contains("next") && (lowerText.contains("video") || lowerContainer.contains("web document")) {
        return (
            action: "click / keypress (Shift+N)",
            purpose: "Skip to next video in queue",
            destination: "Adjacent video player",
            human_intent: "Advance to the next recommendation in queue"
        )
    }
    if tokens.contains("previous") && (lowerText.contains("video") || lowerContainer.contains("web document")) {
        return (
            action: "click",
            purpose: "Skip to previous video in queue",
            destination: "Previous video player",
            human_intent: "Replay or navigate back to the previous video"
        )
    }

    // 15. Social & Video Platform Actions
    if lowerText == "guide" || (tokens.contains("guide") && lowerContainer.contains("web document")) {
        return (
            action: "click",
            purpose: "Toggle navigation guide drawer",
            destination: "Navigation drawer (expanded/collapsed)",
            human_intent: "Expand or collapse primary navigation sidebar"
        )
    }
    if tokens.contains("like") && !tokens.contains("dislike") {
        return (
            action: "toggle",
            purpose: "Appraise video positively",
            destination: "Liked state (+1 count)",
            human_intent: "Support creator and train recommendation algorithm"
        )
    }
    if tokens.contains("dislike") {
        return (
            action: "toggle",
            purpose: "Signal negative content appraisal",
            destination: "Disliked state",
            human_intent: "Downvote content to reduce similar recommendations"
        )
    }
    if tokens.contains("subscribe") || tokens.contains("subscribed") {
        return (
            action: "click",
            purpose: "Subscribe to channel",
            destination: "Subscriptions feed",
            human_intent: "Follow creator to receive updates when new content is uploaded"
        )
    }
    if tokens.contains("share") {
        return (
            action: "click",
            purpose: "Open content share modal",
            destination: "Share options modal",
            human_intent: "Copy shareable link or send to contacts on social apps"
        )
    }
    if tokens.contains("remix") || tokens.contains("clip") {
        return (
            action: "click",
            purpose: "Create short video remix or snippet",
            destination: "Remix editor",
            human_intent: "Sample audio or video segment to create derivative content"
        )
    }
    if tokens.contains("comments") || tokens.contains("comment") {
        return (
            action: "click",
            purpose: "Open audience discussion thread",
            destination: "Comments section",
            human_intent: "Read audience reactions or write a public comment"
        )
    }
    if lowerText == "home" || lowerText == "youtube home" {
        return (
            action: "click",
            purpose: "Return to main recommendations feed",
            destination: "Home feed",
            human_intent: "Explore general recommended content"
        )
    }
    if lowerText == "shorts" {
        return (
            action: "click",
            purpose: "Switch to vertical short-form feed",
            destination: "Shorts player",
            human_intent: "Watch entertaining vertical short-form videos"
        )
    }
    if lowerText == "subscriptions" {
        return (
            action: "click",
            purpose: "View feed from followed channels",
            destination: "Subscriptions feed",
            human_intent: "Check newly uploaded videos from subscribed creators"
        )
    }
    if lowerText == "history" {
        return (
            action: "click",
            purpose: "Browse watch history",
            destination: "History page",
            human_intent: "Revisit past videos or activities"
        )
    }
    if lowerText == "playlists" || lowerText == "library" {
        return (
            action: "click",
            purpose: "Browse saved video lists",
            destination: "Playlists page",
            human_intent: "Play or organize custom video collections"
        )
    }
    if lowerText == "watch later" {
        return (
            action: "click",
            purpose: "Open queued videos",
            destination: "Watch Later list",
            human_intent: "Watch videos bookmarked for later viewing"
        )
    }
    if lowerText == "liked videos" {
        return (
            action: "click",
            purpose: "Open liked videos archive",
            destination: "Liked videos archive",
            human_intent: "Replay favorite videos"
        )
    }
    if lowerText.contains("new content available") {
        let channelName = rawText.components(separatedBy: ".").first ?? rawText
        return (
            action: "click",
            purpose: "Open '\(channelName)' channel feed",
            destination: "\(channelName) channel page",
            human_intent: "Browse latest uploads from subscribed creator \(channelName)"
        )
    }
    if (lowerContainer.contains("web document") || appName.contains("Chrome") || appName.contains("Safari")) && lowerRole == "axlink" && (lowerText.contains("ago") || lowerText.contains("views") || lowerText.contains("watching") || lowerText.contains("streamed")) {
        let cleanVideoTitle = rawText.components(separatedBy: "\n").first ?? rawText
        return (
            action: "click",
            purpose: "Watch video: '\(cleanVideoTitle)'",
            destination: "YouTube video player",
            human_intent: "Stream and watch '\(cleanVideoTitle)' video"
        )
    }
    if lowerContainer.contains("sidebar") && (lowerRole == "axlink" || lowerRole == "axbutton") {
        let cleanConvTitle = rawText.components(separatedBy: "\n").first ?? rawText
        return (
            action: "click",
            purpose: "Open conversation: '\(cleanConvTitle)'",
            destination: "Conversation workspace",
            human_intent: "Resume or review discussion in '\(cleanConvTitle)'"
        )
    }

    // 16. Search Inputs and Search Submission
    if tokens.contains("search") || tokens.contains("find") || lowerRole.contains("searchfield") {
        if lowerRole.contains("button") {
            return (
                action: "click",
                purpose: "Execute search query for entered keywords",
                destination: "Search results feed",
                human_intent: "Submit search query to locate matching videos, channels, or documents"
            )
        }
        return (
            action: "type / submit",
            purpose: "Input search keywords",
            destination: "Search query buffer",
            human_intent: "Type keywords to discover specific content, channels, or documents"
        )
    }

    // 17. User Profile & Notifications
    if tokens.contains("notifications") || tokens.contains("alerts") {
        return (
            action: "click",
            purpose: "Open notifications feed",
            destination: "Notifications panel",
            human_intent: "Review recent subscriber alerts, mentions, and updates"
        )
    }
    if tokens.contains("account") || tokens.contains("profile") || lowerText.contains("person") || lowerText.contains("avatar") {
        return (
            action: "click",
            purpose: "Open user profile & account menu",
            destination: "Account settings dropdown",
            human_intent: "Switch user profiles, manage channel settings, or sign out"
        )
    }

    // 18. Build, Compilation, Run, Debug
    if tokens.contains("run") || tokens.contains("debug") || tokens.contains("compile") || tokens.contains("build") {
        return (
            action: "click / keypress (Cmd+R)",
            purpose: "Run application / execute build step",
            destination: "Console output / runtime",
            human_intent: "Compile source code and execute runtime verification"
        )
    }

    // 19. Context-Aware Deterministic Fallbacks (NO generic robotic text)
    if lowerRole.contains("heading") {
        return (
            action: "read",
            purpose: "Section header: '\(displaySubject)'",
            destination: "Informational heading",
            human_intent: "Scan document structure and identify content section"
        )
    }
    if lowerRole.contains("link") {
        return (
            action: "click",
            purpose: "Navigate to '\(displaySubject)'",
            destination: "\(displaySubject) page",
            human_intent: "Follow hyperlink to open referenced webpage or section"
        )
    }
    if lowerRole.contains("checkbox") {
        return (
            action: "toggle",
            purpose: "Toggle '\(displaySubject)' option",
            destination: "Updated option state",
            human_intent: "Enable or disable '\(displaySubject)' setting"
        )
    }
    if lowerRole.contains("radio") {
        return (
            action: "select",
            purpose: "Select '\(displaySubject)'",
            destination: "\(displaySubject) selected state",
            human_intent: "Activate '\(displaySubject)' among mutually exclusive options"
        )
    }
    if lowerRole.contains("textfield") || lowerRole.contains("textarea") || lowerRole.contains("combobox") {
        return (
            action: "type",
            purpose: "Input data into '\(displaySubject)'",
            destination: "Input buffer",
            human_intent: "Provide required parameter or text query"
        )
    }
    if lowerContainer.contains("sidebar") || lowerContainer.contains("nav") {
        return (
            action: "click",
            purpose: "Navigate to '\(displaySubject)'",
            destination: "\(displaySubject) workspace view",
            human_intent: "Switch view to '\(displaySubject)' to inspect related resources"
        )
    }
    if lowerContainer.contains("toolbar") {
        return (
            action: "click",
            purpose: "Activate '\(displaySubject)' toolbar tool",
            destination: "\(displaySubject) tool action",
            human_intent: "Apply '\(displaySubject)' command within \(appName)"
        )
    }
    if lowerContainer.contains("dialog") || lowerContainer.contains("sheet") {
        return (
            action: "click",
            purpose: "Confirm modal option: '\(displaySubject)'",
            destination: "Dismissed dialog",
            human_intent: "Proceed with or dismiss modal dialog using '\(displaySubject)'"
        )
    }
    if lowerRole.contains("button") {
        return (
            action: "click",
            purpose: "Select '\(displaySubject)'",
            destination: "\(displaySubject) action target",
            human_intent: "Engage with '\(displaySubject)' in \(appName)"
        )
    }

    return (
        action: "inspect",
        purpose: "Display: '\(displaySubject)'",
        destination: "Screen viewport",
        human_intent: "View informational content"
    )
}

var rawNodes: [Node] = []
var nodeCounter = 0

func traverse(
    element: AXUIElement,
    appName: String,
    depth: Int = 0,
    currentContainer: String = "window",
    parentRole: String = "",
    parentTitle: String = "",
    frontWindowRects: [CGRect] = []
) {
    if depth > 45 { return }
    
    var nextContainer = currentContainer
    let role = (getAttribute(element, kAXRoleAttribute as String) as? String) ?? ""
    let subrole = ((getAttribute(element, kAXSubroleAttribute as String) as? String) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    let rawTitle = ((getAttribute(element, kAXTitleAttribute as String) as? String) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    let desc = ((getAttribute(element, kAXDescriptionAttribute as String) as? String) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    let val = ((getAttribute(element, kAXValueAttribute as String) as? String) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    let placeholder = ((getAttribute(element, "AXPlaceholderValue") as? String)
        ?? (getAttribute(element, kAXHelpAttribute as String) as? String) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)

    var effectiveTitle = !rawTitle.isEmpty ? rawTitle : (!desc.isEmpty ? desc : (!val.isEmpty ? val : placeholder))

    let interactiveRoles: Set<String> = [
        "AXButton", "AXLink", "AXTextField", "AXTextArea", "AXSearchField",
        "AXComboBox", "AXPopUpButton", "AXCheckBox", "AXRadioButton", "AXSlider", "AXMenuItem", "AXTabItem"
    ]
    let isInteractive = interactiveRoles.contains(role)
    let isParentInteractive = interactiveRoles.contains(parentRole)

    if role == "AXWebArea" {
        nextContainer = "\(appName) web document"
    } else if role == "AXTabGroup" {
        nextContainer = "\(appName) tabs"
    } else if role == "AXToolbar" {
        nextContainer = "\(appName) toolbar"
    } else if role == "AXMenuBar" || role == "AXMenu" {
        nextContainer = "\(appName) menu"
    } else if role == "AXSheet" || role == "AXDialog" {
        nextContainer = "\(appName) dialog"
    } else if role == "AXScrollArea" && (currentContainer == "window" || currentContainer == "\(appName) window") {
        nextContainer = "\(appName) workspace"
    } else if role == "AXSplitGroup" || role == "AXGroup" {
        let groupDesc = (desc + " " + rawTitle).lowercased()
        if groupDesc.contains("timeline") {
            nextContainer = "\(appName) timeline"
        } else if groupDesc.contains("sidebar") || groupDesc.contains("nav") {
            nextContainer = "\(appName) sidebar"
        } else if groupDesc.contains("canvas") || groupDesc.contains("viewport") {
            nextContainer = "\(appName) viewport"
        }
    }

    if appName == "Preview" && (nextContainer.contains("workspace") || role.contains("DataDetector")) {
        if role == "AXTextArea" || role.contains("DataDetector") || role == "AXStaticText" {
            return
        }
    }

    if role == "AXGroup" && (effectiveTitle.lowercased() == "tooltip" || desc.lowercased() == "tooltip") {
        return
    }

    // Child text borrowing: If an interactive button/link has an empty title, borrow from immediate child
    if isInteractive && effectiveTitle.isEmpty {
        if let children = getChildren(of: element) {
            for child in children {
                let cRaw = ((getAttribute(child, kAXTitleAttribute as String) as? String) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
                let cDesc = ((getAttribute(child, kAXDescriptionAttribute as String) as? String) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
                let cVal = ((getAttribute(child, kAXValueAttribute as String) as? String) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
                let cText = !cRaw.isEmpty ? cRaw : (!cDesc.isEmpty ? cDesc : cVal)
                if !cText.isEmpty {
                    effectiveTitle = cText
                    break
                }
            }
        }
    }

    // Drop text / image inside interactive parent (the parent control is the action target!)
    if (role == "AXStaticText" || role == "AXImage") && isParentInteractive {
        // Child is consumed by parent interactive element; do not emit separately
    } else if let bounds = getBounds(element) {
        let centerX = bounds.x + (bounds.w / 2)
        let centerY = bounds.y + (bounds.h / 2)
        
        if bounds.w > 4 && bounds.h > 4 &&
           bounds.x + bounds.w > 0 && bounds.y + bounds.h > 0 &&
           bounds.x < screenW && bounds.y < screenH {
            
            var isOccluded = false
            let point = CGPoint(x: centerX, y: centerY)
            for frontRect in frontWindowRects {
                if frontRect.contains(point) {
                    isOccluded = true
                    break
                }
            }

            let isFullScreenRoot = bounds.w >= (screenW * 0.95) && bounds.h >= (screenH * 0.95) && (role == "AXWindow" || role == "AXApplication")
            let isGiantGroup = role == "AXGroup" && bounds.w >= (screenW * 0.7) && bounds.h >= (screenH * 0.45)
            
            if !isOccluded && !isFullScreenRoot && !isGiantGroup {
                var shouldInclude = false
                
                if isInteractive {
                    shouldInclude = true
                } else {
                    switch role {
                    case "AXHeading":
                        if !effectiveTitle.isEmpty {
                            shouldInclude = true
                        }
                    case "AXStaticText":
                        if !effectiveTitle.isEmpty && !isParentInteractive {
                            let trimmed = effectiveTitle.trimmingCharacters(in: .whitespacesAndNewlines)
                            let isAsciiArt = trimmed.contains("┌") || trimmed.contains("└") || trimmed.contains("│") || trimmed.contains("├") || trimmed.contains("─") || trimmed.contains("┬") || trimmed.contains("┴") || trimmed.contains("┼")
                            let isPurePunctuation = trimmed.count <= 2 && trimmed.allSatisfy { "!@#$%^&*()_+-=[]{}|;':\",./<>?`~".contains($0) }
                            if !isAsciiArt && !isPurePunctuation {
                                shouldInclude = true
                            }
                        }
                    case "AXImage":
                        if !desc.isEmpty || !rawTitle.isEmpty {
                            shouldInclude = true
                        }
                    case "AXGroup":
                        let actions = getActionNames(element)
                        if (actions.contains("AXPress") || actions.contains("AXConfirm")) && !effectiveTitle.isEmpty {
                            shouldInclude = true
                        }
                    default:
                        break
                    }
                }

                if shouldInclude {
                    nodeCounter += 1
                    let (act, purp, dest, intent) = inferCognitiveModel(
                        role: role,
                        subrole: subrole,
                        title: effectiveTitle,
                        desc: desc.isEmpty ? nil : desc,
                        placeholder: placeholder.isEmpty ? nil : placeholder,
                        val: val.isEmpty ? nil : val,
                        container: nextContainer,
                        appName: appName,
                        bounds: (x: bounds.x, y: bounds.y, w: bounds.w, h: bounds.h)
                    )
                    rawNodes.append(Node(
                        id: "el_\(nodeCounter)",
                        app: appName,
                        role: role,
                        title: effectiveTitle,
                        description: desc.isEmpty ? nil : desc,
                        placeholder: placeholder.isEmpty ? nil : placeholder,
                        value: val.isEmpty ? nil : val,
                        container: nextContainer,
                        action: act,
                        purpose: purp,
                        destination: dest,
                        human_intent: intent,
                        x: centerX,
                        y: centerY,
                        w: bounds.w,
                        h: bounds.h
                    ))
                }
            }
        }
    }

    if let children = getChildren(of: element) {
        for child in children {
            traverse(
                element: child,
                appName: appName,
                depth: depth + 1,
                currentContainer: nextContainer,
                parentRole: role,
                parentTitle: effectiveTitle,
                frontWindowRects: frontWindowRects
            )
        }
    }
}

// Extract MenuBar items from frontmost app
func addMenuBarNodes(for frontApp: NSRunningApplication) {
    let axApp = AXUIElementCreateApplication(frontApp.processIdentifier)
    var mbVal: CFTypeRef?
    if AXUIElementCopyAttributeValue(axApp, kAXMenuBarAttribute as CFString, &mbVal) == .success,
       let mb = mbVal {
        if let items = getChildren(of: mb as! AXUIElement) {
            for item in items {
                let title = (getAttribute(item, kAXTitleAttribute as String) as? String) ?? ""
                if let bounds = getBounds(item), !title.isEmpty && bounds.w > 4 && bounds.h > 4 {
                    let (act, purp, dest, intent) = inferCognitiveModel(
                        role: "AXMenuBarItem",
                        subrole: "",
                        title: title,
                        desc: nil,
                        placeholder: nil,
                        val: nil,
                        container: "system menu bar",
                        appName: frontApp.localizedName ?? "System",
                        bounds: (x: bounds.x, y: bounds.y, w: bounds.w, h: bounds.h)
                    )
                    nodeCounter += 1
                    rawNodes.append(Node(
                        id: "el_\(nodeCounter)",
                        app: frontApp.localizedName ?? "System",
                        role: "AXMenuBarItem",
                        title: title,
                        description: "Application menu",
                        placeholder: nil,
                        value: nil,
                        container: "system menu bar",
                        action: act,
                        purpose: purp,
                        destination: dest,
                        human_intent: intent,
                        x: bounds.x + (bounds.w / 2),
                        y: bounds.y + (bounds.h / 2),
                        w: bounds.w,
                        h: bounds.h
                    ))
                }
            }
        }
    }
}

// Extract Dock items from macOS Dock
func addDockNodes() {
    guard let dock = NSRunningApplication.runningApplications(withBundleIdentifier: "com.apple.dock").first else { return }
    let axDock = AXUIElementCreateApplication(dock.processIdentifier)
    if let children = getChildren(of: axDock) {
        for child in children {
            if let items = getChildren(of: child) {
                for item in items {
                    let title = (getAttribute(item, kAXTitleAttribute as String) as? String) ?? ""
                    if let bounds = getBounds(item), !title.isEmpty && bounds.w > 4 && bounds.h > 4 {
                        let (act, purp, dest, intent) = inferCognitiveModel(
                            role: "AXDockItem",
                            subrole: "",
                            title: title,
                            desc: nil,
                            placeholder: nil,
                            val: nil,
                            container: "macOS dock",
                            appName: "macOS Dock",
                            bounds: (x: bounds.x, y: bounds.y, w: bounds.w, h: bounds.h)
                        )
                        nodeCounter += 1
                        rawNodes.append(Node(
                            id: "el_\(nodeCounter)",
                            app: "macOS Dock",
                            role: "AXDockItem",
                            title: title,
                            description: "Dock application icon",
                            placeholder: nil,
                            value: nil,
                            container: "macOS dock",
                            action: act,
                            purpose: purp,
                            destination: dest,
                            human_intent: intent,
                            x: bounds.x + (bounds.w / 2),
                            y: bounds.y + (bounds.h / 2),
                            w: bounds.w,
                            h: bounds.h
                        ))
                    }
                }
            }
        }
    }
}

let ignoredAppNames: Set<String> = [
    "momentum-hud", "momentum", "tauri", "terminal", "iterm", "iterm2", "alacritty", "warp", "kitty", "window server", "dock", "finder", "preview"
]

struct WindowTarget {
    let app: NSRunningApplication
    let name: String
    let bounds: CGRect
    let element: AXUIElement
}

var targets: [WindowTarget] = []

// Check CLI arguments for specific app target
if CommandLine.arguments.count > 1 {
    let query = CommandLine.arguments[1].lowercased().trimmingCharacters(in: .whitespacesAndNewlines)
    if !query.isEmpty && query != "--all" {
        if let app = NSWorkspace.shared.runningApplications.first(where: { a in
            guard a.activationPolicy == .regular else { return false }
            let name = (a.localizedName ?? "").lowercased()
            let bundle = (a.bundleIdentifier ?? "").lowercased()
            return name.contains(query) || bundle.contains(query)
        }) {
            let axApp = getAXUIElement(pid: app.processIdentifier)
            AXUIElementSetAttributeValue(axApp, "AXEnhancedUserInterface" as CFString, kCFBooleanTrue)
            AXUIElementSetAttributeValue(axApp, "AXManualAccessibility" as CFString, kCFBooleanTrue)
            
            var targetEl: AXUIElement = axApp
            var mainWinVal: CFTypeRef?
            if AXUIElementCopyAttributeValue(axApp, kAXMainWindowAttribute as CFString, &mainWinVal) == .success,
               let mw = mainWinVal {
                targetEl = mw as! AXUIElement
            }
            targets.append(WindowTarget(app: app, name: app.localizedName ?? "App", bounds: CGRect(x: 0, y: 0, width: screenW, height: screenH), element: targetEl))
        }
    }
}

// Default: Inspect visible on-screen windows in z-order
if targets.isEmpty {
    var coveredRects: [CGRect] = []
    var seenPids = Set<pid_t>()

    // 1. ALWAYS place the active frontmost application FIRST!
    if let front = NSWorkspace.shared.frontmostApplication,
       front.activationPolicy == .regular,
       !ignoredAppNames.contains((front.localizedName ?? "").lowercased()) {
        let pid = front.processIdentifier
        seenPids.insert(pid)
        let axApp = getAXUIElement(pid: pid)
        AXUIElementSetAttributeValue(axApp, "AXEnhancedUserInterface" as CFString, kCFBooleanTrue)
        AXUIElementSetAttributeValue(axApp, "AXManualAccessibility" as CFString, kCFBooleanTrue)
        
        var targetEl: AXUIElement = axApp
        var winVal: CFTypeRef?
        var frontBounds = CGRect(x: 0, y: 0, width: screenW, height: screenH)
        if AXUIElementCopyAttributeValue(axApp, kAXMainWindowAttribute as CFString, &winVal) == .success,
           let mw = winVal {
            targetEl = mw as! AXUIElement
        } else if AXUIElementCopyAttributeValue(axApp, kAXFocusedWindowAttribute as CFString, &winVal) == .success,
                  let fw = winVal {
            targetEl = fw as! AXUIElement
        } else if AXUIElementCopyAttributeValue(axApp, kAXWindowsAttribute as CFString, &winVal) == .success,
                  let wins = winVal as? [AXUIElement], let first = wins.first {
            targetEl = first
        }

        if let b = getBounds(targetEl) {
            frontBounds = CGRect(x: b.x, y: b.y, width: b.w, height: b.h)
        }
        targets.append(WindowTarget(app: front, name: front.localizedName ?? "App", bounds: frontBounds, element: targetEl))
        coveredRects.append(frontBounds)
    }

    // 2. Add other on-screen windows from CGWindowList in z-order ONLY if not covered
    let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
    if let infoList = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] {
        for info in infoList {
            let layer = info[kCGWindowLayer as String] as? Int ?? -1
            let owner = info[kCGWindowOwnerName as String] as? String ?? ""
            let pid = info[kCGWindowOwnerPID as String] as? pid_t ?? 0
            let boundsDict = info[kCGWindowBounds as String] as? [String: CGFloat] ?? [:]
            let x = boundsDict["X"] ?? 0
            let y = boundsDict["Y"] ?? 0
            let w = boundsDict["Width"] ?? 0
            let h = boundsDict["Height"] ?? 0
            let rect = CGRect(x: x, y: y, width: w, height: h)

            if layer == 0 && w > 120 && h > 120 && !ignoredAppNames.contains(owner.lowercased()) && !seenPids.contains(pid) {
                var isCovered = false
                for frontRect in coveredRects {
                    let intersection = frontRect.intersection(rect)
                    if !intersection.isNull {
                        let intersectArea = intersection.width * intersection.height
                        let rectArea = rect.width * rect.height
                        if rectArea > 0 && (intersectArea / rectArea) > 0.40 {
                            isCovered = true
                            break
                        }
                    }
                }

                if !isCovered {
                    if let app = NSRunningApplication(processIdentifier: pid), app.activationPolicy == .regular {
                        seenPids.insert(pid)
                        let axApp = getAXUIElement(pid: pid)
                        AXUIElementSetAttributeValue(axApp, "AXEnhancedUserInterface" as CFString, kCFBooleanTrue)
                        AXUIElementSetAttributeValue(axApp, "AXManualAccessibility" as CFString, kCFBooleanTrue)
                        
                        var targetEl: AXUIElement = axApp
                        var wVal: CFTypeRef?
                        if AXUIElementCopyAttributeValue(axApp, kAXMainWindowAttribute as CFString, &wVal) == .success,
                           let mw = wVal {
                            targetEl = mw as! AXUIElement
                        } else if AXUIElementCopyAttributeValue(axApp, kAXFocusedWindowAttribute as CFString, &wVal) == .success,
                                  let fw = wVal {
                            targetEl = fw as! AXUIElement
                        }
                        
                        targets.append(WindowTarget(app: app, name: app.localizedName ?? owner, bounds: rect, element: targetEl))
                        coveredRects.append(rect)
                    }
                }
            }
        }
    }
}

// 1. Add top menu bar from frontmost app (if no specific app requested or if frontmost matches)
if let front = NSWorkspace.shared.frontmostApplication {
    addMenuBarNodes(for: front)
}

// 2. Traverse target windows
var frontRectsAccumulator: [CGRect] = []

for target in targets {
    traverse(
        element: target.element,
        appName: target.name,
        depth: 0,
        currentContainer: "\(target.name) window",
        parentRole: "",
        parentTitle: "",
        frontWindowRects: frontRectsAccumulator
    )
    frontRectsAccumulator.append(target.bounds)
}

// 3. Add bottom macOS Dock items
addDockNodes()

// Spatial deduplication: merge overlapping duplicate nodes
var finalNodes: [Node] = []
let dedupeInteractiveRoles: Set<String> = [
    "AXButton", "AXLink", "AXTextField", "AXTextArea", "AXSearchField",
    "AXCheckBox", "AXRadioButton", "AXComboBox", "AXPopUpButton", "AXSlider",
    "AXHeading", "AXDockItem", "AXMenuBarItem", "AXTabItem", "AXMenuItem"
]

for node in rawNodes {
    var isDuplicate = false
    let nodeText = (!node.title.isEmpty ? node.title : (node.description ?? (node.value ?? ""))).trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
    
    for (idx, existing) in finalNodes.enumerated() {
        let dx = abs(node.x - existing.x)
        let dy = abs(node.y - existing.y)
        let dw = abs(node.w - existing.w)
        let dh = abs(node.h - existing.h)
        let existingText = (!existing.title.isEmpty ? existing.title : (existing.description ?? (existing.value ?? ""))).trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        
        let isSameText = (nodeText == existingText) && !nodeText.isEmpty
        let isCenterClose = dx < 10 && dy < 10 && dw < 20 && dh < 20
        let isRowSameText = isSameText && dy < 8 && dx < 80
        
        if isCenterClose || isRowSameText {
            isDuplicate = true
            let nodeIsInteractive = dedupeInteractiveRoles.contains(node.role)
            let existingIsInteractive = dedupeInteractiveRoles.contains(existing.role)
            
            if nodeIsInteractive && !existingIsInteractive {
                finalNodes[idx] = node
            } else if nodeIsInteractive && existingIsInteractive {
                let nodeLen = node.title.count + (node.description?.count ?? 0)
                let existLen = existing.title.count + (existing.description?.count ?? 0)
                if nodeLen > existLen {
                    finalNodes[idx] = node
                }
            }
            break
        }
    }
    
    if !isDuplicate {
        finalNodes.append(node)
    }
}

let encoder = JSONEncoder()
if let jsonData = try? encoder.encode(finalNodes),
   let jsonString = String(data: jsonData, encoding: .utf8) {
    print(jsonString)
} else {
    print("[]")
}
