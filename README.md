# Portloom

**Tes ports, sous contrôle.** Un moniteur de ports natif en Rust pour reconnaître les projets, applications et services qui tournent sur ta machine, puis arrêter le bon processus.

[![Vérification et distribution](https://github.com/ronael/portloom/actions/workflows/build.yml/badge.svg)](https://github.com/ronael/portloom/actions/workflows/build.yml)
[![MIT](https://img.shields.io/badge/licence-MIT-79dbc8)](LICENSE)

![Interface Portloom : projets, services et inspecteur de processus](docs/assets/preview.png)

## Installer et lancer

Les archives exécutables sont disponibles dans les [releases GitHub](https://github.com/ronael/portloom/releases). Avec l’archive npm téléchargée :

```sh
npm install --global ./portloom-1.0.1.tgz
portloom
```

Le paquet [npm `portloom`](https://www.npmjs.com/package/portloom) est également prévu pour `npm install --global portloom`. Si npm affiche encore `0.0.0-stage`, utilise l’archive de la release : cette version d’attente ne contient pas l’application.

Le lanceur fait `exec` du binaire Rust. Aucun processus Node ou Bun ne reste en mémoire. Aucun téléchargement ni aucune compilation à l’installation du paquet. Le binaire peut aussi être utilisé directement, sans npm.

macOS et Linux glibc sont les cibles du projet, en ARM64 et x64. La CI construit et teste chaque cible sur un runner natif ; le contenu de chaque archive indique les plateformes réellement incluses. `lsof` et `ps` sont requis (`lsof` peut être à installer sous Linux). Terminal minimal : **60 × 20**. Windows et Alpine/musl ne sont pas pris en charge.

## Voir ce qui tourne

- Ports TCP en écoute et sockets UDP liés à un port local, regroupés par processus.
- Catégories explicites : site/API probable, projet, base de données, service, application, système ou inconnu.
- Inspecteur avec commande, exécutable, dossier, adresses, CPU, mémoire et autres ports du processus.
- Recherche, filtres, souris, détails défilables et export JSON.

```sh
portloom --demo             # simulation sans effet sur les processus réels
portloom --json             # un relevé pour tes scripts
portloom --interval 1000    # intervalle de 500 à 60 000 ms
portloom --help
portloom completions zsh
```

| Touche | Action |
| --- | --- |
| `0` à `5` | Tous / Projets / Services / Apps / Système / Inconnus |
| `Tab` / `/` | Filtrer le protocole / rechercher |
| `↑↓` / `Entrée` | Sélectionner / ouvrir le détail |
| `k` / `K` | Préparer SIGTERM / SIGKILL |
| `Échap` | Annuler ou fermer |
| `Espace` / `r` | Pause / actualisation |
| `?` / `q` / `Ctrl+C` | Aide / quitter |

## Arrêter un processus

L’arrêt concerne **un processus et tous ses ports**. Portloom affiche cet impact, puis exige une confirmation : `O` pour un projet, saisie du PID exact pour les catégories sensibles. L’identité, la propriété et les ports sont revérifiés avant le signal. Les processus système, ceux d’un autre utilisateur, Portloom et son terminal sont protégés.

SIGTERM ne devient jamais automatiquement SIGKILL. Aucun arrêt collectif d’enfants ou de conteneurs. Les catégories sont des indices issus de la commande, du chemin et du contexte ; un numéro de port ne prouve pas qu’un service est un site. Aucun test HTTP n’est effectué.

Aucune télémétrie ni élévation sudo. Les commandes et exports peuvent contenir des arguments privés : relis-les avant partage. La visibilité dépend des permissions du compte. La vérification d’identité et l’envoi d’un signal ne sont pas atomiques ; une course résiduelle reste possible, notamment sous macOS où `ps` expose l’heure de début à la seconde.

## Développer

Prérequis : Rust via rustup (version fixée dans `rust-toolchain.toml`), Node 22 ou plus récent pour les scripts, Python 3 pour les tests réseau et terminal, `lsof` et `ps`. Aucun `npm install` nécessaire.

```sh
git clone https://github.com/ronael/portloom.git
cd portloom
npm run verify
npm run build
npm run demo
python3 scripts/dev/terminal_smoke.py
npm run pack
node scripts/release/smoke.mjs
```

`npm run install:local` installe des liens dans `~/.local/bin` vers le binaire compilé. `cargo run --locked --release -- --demo` fonctionne aussi. Les captures natives de `npm run preview` et les fichiers de compilation restent dans `.tooling`, hors Git.

## Architecture

```text
crates/domain/        Modèles, classification et politique d’arrêt purs
crates/application/   Ports injectables et cas d’usage StopProcess
src/infrastructure/   Adaptateurs lsof, ps, contexte projet, signaux, démo
src/presentation/     CLI, état, composants Ratatui et design system
src/composition/      Assemblage des implémentations
tests/                Contrats, protections, UI, réseau et distribution
scripts/              Vérification, compilation et livraison
distribution/npm/     Métadonnées, lanceur et README du paquet
docs/                 Architecture, décisions, design et distribution
```

Le graphe Cargo impose les dépendances du cœur métier. Les contrôles CI vérifient aussi les frontières de modules et la limite de dix fichiers directement par dossier. Voir [l’architecture](docs/architecture/architecture.md), [les décisions et références](docs/architecture/decisions.md), [la contribution](.github/CONTRIBUTING.md) et [la distribution npm](docs/distribution/npm.md).

## Support et licence

Créé par [ronael](https://github.com/ronael) (npm : [ronaeld](https://www.npmjs.com/~ronaeld)). Les [issues](https://github.com/ronael/portloom/issues) accueillent les bugs et propositions. Pour une vulnérabilité, voir [la politique de sécurité](.github/SECURITY.md).

Portloom est sous [licence MIT](LICENSE). Les notices des bibliothèques Rust sont incluses dans les archives. Le paquet npm embarque également les sources Rust et leurs tests.
