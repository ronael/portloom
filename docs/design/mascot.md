# Pip, Portloom's pixel companion

## Reference lock

The user requested the visual weight and animation of a filled terminal character like Claude Code's. Pip uses an original antenna, bevelled shell, two inset eyes, articulated arms and two contact-pin feet. The existing Portloom design system remains the primary reference for its dark canvas, monospace density and cyan identity; amber and red keep their confirmation/error roles.

| Decision | Reference | Application |
| --- | --- | --- |
| Filled, detailed pixel silhouette | User brief; [Claude Code terminal identity](https://github.com/anthropics/claude-code) | A compact 18 × 6 pixel sprite, rendered as 9 × 3 terminal cells with quarter-block characters. Thin eye cutouts, a lit upper edge and a shaded lower edge give it depth. |
| Expressive movement | User request; Refero motion guidance | A short hand wave, double blink and sideways glance while idle; stepping feet and a pulsing antenna during scans. Pause closes the eyes. Confirmation and errors freeze the pose. |
| Preserve semantic color roles | Existing `design.rs` | Cyan shell, derived light/shadow shades; muted pause, amber confirmation, red error or force-stop. Text remains authoritative. |
| Render changed poses only | [Ratatui rendering](https://ratatui.rs/concepts/rendering/) | The existing event loop computes deterministic poses. No animation thread or dependency. Working poses change at most once per 180 ms; idle poses have long still intervals. |
| Reduced motion | Refero motion/accessibility guidance | `a` toggles motion for the session; `--no-motion` or `PORTLOOM_NO_MOTION` starts with a still sprite. State expressions and textual feedback remain available. |
| Preserve the port table | Existing 60 × 20 minimum | Four header rows and two toolbar rows retain the previous combined height. The sprite never overlaps counts, the table or confirmation controls. |

The sprite and its timing are presentation-only. They cannot trigger application actions. Quarter-block rendering uses the terminal's regular character grid, without emoji fonts, images or terminal extensions. A clear row below the compact sprite separates its feet from navigation. Preview frames use demo data; the animated preview shows idle, scanning and pause behavior.
