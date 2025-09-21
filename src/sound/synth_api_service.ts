import type { noteDTO } from "../types/note";
import { AudioEngineOrchestrator } from "./audio_engine_orchestrator";

const MIDI_EVENT_SIZE = 4;
const MIDI_QUEUE_CAPACITY = 64;
const MIDI_BUFFER_SIZE = MIDI_QUEUE_CAPACITY * MIDI_EVENT_SIZE;

export class SynthApi {
  private static soundEngine: AudioEngineOrchestrator;

  private static midi_queue_buffer: SharedArrayBuffer;
  private static midi_queue_array: Uint8Array;
  private static midi_write_index: Int32Array;

  constructor() {
    SynthApi.soundEngine = AudioEngineOrchestrator.getInstance();

    SynthApi.midi_queue_buffer = new SharedArrayBuffer(MIDI_BUFFER_SIZE);
    SynthApi.midi_queue_array = new Uint8Array(SynthApi.midi_queue_buffer);

    const midi_control_size = 2 * Int32Array.BYTES_PER_ELEMENT;
    SynthApi.midi_queue_buffer = new SharedArrayBuffer(midi_control_size + MIDI_BUFFER_SIZE);

    SynthApi.midi_write_index = new Int32Array(SynthApi.midi_queue_buffer, 0, 2);

    SynthApi.midi_queue_array = new Uint8Array(SynthApi.midi_queue_buffer, midi_control_size);
  }

  async init() {
    await SynthApi.soundEngine.init(SynthApi.midi_queue_buffer);
  }

  static async playNote(note: noteDTO) {
    SynthApi.writeToMidiQueue(1, note.value, note.velocity ?? 100);
  }

  static async stopNote(note: noteDTO) {
    SynthApi.writeToMidiQueue(1, note.value, 0);
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

  public destroy() {
    SynthApi.soundEngine.release();
  }
}
