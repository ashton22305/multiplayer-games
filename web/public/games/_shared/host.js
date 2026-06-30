// Generic iframe host for macroquad games. Loaded after mq_js_bundle.js (which
// defines the global `load()` and `wasm_memory`). The game wasm name comes from
// this script's `data-wasm` attribute (written by web/scripts/build-games.mjs).
//
// Provides two JS-side bridges the engine calls into:
//   - host_emit: push a HostEvent up to the parent window (postMessage).
//   - net_*: own a WebSocket to the (future) server and exchange binary frames.
// Both decode/encode bytes directly from wasm linear memory.
//
// Cross-origin: when games are served from a separate origin, GameFrame.vue
// passes ?parent=<encodedOrigin> so we know the exact target for postMessage
// rather than relying on same-origin assumption.
"use strict";
(function () {
  const script = document.currentScript;
  const wasmName = (script && script.dataset.wasm) || "game.wasm";
  const decoder = new TextDecoder("utf-8");

  // Resolve the allowed parent origin from the query string; fall back to '*'
  // only when running standalone (no parent param), not when cross-origin.
  const params = new URLSearchParams(location.search);
  const parentOrigin = params.get("parent") || (window.parent !== window ? window.location.origin : "*");

  function readBytes(ptr, len) {
    return new Uint8Array(wasm_memory.buffer, ptr, len);
  }

  let socket = null;
  const inbox = []; // received Uint8Array messages, FIFO

  miniquad_add_plugin({
    register_plugin: function (importObject) {
      const env = importObject.env;

      // Game -> parent UI.
      env.host_emit = function (ptr, len) {
        const json = decoder.decode(readBytes(ptr, len));
        try {
          window.parent.postMessage(
            { source: "mq-game", event: JSON.parse(json) },
            parentOrigin,
          );
        } catch (e) {
          console.error("host_emit: bad event JSON", e);
        }
      };

      // Game <-> server WebSocket.
      env.net_connect = function (ptr, len) {
        const raw = decoder.decode(readBytes(ptr, len));
        const url = new URL(raw, window.location.href).href.replace(/^http/, "ws");
        try {
          socket = new WebSocket(url);
          socket.binaryType = "arraybuffer";
          socket.onmessage = (e) => {
            if (e.data instanceof ArrayBuffer) inbox.push(new Uint8Array(e.data));
          };
          socket.onclose = () => {
            socket = null;
          };
          socket.onerror = () => {};
        } catch (e) {
          console.error("net_connect", e);
        }
      };

      env.net_send = function (ptr, len) {
        if (socket && socket.readyState === WebSocket.OPEN) {
          // Copy out of wasm memory; the buffer may be reused after this call.
          socket.send(readBytes(ptr, len).slice());
        }
      };

      // Copy the next queued message into wasm memory; return its length, or -1.
      env.net_poll = function (ptr, cap) {
        if (inbox.length === 0) return -1;
        const msg = inbox[0];
        if (msg.length > cap) {
          inbox.shift();
          return -1;
        }
        new Uint8Array(wasm_memory.buffer, ptr, msg.length).set(msg);
        inbox.shift();
        return msg.length;
      };
    },
  });

  load(wasmName);
})();
