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
    var x: Double
    var y: Double
    var w: Double
    var h: Double
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
    let isWeb = currentContainer.contains("web document")
    // Deep traversal for web content (YouTube/Chromium DOMs are 15-35 levels deep)
    let maxDepth = isWeb ? 36 : 16
    if depth > maxDepth { return }
    
    var nextContainer = currentContainer
    let role = (getAttribute(element, kAXRoleAttribute as String) as? String) ?? ""
    let rawTitle = (getAttribute(element, kAXTitleAttribute as String) as? String) ?? ""
    let desc = getAttribute(element, kAXDescriptionAttribute as String) as? String
    let placeholder = (getAttribute(element, "AXPlaceholderValue") as? String)
        ?? (getAttribute(element, kAXHelpAttribute as String) as? String)
    let val = getAttribute(element, kAXValueAttribute as String) as? String

    // Pick the most descriptive text available for title
    let effectiveTitle = (!rawTitle.isEmpty ? rawTitle : (val ?? (desc ?? ""))).trimmingCharacters(in: .whitespacesAndNewlines)

    // Dynamic container hierarchy
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
        let groupDesc = ((desc ?? "") + " " + rawTitle).lowercased()
        if groupDesc.contains("timeline") {
            nextContainer = "\(appName) timeline"
        } else if groupDesc.contains("sidebar") || groupDesc.contains("nav") {
            nextContainer = "\(appName) sidebar"
        } else if groupDesc.contains("canvas") || groupDesc.contains("viewport") {
            nextContainer = "\(appName) viewport"
        }
    }

    // 1. Ignore Apple Live Text OCR artifacts in Preview document workspace
    if appName == "Preview" && (nextContainer.contains("workspace") || role.contains("DataDetector")) {
        if role == "AXTextArea" || role.contains("DataDetector") || role == "AXStaticText" {
            return
        }
    }

    // 2. Ignore non-interactive hover tooltips
    if role == "AXGroup" && (effectiveTitle.lowercased() == "tooltip" || desc?.lowercased() == "tooltip") {
        return
    }

    if let bounds = getBounds(element) {
        let centerX = bounds.x + (bounds.w / 2)
        let centerY = bounds.y + (bounds.h / 2)
        
        // 3. Filter out off-screen or invisible elements
        if bounds.w > 4 && bounds.h > 4 &&
           bounds.x + bounds.w > 0 && bounds.y + bounds.h > 0 &&
           bounds.x < screenW && bounds.y < screenH {
            
            // 4. Occlusion culling: If a window in front covers this element, it is invisible
            var isOccluded = false
            let point = CGPoint(x: centerX, y: centerY)
            for frontRect in frontWindowRects {
                if frontRect.contains(point) {
                    isOccluded = true
                    break
                }
            }

            // 5. Skip giant root/viewport containers to prevent full-screen covering boxes
            let isFullScreenRoot = bounds.w >= (screenW * 0.95) && bounds.h >= (screenH * 0.95) && (role == "AXWindow" || role == "AXApplication")
            let isGiantGroup = role == "AXGroup" && bounds.w >= (screenW * 0.7) && bounds.h >= (screenH * 0.45)
            
            if !isOccluded && !isFullScreenRoot && !isGiantGroup {
                var shouldInclude = false
                
                switch role {
                case "AXButton", "AXTextField", "AXTextArea", "AXSearchField", "AXComboBox", "AXPopUpButton", "AXCheckBox", "AXRadioButton", "AXSlider", "AXMenuItem", "AXTabItem":
                    shouldInclude = true
                case "AXLink":
                    shouldInclude = true
                case "AXHeading":
                    // Headings (h1, h2, h3 in web pages) are crucial content landmarks
                    if !effectiveTitle.isEmpty {
                        shouldInclude = true
                    }
                case "AXStaticText":
                    // Include static text if meaningful and not duplicate of immediate interactive parent
                    if !effectiveTitle.isEmpty {
                        let isParentInteractive = (parentRole == "AXButton" || parentRole == "AXLink")
                        let isExactParentDuplicate = isParentInteractive && (parentTitle.lowercased() == effectiveTitle.lowercased())
                        if !isExactParentDuplicate {
                            shouldInclude = true
                        }
                    }
                case "AXImage":
                    if (desc != nil && !desc!.isEmpty) || !rawTitle.isEmpty {
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

                if shouldInclude {
                    nodeCounter += 1
                    rawNodes.append(Node(
                        id: "el_\(nodeCounter)",
                        app: appName,
                        role: role,
                        title: effectiveTitle,
                        description: desc,
                        placeholder: placeholder,
                        value: val,
                        container: nextContainer,
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

let ignoredAppNames: Set<String> = [
    "momentum-hud", "momentum", "tauri", "terminal", "iterm", "iterm2", "alacritty", "warp", "kitty", "window server", "dock", "finder"
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

// Default: Inspect visible on-screen windows in z-order (front to back)
if targets.isEmpty {
    let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
    guard let infoList = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] else {
        exit(0)
    }

    var coveredRects: [CGRect] = []
    var seenPids = Set<pid_t>()

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

        if layer == 0 && w > 120 && h > 120 && !ignoredAppNames.contains(owner.lowercased()) {
            var isCovered = false
            for frontRect in coveredRects {
                if frontRect.contains(rect) {
                    isCovered = true
                    break
                }
            }

            if !isCovered && !seenPids.contains(pid) {
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

// Fallback: If no visible windows found, inspect frontmost app
if targets.isEmpty {
    if let front = NSWorkspace.shared.frontmostApplication, front.activationPolicy == .regular {
        let name = front.localizedName ?? "App"
        if !ignoredAppNames.contains(name.lowercased()) {
            let axApp = getAXUIElement(pid: front.processIdentifier)
            AXUIElementSetAttributeValue(axApp, "AXEnhancedUserInterface" as CFString, kCFBooleanTrue)
            AXUIElementSetAttributeValue(axApp, "AXManualAccessibility" as CFString, kCFBooleanTrue)
            
            var targetEl: AXUIElement = axApp
            var mainWinVal: CFTypeRef?
            if AXUIElementCopyAttributeValue(axApp, kAXMainWindowAttribute as CFString, &mainWinVal) == .success,
               let mw = mainWinVal {
                targetEl = mw as! AXUIElement
            }
            targets.append(WindowTarget(app: front, name: name, bounds: CGRect(x: 0, y: 0, width: screenW, height: screenH), element: targetEl))
        }
    }
}

// Traverse target windows
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

// Spatial deduplication: merge overlapping duplicate nodes
var finalNodes: [Node] = []

for node in rawNodes {
    var isDuplicate = false
    let nodeText = (!node.title.isEmpty ? node.title : (node.description ?? (node.value ?? ""))).trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
    
    for (idx, existing) in finalNodes.enumerated() {
        let dx = abs(node.x - existing.x)
        let dy = abs(node.y - existing.y)
        let dw = abs(node.w - existing.w)
        let dh = abs(node.h - existing.h)
        let existingText = (!existing.title.isEmpty ? existing.title : (existing.description ?? (existing.value ?? ""))).trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        
        // Only deduplicate if centers are extremely close AND text is identical or one is empty
        if dx < 6 && dy < 6 && dw < 12 && dh < 12 && (nodeText == existingText || nodeText.isEmpty || existingText.isEmpty) {
            isDuplicate = true
            let nodeIsInteractive = ["AXButton", "AXLink", "AXTextField", "AXSearchField", "AXCheckBox", "AXRadioButton", "AXComboBox", "AXPopUpButton", "AXHeading"].contains(node.role)
            let existingIsInteractive = ["AXButton", "AXLink", "AXTextField", "AXSearchField", "AXCheckBox", "AXRadioButton", "AXComboBox", "AXPopUpButton", "AXHeading"].contains(existing.role)
            
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
