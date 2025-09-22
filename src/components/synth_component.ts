import type { WaveType } from "../sound/rust-synth/build/rust_synth";
import { OscKey, type SynthApi } from "../sound/synth_api_service";

export class SynthComponent {
  api: SynthApi;
  constructor(api: SynthApi) {
    this.api = api;

    const container = document.createElement("div");
    container.className = "synthcontainer";
    document.body.appendChild(container);

    const btn = document.createElement("button");
    btn.addEventListener("click", () => this.create_oscillator(container));
    btn.innerText = "ajouter un oscillateur";
    container.appendChild(btn);
  }

  private create_oscillator(main_container: HTMLElement) {
    const INDEX = this.api.create_oscillator();
    const container = document.createElement("div");
    container.className = "oscContainer";

    const h2 = document.createElement("h2");
    h2.innerText = `Oscillator ${INDEX}`;

    // Select waveform
    const waveformLabel = document.createElement("label");
    waveformLabel.innerText = "Waveform: ";

    const waveformSelect = document.createElement("select");
    ["sine", "square", "sawtooth", "triangle"].forEach((wave, index) => {
      const option = document.createElement("option");
      option.value = "index";
      option.text = wave;
      waveformSelect.appendChild(option);
    });

    waveformSelect.addEventListener("change", (e) => {
      this.api.update_oscillator(
        parseInt((e.target as HTMLSelectElement).value) as WaveType,
        OscKey.WAVEFORM,
        INDEX
      );
    });

    container.appendChild(h2);
    container.appendChild(waveformLabel);
    container.appendChild(waveformSelect);

    const attack = this.create_slider(container, "Attack (ms)", 200, 10000, 1, 500);
    const decay = this.create_slider(container, "Decay (ms)", 200, 10000, 1, 500);
    const sustain = this.create_slider(container, "Sustain (%)", 0, 100, 1, 50);
    const release = this.create_slider(container, "Release (ms)", 0, 10000, 1, 500);
    const delay = this.create_slider(container, "Delay (ms)", 0, 10000, 1, 0);
    const frequency = this.create_slider(container, "Shift (semitones)", -36, 36, 1, 0);
    const phase = this.create_slider(container, "Phase", 0.05, 0.95, 0.05, 0);
    const gain = this.create_slider(container, "Gain (%)", 0, 100, 1, 50);

    // --- Écouteurs branchés ici ---
    attack.addEventListener("change", () =>
      this.api.update_oscillator(INDEX, OscKey.ATTACK, Number(attack.value))
    );

    decay.addEventListener("change", () =>
      this.api.update_oscillator(INDEX, OscKey.DECAY, Number(decay.value))
    );

    sustain.addEventListener("change", () =>
      this.api.update_oscillator(INDEX, OscKey.SUSTAIN, Number(sustain.value))
    );

    release.addEventListener("change", () =>
      this.api.update_oscillator(INDEX, OscKey.RELEASE, Number(release.value))
    );

    delay.addEventListener("change", () =>
      this.api.update_oscillator(INDEX, OscKey.DELAY, Number(delay.value))
    );

    frequency.addEventListener("change", () =>
      this.api.update_oscillator(INDEX, OscKey.PITCH, Number(frequency.value))
    );

    phase.addEventListener("change", () =>
      this.api.update_oscillator(INDEX, OscKey.PHASE, Number(phase.value))
    );

    gain.addEventListener("change", () =>
      this.api.update_oscillator(INDEX, OscKey.GAIN, Number(gain.value))
    );

    // Bouton suppression
    const delBtn = document.createElement("button");
    delBtn.innerText = "supprimer";
    delBtn.addEventListener("click", () => {
      container.remove();
      this.api.remove_oscillator(INDEX);
    });

    container.appendChild(delBtn);

    main_container.appendChild(container);
  }

  private create_slider = (
    container: HTMLDivElement,
    labelText: string,
    min: number,
    max: number,
    step: number,
    value: number
  ) => {
    const wrapper = document.createElement("div");
    wrapper.className = "slider-wrapper";

    const input = document.createElement("input");
    input.type = "range";
    input.min = String(min);
    input.max = String(max);
    input.step = String(step);
    input.value = String(value);

    const label = document.createElement("label");
    label.textContent = `${labelText} ${input.value}`;

    wrapper.appendChild(label);
    wrapper.appendChild(input);

    container.appendChild(wrapper);

    // Je retourne l’input (plus simple pour brancher les events ensuite)
    return input;
  };
}
