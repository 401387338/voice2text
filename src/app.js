// ============================================================
// Voice2Text — 系统托盘应用前端
// ============================================================

const invoke = window.__TAURI__?.core?.invoke;
const listen = window.__TAURI__?.event?.listen;

// DOM
const statusBadge = document.getElementById("status-badge");
const statusLabel = document.getElementById("status-label");
const timerEl = document.getElementById("timer");
const historyList = document.getElementById("history-list");
const btnClearHistory = document.getElementById("btn-clear-history");

// 状态
let isRecording = false;
let timerInterval = null;
let recordingStartTime = 0;
const history = [];

// --- 初始化 ---
document.addEventListener("DOMContentLoaded", async () => {
  if (typeof invoke !== "function") {
    statusLabel.textContent = "Tauri API 未加载";
    return;
  }

  // 监听后端事件
  if (typeof listen === "function") {
    listen("recording-started", () => {
      isRecording = true;
      recordingStartTime = Date.now();
      updateStatus("recording");
      startTimer();
    });

    listen("recording-stopped", (event) => {
      isRecording = false;
      stopTimer();
      updateStatus("done");
      const text = event.payload || "";
      addToHistory(text);

      // 2秒后恢复等待状态
      setTimeout(() => {
        if (!isRecording) updateStatus("idle");
      }, 2000);
    });
  }

  // 检查当前录音状态
  try {
    const status = await invoke("get_recording_status");
    if (status) {
      isRecording = true;
      updateStatus("recording");
    }
  } catch (e) {
    // 忽略
  }

  updateStatus("idle");
});

// --- 键盘快捷键（在窗口内时） ---
document.addEventListener("keydown", async (e) => {
  // Ctrl+Shift+F9 = start, Ctrl+Shift+F10 = stop (只在窗口聚焦时)
  if (e.ctrlKey && e.shiftKey && e.key === "F9") {
    e.preventDefault();
    if (!isRecording) await startRecording();
  }
  if (e.ctrlKey && e.shiftKey && e.key === "F10") {
    e.preventDefault();
    if (isRecording) await stopRecording();
  }
  if (e.key === "Escape" && isRecording) {
    e.preventDefault();
    await stopRecording();
  }
});

// --- 按钮事件 ---
btnClearHistory.addEventListener("click", () => {
  history.length = 0;
  renderHistory();
});

// --- 核心 ---
async function toggleRecording() {
  if (isRecording) {
    await stopRecording();
  } else {
    await startRecording();
  }
}

async function startRecording() {
  try {
    await invoke("start_recording");
    isRecording = true;
    recordingStartTime = Date.now();
    updateStatus("recording");
    startTimer();
  } catch (e) {
    console.error("启动录音失败:", e);
  }
}

async function stopRecording() {
  isRecording = false;
  stopTimer();
  updateStatus("processing");

  try {
    const text = await invoke("stop_recording");
    updateStatus("done");
    if (text) addToHistory(text);
    setTimeout(() => {
      if (!isRecording) updateStatus("idle");
    }, 2000);
  } catch (e) {
    updateStatus("idle");
    console.error("停止录音失败:", e);
  }
}

// --- UI 更新 ---
function updateStatus(state) {
  statusBadge.classList.remove("idle", "recording", "processing", "done");
  switch (state) {
    case "idle":
      statusBadge.classList.add("idle");
      statusLabel.textContent = "等待快捷键 Ctrl+Shift+R";
      break;
    case "recording":
      statusBadge.classList.add("recording");
      statusLabel.textContent = "录音中... 再次按快捷键停止";
      break;
    case "processing":
      statusBadge.classList.add("processing");
      statusLabel.textContent = "识别中...";
      break;
    case "done":
      statusBadge.classList.add("done");
      statusLabel.textContent = "已粘贴到光标位置";
      break;
  }
}

function startTimer() {
  timerEl.textContent = "00:00";
  timerInterval = setInterval(() => {
    const elapsed = Math.floor((Date.now() - recordingStartTime) / 1000);
    const min = String(Math.floor(elapsed / 60)).padStart(2, "0");
    const sec = String(elapsed % 60).padStart(2, "0");
    timerEl.textContent = `${min}:${sec}`;
  }, 200);
}

function stopTimer() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
}

function addToHistory(text) {
  history.unshift({
    text,
    time: new Date().toLocaleTimeString("zh-CN"),
  });
  if (history.length > 20) history.pop();
  renderHistory();
}

function renderHistory() {
  if (history.length === 0) {
    historyList.innerHTML =
      '<span class="placeholder-text">按 Ctrl+Shift+R 开始录音<br>识别结果将自动粘贴到光标位置</span>';
    return;
  }
  historyList.innerHTML = history
    .map(
      (item, i) =>
        `<div class="history-item">
      <span class="history-time">${item.time}</span>
      <span class="history-text">${escapeHtml(item.text)}</span>
    </div>`
    )
    .join("");
}

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}
