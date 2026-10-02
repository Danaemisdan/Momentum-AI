import Cocoa
import CoreGraphics

if CommandLine.arguments.count < 3 {
    print("Usage: mac_click <x> <y>")
    exit(1)
}

if let x = Double(CommandLine.arguments[1]), let y = Double(CommandLine.arguments[2]) {
    let point = CGPoint(x: x, y: y)
    
    // Mouse Down
    if let mouseDown = CGEvent(mouseEventSource: nil, mouseType: .leftMouseDown, mouseCursorPosition: point, mouseButton: .left) {
        mouseDown.post(tap: .cghidEventTap)
    }
    
    // Short delay
    Thread.sleep(forTimeInterval: 0.05)
    
    // Mouse Up
    if let mouseUp = CGEvent(mouseEventSource: nil, mouseType: .leftMouseUp, mouseCursorPosition: point, mouseButton: .left) {
        mouseUp.post(tap: .cghidEventTap)
    }
    
    print("Clicked at (\(x), \(y))")
}
