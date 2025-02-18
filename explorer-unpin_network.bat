@echo off

reg add "HKCU\Software\Classes\CLSID\{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}" /v "System.IsPinnedToNameSpaceTree" /t REG_DWORD /d 0 /f >nul 2>&1

echo Network folder has been unpinned from File Explorer's Navigation Panel.

echo Restarting File Explorer...

@taskkill /F /IM explorer.exe
start explorer.exe

pause
