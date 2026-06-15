use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Mutex};

/// 持久化 Whisper 引擎（Python 进程常驻，模型只加载一次）
pub struct WhisperEngine {
    request_tx: mpsc::Sender<String>,
    response_rx: Mutex<mpsc::Receiver<String>>,
    _child: Mutex<Child>,
}

impl WhisperEngine {
    pub fn new(_model_path: &str) -> Result<Self> {
        let script = find_script("whisper_server.py");
        println!("[Whisper] 启动持久服务: {}", script);

        // 启动 Python 进程
        let mut child = Command::new("python")
            .env("PYTHONUTF8", "1")
            .env("PYTHONIOENCODING", "utf-8")
            .env("HF_ENDPOINT", "https://hf-mirror.com")
            .arg(&script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("启动 whisper_server.py 失败")?;

        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");

        let (request_tx, request_rx) = mpsc::channel::<String>();
        let (response_tx, response_rx) = mpsc::channel::<String>();

        // 通信线程
        std::thread::spawn(move || {
            let mut stdin = stdin;
            let mut reader = BufReader::new(stdout);

            loop {
                match request_rx.recv() {
                    Ok(req) => {
                        if writeln!(stdin, "{}", req).is_err() {
                            break;
                        }
                        if stdin.flush().is_err() {
                            break;
                        }
                        let mut resp = String::new();
                        if reader.read_line(&mut resp).is_err() {
                            break;
                        }
                        let _ = response_tx.send(resp);
                    }
                    Err(_) => break,
                }
            }
        });

        // 发送预热请求让模型加载
        let warmup = serde_json::json!({"wav": "WARMUP"});
        let _ = request_tx.send(warmup.to_string());
        // 等待预热完成
        if let Ok(resp) = response_rx.recv() {
            println!("[Whisper] 预热完成");
        }

        println!("[Whisper] 持久服务已启动");

        Ok(Self {
            request_tx,
            response_rx: Mutex::new(response_rx),
            _child: Mutex::new(child),
        })
    }

    pub fn transcribe(&mut self, audio_samples: &[i16], sample_rate: u32) -> Result<String> {
        if audio_samples.is_empty() {
            return Ok(String::new());
        }

        // 1. 重采样 + 保存 WAV
        let samples_16k = if sample_rate != 16000 {
            resample_i16(audio_samples, sample_rate, 16000)
        } else {
            audio_samples.to_vec()
        };

        let duration = samples_16k.len() as f64 / 16000.0;
        if duration < 0.3 {
            return Ok("(录音时间太短)".to_string());
        }

        println!(
            "[Whisper] 识别中: {} 采样, {:.1} 秒",
            samples_16k.len(),
            duration
        );

        let temp_dir = std::env::temp_dir();
        let wav_path = temp_dir.join("voice2text_temp.wav");
        save_wav(&wav_path, &samples_16k, 16000)?;

        // 2. 发送请求到持久进程
        let t0 = std::time::Instant::now();
        let req = serde_json::json!({
            "wav": wav_path.to_string_lossy(),
            "lang": "zh",
        });
        let req_str = req.to_string();

        self.request_tx
            .send(req_str)
            .map_err(|e| anyhow::anyhow!("whisper 进程已退出: {}", e))?;

        let resp = self
            .response_rx
            .lock()
            .unwrap()
            .recv()
            .map_err(|e| anyhow::anyhow!("whisper 无响应: {}", e))?;

        let elapsed = t0.elapsed();
        println!("[Whisper] 总耗时: {:.1}s", elapsed.as_secs_f64());

        // 清理
        let _ = std::fs::remove_file(&wav_path);

        // 解析响应
        let resp: serde_json::Value =
            serde_json::from_str(&resp).unwrap_or(serde_json::json!({"error": "invalid json"}));

        if let Some(err) = resp["error"].as_str() {
            return Err(anyhow::anyhow!("识别错误: {}", err));
        }

        let raw_text = resp["text"].as_str().unwrap_or("").to_string();
        if raw_text.is_empty() {
            return Ok("(未识别到语音)".to_string());
        }

        Ok(clean_output(&raw_text))
    }
}

fn find_script(name: &str) -> String {
    for p in &[
        name.to_string(),
        format!("src-tauri/{}", name),
        // Release 模式下脚本在 exe 同目录
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|d| d.join(name)))
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_default(),
    ] {
        if Path::new(&p).exists() {
            return p.to_string();
        }
    }
    name.to_string()
}

fn save_wav(path: &Path, samples: &[i16], sample_rate: u32) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for &s in samples {
        writer.write_sample(s)?;
    }
    writer.finalize()?;
    Ok(())
}

fn clean_output(text: &str) -> String {
    text.replace('(', "")
        .replace(')', "")
        .replace('（', "")
        .replace('）', "")
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("")
}

fn resample_i16(samples: &[i16], from_rate: u32, to_rate: u32) -> Vec<i16> {
    if from_rate == to_rate || samples.is_empty() {
        return samples.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio).round() as usize;
    let mut out = Vec::with_capacity(new_len);
    for i in 0..new_len {
        let start = (i as f64 * ratio).round() as usize;
        let end = ((i + 1) as f64 * ratio).round() as usize;
        let end = end.min(samples.len());
        if end <= start {
            if start < samples.len() {
                out.push(samples[start]);
            }
        } else {
            let sum: i64 = samples[start..end].iter().map(|&s| s as i64).sum();
            out.push((sum / (end - start) as i64) as i16);
        }
    }
    out
}
