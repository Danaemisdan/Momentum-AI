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
    title: String,
    desc: String?,
    placeholder: String?,
    val: String?,
    container: String,
    appName: String
) -> (action: String, purpose: String, destination: String, human_intent: String) {
    let lowerRole = role.lowercased()
    let text = (!title.isEmpty ? title : (desc ?? (val ?? (placeholder ?? "")))).trimmingCharacters(in: .whitespacesAndNewlines)
    let lowerText = text.lowercased()
    let lowerContainer = container.lowercased()

    // 1. macOS Dock items
    if lowerContainer.contains("dock") || lowerRole.contains("dockitem") {
        return (
            action: "click",
            purpose: "Launch or bring \(text) into foreground",
            destination: "Application surface: \(text)",
            human_intent: "Switch to or launch \(text) to perform tasks in that application"
        )
    }

    // 2. System / App MenuBar items
    if lowerContainer.contains("menu bar") || lowerRole.contains("menubaritem") {
        return (
            action: "click",
            purpose: "Open \(text) application menu",
            destination: "\(text) dropdown menu",
            human_intent: "Access system-level commands, preferences, file operations, or tools in \(text)"
        )
    }

    // 3. Search inputs and buttons
    if lowerText.contains("search") || lowerText.contains("find") || lowerRole.contains("searchfield") {
        if lowerRole.contains("button") {
            if lowerText.contains("voice") {
                return (
                    action: "click",
                    purpose: "Activate microphone voice search",
                    destination: "Voice search listener",
                    human_intent: "Speak a search query aloud instead of typing"
                )
            }
            return (
                action: "click",
                purpose: "Execute keyword search query",
                destination: "Search results view",
                human_intent: "Submit entered keyword to locate matching content or videos"
            )
        }
        return (
            action: "type / submit",
            purpose: "Input search keywords",
            destination: "Search results view",
            human_intent: "Locate specific content, videos, channels, or files matching a query"
        )
    }

    // 4. Browser Tabs and Window Controls
    if lowerContainer.contains("tabs") || lowerText.contains("tab") {
        if lowerText.contains("new tab") || lowerText == "+" {
            return (
                action: "click",
                purpose: "Open new empty browser tab",
                destination: "New blank tab",
                human_intent: "Start a fresh web browsing session or open a new webpage"
            )
        }
        if lowerText.contains("close") {
            return (
                action: "click",
                purpose: "Close specific browser tab",
                destination: "Remaining tabs",
                human_intent: "Free up system memory and declutter the open tab bar"
            )
        }
        if lowerRole.contains("radio") {
            let cleanTitle = text.components(separatedBy: " - ").first ?? text
            return (
                action: "click",
                purpose: "Switch active browser tab to \(cleanTitle)",
                destination: "Active tab: \(cleanTitle)",
                human_intent: "Bring \(cleanTitle) into view to interact with that webpage"
            )
        }
    }

    // 5. Window state controls
    if lowerText.contains("close the window") || (lowerContainer.contains("window") && lowerRole == "axbutton" && text.isEmpty) {
        return (
            action: "click",
            purpose: "Close / Minimize / Zoom window",
            destination: "Modified window state",
            human_intent: "Manage desktop window layout and free screen space"
        )
    }

    // 6. Address Bar
    if lowerText.contains("address") || lowerText.contains("url") {
        return (
            action: "type / submit",
            purpose: "Enter website URL or search Google",
            destination: "Target web address or search results",
            human_intent: "Navigate directly to a website domain or search Google"
        )
    }

    // 7. Video Player Controls
    if lowerText.contains("pause") || lowerText.contains("play") {
        return (
            action: "click / keypress (k)",
            purpose: "Toggle audio/video playback",
            destination: "Playback state: paused/playing",
            human_intent: "Temporarily pause video to examine a frame or take a break"
        )
    }
    if lowerText.contains("mute") || lowerText.contains("volume") {
        return (
            action: "click / keypress (m)",
            purpose: "Toggle audio output mute",
            destination: "Audio state: muted/unmuted",
            human_intent: "Silence audio output without disrupting video playback"
        )
    }
    if lowerText.contains("seek") || lowerRole.contains("slider") {
        return (
            action: "drag / click",
            purpose: "Scrub media playback timeline",
            destination: "Target timestamp in media",
            human_intent: "Jump forward or backward to a specific point in the video"
        )
    }
    if lowerText.contains("full screen") || lowerText.contains("cinema") {
        return (
            action: "click / keypress (f)",
            purpose: "Expand viewing dimensions",
            destination: "Full-screen viewport",
            human_intent: "Maximize video to fill screen without UI distractions"
        )
    }
    if lowerText.contains("next video") || lowerText.contains("previous video") {
        return (
            action: "click",
            purpose: "Skip to adjacent video in queue",
            destination: "Adjacent video player",
            human_intent: "Advance to the next recommendation or replay the previous one"
        )
    }
    if lowerText.contains("subtitles") || lowerText.contains("caption") {
        return (
            action: "toggle",
            purpose: "Toggle text caption overlay",
            destination: "Captions state: on/off",
            human_intent: "Read spoken dialogue in text form"
        )
    }

    // 8. Social / Engagement Controls
    if lowerText.contains("like") && !lowerText.contains("dislike") {
        return (
            action: "toggle",
            purpose: "Positive content appraisal",
            destination: "Liked state (+1 count)",
            human_intent: "Support creator and train the algorithm to recommend similar content"
        )
    }
    if lowerText.contains("dislike") {
        return (
            action: "toggle",
            purpose: "Negative content appraisal",
            destination: "Disliked state",
            human_intent: "Signal dissatisfaction and hide similar low-quality recommendations"
        )
    }
    if lowerText.contains("subscribe") || lowerText.contains("follow") {
        return (
            action: "click",
            purpose: "Add creator to personal subscription feed",
            destination: "Subscribed state",
            human_intent: "Receive future uploads and support creator's channel"
        )
    }
    if lowerText.contains("share") {
        return (
            action: "click",
            purpose: "Open content sharing sheet",
            destination: "Share modal / URL copy dialog",
            human_intent: "Copy video link or share with friends on external platforms"
        )
    }
    if lowerText.contains("remix") || lowerText.contains("clip") {
        return (
            action: "click",
            purpose: "Create derivative short video or audio snippet",
            destination: "Shorts remix editor",
            human_intent: "Sample audio or video segment to create a new Short"
        )
    }
    if lowerText.contains("comments") {
        return (
            action: "click",
            purpose: "Open audience discussion thread",
            destination: "Comments section / panel",
            human_intent: "Read reactions, thoughts, or write a public comment"
        )
    }

    // 9. Navigation / Feeds
    if lowerText == "home" {
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
    if lowerText == "downloads" {
        return (
            action: "click",
            purpose: "Access offline media",
            destination: "Downloads page",
            human_intent: "Watch saved videos without an internet connection"
        )
    }

    // 10. Generic Roles
    if lowerRole.contains("heading") {
        return (
            action: "read",
            purpose: "Content section landmark",
            destination: "Informational heading",
            human_intent: "Understand document structure or topic headline"
        )
    }
    if lowerRole.contains("link") {
        return (
            action: "click",
            purpose: "Follow hyperlink to \(text)",
            destination: "\(text) page",
            human_intent: "Navigate to external page or section"
        )
    }
    if lowerRole.contains("button") {
        return (
            action: "click",
            purpose: "Trigger \(text) function",
            destination: "\(text) outcome",
            human_intent: "Execute specific action in \(appName)"
        )
    }
    if lowerRole.contains("textfield") || lowerRole.contains("textarea") || lowerRole.contains("combobox") {
        return (
            action: "type",
            purpose: "Input data into \(text)",
            destination: "Input buffer",
            human_intent: "Provide required text input or parameters"
        )
    }

    return (
        action: "inspect",
        purpose: "Visual display element",
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
                        title: effectiveTitle,
                        desc: desc.isEmpty ? nil : desc,
                        placeholder: placeholder.isEmpty ? nil : placeholder,
                        val: val.isEmpty ? nil : val,
                        container: nextContainer,
                        appName: appName
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
                        title: title,
                        desc: nil,
                        placeholder: nil,
                        val: nil,
                        container: "system menu bar",
                        appName: frontApp.localizedName ?? "System"
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
                            title: title,
                            desc: nil,
                            placeholder: nil,
                            val: nil,
                            container: "macOS dock",
                            appName: "macOS Dock"
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
        var mainWinVal: CFTypeRef?
        var frontBounds = CGRect(x: 0, y: 0, width: screenW, height: screenH)
        if AXUIElementCopyAttributeValue(axApp, kAXMainWindowAttribute as CFString, &mainWinVal) == .success,
           let mw = mainWinVal {
            targetEl = mw as! AXUIElement
            if let b = getBounds(targetEl) {
                frontBounds = CGRect(x: b.x, y: b.y, width: b.w, height: b.h)
            }
        }
        targets.append(WindowTarget(app: front, name: front.localizedName ?? "App", bounds: frontBounds, element: targetEl))
        coveredRects.append(frontBounds)
    }

    // 2. Add other on-screen windows from CGWindowList in z-order
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
                    if frontRect.contains(rect) {
                        isCovered = true
                        break
                    }
                }

                if !isCovered {
                    if let app = NSRunningApplication(processIdentifier: pid), app.activationPolicy == .regular {
                        seenPids.insert(pid)
                        let axApp = getAXUIElement(pid: pid)
                        AXUIElementSetAttributeValue(axApp, "AXEnhancedUserInterface" as CFString, kCFBooleanTrue)
                        AXUIElementSetAttributeValue(axApp, "AXManualAccessibility" as CFString, kCFBooleanTrue)
                        
                        var targetEl: AXUIElement = axApp
                        var mainWinVal: CFTypeRef?
                        if AXUIElementCopyAttributeValue(axApp, kAXMainWindowAttribute as CFString, &mainWinVal) == .success,
                           let mw = mainWinVal {
                            targetEl = mw as! AXUIElement
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
