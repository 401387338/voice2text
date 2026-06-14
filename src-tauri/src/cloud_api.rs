use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 豆包 ASR 云端 API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudApiConfig {
    /// API 密钥
    pub api_key: String,
    /// 资源 ID
    pub resource_id: String,
    /// 是否启用云端识别
    pub enabled: bool,
}

impl Default for CloudApiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            resource_id: String::new(),
            enabled: false,
        }
    }
}

/// 豆包 ASR WebSocket 流式识别结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRecognitionResult {
    /// 识别的文本片段
    pub text: String,
    /// 是否是最终结果
    pub is_final: bool,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
}

/// 豆包云 API 客户端
pub struct CloudApiClient {
    config: CloudApiConfig,
}

impl CloudApiClient {
    /// 创建新的云端 API 客户端
    pub fn new(config: CloudApiConfig) -> Self {
        Self { config }
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.api_key.is_empty()
    }

    /// 流式发送音频数据到豆包 ASR
    /// 返回识别结果流（异步迭代器）
    ///
    /// 豆包 ASR 2.0 WebSocket 端点:
    /// wss://openspeech.bytedance.com/api/v3/sauc/bigmodel
    ///
    /// 协议:
    /// 1. 发送 StartConnection 消息（含 api_key, resource_id）
    /// 2. 流式发送音频帧 (PCM 16kHz 16bit 单声道)
    /// 3. 接收识别结果（partial/final）
    /// 4. 发送 FinishConnection 结束
    pub async fn transcribe_streaming(
        &self,
        audio_data: &[i16],
        sample_rate: u32,
    ) -> Result<Vec<CloudRecognitionResult>> {
        if !self.is_enabled() {
            anyhow::bail!("云端 API 未启用，请配置 api_key");
        }

        println!(
            "[云API] 发送音频数据到豆包 ASR，长度: {} 采样，采样率: {}Hz",
            audio_data.len(),
            sample_rate
        );

        // ==========================================
        // 豆包 ASR WebSocket 集成占位
        //
        // 正式版本需:
        // 1. 使用 tokio-tungstenite 建立 WebSocket 连接
        // 2. 发送 StartConnection JSON 消息
        // 3. 将 audio_data 转为 PCM 16kHz 16bit，分帧发送
        // 4. 接收并解析识别结果 JSON 消息
        // 5. 发送 FinishConnection 结束
        //
        // 参考文档: https://www.volcengine.com/docs/6561/1354868
        //
        // 示例代码框架:
        //
        // use tokio_tungstenite::{connect_async, tungstenite::Message};
        // use futures_util::SinkExt;
        //
        // let ws_url = "wss://openspeech.bytedance.com/api/v3/sauc/bigmodel";
        // let (mut ws, _) = connect_async(ws_url).await?;
        //
        // // 发送开始连接
        // ws.send(Message::Text(json!({
        //     "type": "StartConnection",
        //     "api_key": self.config.api_key,
        //     "resource_id": self.config.resource_id,
        //     "format": "pcm",
        //     "rate": 16000,
        //     "bits": 16,
        //     "channel": 1,
        //     "language": "zh-CN",
        // }).to_string())).await?;
        //
        // // 分帧发送音频...
        //
        // // 接收结果...
        // ==========================================

        let dummy_result = CloudRecognitionResult {
            text: format!(
                "[云端识别] 豆包 API 集成开发中 — 收到 {} 采样音频",
                audio_data.len()
            ),
            is_final: true,
            confidence: 0.0,
        };

        Ok(vec![dummy_result])
    }

    /// 更新配置
    pub fn update_config(&mut self, config: CloudApiConfig) {
        self.config = config;
    }
}
