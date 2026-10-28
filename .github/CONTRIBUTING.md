# Contribuer

Les bugs et propositions sont bienvenus dans les issues. Pour un changement de comportement, décrire le problème, un exemple concret et la validation prévue. Les échanges peuvent être en français ou en anglais.

## Environnement

Installer rustup, Node 22 ou plus récent, Python 3 et `lsof`. Le projet fixe Rust dans `rust-toolchain.toml`. Les scripts sélectionnent une éventuelle toolchain locale dans `.tooling` sans modifier les outils globaux. Aucun paquet npm n’est nécessaire au développement.

```sh
npm run verify
npm run build
python3 scripts/dev/terminal_smoke.py
npm run pack
node scripts/release/smoke.mjs
```

## Conventions

- Garder le domaine et les cas d’usage sans accès système ; injecter leurs dépendances par les ports existants ou un nouveau contrat justifié.
- Conserver au plus dix fichiers directement par dossier de sources ; regrouper par responsabilité.
- Ajouter un test pour une règle métier ou une régression. Les tests d’intégration ne doivent arrêter que leurs propres fixtures.
- Préserver les confirmations, protections système, raccourcis et restauration du terminal.
- Utiliser les tokens du design system pour les composants Ratatui.
- Mettre à jour les documents si les commandes, plateformes ou comportements publics changent.
- Ne pas suivre `.tooling`, `dist`, les binaires, les exports de processus réels ou les secrets.

## Pull requests

Expliquer le problème résolu, le comportement obtenu et les vérifications effectuées. Le workflow teste macOS et Linux sur ARM64 et x64. Les mises à jour de dépendances suivent les mêmes contrôles. Les contributions sont proposées sous la licence MIT du projet.
