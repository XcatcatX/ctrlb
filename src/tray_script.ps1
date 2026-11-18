param(
    [int]$ParentId,
    [string]$LinkFile
)

# Tray icon of Datei-Browser.
# It tells the program what was clicked by writing "show" or "quit" to the output.
# The link is read from the file $LinkFile. The text ::exit:: in that file closes the icon.

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$script:currentLink = ''
$script:tickCount = 0
$waitingText = 'Link: waiting for tunnel...'

function New-AppIcon {
    $bitmap = [System.Drawing.Bitmap]::new(32, 32)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.Clear([System.Drawing.Color]::FromArgb(210, 59, 59))
    $font = [System.Drawing.Font]::new('Segoe UI', [single]18, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
    $graphics.DrawString('D', $font, [System.Drawing.Brushes]::White, 6, 4)
    $graphics.Dispose()
    return [System.Drawing.Icon]::FromHandle($bitmap.GetHicon())
}

function Send-Command([string]$name) {
    [Console]::Out.WriteLine($name)
    [Console]::Out.Flush()
}

function Close-Tray {
    $notify.Visible = $false
    $notify.Dispose()
    [System.Windows.Forms.Application]::Exit()
}

$notify = New-Object System.Windows.Forms.NotifyIcon
$notify.Icon = New-AppIcon
$notify.Text = 'Datei-Browser'

$menu = New-Object System.Windows.Forms.ContextMenuStrip
$linkItem = $menu.Items.Add($waitingText)
$terminalItem = $menu.Items.Add('Open terminal')
$closeItem = $menu.Items.Add('Close')
$notify.ContextMenuStrip = $menu

$linkItem.add_Click({
    if ($script:currentLink -ne '') {
        [System.Windows.Forms.Clipboard]::SetText($script:currentLink)
        $notify.ShowBalloonTip(2000, 'Datei-Browser', 'Link copied.', [System.Windows.Forms.ToolTipIcon]::Info)
    }
})
$terminalItem.add_Click({ Send-Command 'show' })
$closeItem.add_Click({ Send-Command 'quit' })
$notify.add_DoubleClick({ Send-Command 'show' })

$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 500
$timer.add_Tick({
    $script:tickCount++
    if (($script:tickCount % 4) -eq 0) {
        if (-not (Get-Process -Id $ParentId -ErrorAction SilentlyContinue)) {
            Close-Tray
            return
        }
    }
    try {
        $text = ([System.IO.File]::ReadAllText($LinkFile)).Trim()
    } catch {
        return
    }
    if ($text -eq '::exit::') {
        Close-Tray
        return
    }
    if ($text -ne $script:currentLink) {
        $script:currentLink = $text
        if ($text -eq '') { $linkItem.Text = $waitingText } else { $linkItem.Text = $text }
    }
})
$timer.Start()

$notify.Visible = $true
[System.Windows.Forms.Application]::Run()
