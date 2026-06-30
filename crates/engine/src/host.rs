//! Bridge from a running game (inside the wasm/iframe) up to the Vue host.
//!
//! [`emit`] serializes a [`protocol::HostEvent`] to JSON and hands the bytes to
//! `host_emit`, a JS import provided by the iframe's `host.js`, which reads them
//! from wasm memory and forwards them to the parent window via `postMessage`.

use protocol::HostEvent;

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn host_emit(ptr: *const u8, len: u32);
}

/// Send an event to the host UI. No-op outside the browser (e.g. native tests).
pub fn emit(event: &HostEvent) {
    let Ok(json) = serde_json::to_string(event) else {
        return;
    };

    #[cfg(target_arch = "wasm32")]
    // SAFETY: `host_emit` reads `len` bytes at `ptr` synchronously and returns
    // before `json` is dropped; the pointer stays valid for the whole call.
    unsafe {
        host_emit(json.as_ptr(), json.len() as u32);
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = json;
}
