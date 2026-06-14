#!/usr/bin/env python3
"""持久化 whisper 服务 — 模型常驻内存，stdin/stdout 通信
协议: 每行一个 JSON 请求 {"wav": "路径", "lang": "zh", "model": "medium"}
      响应: JSON {"text": "识别结果"} 或 {"error": "错误"}
"""

import sys, os, json, wave, time
import numpy as np

# ---- CUDA DLL 路径 ----
if sys.platform == "win32":
    import site
    _site_packages = site.getsitepackages()[0] if site.getsitepackages() else ""
    for _d in [
        os.path.join(_site_packages, "nvidia", "cublas", "bin"),
        os.path.join(_site_packages, "ctranslate2"),
    ]:
        if os.path.isdir(_d):
            os.environ["PATH"] = _d + ";" + os.environ.get("PATH", "")
            try: os.add_dll_directory(_d)
            except: pass

# ---- 全局模型缓存 ----
_model = None
_model_size = None
_device = None

def get_model(model_size="medium"):
    global _model, _model_size, _device
    if _model is not None and _model_size == model_size:
        return _model

    from faster_whisper import WhisperModel
    for device, compute in [("cuda", "float16"), ("cpu", "auto")]:
        try:
            t0 = time.time()
            _model = WhisperModel(model_size, device=device, compute_type=compute)
            _model_size = model_size
            _device = f"{device}/{compute}"
            print(f"[server] model loaded: {model_size} on {_device} ({time.time()-t0:.1f}s)", file=sys.stderr, flush=True)
            return _model
        except Exception as e:
            print(f"[server] {device} fail: {e}", file=sys.stderr, flush=True)
            if device == "cpu":
                raise
    raise RuntimeError("No device available")

def transcribe(req):
    wav_path = req["wav"]
    lang = req.get("lang", "zh")
    model_size = req.get("model", "medium")

    model = get_model(model_size)

    with wave.open(wav_path, 'rb') as w:
        frames = w.readframes(w.getnframes())
        audio = np.frombuffer(frames, dtype=np.int16).astype(np.float32) / 32768.0

    t0 = time.time()
    segments, info = model.transcribe(audio, language=lang, beam_size=5)
    texts = [s.text.strip() for s in segments if s.text.strip()]
    elapsed = time.time() - t0

    result = "".join(texts)
    print(f"[server] audio={info.duration:.1f}s infer={elapsed:.1f}s ({_device})",
          file=sys.stderr, flush=True)
    return result

# ---- 主循环 ----
if __name__ == "__main__":
    print("[server] ready", file=sys.stderr, flush=True)
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
            # 预热请求：只加载模型不推理
            if req.get("wav") == "WARMUP":
                get_model(req.get("model", "medium"))
                resp = json.dumps({"text": ""})
            else:
                text = transcribe(req)
                resp = json.dumps({"text": text}, ensure_ascii=False)
        except Exception as e:
            resp = json.dumps({"error": str(e)}, ensure_ascii=False)
        print(resp, flush=True)
