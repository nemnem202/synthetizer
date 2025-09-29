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
    const {
      sharedBuffer,
      midi_queue_buffer,
      ringBufferSize,
      osc_queue_buffer,
      fx_queue_buffer,
      sample_event_buffer,
      sample_buffer,
    } = e.data;

    const buffers = [
      sharedBuffer,
      midi_queue_buffer,
      osc_queue_buffer,
      fx_queue_buffer,
      sample_event_buffer,
      sample_buffer,
    ];

    const all_valid =
      typeof ringBufferSize === "number" &&
      buffers.every((buf) => buf instanceof SharedArrayBuffer);

    if (!all_valid) {
      console.log(
        "error - invalid buffers:",
        "audio buffer valid:",
        sharedBuffer instanceof SharedArrayBuffer,
        "midi buffer valid:",
        midi_queue_buffer instanceof SharedArrayBuffer,
        "osc buffer valid:",
        osc_queue_buffer instanceof SharedArrayBuffer,
        "ring buffer size:",
        typeof ringBufferSize === "number",
        "sample_event_buffer",
        sample_event_buffer instanceof SharedArrayBuffer,
        "sample_buffer: ",
        sample_buffer instanceof SharedArrayBuffer
      );

      return;
    }

    const indexes = new Int32Array(sharedBuffer, 0, 3);
    flag = indexes.subarray(0, 1);

    init_audio_thread(
      sharedBuffer,
      ringBufferSize,
      midi_queue_buffer,
      osc_queue_buffer,
      fx_queue_buffer,
      sample_event_buffer,
      sample_buffer
    );

    console.log("[RUST WORKER] initialisation done, processing loop...");
    start_audio_processing_loop();
  }
};
