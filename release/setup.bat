@echo off
title Voice2Text 安装程序
echo ============================================
echo   Voice2Text v0.1.0 一键安装
echo ============================================
echo.

:: 检查 Python
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [X] 未找到 Python，正在下载...
    curl -L -o python-installer.exe https://mirrors.huaweicloud.com/python/3.12.9/python-3.12.9-amd64.exe
    python-installer.exe /quiet InstallAllUsers=1 PrependPath=1
    del python-installer.exe
    echo [!] 请重启命令行后重新运行本脚本
    pause
    exit /b
)
echo [OK] Python 已安装:
python --version

:: 安装依赖
echo.
echo [*] 安装语音识别依赖...
pip install faster-whisper ctranslate2 numpy -q
if %errorlevel% neq 0 (
    echo [X] pip 安装失败，请检查网络
    pause
    exit /b
)
echo [OK] 依赖安装完成

:: GPU 加速（可选）
echo.
echo [*] 检测 GPU...
nvidia-smi >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] 检测到 NVIDIA GPU，安装 CUDA 加速...
    pip install nvidia-cublas-cu12 -q
) else (
    echo [!] 未检测到 NVIDIA GPU，使用 CPU 推理
)

:: 创建桌面快捷方式
echo.
echo [*] 创建桌面快捷方式...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ws=New-Object -ComObject WScript.Shell;$s=$ws.CreateShortcut([Environment]::GetFolderPath('Desktop')+'\Voice2Text.lnk');$s.TargetPath='%~dp0Voice2Text\voice2text.exe';$s.WorkingDirectory='%~dp0Voice2Text';$s.Save()"
echo [OK] 桌面快捷方式已创建

echo.
echo ============================================
echo   安装完成！
echo   双击桌面 Voice2Text 图标启动
echo   快捷键: Ctrl+Alt+Z 开始 / Ctrl+Alt+X 停止
echo ============================================
pause
