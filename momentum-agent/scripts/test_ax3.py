import ApplicationServices
import AppKit

def main():
    app = AppKit.NSWorkspace.sharedWorkspace().frontmostApplication()
    ax_app = ApplicationServices.AXUIElementCreateApplication(app.processIdentifier())
    
    err, children = ApplicationServices.AXUIElementCopyAttributeValue(ax_app, "AXChildren", None)
    if not children:
        return
        
    for child in children:
        err, role = ApplicationServices.AXUIElementCopyAttributeValue(child, "AXRole", None)
        err, pos_val = ApplicationServices.AXUIElementCopyAttributeValue(child, "AXPosition", None)
        err, size_val = ApplicationServices.AXUIElementCopyAttributeValue(child, "AXSize", None)
        
        # In PyObjC, AXValue is a core foundation object but its value can be extracted
        # Usually it translates directly if we cast it, or we use AXValueGetValue
        
        print(f"Role: {role}, PosVal Type: {type(pos_val)}")

main()
