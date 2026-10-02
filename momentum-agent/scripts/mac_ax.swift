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
            
            // We only care about interactive or text elements
            let isInteractive = ["AXButton", "AXTextField", "AXTextArea", "AXLink", "AXMenuItem", "AXStaticText", "AXImage", "AXGroup"].contains(role)
            
            if isInteractive || depth < 2 {
                nodeCounter += 1
                let node = Node(
                    id: "el_\(nodeCounter)",
                    role: role,
                    title: title,
                    description: desc,
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

if let frontApp = NSWorkspace.shared.frontmostApplication {
    let pid = frontApp.processIdentifier
    let axApp = getAXUIElement(pid: pid)
    
    traverse(element: axApp)
    
    let encoder = JSONEncoder()
    if let jsonData = try? encoder.encode(allNodes),
       let jsonString = String(data: jsonData, encoding: .utf8) {
        print(jsonString)
    } else {
        print("[]")
    }
}
