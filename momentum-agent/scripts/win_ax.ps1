# Native Windows UI Automation Extraction for Momentum AI
$ErrorActionPreference = "SilentlyContinue"

try {
    Add-Type @"
        using System;
        using System.Runtime.InteropServices;
        public class Win32Ax {
            [DllImport("user32.dll")]
            public static extern IntPtr GetForegroundWindow();
        }
"@
} catch {}

try {
    [void][System.Reflection.Assembly]::LoadWithPartialName("UIAutomationClient")
    [void][System.Reflection.Assembly]::LoadWithPartialName("UIAutomationTypes")

    $root = $null
    try {
        $hwnd = [Win32Ax]::GetForegroundWindow()
        if ($hwnd -ne [IntPtr]::Zero) {
            $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        }
    } catch {}

    if ($null -eq $root) {
        $root = [System.Windows.Automation.AutomationElement]::FocusedElement
    }

    if ($null -eq $root) {
        Write-Output "[]"
        exit 0
    }

    # If focused element is deep inside, climb to root window if needed
    $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
    $candidate = $root
    while ($true) {
        $parent = $walker.GetParent($candidate)
        if ($null -eq $parent -or $parent -eq [System.Windows.Automation.AutomationElement]::RootElement) {
            break
        }
        $candidate = $parent
    }
    if ($null -ne $candidate) {
        $root = $candidate
    }

    # Query interactive controls: Button, Edit, Hyperlink, ListItem, MenuItem, TabItem
    $cTypes = @(
        [System.Windows.Automation.ControlType]::Button,
        [System.Windows.Automation.ControlType]::Edit,
        [System.Windows.Automation.ControlType]::Hyperlink,
        [System.Windows.Automation.ControlType]::ListItem,
        [System.Windows.Automation.ControlType]::MenuItem,
        [System.Windows.Automation.ControlType]::TabItem
    )

    $results = @()
    $idx = 0

    foreach ($ct in $cTypes) {
        if ($idx -ge 50) { break }
        $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, $ct)
        $children = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
        if ($null -ne $children) {
            foreach ($child in $children) {
                if ($idx -ge 50) { break }
                try {
                    $rect = $child.Current.BoundingRectangle
                    if ($rect.Width -gt 0 -and $rect.Height -gt 0 -and -not $child.Current.IsOffscreen) {
                        $name = $child.Current.Name
                        $type = $ct.ProgrammaticName.Replace("ControlType.", "")
                        $results += @{
                            id = "$idx"
                            role = $type
                            title = if ($name) { $name } else { "" }
                            x = [math]::Round($rect.X + ($rect.Width / 2))
                            y = [math]::Round($rect.Y + ($rect.Height / 2))
                            description = if ($child.Current.HelpText) { $child.Current.HelpText } else { "" }
                            container = "window"
                        }
                        $idx++
                    }
                } catch {}
            }
        }
    }

    $json = $results | ConvertTo-Json -Compress
    if ($null -eq $json -or $json -eq "") {
        Write-Output "[]"
    } else {
        Write-Output $json
    }
} catch {
    Write-Output "[]"
}
