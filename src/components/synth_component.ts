import { EchoParams, Effects, OscKey, SynthApi } from "../sound/synth_api_service";

const keys = ["q", "z", "s", "e", "d", "f", "t", "g", "y", "h", "u", "j"];

export class SynthComponent {
  api: SynthApi;
  constructor(api: SynthApi) {
    this.api = api;
    document.body.innerHTML = "";
    const container = document.createElement("div");
    container.className = "synthcontainer";
    this.create_mixer(document.body);

    const h2 = document.createElement("h2");
    h2.textContent = "Oscillateurs";

    const btn = document.createElement("button");
    btn.addEventListener("click", () => {
      this.create_sampler(container);
    });
    btn.innerText = "ajouter un sampler";

    const btn_200 = document.createElement("button");
    btn_200.addEventListener("click", () => {
      Array.from({ length: 200 }).map(() => this.create_sampler(container));
    });
    btn_200.innerText = "ajouter 200 samplers";

    document.body.appendChild(h2);
    document.body.appendChild(btn);
    document.body.appendChild(btn_200);

    document.body.appendChild(container);

    this.listen_keys();
  }

  private create_sampler(main_container: HTMLElement) {
    const INDEX = this.api.create_sampler();
    const container = document.createElement("div");
    container.className = "oscContainer";

    const h2 = document.createElement("h2");
    h2.innerText = `Sampler ${INDEX}`;

    const input_option = document.createElement("input");
    input_option.type = "file";
    input_option.accept = ".wav";
    input_option.innerText = "importer votre waveform";

    input_option.addEventListener("input", () =>
      this.api.handle_sample(input_option.files, false, INDEX)
    );

    const hq_input_option = document.createElement("input");
    hq_input_option.id = `hq-${INDEX}`;
    hq_input_option.type = "file";
    hq_input_option.accept = ".wav";
    hq_input_option.innerText = "importer votre waveform";

    hq_input_option.addEventListener("input", () =>
      this.api.handle_sample(hq_input_option.files, true, INDEX)
    );

    const hq_label_input = document.createElement("label");
    hq_label_input.htmlFor = `hq-${INDEX}`; // INDEX doit être défini
    hq_label_input.textContent = "hq?";

    container.appendChild(h2);
    container.appendChild(input_option);
    container.appendChild(hq_label_input);
    container.appendChild(hq_input_option);

    const attack = this.create_slider(container, "Attack (ms)", 0, 10000, 1, 0);
    const decay = this.create_slider(container, "Decay (ms)", 10, 10000, 1, 500);
    const sustain = this.create_slider(container, "Sustain (%)", 0, 100, 1, 20);
    const release = this.create_slider(container, "Release (ms)", 0, 10000, 1, 500);
    const delay = this.create_slider(container, "Delay (ms)", 0, 10000, 1, 0);
    const frequency = this.create_slider(container, "Shift (semitones)", -36, 36, 1, 0);
    const frequency_full = this.create_slider(container, "Shift (full range)", -12, 12, 0.1, 0);
    const phase = this.create_slider(container, "Phase", 0.05, 0.95, 0.05, 0);
    const gain = this.create_slider(container, "Gain (%)", 0, 100, 1, 50);
    const pan = this.create_slider(container, "Pan (%)", -1, 1, 0.1, 0);

    // --- Écouteurs branchés ici ---
    attack.addEventListener("input", () =>
      this.api.update_sampler(INDEX, OscKey.ATTACK, Number(attack.value))
    );

    decay.addEventListener("input", () =>
      this.api.update_sampler(INDEX, OscKey.DECAY, Number(decay.value))
    );

    sustain.addEventListener("input", () =>
      this.api.update_sampler(INDEX, OscKey.SUSTAIN, Number(sustain.value))
    );

    release.addEventListener("input", () =>
      this.api.update_sampler(INDEX, OscKey.RELEASE, Number(release.value))
    );

    delay.addEventListener("input", () =>
      this.api.update_sampler(INDEX, OscKey.DELAY, Number(delay.value))
    );

    frequency.addEventListener("input", () =>
      this.api.update_sampler(INDEX, OscKey.PITCH, Number(frequency.value))
    );

    frequency_full.addEventListener("input", () =>
      this.api.update_sampler(INDEX, OscKey.PITCH, Number(frequency_full.value))
    );

    phase.addEventListener("input", () =>
      this.api.update_sampler(INDEX, OscKey.PHASE, Number(phase.value))
    );

    gain.addEventListener("input", () =>
      this.api.update_sampler(INDEX, OscKey.GAIN, Number(gain.value))
    );

    pan.addEventListener("input", () => {
      this.api.update_sampler(INDEX, OscKey.PAN, Number(pan.value));
    });

    // Bouton suppression
    const delBtn = document.createElement("button");
    delBtn.innerText = "supprimer";
    delBtn.addEventListener("click", () => {
      container.remove();
      this.api.remove_sampler(INDEX);
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

    input.addEventListener("input", () => (label.textContent = `${labelText} ${input.value}`));

    wrapper.appendChild(input);
    wrapper.appendChild(label);

    container.appendChild(wrapper);

    // Je retourne l’input (plus simple pour brancher les events ensuite)
    return input;
  };

  private listen_keys() {
    let playedkeys: string[] = [];
    window.addEventListener("keydown", (e) => {
      const index = keys.indexOf(e.key.toLowerCase());

      if (index === -1 || playedkeys.includes(e.key.toLowerCase())) return;
      playedkeys.push(e.key.toLowerCase());
      SynthApi.playNote({ value: 72 + index, velocity: 50 });
    });

    window.addEventListener("keyup", (e) => {
      const index = keys.indexOf(e.key.toLowerCase());

      if (index === -1) return;
      playedkeys = playedkeys.filter((k) => k !== e.key.toLowerCase());
      SynthApi.stopNote(72 + index);
    });
  }

  private create_mixer(main_container: HTMLElement) {
    const container = document.createElement("div");
    container.className = "mixer-container";

    const h2 = document.createElement("h2");
    h2.innerText = `Mixer`;

    const btn = document.createElement("button");
    btn.innerText = "Ajouter un Effet";

    const fx_select = document.createElement("select");

    ["Echo", "Filter"].forEach((fx, index) => {
      const option = document.createElement("option");
      option.value = `${index}`;
      option.text = fx;
      fx_select.appendChild(option);
    });

    btn.addEventListener("click", () => {
      const type = fx_select.value === "0" ? Effects.ECHO : Effects.FILTER;
      const id = this.api.add_fx(type);
      this.create_mixer_module(id, container, type);
    });

    container.appendChild(h2);
    container.appendChild(btn);
    container.appendChild(fx_select);
    main_container.appendChild(container);
  }

  private create_mixer_module(id: number, container: HTMLElement, type: Effects) {
    const module_container = document.createElement("div");
    module_container.className = "mixer-module-container";

    let module: HTMLDivElement;

    if (type === Effects.ECHO) {
      module = this.create_echo_module(id);
    } else if (type === Effects.FILTER) {
      module = this.create_filter_module(id);
    } else {
      module = document.createElement("div");
      module.innerText = "error loading module";
    }

    const btn = document.createElement("button");
    btn.innerText = "supprimer " + id;
    btn.addEventListener("click", () => {
      this.api.remove_fx(id);
      module.remove();
    });
    module_container.appendChild(module);
    module_container.appendChild(btn);

    container.appendChild(module_container);
  }

  create_echo_module(id: number): HTMLDivElement {
    const echo = document.createElement("div");
    echo.className = "mixer-effect";
    echo.classList.add("echo-effect");

    const h2 = document.createElement("h2");
    h2.textContent = "Echo";

    const delay = this.create_slider(echo, "Delay (ms)", 10, 2000, 1, 300);
    const feedback = this.create_slider(echo, "Feedback (%)", 0, 1, 0.1, 0.7);
    const offset_r = this.create_slider(echo, "offset_r (ms)", 0, 100, 1, 10);
    const offset_l = this.create_slider(echo, "offset_l (ms)", 0, 100, 1, 50);

    delay.addEventListener("input", () => {
      this.api.edit_fx(id, EchoParams.DELAY, parseInt(delay.value));
    });
    feedback.addEventListener("input", () => {
      this.api.edit_fx(id, EchoParams.FEEDBACK, parseFloat(feedback.value));
    });
    offset_l.addEventListener("input", () => {
      this.api.edit_fx(id, EchoParams.L_DELAY_OFFSET, parseInt(offset_l.value));
    });
    offset_r.addEventListener("input", () => {
      this.api.edit_fx(id, EchoParams.R_DELAY_OFFSET, parseInt(offset_r.value));
    });
    return echo;
  }

  create_filter_module(id: number): HTMLDivElement {
    const filter = document.createElement("div");
    filter.className = "mixer-effect";
    filter.classList.add("filter-effect");

    const h2 = document.createElement("h2");
    h2.textContent = "Filter";
    return filter;
  }
}
