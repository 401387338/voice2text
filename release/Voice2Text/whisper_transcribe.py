#!/usr/bin/env python3
"""语音识别 — faster-whisper, 优先 GPU 自动回退 CPU"""
import sys, os

# ---- 必须在导入 faster_whisper 之前设置 CUDA 12 DLL 路径 ----
if sys.platform == "win32":
    import site
    _sp = site.getsitepackages()[0] if site.getsitepackages() else ""
    for _d in [
        os.path.join(_sp, "nvidia", "cublas", "bin"),
        os.path.join(_sp, "ctranslate2"),
    ]:
        if os.path.isdir(_d):
            os.environ["PATH"] = _d + ";" + os.environ.get("PATH", "")
            try: os.add_dll_directory(_d)
            except: pass

import wave, time
import numpy as np

def transcribe(wav_path, language="zh", model_size="medium"):
    from faster_whisper import WhisperModel

    audio = None
    with wave.open(wav_path, 'rb') as w:
        frames = w.readframes(w.getnframes())
        audio = np.frombuffer(frames, dtype=np.int16).astype(np.float32) / 32768.0

    last_err = None
    for device, compute in [("cuda", "float16"), ("cpu", "auto")]:
        try:
            print(f"[Whisper] try {device}/{compute}")
            model = WhisperModel(model_size, device=device, compute_type=compute)
            segments, info = model.transcribe(audio, language=language, beam_size=5)
            texts = [s.text.strip() for s in segments if s.text.strip()]
            result = "".join(texts)
            elapsed = info.duration  # approximate
            print(f"[Whisper] {device}/{compute} ok, lang={info.language} p={info.language_probability:.2f}")
            return result
        except Exception as e:
            last_err = str(e)
            print(f"[Whisper] {device}/{compute} fail: {last_err[:80]}")
            if device == "cpu":
                raise
    raise RuntimeError(last_err or "unknown")

if __name__ == "__main__":
    wav = sys.argv[1]
    out_file = sys.argv[2]
    lang = sys.argv[3] if len(sys.argv) > 3 else "zh"
    model = sys.argv[4] if len(sys.argv) > 4 else "medium"

    try:
        text = transcribe(wav, lang, model)
        with open(out_file, "w", encoding="utf-8") as f:
            f.write(text)
    except Exception as e:
        with open(out_file, "w", encoding="utf-8") as f:
            f.write(f"ERROR:{e}")
        sys.exit(1)
