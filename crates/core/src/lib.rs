//! Quaero core: pure shared types and logic.
//!
//! This crate is intentionally free of any Tauri dependency (ADR-0011): it
//! holds the typed IPC contract and pure domain-free logic so it can be tested
//! in isolation. Tauri depends on `core`, never the other way around.

pub mod chat;
pub mod citation_candidates;
pub mod domain;
pub mod evidence;
pub mod export;
pub mod hash;
pub mod persistence;
pub mod privacy;
pub mod verify;

use serde::{Deserialize, Serialize};

/// Request payload for the `ping` round-trip command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingRequest {
    pub message: String,
}

/// Response payload for the `ping` round-trip command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingResponse {
    pub reply: String,
}

/// Pure handler for the `ping` round-trip: echoes the message back as a pong.
pub fn ping(request: PingRequest) -> PingResponse {
    PingResponse {
        reply: format!("pong: {}", request.message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_returns_pong_echoing_the_message() {
        let response = ping(PingRequest {
            message: "ciao".into(),
        });
        assert_eq!(response.reply, "pong: ciao");
    }

    #[test]
    fn ping_payloads_survive_json_round_trip() {
        let request = PingRequest {
            message: "ciao".into(),
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: PingRequest = serde_json::from_str(&encoded).unwrap();
        assert_eq!(request, decoded);

        let response = PingResponse {
            reply: "pong: ciao".into(),
        };
        let encoded = serde_json::to_string(&response).unwrap();
        // The wire shape the frontend depends on.
        assert_eq!(encoded, r#"{"reply":"pong: ciao"}"#);
        let decoded: PingResponse = serde_json::from_str(&encoded).unwrap();
        assert_eq!(response, decoded);
    }
}
