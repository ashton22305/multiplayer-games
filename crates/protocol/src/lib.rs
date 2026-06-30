//! Types shared between the engine, games, the Vue host, and a future Rust
//! game server. Kept dependency-light so it can be reused on the server.
//!
//! The TypeScript mirror lives at `web/src/types/protocol.ts`. Serialization
//! round-trip tests below verify the exact JSON shapes; if you change the serde
//! attributes here, update the TS file and the tests together.

use serde::{Deserialize, Serialize};

/// Live game lifecycle state, surfaced to the host UI (e.g. the stats sidebar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameStatus {
    Loading,
    Playing,
    Paused,
    GameOver,
}

/// Events emitted from a running game (inside the wasm/iframe) up to the Vue
/// host via `postMessage`. The JSON shape is `{ "type": "...", ...fields }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HostEvent {
    Ready,
    ScoreChanged { score: u32 },
    GameOver { score: u32 },
    StatusChanged { status: GameStatus },
    PlayersOnline { count: u32 },
}

/// Placeholder client -> server message. Expanded once the server exists.
// TODO(server): extend when the authoritative WebSocket server is built.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    Ping,
}

/// Placeholder server -> client message. Expanded once the server exists.
// TODO(server): extend when the authoritative WebSocket server is built.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn ser<T: Serialize>(v: &T) -> Value {
        serde_json::to_value(v).unwrap()
    }

    fn round_trip<T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug>(v: T) {
        let json = serde_json::to_string(&v).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    // GameStatus: serializes to camelCase strings matching the TS union.
    #[test]
    fn game_status_serializes_to_camel_case() {
        assert_eq!(ser(&GameStatus::Loading), json!("loading"));
        assert_eq!(ser(&GameStatus::Playing), json!("playing"));
        assert_eq!(ser(&GameStatus::Paused), json!("paused"));
        assert_eq!(ser(&GameStatus::GameOver), json!("gameOver"));
    }

    #[test]
    fn game_status_round_trips() {
        for v in [
            GameStatus::Loading,
            GameStatus::Playing,
            GameStatus::Paused,
            GameStatus::GameOver,
        ] {
            round_trip(v);
        }
    }

    // HostEvent: tagged with "type" field, variant names in camelCase.
    #[test]
    fn host_event_ready() {
        assert_eq!(ser(&HostEvent::Ready), json!({ "type": "ready" }));
    }

    #[test]
    fn host_event_score_changed() {
        assert_eq!(
            ser(&HostEvent::ScoreChanged { score: 42 }),
            json!({ "type": "scoreChanged", "score": 42 }),
        );
    }

    #[test]
    fn host_event_game_over() {
        assert_eq!(
            ser(&HostEvent::GameOver { score: 7 }),
            json!({ "type": "gameOver", "score": 7 }),
        );
    }

    #[test]
    fn host_event_status_changed() {
        assert_eq!(
            ser(&HostEvent::StatusChanged {
                status: GameStatus::Playing
            }),
            json!({ "type": "statusChanged", "status": "playing" }),
        );
    }

    #[test]
    fn host_event_players_online() {
        assert_eq!(
            ser(&HostEvent::PlayersOnline { count: 3 }),
            json!({ "type": "playersOnline", "count": 3 }),
        );
    }

    #[test]
    fn host_event_round_trips() {
        for v in [
            HostEvent::Ready,
            HostEvent::ScoreChanged { score: 0 },
            HostEvent::GameOver { score: 100 },
            HostEvent::StatusChanged {
                status: GameStatus::Paused,
            },
            HostEvent::PlayersOnline { count: 1 },
        ] {
            round_trip(v);
        }
    }

    // ClientMsg / ServerMsg: postcard round-trips (used by net.rs).
    #[test]
    fn client_msg_postcard_round_trip() {
        let msg = ClientMsg::Ping;
        let bytes = postcard::to_allocvec(&msg).unwrap();
        let back: ClientMsg = postcard::from_bytes(&bytes).unwrap();
        assert!(matches!(back, ClientMsg::Ping));
    }

    #[test]
    fn server_msg_postcard_round_trip() {
        let msg = ServerMsg::Pong;
        let bytes = postcard::to_allocvec(&msg).unwrap();
        let back: ServerMsg = postcard::from_bytes(&bytes).unwrap();
        assert!(matches!(back, ServerMsg::Pong));
    }
}
