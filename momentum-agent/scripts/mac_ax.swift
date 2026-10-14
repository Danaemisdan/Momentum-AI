import Cocoa
import ApplicationServices
import CoreGraphics
import Foundation

let startTime = CFAbsoluteTimeGetCurrent()

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
    var x: Double
    var y: Double
    var w: Double
    var h: Double
}

var allNodes: [Node] = []
var nodeCounter = 0

func traverse(
    element: AXUIElement,
    appName: String,
    depth: Int = 0,
    currentContainer: String = "window",
    parentRole: String = "",
    parentTitle: String = ""
) {
    let isWeb = currentContainer.contains("web document")
    // Allow deep traversal for web areas (Chromium DOMs are 15-30 levels deep)
    let maxDepth = isWeb ? 36 : 16
    if depth > maxDepth { return }
    
    var nextContainer = currentContainer
    let role = (getAttribute(element, kAXRoleAttribute as String) as? String) ?? ""
    let title = (getAttribute(element, kAXTitleAttribute as String) as? String) ?? ""
    let desc = getAttribute(element, kAXDescriptionAttribute as String) as? String
    let placeholder = (getAttribute(element, "AXPlaceholderValue") as? String)
        ?? (getAttribute(element, kAXHelpAttribute as String) as? String)
    let val = getAttribute(element, kAXValueAttribute as String) as? String

    // Dynamic container hierarchy detection
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
        let groupDesc = ((desc ?? "") + " " + title).lowercased()
        if groupDesc.contains("timeline") {
            nextContainer = "\(appName) timeline"
        } else if groupDesc.contains("sidebar") || groupDesc.contains("nav") {
            nextContainer = "\(appName) sidebar"
        } else if groupDesc.contains("canvas") || groupDesc.contains("viewport") {
            nextContainer = "\(appName) viewport"
        }
    }

    if let bounds = getBounds(element) {
        // Filter out invisible, zero-sized, or entirely off-screen elements
        if bounds.w > 4 && bounds.h > 4 &&
           bounds.x + bounds.w > 0 && bounds.y + bounds.h > 0 &&
           bounds.x < screenW && bounds.y < screenH {
            
            // Skip root window/application background bounds to prevent full-screen covering boxes
            let isFullScreenRoot = bounds.w >= (screenW * 0.98) && bounds.h >= (screenH * 0.98) && (role == "AXWindow" || role == "AXApplication")
            
            if !isFullScreenRoot {
                var shouldInclude = false
                
                switch role {
                case "AXButton", "AXTextField", "AXTextArea", "AXSearchField", "AXComboBox", "AXPopUpButton", "AXCheckBox", "AXRadioButton", "AXSlider", "AXMenuItem", "AXTabItem":
                    shouldInclude = true
                case "AXLink":
                    shouldInclude = true
                case "AXStaticText":
                    let textContent = !title.isEmpty ? title : (val ?? "")
                    let clean = textContent.trimmingCharacters(in: .whitespacesAndNewlines)
                    // Only include text if it's meaningful and not a duplicate child of an interactive button/link
                    if !clean.isEmpty && clean != parentTitle && parentRole != "AXButton" && parentRole != "AXLink" {
                        shouldInclude = true
                    }
                case "AXImage":
                    // Only include images with accessibility labels / descriptions
                    if (desc != nil && !desc!.isEmpty) || !title.isEmpty {
                        shouldInclude = true
                    }
                case "AXGroup":
                    // Include groups that serve as custom interactive controls
                    let actions = getActionNames(element)
                    if (actions.contains("AXPress") || actions.contains("AXConfirm")) && ((desc != nil && !desc!.isEmpty) || !title.isEmpty) {
                        shouldInclude = true
                    }
                default:
                    break
                }

                if shouldInclude {
                    nodeCounter += 1
                    allNodes.append(Node(
                        id: "el_\(nodeCounter)",
                        app: appName,
                        role: role,
                        title: title,
                        description: desc,
                        placeholder: placeholder,
                        value: val,
                        container: nextContainer,
                        x: bounds.x + (bounds.w / 2),
                        y: bounds.y + (bounds.h / 2),
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
                parentTitle: title
            )
        }
    }
}

let ignoredAppNames: Set<String> = [
    "momentum-hud", "momentum", "tauri", "terminal", "iterm", "iterm2", "alacritty", "warp", "kitty", "window server", "dock", "finder"
]

var targetApps: [(NSRunningApplication, String)] = []

// 1. Explicit app target query from CLI argument:
if CommandLine.arguments.count > 1 {
    let query = CommandLine.arguments[1].lowercased().trimmingCharacters(in: .whitespacesAndNewlines)
    if !query.isEmpty && query != "--all" {
        if let app = NSWorkspace.shared.runningApplications.first(where: { a in
            guard a.activationPolicy == .regular else { return false }
            let name = (a.localizedName ?? "").lowercased()
            let bundle = (a.bundleIdentifier ?? "").lowercased()
            return name.contains(query) || bundle.contains(query)
        }) {
            targetApps.append((app, app.localizedName ?? "App"))
        }
    }
}

// 2. Default: Inspect all visible regular GUI applications currently on-screen:
if targetApps.isEmpty {
    let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
    var seenPids = Set<pid_t>()
    
    if let infoList = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] {
        for info in infoList {
            let layer = info[kCGWindowLayer as String] as? Int ?? -1
            let pid = info[kCGWindowOwnerPID as String] as? pid_t ?? 0
            let bounds = info[kCGWindowBounds as String] as? [String: CGFloat] ?? [:]
            let w = bounds["Width"] ?? 0
            let h = bounds["Height"] ?? 0
            
            // Standard GUI window layer 0 with realistic size
            if layer == 0 && w > 120 && h > 120 && !seenPids.contains(pid) {
                seenPids.insert(pid)
                if let app = NSRunningApplication(processIdentifier: pid), app.activationPolicy == .regular {
                    let name = app.localizedName ?? ""
                    if !name.isEmpty && !ignoredAppNames.contains(name.lowercased()) {
                        targetApps.append((app, name))
                    }
                }
            }
        }
    }
}

for (app, name) in targetApps {
    let axApp = getAXUIElement(pid: app.processIdentifier)
    // Enable enhanced accessibility (essential for Chromium/Electron web areas)
    AXUIElementSetAttributeValue(axApp, "AXEnhancedUserInterface" as CFString, kCFBooleanTrue)
    AXUIElementSetAttributeValue(axApp, "AXManualAccessibility" as CFString, kCFBooleanTrue)
    
    traverse(element: axApp, appName: name, depth: 0, currentContainer: "\(name) window")
}

let encoder = JSONEncoder()
if let jsonData = try? encoder.encode(allNodes),
   let jsonString = String(data: jsonData, encoding: .utf8) {
    print(jsonString)
} else {
    print("[]")
}
