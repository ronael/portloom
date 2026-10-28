# Direction conservée — Portloom

Direction : interface terminal sombre et compacte, centrée sur la séparation projets/services/apps/système. Les catégories, leurs indices et l’impact de l’arrêt sont visibles avant toute action.

Référence principale : [OpenCode](https://opencode.ai/docs/themes/) pour le canevas sombre, la densité et les raccourcis visibles. Référence secondaire : [btop](https://github.com/aristocratos/btop) uniquement pour le tableau dominant et l’inspecteur latéral. Le retour utilisateur sur la difficulté à distinguer sites et services impose les catégories explicites, leur raison et l’impact de l’arrêt.

| Décision | Application native |
|---|---|
| Profondeur sombre | Canvas #101216, surface #171a20, overlay #20242c |
| Accent cyan | #79dbc8 pour projet, focus et sélection des vues |
| Prudence explicite | Ambre #f2c47b, rouge #f28f95 pour forcer |
| Information principale | Port, protocole, usage, nom du projet/service, action |
| Détail continu | Inspecteur de 38 colonnes dès 110 ; Entrée en petit écran |
| Confirmation | Tous les ports du PID ; O pour projet, PID + Entrée sinon |
| Terminal minimal | 60 × 20 ; en dessous, aucune action d’arrêt au clavier |
| Zéro image de décoration | Traits, textes et couleurs natifs Ratatui |

Validation : rendus natifs Ratatui à 140×40, 80×24 et 60×20, vue Système et confirmation sensible ; tests de focus, scrolling, annulation, saisie PID, Unicode et contrôles ANSI. La capture publique est dans `docs/assets/preview.png` ; `npm run preview` produit les rendus détaillés dans `.tooling/previews`.
