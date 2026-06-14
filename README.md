# Voice2Text 🎙️

> 跨平台 PC 端语音转文字工具 — 快捷键触发，自动粘贴到当前位置

基于 Whisper + Tauri 构建，**本地离线识别**，保护隐私。按下快捷键开始录音，说完后停止，文字自动粘贴到当前光标位置。

## ✨ 特性

- 🎤 **全局快捷键** — 任意应用中一键触发录音/停止
- 🧠 **本地 AI 识别** — whisper.cpp / faster-whisper，无需联网
- 📋 **自动粘贴** — 识别结果自动 Ctrl+V 到当前光标位置
- 🖥️ **系统托盘运行** — 后台常驻，不占屏幕空间
- 🚀 **GPU 加速** — 支持 CUDA（NVIDIA）或 CPU 推理
- 🔒 **隐私安全** — 所有数据本地处理，不上传云端
- 🌍 **跨平台** — Windows / macOS / Linux

## 🎬 使用方法

| 操作 | Windows/Linux | macOS |
|------|---------------|-------|
| 开始录音 | `Ctrl+Alt+Z` | `Cmd+Option+Z` |
| 停止录音 & 识别 | `Ctrl+Alt+X` | `Cmd+Option+X` |

1. 按 `Ctrl+Alt+Z` (Mac: `Cmd+Option+Z`) → 「嘀」一声 → 开始说话
2. 说完按 `Ctrl+Alt+X` (Mac: `Cmd+Option+X`) → 「嘀嘀」两声 → 自动识别
3. 文字自动粘贴到光标位置

> 点击系统托盘图标可打开设置窗口，右键可退出。

## 📋 环境要求

| 依赖 | 版本 | 说明 |
|------|------|------|
| Rust | 1.70+ | 后端语言 |
| Node.js | 18+ | 前端 / Tauri CLI |
| Python | 3.10+ | 语音识别引擎 |
| CUDA | 12.x (可选) | GPU 加速，需 NVIDIA 显卡 |

### Python 依赖

```bash
pip install faster-whisper ctranslate2 numpy
# GPU 加速（CUDA 12）
pip install nvidia-cublas-cu12 nvidia-cuda-runtime-cu12
```

## 🚀 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/your-username/voice2text.git
cd voice2text
```

### 2. 安装依赖

```bash
npm install            # Tauri CLI + 前端依赖
pip install -r requirements.txt  # Python 语音识别
```

### 3. 启动（开发模式）

```bash
npx tauri dev
```

首次启动会自动下载 whisper 模型（~1.5GB），之后缓存到本地。

### 4. 打包为独立应用

```bash
npx tauri build
```

生成的可执行文件在 `src-tauri/target/release/` 下。

## 🏗️ 架构

```
┌─────────────────────────────────────────┐
│                 用户                     │
│   Win+Shift+Z 开始 / Win+Shift+X 停止    │
└──────────────┬──────────────────────────┘
               │ 全局快捷键
┌──────────────▼──────────────────────────┐
│           Tauri 桌面壳                   │
│  ┌──────────────────────────────────┐   │
│  │  系统托盘 (隐藏窗口)              │   │
│  │  src-tauri/src/lib.rs            │   │
│  └──────────────────────────────────┘   │
│  ┌────────────┬──────────────────────┐  │
│  │ 音频采集    │  语音识别引擎         │  │
│  │ cpal 立体声 │  Python 持久进程      │  │
│  │ → 单声道    │  faster-whisper      │  │
│  │ → 16kHz    │  CUDA/CPU int8       │  │
│  └────────────┴──────────────────────┘  │
│  ┌──────────────────────────────────┐   │
│  │  剪贴板 + 模拟粘贴                │   │
│  │  arboard + PowerShell SendKeys   │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

```
voice2text/
├── src/                        # 前端 (HTML/CSS/JS)
│   ├── index.html              # 状态窗口
│   ├── styles.css              # 深色主题
│   └── app.js                  # Tauri 事件监听
├── src-tauri/                  # Rust 后端
│   ├── Cargo.toml              # Rust 依赖
│   ├── tauri.conf.json         # Tauri 配置
│   ├── capabilities/default.json
│   ├── whisper_server.py       # 持久化语音识别服务
│   ├── whisper_transcribe.py   # 单次识别脚本（备用）
│   └── src/
│       ├── main.rs             # 入口
│       ├── lib.rs              # 应用初始化、快捷键、托盘
│       ├── audio_capture.rs    # 麦克风采集（独立线程）
│       ├── whisper_engine.rs   # Rust ↔ Python 通信
│       ├── cloud_api.rs        # 云端 API 客户端（占位）
│       └── commands.rs         # Tauri IPC 命令
├── icons/                      # 应用图标
├── 启动Voice2Text.bat          # Windows 一键启动脚本
├── package.json                # Node.js 配置
└── README.md                   # 本文件
```

## 🧠 模型选择

| 模型 | 大小 | 内存占用 | 中文质量 | 推荐场景 |
|------|------|----------|----------|----------|
| `tiny` | 75MB | ~150MB | ❌ 不可用 | 测试 |
| `base` | 142MB | ~280MB | ⚠️ 勉强 | 低配机器 |
| `small` | 466MB | ~900MB | ✅ 可用 | 笔记本 |
| `medium` | 1.5GB | ~3GB | 👍 推荐 | **日常使用** |
| `large-v3` | 3GB | ~6GB | 🏆 最佳 | 高配台式 |

默认使用 `medium`，可在 `whisper_server.py` 中修改。

## ⌨️ 自定义快捷键

编辑 `src-tauri/src/lib.rs` 中的快捷键注册：

```rust
// 开始录音
app.global_shortcut()
    .on_shortcut("Ctrl+Alt+Z", move |...| { ... });

// 停止录音
app.global_shortcut()
    .on_shortcut("Ctrl+Alt+X", move |...| { ... });
```

支持的修饰键：`Super` (Win), `Ctrl`, `Alt`, `Shift`

## 🔧 常见问题

### 识别很慢？

1. 确认安装了 CUDA 12 + `nvidia-cublas-cu12`
2. 确认 `faster-whisper` 使用 `device="cuda"`
3. 查看终端日志确认 GPU 加载成功

### 快捷键无效？

- `Super` 即 Windows 键
- 某些笔记本功能键需按 `Fn`
- 查看终端日志确认「快捷键已注册」

### 识别结果乱码？

- 确保 `PYTHONUTF8=1` 环境变量已设置
- 检查麦克风是否为默认设备

### 没有声音/识别为音乐？

- 检查麦克风权限
- 确保音频是单声道 16kHz（代码已自动转换）

## 📄 技术栈

- [Tauri 2.x](https://tauri.app) — 跨平台桌面框架
- [faster-whisper](https://github.com/SYSTRAN/faster-whisper) — 高速 Whisper 推理
- [CTranslate2](https://github.com/OpenNMT/CTranslate2) — 推理加速引擎
- [cpal](https://github.com/RustAudio/cpal) — 跨平台音频采集
- [arboard](https://github.com/1Password/arboard) — 剪贴板操作

## 📜 许可证

MIT License

---

**Voice2Text** — 让语音输入像呼吸一样自然。
