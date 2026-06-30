use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameStatus {
    Loading,
    Playing,
    Paused,
    GameOver,
}

/// Events emitted from a running game up to the Vue host via `postMessage`.
/// The JSON shape is `{ "type": "...", ...fields }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HostEvent {
    Ready,
    ScoreChanged { score: u32 },
    GameOver { score: u32 },
    StatusChanged { status: GameStatus },
    PlayersOnline { count: u32 },
}

/// Placeholder client -> server message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    Ping,
}

/// Placeholder server -> client message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    Pong,
}
