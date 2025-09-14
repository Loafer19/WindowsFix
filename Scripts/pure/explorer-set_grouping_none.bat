@echo off
setlocal EnableDelayedExpansion

echo Setting folder view to Details with Group By None...

:: Set the default "All Items" template to Details view with Group By None in the registry
reg add "HKCU\Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\Bags\AllFolders\Shell" /v "FolderType" /t REG_SZ /d "NotSpecified" /f >nul 2>&1
reg add "HKCU\Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\Bags\AllFolders\Shell" /v "GroupBy" /t REG_SZ /d "System.Null" /f >nul 2>&1
reg add "HKCU\Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\Bags\AllFolders\Shell" /v "Sort" /t REG_SZ /d "System.Null" /f >nul 2>&1
reg add "HKCU\Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\Bags\AllFolders\Shell" /v "ViewMode" /t REG_DWORD /d 1 /f >nul 2>&1

:: Reset all folder views by clearing the Bags registry key
echo Clearing existing folder views...
reg delete "HKCU\Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\Bags" /f >nul 2>&1

:: Restart Explorer to apply changes
echo Restarting Windows Explorer...
taskkill /f /im explorer.exe >nul 2>&1
start explorer.exe

echo Applying view settings to all folders...
powershell -Command "$shell = New-Object -ComObject Shell.Application; $folder = $shell.Namespace(0); $folder.Self.InvokeVerb('Reset Folders')"

echo Done! Folder grouping should now be set to None for all folders.
pause
