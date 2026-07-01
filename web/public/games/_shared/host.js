"use strict";
// Generic iframe host for macroquad games. Loaded after mq_js_bundle.js.
// The game name comes from the ?game= query parameter written by GameFrame.vue.
// The wasm binary is resolved relative to this script's directory:
//   _shared/host.js  →  ../<game>/<game>.wasm
(function () {
  const params = new URLSearchParams(location.search);
  const gameName = params.get("game") || "unknown";
  const wasmPath = `../${gameName}/${gameName}.wasm`;
  const decoder = new TextDecoder("utf-8");

  // Resolve the allowed parent origin from the query string; fall back to same-origin.
  const parentOrigin = params.get("parent") || window.location.origin;

  function readBytes(ptr, len) {
    return new Uint8Array(wasm_memory.buffer, ptr, len);
  }

  miniquad_add_plugin({
    register_plugin: function (importObject) {
      // Game -> parent UI.
      importObject.env.host_emit = function (ptr, len) {
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
    },
  });

  load(wasmPath);
})();
