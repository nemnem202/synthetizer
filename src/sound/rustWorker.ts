// wasmWorker.ts
import initRustSynth, {
  init_audio_thread,
  start_audio_processing_loop,
} from "./rust-synth/build/rust_synth.js";

let wasmReady = false;
let flag: Int32Array;

const initModule = async () => {
  await initRustSynth();
  console.log("[RUST WORKER] Rust WASM ready in Worker!");
  self.postMessage({ type: "module_end_init" });
  wasmReady = true;
};

initModule();

self.onmessage = (e: MessageEvent) => {
  if (e.data.type === "init_wasm") {
    const sharedBuffer = e.data.sharedBuffer;
    const midi_queue_buffer = e.data.midi_queue_buffer;
    const ringBufferSize = e.data.ringBufferSize;
    const osc_queue_buffer = e.data.osc_queue_buffer;

    if (
      !(sharedBuffer instanceof SharedArrayBuffer) ||
      typeof ringBufferSize !== "number" ||
      !(midi_queue_buffer instanceof SharedArrayBuffer) ||
      !(osc_queue_buffer instanceof SharedArrayBuffer)
    ) {
      console.log(
        "error - invalid buffers:",
        "audio buffer valid:",
        sharedBuffer instanceof SharedArrayBuffer,
        "midi buffer valid:",
        midi_queue_buffer instanceof SharedArrayBuffer,
        "osc buffer valid:",
        osc_queue_buffer instanceof SharedArrayBuffer,
        "ring buffer size:",
        ringBufferSize
      );
      return;
    }

    const indexes = new Int32Array(sharedBuffer, 0, 3);
    flag = indexes.subarray(0, 1);

    init_audio_thread(sharedBuffer, ringBufferSize, midi_queue_buffer, osc_queue_buffer);

    console.log("[RUST WORKER] initialisation done, processing loop...");
    start_audio_processing_loop();
  }
};
