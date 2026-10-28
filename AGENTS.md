# Portloom

- Rust 1.90.0, Ratatui 0.30.2, Crossterm ; conserver le lockfile Cargo.
- Le Rust local de `.tooling` est isolé. Ne pas modifier les outils globaux.
- Workspace : `crates/domain` pur ← `crates/application` et ses ports ← adaptateurs de `src/infrastructure` et `src/presentation` ; `src/composition` assemble les implémentations. Les dépendances du cœur sont vérifiées depuis les manifestes Cargo.
- Pas de dépendance présentation → infrastructure. Pas d’IO, de framework UI ni de signal OS dans domain/application.
- Maximum dix fichiers directement par dossier, hors dépendances et sorties générées `.tooling`, `target`, `dist`, `node_modules`, `.git`.
- Tout arrêt passe par StopProcess et une confirmation explicite. PID exact obligatoire pour les catégories sensibles ; services système bloqués.
- Pas de sudo dans l’application, de signaux à un groupe, d’escalade automatique ou de télémétrie.
- Les tests réels arrêtent uniquement leurs propres fixtures et les nettoient avec un garde RAII.
- Tokens visuels dans `src/presentation/design.rs`. Référence et décisions dans `docs/design`.
- Pour les futurs changements visuels, montrer un aperçu puis attendre la validation du propriétaire avant l’intégration. La chauve-souris et sa palette dans `docs/design/mascot.md` sont validées.
- `npm run verify` contrôle architecture, formatage, Clippy, tests Rust et lanceur npm. `npm run preview` produit les rendus Ratatui.
- `npm run build`, `npm run pack`, puis `node scripts/release/smoke.mjs` vérifient la livraison locale.
- La racine npm est privée ; seule l’archive construite dans `dist/npm` est destinée à publication. Ne jamais publier implicitement.
- Les archives locales éventuelles dans `.tooling/legacy-typescript` et les rapports dans `.tooling/verification-history` ne font pas partie du dépôt public. Ne pas les supprimer ni les publier.
- Les contacts publics du mainteneur passent par les issues et le signalement privé GitHub ; ne pas ajouter d’adresse personnelle.
- Anglais par défaut ; traductions anglais/français dans presentation/i18n. Le domaine transporte des raisons typées, jamais des messages localisés. Garder les noms de champs JSON stables.
- Le choix de langue intervient au premier lancement interactif, jamais à l’installation npm. Aucun prompt en script, CI, JSON ou démo ; ne rien enregistrer sans action explicite.
- HTTP est une action explicite limitée aux adresses numériques loopback : HEAD, délai réseau total de deux secondes, en-têtes bornés, aucun suivi de redirection ni détection HTTPS implicite. Revalider le propriétaire avant une ouverture navigateur.
