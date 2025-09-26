# Synthétiseur Rust / WebAssembly Ultra-Performance

Découvrez mon projet de **synthétiseur** développé en **Rust** et compilé en **WebAssembly** pour des performances ultra-hautes et un contrôle **low-level**.  
**J'ai réussi à jouer jusqu'à 500 samplers simultanément sans artefact audible.**

## Fonctionnalités

- Jouer des notes via un **contrôleur MIDI** ou le **clavier** :

q, z, s, e, d, f, t, g, y, h, u, j
_(dans l'ordre des demi-tons)_

- Interface réactive et interactive.
- Intégration prévue dans une application d'apprentissage de musique en ligne.

## Lancer l'application avec Docker

Assurez-vous que **Docker** est installé sur votre machine.  
Ensuite, vous pouvez lancer l'application avec ces commandes :

docker build -f Dockerfile -t rust-wasm-synth .
docker run -p 4173:80 rust-wasm-synth

## Accès à l'application

L'application sera ensuite accessible via : [http://localhost:4173/](http://localhost:4173/)

- Version en ligne : [https://synthetizer-production.up.railway.app/](https://synthetizer-production.up.railway.app/)

## Usage

1. Cliquez sur le bouton **"Commencer"** pour créer vos premières sources sonores.
2. Jouez avec votre **clavier** ou **contrôleur MIDI**.
3. Explorez et modifiez les paramètres pour expérimenter les sons générés par le synthétiseur.

## Suivi du projet

Pour découvrir le projet et suivre son évolution, rendez-vous sur [mon GitHub](https://github.com/nemnem202).
