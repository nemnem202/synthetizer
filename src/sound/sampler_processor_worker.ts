import init, { generate_c0_table } from "./rust-sample-processor/build/rust_sample_processor.js";
import type { InitInput } from "./rust-synth/build/rust_synth.js";

let wasm_ready: Promise<InitInput> | null = null;

// fonction d'initialisation unique
async function init_wasm() {
  if (!wasm_ready) {
    wasm_ready = init(); // init() retourne une Promise
  }
  await wasm_ready;
}

self.onmessage = async (e: MessageEvent) => {
  await init_wasm(); // s'assurer que le module Rust est prêt

  const samples = e.data.samples as Float32Array;

  // appel de la fonction Rust
  const output: Float32Array = generate_c0_table(samples);

  // renvoyer le résultat
  self.postMessage({ output });
};
