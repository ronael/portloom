# Design system terminal

Source unique : `src/presentation/design.rs`. Tous les composants utilisent ses couleurs sémantiques, la suppression des séquences de contrôle, le troncage en cellules et le retour à la ligne Unicode. Typographie monospace du terminal, sans dépendance à Nerd Fonts.

Palette : canvas #101216 ; surface #171a20 ; raised #20242c ; sélection #243338 ; texte #e7e9ee ; secondaire #9ba6b7 ; discret #798598 ; bordure #323a47 ; accent #79dbc8 ; confirmation #a3d9a5 ; prudence #f2c47b ; danger #f28f95.

Espacement en cellules : marge extérieure 1, séparation des panneaux 1, ligne vide entre blocs d’information. Barre supérieure avec marque, état et compteurs. Historique compact : nombre d’endpoints observés, jamais débit réseau. Vues et recherche restent au-dessus du tableau ; messages et aide clavier restent en bas.

La couleur n’est jamais la seule information : SITE/API, PROJET, BASE, SERVICE, APPLI, SYSTÈME, INCONNU ; CONFIRMER, PRUDENCE, PROTÉGÉ. Le système explique son niveau de confiance et n’attribue pas la qualité de site à partir d’un port conventionnel.

Le focus de l’inspecteur est cyan. Les boutons d’arrêt sont ambre, le dialogue SIGKILL rouge. Seul le clic gauche déclenche une action. La recherche capture les raccourcis tant qu’elle est active. Les détails et confirmations ont un contenu défilable et une action fixe.
