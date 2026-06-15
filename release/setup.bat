@echo off
title Voice2Text Setup
echo ============================================
echo   Voice2Text v0.1.0 Setup
echo ============================================
echo.

python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Python not found.
    echo Please install Python 3.10+ from https://python.org
    echo Make sure to check "Add Python to PATH" during install.
    pause
    exit /b 1
)
echo [OK] Python found:
python --version

echo.
echo [*] Installing dependencies...
pip install faster-whisper ctranslate2 numpy --quiet
if %errorlevel% neq 0 (
    echo [ERROR] pip install failed. Check your internet connection.
    pause
    exit /b 1
)
echo [OK] Dependencies installed.

echo.
echo [*] Checking GPU...
nvidia-smi >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] NVIDIA GPU detected. Installing CUDA acceleration...
    pip install nvidia-cublas-cu12 --quiet
) else (
    echo [!] No NVIDIA GPU detected. Using CPU mode.
)

echo.
echo [*] Creating desktop shortcut...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ws=New-Object -ComObject WScript.Shell;$s=$ws.CreateShortcut([Environment]::GetFolderPath('Desktop')+'\Voice2Text.lnk');$s.TargetPath='%~dp0Voice2Text\voice2text.exe';$s.WorkingDirectory='%~dp0Voice2Text';$s.Save()"
echo [OK] Desktop shortcut created.

echo.
echo ============================================
echo   Setup complete!
echo   Double-click Voice2Text on your desktop.
echo   Hotkeys: Ctrl+Alt+Z start / Ctrl+Alt+X stop
echo ============================================
pause
