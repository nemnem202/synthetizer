import { MidiController } from "../midi/midi_controller_service";
import { SynthApi } from "../sound/synth_api_service";

const midi_controller = new MidiController();

const synth_api = new SynthApi().init();
