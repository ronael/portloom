# Décisions et références

## Trois crates, dépendances vers le métier

Le domaine et les cas d’usage ont des manifestes indépendants. Le workspace partage version, toolchain, règles de lint et lockfile. Les adaptateurs et l’interface restent réunis dans la crate exécutable, avec des frontières vérifiées en CI. Cela donne des contraintes de compilation au cœur sans multiplier les paquets.

Références : [Cargo workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html), [modules et visibilité Rust](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html), [règle de dépendance de la Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html).

## Injection explicite

Des traits étroits représentent la lecture, le contrôle des processus et le temps. La composition injecte leurs implémentations dans le cas d’usage. Les doubles de test couvrent les changements d’identité, les permissions, les nouveaux ports et la confirmation sans envoyer de signal réel. Aucun conteneur d’injection n’est nécessaire.

## Builds reproductibles et maintenance

`Cargo.lock` est suivi car Portloom livre un exécutable. La version Rust est fixée dans `rust-toolchain.toml` ; les commandes CI utilisent `--locked`. Dependabot propose les mises à jour sans fusion automatique. Le maximum de dix fichiers par dossier est une convention de ce projet, pas une règle universelle de SOLID.

Référence : [Cargo.toml et Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html).

## CI et distribution

Les actions GitHub sont figées par SHA complet, le token CI possède seulement `contents: read`, les identifiants du checkout ne sont pas conservés, les jobs ont une durée maximale. Les quatre cibles sont testées nativement. L’archive npm contient les binaires et sources, sans dépendance npm ni script d’installation ; les plateformes sont déduites des binaires présents et vérifiés par SHA-256.

Références : [sécurité des workflows GitHub](https://docs.github.com/en/actions/reference/security/secure-use), [runners GitHub](https://docs.github.com/en/actions/reference/runners/github-hosted-runners), [manifeste npm](https://docs.npmjs.com/cli/v11/configuring-npm/package-json/).

## Dépôt public et licence

README, capture, guide de contribution, politique de sécurité, formulaires d’issues, description, sujets et tags rendent le projet identifiable et utilisable. Les artefacts locaux, anciennes versions et secrets ne sont pas suivis. MIT autorise notamment l’utilisation commerciale et la modification sous réserve de conserver les mentions requises.

Références : [README GitHub](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes), [sujets GitHub](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/classifying-your-repository-with-topics), [licence MIT](https://choosealicense.com/licenses/mit/).
