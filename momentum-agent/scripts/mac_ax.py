import ApplicationServices
import AppKit
import json
import sys

def get_frontmost_app():
    return AppKit.NSWorkspace.sharedWorkspace().frontmostApplication()

def get_ax_app(pid):
    return ApplicationServices.AXUIElementCreateApplication(pid)

def main():
    app = get_frontmost_app()
    if not app:
        print(json.dumps([]))
        return
        
    pid = app.processIdentifier()
    ax_app = get_ax_app(pid)
    
    print(f"PID: {pid}")
    print(f"App: {app.localizedName()}")
    
if __name__ == "__main__":
    main()
