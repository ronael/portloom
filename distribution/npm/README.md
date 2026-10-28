# Portloom

**Tes ports, sous contrôle.**

Un moniteur de ports natif en Rust pour voir quel projet, quelle application ou quel service occupe un port, comprendre son rôle et arrêter le bon processus.

## Installation

```sh
npm install --global portloom
portloom
```

Le binaire est inclus dans le paquet. Aucune compilation ni aucun téléchargement supplémentaire à l’installation. Le lanceur exécute directement le programme Rust : aucun processus Node ou Bun ne reste derrière l’interface.

### Compatibilité de cette archive

<!-- SUPPORTED_PLATFORMS -->

`lsof` et `ps` doivent être disponibles. Ils sont fournis sur macOS ; sous Linux, `lsof` peut être à installer. Terminal minimal : **60 colonnes × 20 lignes**. Windows et Alpine/musl ne sont pas pris en charge.

Les cibles qui ne figurent pas dans le tableau ne sont pas incluses dans cette version du paquet.

![Interface Portloom](https://raw.githubusercontent.com/ronael/portloom/main/docs/assets/preview.png)

## Ce que tu peux faire

- Voir les ports TCP en écoute et les sockets UDP liés à un port local.
- Distinguer les sites/API probables, projets, bases de données, services partagés, applications et services système.
- Retrouver un processus par son port, son PID, son projet, sa commande ou son dossier.
- Inspecter sa commande, son exécutable, ses adresses, son CPU, sa mémoire et tous ses ports.
- Préparer un arrêt normal ou forcé, avec confirmation et vérification de l’identité.
- Exporter un relevé JSON ou explorer l’interface en mode démonstration.

```sh
portloom --demo             # démonstration sans effet sur tes processus
portloom --json             # un relevé JSON pour tes scripts
portloom --interval 1000    # rafraîchissement toutes les 1 000 ms
portloom --help
portloom --version
portloom completions zsh    # génère la complétion du shell
```

## Raccourcis

| Touche | Action |
| --- | --- |
| `0` / `1` / `2` / `3` / `4` / `5` | Tous / Projets / Services / Apps / Système / Inconnus |
| `Tab` | Filtrer TCP et UDP |
| `/` | Rechercher |
| `↑` / `↓` | Sélectionner un port |
| `Entrée` | Ouvrir ou quitter le détail |
| `↑` / `↓` ou molette dans le détail | Faire défiler les informations |
| `k` / `K` | Préparer SIGTERM / SIGKILL |
| `Échap` | Annuler ou fermer |
| `Espace` / `r` | Mettre en pause / actualiser |
| `?` / `q` | Aide / quitter |

## Arrêter en connaissance de cause

Pour un projet, `O` confirme l’arrêt. Pour une base, un service partagé, une application ou un usage inconnu, il faut saisir le **PID exact**, puis Entrée. Les processus système, ceux d’un autre utilisateur, le moniteur et son terminal sont protégés.

L’arrêt vise **un processus et tous ses ports**. Portloom relit son identité et ses ports avant le signal ; SIGTERM ne devient jamais automatiquement SIGKILL. Les enfants et les conteneurs ne sont pas arrêtés collectivement.

Les catégories reposent sur la commande, l’exécutable et le dossier. Le seul numéro d’un port ne suffit pas à identifier un site. La mention « site / API probable » n’est pas un test HTTP. Sous macOS, la vérification d’identité et le signal ne sont pas atomiques : une course résiduelle reste possible.

## Données et dépendances

Aucune télémétrie ni élévation sudo. La visibilité des processus dépend des permissions du compte. Le JSON peut contenir des arguments de commande locaux : vérifie leur contenu avant de le partager.

Le paquet n’a **aucune dépendance npm**. Les bibliothèques Rust sont compilées dans le binaire ; leurs notices sont jointes dans `native/`.

## Sources Rust

Les sources de cette version sont incluses dans **`source/`** : code Rust, manifeste Cargo, lockfile, chaîne de compilation et tests Rust. Tu peux les consulter dans l’onglet Code de npm ou dans le paquet installé. Le fichier `source/README.md` explique comment les compiler et les tester.

## Auteur, support et licence

Créé par **[ronael](https://github.com/ronael)** (npm : [ronaeld](https://www.npmjs.com/~ronaeld)). [Code source et contribution](https://github.com/ronael/portloom) · [Signaler un problème](https://github.com/ronael/portloom/issues) · [Sécurité](https://github.com/ronael/portloom/security/policy). Joins la version, ton système et les étapes de reproduction, en retirant les informations privées.

Distribué sous **licence MIT**. Le texte complet est inclus dans le fichier `LICENSE`.
