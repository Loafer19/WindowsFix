# Get the Shell object
$shell = New-Object -ComObject Shell.Application

# Access the Quick Access namespace
$quickAccess = $shell.Namespace("shell:::{679f85cb-0220-4080-b29b-5540cc05aab6}")

# List of folders to keep pinned
$keepPinned = @("Desktop", "Downloads", "Pictures")

# Get all pinned items and filter out the ones to keep
foreach ($item in $quickAccess.Items()) {
    if ($item.IsFolder -and $keepPinned -notcontains $item.Name) {
        # Unpin the folder
        $item.InvokeVerb("unpinfromhome")
    }
}

Write-Host "Finished unpinning folders except Desktop, Downloads, and Pictures."

pause
