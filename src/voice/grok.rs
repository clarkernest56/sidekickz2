use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn, error};

pub struct GrokVoiceAgent {
    api_key: String,
}

impl GrokVoiceAgent {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("GROK_API_KEY").context("GROK_API_KEY environment variable is required")?;
        Ok(Self { api_key })
    }

    /// Connects to the Grok Voice API S2S WebSocket and handles the real-time interaction.
    /// Uses persona "Eve" (female, energetic).
    pub async fn connect_and_start_session(&self) -> Result<()> {
        let url = "wss://api.x.ai/v1/realtime?model=grok-2-realtime-preview";
        
        let target = format!("{}&api_key={}", url, self.api_key);
        let (mut ws_stream, _) = connect_async(&target).await.context("Failed to connect to Grok Voice API WebSocket")?;
        
        info!("🔌 Connected to Grok Voice S2S API");

        // Send initial setup/session configuration
        let setup_msg = json!({
            "type": "session.update",
            "session": {
                "persona": "Eve",
                "traits": ["female", "energetic", "helpful", "witty"],
                "language": "en-US"
            }
        });
        
        ws_stream.send(Message::Text(setup_msg.to_string().into())).await?;
        info!("🎤 Session updated with persona 'Eve'");

        // Real-time loop (receives audio/text frames from Grok)
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Handle JSON events from Grok
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(text.to_string().as_str()) {
                        let event_type = event["type"].as_str().unwrap_or("unknown");
                        match event_type {
                            "response.audio.delta" => {
                                // In a real integration, we'd stream this base64 audio back via Telegram Voice
                                // or play it locally.
                                let _base64_audio = event["delta"].as_str().unwrap_or_default();
                            }
                            "response.text.delta" => {
                                let text_delta = event["delta"].as_str().unwrap_or_default();
                                info!("Grok Eve says: {}", text_delta);
                            }
                            _ => {
                                // Ignore other events for now
                            }
                        }
                    }
                }
                Ok(Message::Binary(_)) => {
                    // Binary audio frames sent directly
                }
                Ok(Message::Close(_)) => {
                    info!("Grok connection closed.");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
