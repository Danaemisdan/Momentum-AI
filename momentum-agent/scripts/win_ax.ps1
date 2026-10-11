# Native Windows UI Automation Extraction for Momentum AI
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$w = Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();' -Name Win32Native -Namespace Win32Functions -PassThru
$hwnd = $w::GetForegroundWindow()

if ($hwnd -eq [IntPtr]::Zero) {
    Write-Output "[]"
    exit 0
}

try {
    $element = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
    if ($null -eq $element) {
        Write-Output "[]"
        exit 0
    }

    $condition = [System.Windows.Automation.Condition]::TrueCondition
    $raw_children = $element.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)

    $results = @()
    $idx = 0
    foreach ($child in $raw_children) {
        if ($idx -ge 80) { break }
        $rect = $child.Current.BoundingRectangle
        if ($rect.Width -gt 0 -and $rect.Height -gt 0 -and -not $child.Current.IsOffscreen) {
            $name = $child.Current.Name
            $type = $child.Current.ControlType.ProgrammaticName.Replace("ControlType.", "")
            if (-not [string]::IsNullOrWhiteSpace($name) -or $type -eq "Edit" -or $type -eq "Button" -or $type -eq "Hyperlink") {
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
