# Explorer TUI

A terminal user interface (TUI) application for managing Windows File Explorer settings, built with Go and Bubbletea.

## Features

- Unpin Network Folder from File Explorer's Navigation Panel
- Set Folder Grouping to None (Details view without grouping)
- Unpin Quick Access Folders (except Desktop, Downloads, Pictures)
- Simple menu-driven interface
- Error handling and status feedback
- Modular code structure for easy maintenance

## Usage

1. Build the application:
   ```bash
   go build -o explorer-tui.exe
   ```

2. Run the application:
   ```bash
   ./explorer-tui.exe
   ```

3. Use arrow keys to navigate the menu
4. Press Enter to select an option
5. Press 'q' or Ctrl+C to quit

## Menu Options

- **Unpin Network Folder**: Modifies the Windows registry to unpin the Network folder from File Explorer's navigation panel, then restarts File Explorer to apply changes.
- **Set Grouping to None**: Sets all folders to Details view with no grouping, clears existing folder view settings, and resets folder views.
- **Unpin Quick Access Folders**: Removes pinned folders from Quick Access except for Desktop, Downloads, and Pictures.
- **Exit**: Quit the application

## Requirements

- Go 1.25.1 or later
- Windows (uses Windows-specific commands and PowerShell)

## Dependencies

- [Bubbletea](https://github.com/charmbracelet/bubbletea) - TUI framework for Go

## Installation

1. Clone or download the project
2. Navigate to the directory
3. Run `go mod tidy` to install dependencies
4. Build with `go build`

## Project Structure

- `main.go`: Entry point of the application
- `ui.go`: Contains the TUI model, view, and update logic
- `operations.go`: Contains the functions for performing Explorer operations (registry modifications, PowerShell commands, etc.)
- `go.mod` and `go.sum`: Go module files

## How it works

The application uses Windows commands and PowerShell scripts to modify File Explorer settings:

### Unpin Network Folder
Uses the `reg` command to modify the registry key that controls whether the Network folder is pinned in File Explorer. After modifying the registry, it restarts the explorer.exe process to apply the changes.

Registry key modified:
```
HKCU\Software\Classes\CLSID\{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}
Value: System.IsPinnedToNameSpaceTree = 0
```

### Set Grouping to None
Modifies multiple registry keys to set the default folder template to Details view with no grouping, clears existing folder view bags, restarts Explorer, and uses PowerShell to reset all folder views.

Registry keys modified:
```
HKCU\Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\Bags\AllFolders\Shell
- FolderType: NotSpecified
- GroupBy: System.Null
- Sort: System.Null
- ViewMode: 1
```

### Unpin Quick Access Folders
Uses PowerShell to access the Quick Access namespace and unpin folders except for the specified keep list (Desktop, Downloads, Pictures).

## License

This project is open source. Feel free to use and modify as needed.
