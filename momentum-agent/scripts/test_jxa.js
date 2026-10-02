ObjC.import('AppKit');
var ws = $.NSWorkspace.sharedWorkspace;
var frontApp = ws.frontmostApplication;
var pid = frontApp.processIdentifier;
console.log("Front PID:", pid);

var sysEvents = Application("System Events");
var processes = sysEvents.processes.whose({unixId: pid});
if (processes.length > 0) {
    var app = processes[0];
    console.log(app.name());
    
    // Attempt to get UI elements
    var windows = app.windows;
    if (windows.length > 0) {
        var win = windows[0];
        console.log("Window Name:", win.name());
        // get UI elements
        var uiElems = win.uiElements();
        console.log("Children:", uiElems.length);
    }
}
