import ApplicationServices
import AppKit

def main():
    app = AppKit.NSWorkspace.sharedWorkspace().frontmostApplication()
    ax_app = ApplicationServices.AXUIElementCreateApplication(app.processIdentifier())
    
    # Try getting children
    err, children = ApplicationServices.AXUIElementCopyAttributeValue(ax_app, "AXChildren", None)
    print(f"Error: {err}, Children: {children}")

main()
