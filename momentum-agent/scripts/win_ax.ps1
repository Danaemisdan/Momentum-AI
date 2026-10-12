# Native Windows UI Automation Extraction for Momentum AI
$ErrorActionPreference = "SilentlyContinue"

try {
    [void][System.Reflection.Assembly]::LoadWithPartialName("UIAutomationClient")
    [void][System.Reflection.Assembly]::LoadWithPartialName("UIAutomationTypes")

    $element = [System.Windows.Automation.AutomationElement]::FocusedElement
    if ($null -eq $element) {
        Write-Output "[]"
        exit 0
    }

    # Focus on the top-level parent window or the focused element itself
    $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
    $root = $element
    while ($true) {
        $parent = $walker.GetParent($root)
        if ($null -eq $parent -or $parent -eq [System.Windows.Automation.AutomationElement]::RootElement) {
            break
        }
        $root = $parent
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
