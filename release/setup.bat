@echo off
title Voice2Text Setup
echo ============================================
echo   Voice2Text v0.1.0 Setup
echo ============================================
echo.

:: Check WebView2 (required by Tauri)
reg query "HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>&1
if %errorlevel% neq 0 (
    echo [!] WebView2 Runtime not found.
    echo Downloading WebView2 installer...
    curl -L -o "%TEMP%\WebView2Setup.exe" "https://go.microsoft.com/fwlink/p/?LinkId=2124703"
    echo Running installer...
    "%TEMP%\WebView2Setup.exe" /silent /install
    del "%TEMP%\WebView2Setup.exe"
    echo [OK] WebView2 installed.
) else (
    echo [OK] WebView2 Runtime detected.
)

echo.
echo [*] Creating desktop shortcut...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ws=New-Object -ComObject WScript.Shell;$s=$ws.CreateShortcut([Environment]::GetFolderPath('Desktop')+'\Voice2Text.lnk');$s.TargetPath='%~dp0Voice2Text\voice2text.exe';$s.WorkingDirectory='%~dp0Voice2Text';$s.Save()"
echo [OK] Desktop shortcut created.

echo.
echo ============================================
echo   Done! Double-click Voice2Text on desktop.
echo   Ctrl+Alt+Z to start / Ctrl+Alt+X to stop.
echo ============================================
pause
