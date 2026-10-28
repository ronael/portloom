# Pip, Portloom's pixel companion

## Reference lock

The user requested the visual weight, fine pixels and animation of a filled terminal character like Claude Code's. The [official terminal screenshot](https://www.anthropic.com/news/enabling-claude-code-to-work-more-autonomously) is the silhouette reference: a broad filled body, spaced inset eyes, tiny extremities and quiet surrounding space. Portloom's dark canvas, monospace density and cyan identity remain the primary product reference; amber and red keep their confirmation/error roles.

Three native-grid studies compared a broad pocket companion, a taller rounded shell and a domed socket. The pocket companion was selected: its face reads clearly at actual terminal size and its 9 × 3 footprint preserves the table. Pip has softened corners, two contact-pin feet and independently articulated hands. The forehead is flat; a thin highlight and shaded feet provide depth without obscuring the eyes. Refero's icon guidance informs optical weight and consistent grid; its motion guidance informs brief gestures and long still intervals. No terminal image extension or extra dependency is required.

| Decision | Reference | Application |
| --- | --- | --- |
| Filled, readable silhouette | User brief; official Claude Code terminal screenshot | A compact 18 × 6 pixel sprite, rendered as 9 × 3 terminal cells with quarter-block characters. A broad rounded shell, two distinct eye cutouts and a thin forehead highlight replace the pointed silhouette. |
| Expressive movement | User request; Refero motion guidance | A short hand wave followed by a wink, a double blink and sideways glances while idle; stepping feet and a forehead glint during scans. Hands remain outside the face. Pause closes both eyes. Confirmation and errors freeze the pose with a small concerned mouth. |
| Preserve semantic color roles | Existing `design.rs` | Cyan shell, derived light/shadow shades; muted pause, amber confirmation, red error or force-stop. Text remains authoritative. |
| Render changed poses only | [Ratatui rendering](https://ratatui.rs/concepts/rendering/) | The existing event loop computes deterministic poses. No animation thread or dependency. Working poses change at most once per 180 ms; idle poses have long still intervals. |
| Reduced motion | Refero motion/accessibility guidance | `a` toggles motion for the session; `--no-motion` or `PORTLOOM_NO_MOTION` starts with a still sprite. State expressions and textual feedback remain available. |
| Preserve the port table | Existing 60 × 20 minimum | Four header rows and two toolbar rows retain the previous combined height. The sprite never overlaps counts, the table or confirmation controls. |

The sprite and its timing are presentation-only. They cannot trigger application actions. Quarter-block rendering uses the terminal's regular character grid, without emoji fonts, images or terminal extensions. A clear row below the compact sprite separates its feet from navigation. Preview frames use demo data; the animated preview shows idle, scanning and pause behavior.
