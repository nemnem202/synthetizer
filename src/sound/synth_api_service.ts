import type { noteDTO } from "../types/note";
import { AudioEngineOrchestrator } from "./audio_engine_orchestrator";

const MIDI_EVENT_SIZE = 4;
const MIDI_QUEUE_CAPACITY = 64;
const MIDI_BUFFER_SIZE = MIDI_QUEUE_CAPACITY * MIDI_EVENT_SIZE;

const OSC_EVENT_SIZE = 8;
const OSC_QUEUE_CAPACITY = 100;
const OSC_BUFFER_SIZE = OSC_QUEUE_CAPACITY * OSC_EVENT_SIZE;

export enum OscKey {
  NONE,
  ATTACK,
  RELEASE,
  DECAY,
  SUSTAIN,
  GAIN,
  DELAY,
  PITCH,
  PHASE,
  SAMPLE_ID,
  PAN,
}

const FX_EVENT_SIZE = 16;
const FX_QUEUE_CAPACITY = 64;
const FX_BUFFER_SIZE = FX_EVENT_SIZE * FX_QUEUE_CAPACITY;

const MAX_SAMPLE_LENGTH = 2 * 8_000_000;
const SAMPLE_EVENT_SIZE = 6 * Int32Array.BYTES_PER_ELEMENT;

export type EffectParams = { index: number; value: number };

export enum Effects {
  ECHO,
  FILTER,
}

export enum EchoParams {
  DELAY,
  FEEDBACK,
  R_DELAY_OFFSET,
  L_DELAY_OFFSET,
  DRY,
  WET,
}

export enum FilterParams {
  FREQUENCY,
  Q,
  TYPE,
  GAIN,
}

export type SampleEvent = {
  sampler_id: number;
  sample_id: number;
  length: number;
  channels: number;
  hq: number;
};

export type SampleData = {
  title: string;
  sample_id: number;
  duration_seconds: number;
  high_quality: boolean;
};

export class SynthApi {
  private static soundEngine: AudioEngineOrchestrator;

  private static midi_queue_buffer: SharedArrayBuffer;
  private static midi_queue_array: Uint8Array;
  private static midi_write_index: Int32Array;

  private static osc_queue_buffer: SharedArrayBuffer;
  private static osc_queue_array: Uint8Array;
  private static osc_write_index: Int32Array;

  public loaded_samples: SampleData[] = [];
  private static sampler_event_buffer: SharedArrayBuffer;
  private static sample_buffer: SharedArrayBuffer;

  private static fx_queue_buffer: SharedArrayBuffer;
  private static fx_queue_int_array: Int32Array;
  private static fx_queue_float_array: Float32Array;
  private static fx_write_index: Int32Array;

  private nmbr_of_samplers = 0;
  private nmbr_of_fx = 0;
  private static sample_event_index = 0;

  private static sample_processor_worker: Worker;

  constructor() {
    SynthApi.soundEngine = AudioEngineOrchestrator.getInstance();

    SynthApi.midi_queue_buffer = new SharedArrayBuffer(MIDI_BUFFER_SIZE);

    SynthApi.init_sample_event();
    SynthApi.init_sample_buffer();
    SynthApi.init_sample_processor_worker();

    SynthApi.init_midi_queue();
    SynthApi.init_osc_queue();
    SynthApi.init_fx_queue();
  }

  private static init_midi_queue() {
    const control_size = 2 * Int32Array.BYTES_PER_ELEMENT;
    SynthApi.midi_queue_buffer = new SharedArrayBuffer(control_size + MIDI_BUFFER_SIZE);

    SynthApi.midi_write_index = new Int32Array(SynthApi.midi_queue_buffer, 0, 2);
    SynthApi.midi_queue_array = new Uint8Array(SynthApi.midi_queue_buffer, control_size);
  }

  private static init_osc_queue() {
    const control_size = 2 * Int32Array.BYTES_PER_ELEMENT;
    SynthApi.osc_queue_buffer = new SharedArrayBuffer(control_size + OSC_BUFFER_SIZE);

    SynthApi.osc_write_index = new Int32Array(SynthApi.osc_queue_buffer, 0, 2);
    SynthApi.osc_queue_array = new Uint8Array(SynthApi.osc_queue_buffer, control_size);
  }

  private static init_fx_queue() {
    const control_size = 2 * Int32Array.BYTES_PER_ELEMENT;
    SynthApi.fx_queue_buffer = new SharedArrayBuffer(control_size + FX_BUFFER_SIZE);

    SynthApi.fx_write_index = new Int32Array(SynthApi.fx_queue_buffer, 0, 2);

    SynthApi.fx_queue_int_array = new Int32Array(
      SynthApi.fx_queue_buffer,
      control_size,
      3 * FX_QUEUE_CAPACITY
    );

    SynthApi.fx_queue_float_array = new Float32Array(
      SynthApi.fx_queue_buffer,
      control_size + 12 * FX_QUEUE_CAPACITY,
      FX_QUEUE_CAPACITY
    );
  }

  async init() {
    await SynthApi.soundEngine.init(
      SynthApi.midi_queue_buffer,
      SynthApi.osc_queue_buffer,
      SynthApi.fx_queue_buffer,
      SynthApi.sampler_event_buffer,
      SynthApi.sample_buffer
    );
  }

  static playNote(note: noteDTO) {
    SynthApi.writeToMidiQueue(1, note.value, note.velocity ?? 100);
  }

  static stopNote(value: number) {
    SynthApi.writeToMidiQueue(1, value, 0);
  }

  private static writeToMidiQueue(event_type: number, note: number, velocity: number) {
    const write_pos = Atomics.load(SynthApi.midi_write_index, 0);
    const read_pos = Atomics.load(SynthApi.midi_write_index, 1);

    const next_write_pos = (write_pos + 1) % MIDI_QUEUE_CAPACITY;

    if (next_write_pos === read_pos) {
      console.warn("Queue MIDI pleine");
      return;
    }

    const event_offset = write_pos * MIDI_EVENT_SIZE;
    SynthApi.midi_queue_array[event_offset] = event_type;
    SynthApi.midi_queue_array[event_offset + 1] = note;
    SynthApi.midi_queue_array[event_offset + 2] = velocity;
    SynthApi.midi_queue_array[event_offset + 3] = 0;

    Atomics.store(SynthApi.midi_write_index, 0, next_write_pos);
  }

  private static writeToOscQueue(
    event_type: number,
    osc_index: number,
    key: OscKey,
    value: number
  ) {
    if (key === OscKey.SAMPLE_ID) {
    } else if (key === OscKey.PITCH) {
      value = this.convert_semitone_to_frequency_shift(value);
    } else if (
      key === OscKey.ATTACK ||
      key === OscKey.DECAY ||
      key === OscKey.RELEASE ||
      key === OscKey.DELAY
    ) {
      value = this.convert_ms_to_sample(value);
    }
    const writePos = Atomics.load(SynthApi.osc_write_index, 0);
    const readPos = Atomics.load(SynthApi.osc_write_index, 1);

    const nextWrite = (writePos + 1) % OSC_QUEUE_CAPACITY;
    if (nextWrite === readPos) {
      console.warn("Queue OSC pleine");
      return;
    }

    const offset = writePos * OSC_EVENT_SIZE;
    SynthApi.osc_queue_array[offset] = event_type & 0xff;
    SynthApi.osc_queue_array[offset + 1] = osc_index & 0xff;
    SynthApi.osc_queue_array[offset + 2] = key & 0xff;

    const view = new DataView(
      SynthApi.osc_queue_array.buffer,
      SynthApi.osc_queue_array.byteOffset + offset + 3,
      4
    );
    view.setFloat32(0, value, true);

    Atomics.store(SynthApi.osc_write_index, 0, nextWrite);
  }

  public create_sampler() {
    const id = this.nmbr_of_samplers;

    SynthApi.writeToOscQueue(0, id, 0, 0);
    this.nmbr_of_samplers++;
    return id;
  }

  public remove_sampler(osc_index: number) {
    SynthApi.writeToOscQueue(1, osc_index, 0, 0);
  }

  public update_sampler(osc_index: number, key: OscKey, value: number) {
    SynthApi.writeToOscQueue(2, osc_index, key, value);
  }

  private static convert_ms_to_sample(ms: number) {
    return Math.floor((ms / 1000) * 44100);
  }

  private static convert_sample_to_ms() {}

  private static convert_semitone_to_frequency_shift(semitone: number) {
    return Math.pow(2, semitone / 12);
  }

  private static write_to_fx_queue(
    id: number,
    event_type: number,
    param_index: number,
    value: number
  ) {
    const write_pos = Atomics.load(SynthApi.fx_write_index, 0);
    const read_pos = Atomics.load(SynthApi.fx_write_index, 1);

    const next_write_pos = (write_pos + 1) % FX_QUEUE_CAPACITY;
    if (next_write_pos === read_pos) {
      console.warn("Queue FX pleine");
      return;
    }

    const int_base = write_pos * 3;
    const float_base = write_pos;

    SynthApi.fx_queue_int_array[int_base] = id;
    SynthApi.fx_queue_int_array[int_base + 1] = event_type;
    SynthApi.fx_queue_int_array[int_base + 2] = param_index;
    SynthApi.fx_queue_float_array[float_base] = value;

    Atomics.store(SynthApi.fx_write_index, 0, next_write_pos);
  }

  add_fx(param_index: number) {
    const id = Number(JSON.parse(JSON.stringify(this.nmbr_of_fx)));
    SynthApi.write_to_fx_queue(id, 0, param_index, 0);
    this.nmbr_of_fx++;
    return id;
  }

  edit_fx(id: number, param_index: EchoParams | FilterParams, param_value: number) {
    SynthApi.write_to_fx_queue(id, 2, param_index, param_value);
  }

  remove_fx(id: number) {
    SynthApi.write_to_fx_queue(id, 1, 0, 0);
  }

  private static init_sample_buffer() {
    SynthApi.sample_buffer = new SharedArrayBuffer(
      Float32Array.BYTES_PER_ELEMENT * MAX_SAMPLE_LENGTH
    );
  }

  private static init_sample_processor_worker() {
    SynthApi.sample_processor_worker = new Worker(
      new URL("./sampler_processor_worker.ts", import.meta.url),
      {
        type: "module",
        name: "sampler_processor_worker",
      }
    );

    SynthApi.sample_processor_worker.postMessage({
      type: "init",
      sample_buffer: SynthApi.sample_buffer,
    });
    SynthApi.sample_processor_worker.onmessage = (e: MessageEvent) => {
      if (!e) return;
      if (e.data.type === "log") {
        console.log(e.data.message);
      } else if (e.data.type === "sampler update") {
        this.notify_sample_event(e.data.event as SampleEvent);
      }
    };
  }

  private static get sample_event_view(): Uint32Array {
    return new Uint32Array(SynthApi.sampler_event_buffer);
  }

  private static init_sample_event() {
    SynthApi.sampler_event_buffer = new SharedArrayBuffer(SAMPLE_EVENT_SIZE);
  }

  public static notify_sample_event(event: SampleEvent) {
    const evt = SynthApi.sample_event_view;
    SynthApi.sample_event_index++;
    evt[0] = SynthApi.sample_event_index;
    evt[1] = event.sampler_id;
    evt[2] = event.sample_id;
    evt[3] = event.length;
    evt[4] = event.channels;
    evt[5] = event.hq;
  }

  public async import_sample(
    files: FileList | null,
    hq: boolean,
    sampler_id: number
  ): Promise<SampleData[] | void> {
    if (!files) return;
    const file = files[0];
    if (file.type !== "audio/wav") {
      console.log("invalid format");
      return;
    }

    console.log("[IMPORT SAMPLE] processing...");

    const array_buffer = await file.arrayBuffer();
    const audio_ctx = new AudioContext();
    const audio_buffer = await audio_ctx.decodeAudioData(array_buffer);

    const channels: Float32Array[] = [];
    for (let i = 0; i < audio_buffer.numberOfChannels; i++) {
      channels.push(audio_buffer.getChannelData(i));
    }

    if (hq) {
      const new_sample_id = this.get_new_sample_id();
      this.handleHqSample(audio_buffer, channels, sampler_id, new_sample_id);
      const new_sample: SampleData = {
        duration_seconds: audio_buffer.duration,
        high_quality: true,
        sample_id: new_sample_id,
        title: file.name,
      };

      this.loaded_samples.push(new_sample);
    } else {
      let total_length = channels[0].length;
      if (channels.length >= 2) {
        total_length += channels[1].length;
      }

      if (total_length > MAX_SAMPLE_LENGTH) {
        console.warn("Sample trop long pour être inséré dans le buffer !");
        return;
      }
      // Création d'un Float32Array sur le buffer partagé
      const buffer_view = new Float32Array(SynthApi.sample_buffer, 0, total_length);

      // Écriture des channels dans le buffer
      buffer_view.set(channels[0], 0);
      if (channels.length >= 2) {
        buffer_view.set(channels[1], channels[0].length);
      }

      const new_sample_id = this.get_new_sample_id();
      // Notifier l'événement
      SynthApi.notify_sample_event({
        sampler_id: sampler_id,
        sample_id: new_sample_id, // tu peux incrémenter si tu veux gérer plusieurs samples
        length: total_length,
        channels: channels.length,
        hq: 0,
      });

      const new_sample: SampleData = {
        duration_seconds: audio_buffer.duration,
        high_quality: false,
        sample_id: this.get_new_sample_id(),
        title: file.name,
      };

      this.loaded_samples.push(new_sample);
    }

    return this.loaded_samples;
  }

  public async set_existing_sample(id: number, sampler_id: number) {
    const sample = this.loaded_samples.find((e) => e.sample_id === id);
    if (!sample) {
      console.error("aucun sample n' a été trouvé");
      return;
    }
    const event: SampleEvent = {
      sample_id: sample.sample_id,
      sampler_id: sampler_id,
      channels: 0,
      hq: 0,
      length: 0,
    };

    SynthApi.notify_sample_event(event);
  }

  private handleHqSample(
    audio_buffer: AudioBuffer,
    channels: Float32Array[],
    sampler_id: number,
    sample_id: number
  ) {
    if (channels[1] && audio_buffer.duration < 5) {
      const interleaved = new Float32Array(channels[0].length + channels[1].length);
      interleaved.set(channels[0], 0);
      interleaved.set(channels[1], channels[0].length);

      SynthApi.sample_processor_worker.postMessage({
        samples: interleaved,
        sampleRate: audio_buffer.sampleRate,
        event: {
          sampler_id: sampler_id,
          sample_id: sample_id,
          length: interleaved.length,
          channels: 2,
          hq: 1,
        },
      });
    } else if (audio_buffer.duration < 10) {
      SynthApi.sample_processor_worker.postMessage({
        samples: channels[0],
        sampleRate: audio_buffer.sampleRate,
        event: {
          sampler_id: sampler_id,
          sample_id: sample_id,
          length: channels[0].length,
          channels: 1,
          hq: 1,
        },
      });
    } else {
      console.log("durée de fichier trop longue !");
      window.alert(
        "Votre fichier est trop long pour être traité avec une haute qualité, essayez en mono ou avec la qualité standart."
      );
      return;
    }

    console.log("worker message sent !");
  }

  private get_new_sample_id(): number {
    let i = 0;
    while (this.loaded_samples.find((e) => e.sample_id === i)) i++;
    return i;
  }
  public destroy() {
    SynthApi.soundEngine.release();
  }
}
