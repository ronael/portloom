# Distribution npm

Le paquet et la commande s’appellent `portloom`. La racine du dépôt reste privée dans `package.json` afin d’empêcher sa publication accidentelle. Seule l’archive produite par `scripts/release/pack.mjs` est destinée au registre.

## Construire une livraison

```sh
npm run verify
npm run build
python3 scripts/dev/terminal_smoke.py
npm run pack
node scripts/release/smoke.mjs
npm publish ./dist/npm/portloom-1.0.1.tgz --dry-run --access public
```

Le générateur lit la version partagée dans `Cargo.toml` et les métadonnées dans `distribution/npm/metadata.json` : description, auteur, dépôt, accueil, issues, licence MIT et mots-clés. Il vérifie la version et le SHA-256 de chaque binaire et notice, puis crée le manifeste à partir d’une liste fermée de champs et de fichiers. Le README du paquet contient uniquement les plateformes embarquées.

Le paquet inclut un lanceur POSIX utilisant `exec`, les binaires et notices de bibliothèques, les sources Rust et tests sous `source/`, la licence, le README et les empreintes. Il n’a ni dépendance npm, ni script d’installation. Le test de distribution l’installe dans un préfixe temporaire et vérifie la commande, l’export JSON, les sources et les métadonnées.

## Plateformes

| Paquet natif | Cible Rust |
| --- | --- |
| darwin-arm64 | aarch64-apple-darwin |
| darwin-x64 | x86_64-apple-darwin |
| linux-arm64 | aarch64-unknown-linux-gnu |
| linux-x64 | x86_64-unknown-linux-gnu |

Le workflow `build.yml` compile et teste chaque cible nativement, puis rassemble les quatre résultats dans l’artefact `portloom-npm`. Il ne publie pas sur npm. Les binaires Linux sont construits sur Ubuntu 22.04 pour glibc ; musl et Windows ne sont pas pris en charge. Une compilation locale ne permet de revendiquer que sa propre plateforme.

## Publication par le mainteneur

Vérifier le commit, la CI et l’empreinte de l’archive finale avant publication. Ne pas reconstruire avec d’autres sources sous un tag de release existant.

```sh
npm publish ./dist/npm/portloom-1.0.1.tgz --access public
npm run release:status
```

La publication réelle est une action explicite du mainteneur. Le script `release:status` lit seulement le registre et retourne un code non nul si la version attendue n’est pas disponible. Une réponse de publication acceptée ne suffit pas à prouver que la version est distribuée.

Si npm affiche `0.0.0-stage` et « Temporary Holding Version », cette fiche correspond à une version d’attente. `npm view portloom` vérifie la version publique ; `npm stage list portloom` indique les publications en attente d’approbation manuelle accessibles au compte. Une liste vide ne permet pas de conclure qu’il faut une approbation manuelle : consulter aussi le résultat du `npm publish` et le statut du registre.

Références : [manifeste npm](https://docs.npmjs.com/cli/v11/configuring-npm/package-json/), [publication par étapes](https://docs.npmjs.com/staged-publishing/), [commande npm stage](https://docs.npmjs.com/cli/v11/commands/npm-stage/).
