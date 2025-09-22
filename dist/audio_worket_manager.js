class AudioWorkletManager extends AudioWorkletProcessor {
  constructor(options) {
    super();

    this.bufferSize = options.processorOptions.bufferSize;
    this.ringBufferSize = options.processorOptions.ringBufferSize;

    const sharedBuffer = options.processorOptions.sharedBuffer;

    const indexes = new Int32Array(sharedBuffer, 0, 3);
    this.flag = indexes.subarray(0, 1);
    this.readIndex = indexes.subarray(1, 2);
    this.writeIndex = indexes.subarray(2, 3);

    const indexesBytes = Int32Array.BYTES_PER_ELEMENT * 3;
    this.ringBuffer = new Float32Array(sharedBuffer, indexesBytes, this.ringBufferSize);
  }

  process(inputs, outputs, parameters) {
    const output = outputs[0];
    const left = output[0];
    const right = output[1];

    let rIndex = Atomics.load(this.readIndex, 0);
    const wIndexNow = Atomics.load(this.writeIndex, 0);

    let underflow = false;

    for (let i = 0; i < left.length; i++) {
      if (rIndex === wIndexNow) {
        left[i] = 0;
        right[i] = 0;
        underflow = true;
      } else {
        const sample = this.ringBuffer[rIndex];
        left[i] = sample;
        right[i] = sample;

        rIndex = (rIndex + 1) % this.ringBufferSize;
      }
    }

    Atomics.store(this.readIndex, 0, rIndex);

    if (underflow) {
      this.port.postMessage({ type: "log", message: "[AUDIO WORKLET] no inputs (underflow)" });
    }

    const wIndex = Atomics.load(this.writeIndex, 0);
    const availableSamples = (wIndex - rIndex + this.ringBufferSize) % this.ringBufferSize;

    if (availableSamples < this.bufferSize && Atomics.load(this.flag, 0) === 1) {
      Atomics.store(this.flag, 0, 0);
      Atomics.notify(this.flag, 0, 1);
    }

    return true;
  }
}

registerProcessor("sound-processor", AudioWorkletManager);
