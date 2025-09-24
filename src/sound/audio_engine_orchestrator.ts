const BUFFER_SIZE = 512;
const BUFFER_QUEUE_LENGTH = 2;
const AUDIO_CHANNELS = 2;
const RING_BUFFER_SIZE = BUFFER_SIZE * BUFFER_QUEUE_LENGTH * AUDIO_CHANNELS;

export class AudioEngineOrchestrator {
  private static instance: AudioEngineOrchestrator;

  private refCount = 0;

  private audioCtx!: AudioContext;
  private workletNode!: AudioWorkletNode;
  private rustWorker!: Worker;
  public shared_sound_buffer: SharedArrayBuffer;

  private constructor() {
    const float32ringBufferBytes = RING_BUFFER_SIZE * Float32Array.BYTES_PER_ELEMENT;

    const indexesBytes = Int32Array.BYTES_PER_ELEMENT * 3;

    this.shared_sound_buffer = new SharedArrayBuffer(indexesBytes + float32ringBufferBytes);

    const indexes = new Int32Array(this.shared_sound_buffer, 0, 3);
    const flag = indexes.subarray(0, 1);

    Atomics.store(flag, 0, 1);
  }

  public static getInstance(): AudioEngineOrchestrator {
    if (!this.instance) {
      this.instance = new AudioEngineOrchestrator();
    }
    return this.instance;
  }

  public async init(
    midi_queue_buffer: SharedArrayBuffer,
    osc_queue_buffer: SharedArrayBuffer,
    fx_queue_buffer: SharedArrayBuffer
  ): Promise<Worker | null> {
    try {
      if (this.audioCtx) return null;

      console.log("[SOUND ENGINE] creation of rust worker...");

      this.rustWorker = new Worker(new URL("./rustWorker.ts", import.meta.url), {
        type: "module",
        name: "rustWorker",
      });

      this.rustWorker.onmessage = (e: MessageEvent) => {
        if (e.data.type === "module_end_init") {
          console.log("[SOUND ENGINE] worker module init end, init wasm...");

          this.rustWorker.postMessage({
            type: "init_wasm",
            sharedBuffer: this.shared_sound_buffer,
            ringBufferSize: RING_BUFFER_SIZE,
            midi_queue_buffer: midi_queue_buffer,
            osc_queue_buffer: osc_queue_buffer,
            fx_queue_buffer: fx_queue_buffer,
          });
        }
      };

      this.audioCtx = new AudioContext({ sampleRate: 44100 });

      await this.audioCtx.resume();

      const workletPath = "/audio_worket_manager.js";
      console.log("[AUDIO ENGINE] Loading AudioWorklet module from:", workletPath);

      await this.audioCtx.audioWorklet.addModule(workletPath);
      console.log("[AUDIO ENGINE] AudioWorklet module successfully loaded");

      this.workletNode = new AudioWorkletNode(this.audioCtx, "sound-processor", {
        outputChannelCount: [2],
        processorOptions: {
          bufferSize: BUFFER_SIZE,
          ringBufferSize: RING_BUFFER_SIZE,
          sharedBuffer: this.shared_sound_buffer,
        },
      });

      this.workletNode.connect(this.audioCtx.destination);

      let lastBufferRequestTime: number | null = null;

      this.workletNode.port.onmessage = (event) => {
        if (
          event.data.type === "log" &&
          event.data.message === "[AUDIO WORKLET] buffer requested"
        ) {
          const now = performance.now();

          if (lastBufferRequestTime !== null) {
            const delta = now - lastBufferRequestTime;
            console.log(`Intervalle depuis dernier buffer request: ${delta.toFixed(2)} ms`);
          }

          lastBufferRequestTime = now;
        } else if (event.data.type === "log") {
          console.log(event.data.message);
        }
      };

      return this.rustWorker;
    } catch (error) {
      console.error("[AUDIO ENGINE]", error);
      return null;
    }
  }

  release() {
    this.refCount--;
    if (this.refCount <= 0) {
      console.log("stop to audio context");
      if (this.workletNode) this.workletNode.disconnect();

      this.audioCtx.close();
      this.rustWorker.terminate();
      this.audioCtx = null!;
      this.workletNode = null!;
      this.rustWorker = null!;
      AudioEngineOrchestrator.instance = null!;
    }
  }
}
