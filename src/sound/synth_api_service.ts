import type { noteDTO } from "../types/note";
import { AudioEngineOrchestrator } from "./audio_engine_orchestrator";

// -------------------- Constantes MIDI --------------------
const MIDI_EVENT_SIZE = 4; // 4 octets par évènement midi
const MIDI_QUEUE_CAPACITY = 64;
const MIDI_BUFFER_SIZE = MIDI_QUEUE_CAPACITY * MIDI_EVENT_SIZE;

// -------------------- Constantes OSC ---------------------
const OSC_EVENT_SIZE = 8; // 8 octets par évènement oscillateur
const OSC_QUEUE_CAPACITY = 100;
const OSC_BUFFER_SIZE = OSC_QUEUE_CAPACITY * OSC_EVENT_SIZE;

export enum OscKey {
  NONE = 0,
  ATTACK = 1,
  RELEASE = 2,
  DECAY = 3,
  SUSTAIN = 4,
  GAIN = 5,
  DELAY = 6,
  PITCH = 7,
  PHASE = 8,
  WAVEFORM = 9,
  PAN = 10,
}

// -------------------- Constantes FX ----------------------

const FX_PARAMS_NUMBER = 16; // nombre de paramètres qu'on peut modifier + id + identifiant
const FX_BUFFER_QUEUE_CAPACITY = 64;
const FX_BUFFER_SIZE = FX_PARAMS_NUMBER * FX_BUFFER_QUEUE_CAPACITY;

export class SynthApi {
  private static soundEngine: AudioEngineOrchestrator;

  // ---- Buffers MIDI ----
  private static midi_queue_buffer: SharedArrayBuffer;
  private static midi_queue_array: Uint8Array;
  private static midi_write_index: Int32Array;

  // ---- Buffers OSC ----

  private static osc_queue_buffer: SharedArrayBuffer;
  private static osc_queue_array: Uint8Array;
  private static osc_write_index: Int32Array;

  // ---- Buffers FX ----

  private static fx_queue_buffer: SharedArrayBuffer;
  private static fx_queue_array: Float32Array;
  private static fx_write_index: Int32Array;

  private nmbr_of_oscillators = 0;

  constructor() {
    SynthApi.soundEngine = AudioEngineOrchestrator.getInstance();

    SynthApi.midi_queue_buffer = new SharedArrayBuffer(MIDI_BUFFER_SIZE);

    // Initialisation des buffers
    SynthApi.init_midi_queue();
    SynthApi.init_osc_queue();
    SynthApi.init_fx_queue();
  }

  private static init_midi_queue() {
    const controlSize = 2 * Int32Array.BYTES_PER_ELEMENT;
    SynthApi.midi_queue_buffer = new SharedArrayBuffer(controlSize + MIDI_BUFFER_SIZE);

    SynthApi.midi_write_index = new Int32Array(SynthApi.midi_queue_buffer, 0, 2);
    SynthApi.midi_queue_array = new Uint8Array(SynthApi.midi_queue_buffer, controlSize);
  }

  private static init_osc_queue() {
    const controlSize = 2 * Int32Array.BYTES_PER_ELEMENT;
    SynthApi.osc_queue_buffer = new SharedArrayBuffer(controlSize + OSC_BUFFER_SIZE);

    SynthApi.osc_write_index = new Int32Array(SynthApi.osc_queue_buffer, 0, 2);
    SynthApi.osc_queue_array = new Uint8Array(SynthApi.osc_queue_buffer, controlSize);
  }

  private static init_fx_queue() {
    const controlSize = 2 * Int32Array.BYTES_PER_ELEMENT;
    SynthApi.fx_queue_buffer = new SharedArrayBuffer(controlSize + FX_BUFFER_SIZE);

    SynthApi.fx_write_index = new Int32Array(SynthApi.fx_queue_buffer, 0, 2);
    SynthApi.fx_queue_array = new Float32Array(SynthApi.fx_queue_buffer, controlSize);
  }

  async init() {
    await SynthApi.soundEngine.init(
      SynthApi.midi_queue_buffer,
      SynthApi.osc_queue_buffer,
      SynthApi.fx_queue_buffer
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

  // -------------------- OSC --------------------

  private static writeToOscQueue(
    event_type: number,
    osc_index: number,
    key: OscKey,
    value: number
  ) {
    if (key === OscKey.WAVEFORM) {
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

  public create_oscillator() {
    const id = this.nmbr_of_oscillators;

    SynthApi.writeToOscQueue(0, id, 0, 0); // 0 = add, key et value ignorés
    console.log(`Oscillateur ${id} créé`);
    this.nmbr_of_oscillators++;
    return id;
  }

  public remove_oscillator(osc_index: number) {
    SynthApi.writeToOscQueue(1, osc_index, 0, 0); // 1 = remove
    console.log(`Oscillateur ${osc_index} supprimé`);
  }

  public update_oscillator(osc_index: number, key: OscKey, value: number) {
    SynthApi.writeToOscQueue(2, osc_index, key, value); // 2 = update
  }

  private static convert_ms_to_sample(ms: number) {
    return Math.floor((ms / 1000) * 44100);
  }

  private static convert_sample_to_ms() {}

  private static convert_semitone_to_frequency_shift(semitone: number) {
    return Math.pow(2, semitone / 12);
  }

  // ----------------------- FX -----------------------------

  private static write_to_fx_queue(id: number, type: number, values: number[]) {
    const writePos = Atomics.load(SynthApi.fx_write_index, 0);
    const readPos = Atomics.load(SynthApi.fx_write_index, 1);

    const nextWrite = (writePos + 1) % FX_BUFFER_QUEUE_CAPACITY;
    if (nextWrite === readPos) {
      console.warn("Queue FX pleine");
      return;
    }

    const offset = writePos * FX_PARAMS_NUMBER;
    SynthApi.fx_queue_array[offset] = type & 0xff;
    SynthApi.fx_queue_array[offset + 1] = id & 0xff;

    for (let i = 0; i < values.length; i++) {
      SynthApi.fx_queue_array[offset + 2 + i] = values[i];
    }

    Atomics.store(SynthApi.fx_write_index, 0, nextWrite);
  }

  public destroy() {
    SynthApi.soundEngine.release();
  }
}
