import { SynthApi } from "../sound/synth_api_service";

export class MidiController {
  private midiAccess: MIDIAccess | null = null;

  constructor() {
    this.requestMidiAccess();
  }

  private async requestMidiAccess() {
    try {
      this.midiAccess = await navigator.requestMIDIAccess({ sysex: false });
      this.onMidiSuccess(this.midiAccess);
    } catch (err) {
      console.warn("Accès au midi refusé !");
    }
  }

  private onMidiSuccess(midiAccess: MIDIAccess) {
    const inputs = Array.from(midiAccess.inputs.values());

    console.log("[Inputs] :");
    console.table(inputs);

    inputs.forEach((input) => {
      input.onmidimessage = (ev) => this.handleMidiMessage(ev);
    });
  }

  private handleMidiMessage(ev: MIDIMessageEvent) {
    if (!ev.data) return;
    const status = ev.data[0];
    const note = ev.data[1];
    const velocity = ev.data[2];

    const isNoteOn = status === 153 && velocity > 0;

    if (isNoteOn) {
      SynthApi.playNote({ value: note });
    } else if (status === 137 || (status === 153 && velocity === 0)) {
      SynthApi.stopNote(note);
    }
  }
}
