import { MidiController } from "../midi/midi_controller_service";
import { SynthApi } from "../sound/synth_api_service";

import "../../style.css";
import { SynthComponent } from "../components/synth_component";

const midi_controller = new MidiController();

document.body.innerHTML = document.body.innerHTML = `
  <div class="synth-presentation">
    <h1>Projet : Synthétiseur Rust / WebAssembly Ultra-Performance</h1>
    <p>
      Découvrez mon projet de <strong>synthétiseur</strong> développé en <strong>Rust</strong> et compilé en <strong>WebAssembly</strong> pour des performances ultra-hautes et un contrôle <em>low-level</em>. Un projet unique : rien de comparable n'a vraiment été fait auparavant !
    </p>
    <p>
      Pour jouer, vous avez deux options : 
      <ul>
        <li>Connecter un contrôleur <strong>MIDI</strong>.</li>
        <li>Utiliser les touches de votre clavier :</li>
      </ul>
      <code>q, z, s, e, d, f, t, g, y, h, u, j</code> <em>(dans l'ordre des demi-tons)</em>
    </p>
    <p>
      Pour commencer, cliquez sur le bouton <strong>"Commencer"</strong> (le bouton sera généré plus tard) afin de créer vos premières sources sonores.
    </p>
    <p>
      Ce synthétiseur sera intégré dans ma prochaine application d'apprentissage de musique en ligne, offrant une expérience interactive et réactive aux utilisateurs.
    </p>
    <p>
      Pour découvrir le projet et suivre son évolution, rendez-vous sur <a href="https://github.com/nemnem202" target="_blank">mon GitHub</a>.
    </p>

        <button id="main_button">Commencer</button>
  </div>

`;

const btn = document.getElementById("main_button");

if (btn) {
  btn.addEventListener("click", () => {
    const synth_api = new SynthApi();
    const synth = new SynthComponent(synth_api);

    synth_api.init();
  });
}
