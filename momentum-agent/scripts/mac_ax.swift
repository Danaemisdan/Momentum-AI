import Cocoa
import ApplicationServices
import Foundation

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
        
        if let pVal = posValue {
            AXValueGetValue(pVal as! AXValue, .cgPoint, &point)
        }
        if let sVal = sizeValue {
            AXValueGetValue(sVal as! AXValue, .cgSize, &size)
        }
        
        return (Double(point.x), Double(point.y), Double(size.width), Double(size.height))
    }
    return nil
}

struct Node: Codable {
    var id: String
    var role: String
    var title: String
    var description: String?
    var placeholder: String?
    var value: String?
    var x: Double
    var y: Double
    var w: Double
    var h: Double
}

var allNodes: [Node] = []
var nodeCounter = 0

func traverse(element: AXUIElement, depth: Int = 0) {
    if depth > 10 { return } // prevent infinite loops
    
    if let bounds = getBounds(element) {
        if bounds.w > 0 && bounds.h > 0 { // Ignore invisible elements
            let role = (getAttribute(element, kAXRoleAttribute as String) as? String) ?? ""
            let title = (getAttribute(element, kAXTitleAttribute as String) as? String) ?? ""
            let desc = getAttribute(element, kAXDescriptionAttribute as String) as? String
            let placeholder = (getAttribute(element, "AXPlaceholderValue") as? String)
                ?? (getAttribute(element, kAXHelpAttribute as String) as? String)
            let val = getAttribute(element, kAXValueAttribute as String) as? String
            
            // We only care about interactive or text elements
            let isInteractive = ["AXButton", "AXTextField", "AXTextArea", "AXSearchField", "AXLink", "AXMenuItem", "AXStaticText", "AXImage", "AXGroup", "AXComboBox", "AXPopUpButton", "AXCheckBox", "AXRadioButton"].contains(role)
            
            if isInteractive || depth < 2 {
                nodeCounter += 1
                let node = Node(
                    id: "el_\(nodeCounter)",
                    role: role,
                    title: title,
                    description: desc,
                    placeholder: placeholder,
                    value: val,
                    x: bounds.x + (bounds.w / 2),
                    y: bounds.y + (bounds.h / 2),
                    w: bounds.w,
                    h: bounds.h
                )
                allNodes.append(node)
            }
        }
    }
    
    if let children = getChildren(of: element) {
        for child in children {
            traverse(element: child, depth: depth + 1)
        }
    }
}

var targetApp: NSRunningApplication? = nil
let ignoredAppNames: Set<String> = [
    "momentum-hud", "momentum", "tauri", "terminal", "iterm", "iterm2", "alacritty", "warp", "kitty"
]

// 1. If an app query argument is provided, search running regular GUI apps:
if CommandLine.arguments.count > 1 {
    let query = CommandLine.arguments[1].lowercased().trimmingCharacters(in: .whitespacesAndNewlines)
    if !query.isEmpty {
        targetApp = NSWorkspace.shared.runningApplications.first { app in
            guard app.activationPolicy == .regular else { return false }
            let name = (app.localizedName ?? "").lowercased()
            let bundle = (app.bundleIdentifier ?? "").lowercased()
            return name.contains(query) || bundle.contains(query)
        }
    }
}

// 2. Fallback: inspect frontmost application if it is NOT one of our developer/HUD tools:
if targetApp == nil {
    if let front = NSWorkspace.shared.frontmostApplication,
       let frontName = front.localizedName?.lowercased(),
       !ignoredAppNames.contains(frontName) {
        targetApp = front
    }
}

// 3. Fallback: pick the first regular GUI app that isn't ignored:
if targetApp == nil {
    targetApp = NSWorkspace.shared.runningApplications.first { app in
        guard app.activationPolicy == .regular, let name = app.localizedName?.lowercased() else { return false }
        return !ignoredAppNames.contains(name)
    }
}

if let app = targetApp {
    // Bring application window to the foreground so clicks and keyboard focus hit it
    app.activate(options: .activateIgnoringOtherApps)
    usleep(150_000)
    let pid = app.processIdentifier
    let axApp = getAXUIElement(pid: pid)
    traverse(element: axApp)
}

let encoder = JSONEncoder()
if let jsonData = try? encoder.encode(allNodes),
   let jsonString = String(data: jsonData, encoding: .utf8) {
    print(jsonString)
} else {
    print("[]")
}
