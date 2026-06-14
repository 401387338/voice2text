use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use std::thread;

/// 录音控制指令
pub enum RecorderCommand {
    Start,
    Stop,
    Shutdown,
}

/// 录音器：在独立线程中运行，通过 channel 通信
/// Sender 是 Send + Sync，Receiver 通过 Mutex 保护
pub struct AudioRecorder {
    command_tx: mpsc::Sender<RecorderCommand>,
    /// 接收最终音频数据（录音停止后发送）
    data_rx: std::sync::Mutex<mpsc::Receiver<Vec<i16>>>,
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("未找到默认麦克风设备")?;

        let config = device
            .default_input_config()
            .context("无法获取默认输入配置")?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        println!(
            "[音频] 设备: {}, 采样率: {}Hz, 通道: {}",
            device.name()?,
            sample_rate,
            channels
        );

        let (command_tx, command_rx) = mpsc::channel::<RecorderCommand>();
        let (data_tx, data_rx) = mpsc::channel::<Vec<i16>>();

        // 在独立线程中初始化 cpal 音频流
        thread::spawn(move || {
            let host = cpal::default_host();
            let device = match host.default_input_device() {
                Some(d) => d,
                None => {
                    eprintln!("[音频线程] 未找到默认麦克风");
                    return;
                }
            };

            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[音频线程] 获取输入配置失败: {}", e);
                    return;
                }
            };

            let err_fn = |err| eprintln!("[音频线程] 错误: {}", err);

            // 持续监听控制指令
            loop {
                match command_rx.recv() {
                    Ok(RecorderCommand::Start) => {
                        // 为本次录音创建新的缓冲通道
                        let (chunk_tx, chunk_rx) = mpsc::channel::<Vec<i16>>();
                        let num_channels = config.channels() as usize;

                        let stream_result = match config.sample_format() {
                            cpal::SampleFormat::I16 => {
                                let tx = chunk_tx.clone();
                                device.build_input_stream(
                                    &config.clone().into(),
                                    move |data: &[i16], _: &_| {
                                        let mono = if num_channels == 1 {
                                            data.to_vec()
                                        } else {
                                            // 立体声→单声道：取平均值
                                            data.chunks(num_channels)
                                                .map(|chunk| {
                                                    let sum: i32 = chunk.iter().map(|&s| s as i32).sum();
                                                    (sum / num_channels as i32) as i16
                                                })
                                                .collect()
                                        };
                                        let _ = tx.send(mono);
                                    },
                                    err_fn,
                                    None,
                                )
                            }
                            cpal::SampleFormat::F32 => {
                                let tx = chunk_tx.clone();
                                device.build_input_stream(
                                    &config.clone().into(),
                                    move |data: &[f32], _: &_| {
                                        let mono: Vec<i16> = if num_channels == 1 {
                                            data.iter()
                                                .map(|&s| (s.max(-1.0).min(1.0) * 32767.0) as i16)
                                                .collect()
                                        } else {
                                            data.chunks(num_channels)
                                                .map(|chunk| {
                                                    let sum: f32 = chunk.iter().sum();
                                                    let avg = sum / num_channels as f32;
                                                    (avg.max(-1.0).min(1.0) * 32767.0) as i16
                                                })
                                                .collect()
                                        };
                                        let _ = tx.send(mono);
                                    },
                                    err_fn,
                                    None,
                                )
                            }
                            _ => {
                                eprintln!("[音频线程] 不支持的采样格式");
                                continue;
                            }
                        };

                        match stream_result {
                            Ok(s) => {
                                if let Err(e) = s.play() {
                                    eprintln!("[音频线程] 播放流失败: {}", e);
                                    continue;
                                }
                                println!("[音频线程] 录音开始");

                                // 等待 Stop 指令
                                loop {
                                    match command_rx.recv() {
                                        Ok(RecorderCommand::Stop) => {
                                            drop(s); // 停止并销毁流
                                            drop(chunk_tx); // 关闭发送端

                                            // 收集所有音频数据块
                                            let mut all_data: Vec<i16> = Vec::new();
                                            while let Ok(chunk) = chunk_rx.recv() {
                                                all_data.extend_from_slice(&chunk);
                                            }
                                            println!(
                                                "[音频线程] 录音停止，共 {} 采样",
                                                all_data.len()
                                            );
                                            let _ = data_tx.send(all_data);
                                            break;
                                        }
                                        Ok(RecorderCommand::Start) => {
                                            // 已在录音中，忽略
                                        }
                                        Ok(RecorderCommand::Shutdown) => {
                                            drop(s);
                                            drop(chunk_tx);
                                            return;
                                        }
                                        Err(_) => return,
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("[音频线程] 创建输入流失败: {}", e);
                            }
                        }
                    }
                    Ok(RecorderCommand::Stop) => {
                        eprintln!("[音频线程] 未在录音中，忽略 Stop");
                    }
                    Ok(RecorderCommand::Shutdown) => {
                        println!("[音频线程] 已关闭");
                        break;
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            command_tx,
            data_rx: std::sync::Mutex::new(data_rx),
            sample_rate,
        })
    }

    pub fn start(&self) -> Result<()> {
        // 清空可能残留的旧数据
        let rx = self.data_rx.lock().unwrap();
        while rx.try_recv().is_ok() {}
        drop(rx);
        self.command_tx
            .send(RecorderCommand::Start)
            .map_err(|e| anyhow::anyhow!("发送 Start 指令失败: {}", e))
    }

    pub fn stop(&self) -> Result<Vec<i16>> {
        self.command_tx
            .send(RecorderCommand::Stop)
            .map_err(|e| anyhow::anyhow!("发送 Stop 指令失败: {}", e))?;

        self.data_rx
            .lock()
            .unwrap()
            .recv()
            .map_err(|e| anyhow::anyhow!("接收音频数据失败: {}", e))
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl Drop for AudioRecorder {
    fn drop(&mut self) {
        let _ = self.command_tx.send(RecorderCommand::Shutdown);
    }
}
