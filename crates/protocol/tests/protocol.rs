use protocol::*;
use serde_json::{json, Value};

fn ser<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}

fn round_trip<
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug,
>(
    v: T,
) {
    let json = serde_json::to_string(&v).unwrap();
    let back: T = serde_json::from_str(&json).unwrap();
    assert_eq!(v, back);
}

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
