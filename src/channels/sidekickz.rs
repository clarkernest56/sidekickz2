use axum::{extract::State, response::IntoResponse, Json};
use reqwest::Client;
use serde_json::json;
use crate::gateway::AppState;
use tracing::{error, info};
use std::env;

pub async fn handle_signalwire_webhook(
    State(_state): State<AppState>,
    Json(_payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("Incoming call from SignalWire");

    // Retrieve Daily API Key from environment
    let daily_api_key = match env::var("DAILY_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            error!("DAILY_API_KEY environment variable is missing");
            let error_swml = json!({
                "version": "1.0.0",
                "sections": {
                    "main": [
                        {
                            "play": {
                                "urls": ["say:An internal error occurred. Daily API key is missing."]
                            }
                        }
                    ]
                }
            });
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(error_swml));
        }
    };

    let client = Client::new();
    let room_res = client
        .post("https://api.daily.co/v1/rooms")
        .header("Authorization", format!("Bearer {}", daily_api_key))
        .json(&json!({
            "properties": {
                "sip": {
                    "display_name": "Sidekickz Caller"
                }
            }
        }))
        .send()
        .await;

    let room_data = match room_res {
        Ok(res) if res.status().is_success() => res.json::<serde_json::Value>().await.unwrap_or(json!({})),
        Ok(res) => {
            error!("Failed to create Daily room. Status: {}", res.status());
            return fallback_swml();
        }
        Err(e) => {
            error!("HTTP request to Daily.co failed: {}", e);
            return fallback_swml();
        }
    };

    // Extract the SIP URI from Daily room properties
    let sip_uri = room_data.get("config")
        .and_then(|c| c.get("sip_endpoint"))
        .and_then(|se| se.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            // Alternatively Daily exposes sip endpoints under `properties.sip.endpoint` or similar depending on exactly what version
            // For now, if we can't find it, we will default to a placeholder or fail safely. 
            // According to daily docs, room creation returns sip info inside `config`.
            room_data.get("sip_endpoint").and_then(|s| s.as_str()).map(|s| s.to_string())
        });

    if let Some(uri) = sip_uri {
        info!("Successfully created Daily room with SIP URI: {}", uri);
        let swml = json!({
            "version": "1.0.0",
            "sections": {
                "main": [
                    {
                        "connect": {
                            "to": {
                                "type": "sip",
                                "endpoint": uri
                            }
                        }
                    }
                ]
            }
        });
        (axum::http::StatusCode::OK, Json(swml))
    } else {
        error!("Could not extract SIP URI from Daily room response");
        fallback_swml()
    }
}

fn fallback_swml() -> (axum::http::StatusCode, Json<serde_json::Value>) {
    let error_swml = json!({
        "version": "1.0.0",
        "sections": {
            "main": [
                {
                    "play": {
                        "urls": ["say:Sorry, we could not connect your call at this time."]
                    }
                }
            ]
        }
    });
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(error_swml))
}
