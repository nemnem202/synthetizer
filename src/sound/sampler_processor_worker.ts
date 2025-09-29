import init, { generate_c0_table } from "./rust-sample-processor/build/rust_sample_processor.js";
import type { InitInput } from "./rust-synth/build/rust_synth.js";

let sample_buffer: SharedArrayBuffer;
let sample_array: Float32Array;
let wasm_ready: Promise<InitInput> | null = null;
// fonction d'initialisation unique
async function init_wasm() {
  if (!wasm_ready) {
    wasm_ready = init();
  }
  await wasm_ready;
}

self.onmessage = async (e: MessageEvent) => {
  if (!wasm_ready && e.data.type != "init") {
    self.postMessage({ type: "log", message: "[SAMPLER PROCESSOR] wasm not init !" });
    return;
  } else if (e.data.type === "init" && !wasm_ready) {
    sample_buffer = e.data.sample_buffer;
    if (!(sample_buffer instanceof SharedArrayBuffer)) {
      self.postMessage({
        type: "log",
        message: "[SAMPLER PROCESSOR] : an error occured, invalid sharredArrayByffer provided!",
      });
      return;
    }
    sample_array = new Float32Array(sample_buffer);
    await init_wasm();
    self.postMessage({
      type: "log",
      message: "[SAMPLER PROCESSOR] : wasm initialized sucessfully !",
    });
  } else {
    const samples = e.data.samples as Float32Array;
    const rate = e.data.sampleRate as number;

    const output: Float32Array = generate_c0_table(samples, rate);
    sample_array.set(output);

    e.data.event.length = output.length;

    self.postMessage({ type: "sampler update", event: e.data.event });
  }
};
