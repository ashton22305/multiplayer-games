//! Client networking seam for the future authoritative server.
//!
//! The game runs in a same-origin iframe, so `host.js` owns a `WebSocket` to
//! `/ws` and the engine exchanges length-prefixed binary frames with it through
//! a few JS imports. Messages are `postcard`-encoded [`protocol`] types, shared
//! with the (future) Rust server.
//!
//! Nothing connects automatically: a game opts in with [`Connection::connect`].
//! Non-networked games never reference the imports, so they pay no cost.
//!
//! TODO(server): this path is unverified end-to-end. Integration tests (game
//! client <-> axum/tokio server) should be added once the server binary exists.

use protocol::{ClientMsg, ServerMsg};

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn net_connect(ptr: *const u8, len: u32);
    fn net_send(ptr: *const u8, len: u32);
    /// Copies the next queued message into `[ptr, ptr+cap)`, returning its
    /// length, or a negative value if the queue is empty / the message is larger
    /// than `cap`.
    fn net_poll(ptr: *mut u8, cap: u32) -> i32;
}

pub struct Connection {
    connected: bool,
    #[cfg(target_arch = "wasm32")]
    rx: Vec<u8>,
}

impl Default for Connection {
    fn default() -> Self {
        Self::new()
    }
}

impl Connection {
    pub fn new() -> Self {
        Self {
            connected: false,
            #[cfg(target_arch = "wasm32")]
            rx: vec![0u8; 8192],
        }
    }

    /// Open a WebSocket. `url` may be relative (e.g. `"/ws"`); host.js resolves
    /// it against the page origin and upgrades the scheme to ws/wss.
    pub fn connect(url: &str) -> Self {
        let mut c = Self::new();
        c.open(url);
        c
    }

    pub fn open(&mut self, url: &str) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            net_connect(url.as_ptr(), url.len() as u32);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = url;
        self.connected = true;
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn send(&self, msg: &ClientMsg) {
        let Ok(bytes) = postcard::to_allocvec(msg) else {
            return;
        };
        #[cfg(target_arch = "wasm32")]
        unsafe {
            net_send(bytes.as_ptr(), bytes.len() as u32);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = bytes;
    }

    /// Drain all messages received since the last call.
    pub fn poll(&mut self) -> Vec<ServerMsg> {
        #[cfg(target_arch = "wasm32")]
        {
            let mut out = Vec::new();
            loop {
                let n = unsafe { net_poll(self.rx.as_mut_ptr(), self.rx.len() as u32) };
                if n < 0 {
                    break;
                }
                if let Ok(msg) = postcard::from_bytes(&self.rx[..n as usize]) {
                    out.push(msg);
                }
            }
            return out;
        }
        #[cfg(not(target_arch = "wasm32"))]
        Vec::new()
    }
}
