@echo off
title Voice2Text Setup
echo ============================================
echo   Voice2Text v0.1.0 Setup
echo ============================================
echo.
echo This will create a desktop shortcut.
echo All dependencies are already included.
echo.

powershell -NoProfile -ExecutionPolicy Bypass -Command "$ws=New-Object -ComObject WScript.Shell;$s=$ws.CreateShortcut([Environment]::GetFolderPath('Desktop')+'\Voice2Text.lnk');$s.TargetPath='%~dp0Voice2Text\voice2text.exe';$s.WorkingDirectory='%~dp0Voice2Text';$s.Save()"
echo [OK] Desktop shortcut created.

echo.
echo ============================================
echo   Done! Double-click Voice2Text on desktop.
echo   Ctrl+Alt+Z to start, Ctrl+Alt+X to stop.
echo ============================================
pause
