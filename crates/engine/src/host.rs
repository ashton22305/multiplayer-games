use protocol::{GameStatus, HostEvent};

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn host_emit(ptr: *const u8, len: u32);
}

#[cfg(target_arch = "wasm32")]
fn emit_json(json: &str) {
    // SAFETY: host_emit reads `len` bytes at `ptr` synchronously and returns
    // before `json` is dropped; the pointer is valid for the full duration of the call.
    unsafe { host_emit(json.as_ptr(), json.len() as u32) }
}

/// Send an event to the host UI. No-op outside the browser.
#[cfg(target_arch = "wasm32")]
pub fn emit(event: &HostEvent) {
    let json = serde_json::to_string(event).expect("HostEvent serialization is infallible");
    emit_json(&json);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn emit(_: &HostEvent) {}

/// Announce that the game (re)started and is now playing.
pub fn emit_playing() {
    emit(&HostEvent::StatusChanged {
        status: GameStatus::Playing,
    });
}

/// Announce that the game ended with the given score.
pub fn emit_game_over(score: u32) {
    emit(&HostEvent::GameOver { score });
    emit(&HostEvent::StatusChanged {
        status: GameStatus::GameOver,
    });
}
