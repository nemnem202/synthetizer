import { MidiController } from "../midi/midi_controller_service";
import { SynthApi } from "../sound/synth_api_service";

import "../../style.css";
import { SynthComponent } from "../components/synth_component";

const midi_controller = new MidiController();

const synth_api = new SynthApi();

const synth = new SynthComponent(synth_api);

synth_api.init();
