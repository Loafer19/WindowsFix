package main

import (
	"os/exec"
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

type statusMsg string

func unpinNetwork() tea.Cmd {
	return func() tea.Msg {
		// Run reg add command
		regCmd := exec.Command("reg", "add", "HKCU\\Software\\Classes\\CLSID\\{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}", "/v", "System.IsPinnedToNameSpaceTree", "/t", "REG_DWORD", "/d", "0", "/f")
		if err := regCmd.Run(); err != nil {
			return statusMsg("Error: Failed to modify registry - " + err.Error())
		}

		// Kill explorer
		killCmd := exec.Command("taskkill", "/F", "/IM", "explorer.exe")
		killCmd.Run() // Ignore error if explorer not running

		// Wait a moment
		time.Sleep(3 * time.Second)

		// Start explorer
		startCmd := exec.Command("cmd", "/c", "start", "explorer.exe")
		if err := startCmd.Run(); err != nil {
			return statusMsg("Error: Failed to restart explorer - " + err.Error())
		}

		return statusMsg("Success: Network folder has been unpinned from File Explorer's Navigation Panel.")
	}
}

func setGroupingNone() tea.Cmd {
	return func() tea.Msg {
		// Set registry for folder view
		regCmds := []*exec.Cmd{
			exec.Command("reg", "add", "HKCU\\Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\Bags\\AllFolders\\Shell", "/v", "FolderType", "/t", "REG_SZ", "/d", "NotSpecified", "/f"),
			exec.Command("reg", "add", "HKCU\\Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\Bags\\AllFolders\\Shell", "/v", "GroupBy", "/t", "REG_SZ", "/d", "System.Null", "/f"),
			exec.Command("reg", "add", "HKCU\\Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\Bags\\AllFolders\\Shell", "/v", "Sort", "/t", "REG_SZ", "/d", "System.Null", "/f"),
			exec.Command("reg", "add", "HKCU\\Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\Bags\\AllFolders\\Shell", "/v", "ViewMode", "/t", "REG_DWORD", "/d", "1", "/f"),
		}
		for _, cmd := range regCmds {
			if err := cmd.Run(); err != nil {
				return statusMsg("Error: Failed to set registry for grouping - " + err.Error())
			}
		}

		// Clear bags
		deleteCmd := exec.Command("reg", "delete", "HKCU\\Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\Bags", "/f")
		deleteCmd.Run() // Ignore if not exists

		// Kill explorer
		killCmd := exec.Command("taskkill", "/F", "/IM", "explorer.exe")
		killCmd.Run()

		// Wait
		time.Sleep(1 * time.Second)

		// Start explorer
		startCmd := exec.Command("cmd", "/c", "start", "explorer.exe")
		if err := startCmd.Run(); err != nil {
			return statusMsg("Error: Failed to restart explorer - " + err.Error())
		}

		// Apply view settings
		psCmd := exec.Command("powershell", "-Command", "$shell = New-Object -ComObject Shell.Application; $folder = $shell.Namespace(0); $folder.Self.InvokeVerb('Reset Folders')")
		psCmd.Run() // Ignore errors

		return statusMsg("Success: Folder grouping set to None and views reset.")
	}
}

func unpinQuickAccess() tea.Cmd {
	return func() tea.Msg {
		psScript := `
$shell = New-Object -ComObject Shell.Application
$quickAccess = $shell.Namespace("shell:::{679f85cb-0220-4080-b29b-5540cc05aab6}")
$keepPinned = @("Desktop", "Downloads", "Pictures")
foreach ($item in $quickAccess.Items()) {
    if ($item.IsFolder -and $keepPinned -notcontains $item.Name) {
        $item.InvokeVerb("unpinfromhome")
    }
}
`
		psCmd := exec.Command("powershell", "-Command", psScript)
		if err := psCmd.Run(); err != nil {
			return statusMsg("Error: Failed to unpin Quick Access folders - " + err.Error())
		}

		return statusMsg("Success: Quick Access folders unpinned except Desktop, Downloads, and Pictures.")
	}
}
