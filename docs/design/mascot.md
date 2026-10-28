# Pip, Portloom's terminal companion

## Reference lock

The existing Portloom design system remains the primary reference: dark canvas, native monospace cells, compact density, cyan identity, amber confirmation and red errors. The user's Claude Code reference contributes a small character beside the product identity, not Claude's character shape or orange palette. Pip is an original network-plug silhouette with two contact pins.

| Decision | Reference | Application |
| --- | --- | --- |
| A small character beside the wordmark | User brief referencing Claude Code's terminal character; [Claude Code](https://github.com/anthropics/claude-code) | A 7 × 3 cell silhouette occupies existing header space. No welcome screen or extra vertical rows. |
| Keep density, typography and semantic colors | Existing `design.rs` tokens | Cyan at rest/work, muted while paused, amber for confirmation, red for errors/force confirmation. |
| Expressions explain application state | Refero motion guidance: motion must provide feedback, continuity or hierarchy | Eyes change on scans, pause, search and confirmation. Textual status remains the source of information. |
| Render only when application state changes | [Ratatui rendering](https://ratatui.rs/concepts/rendering/) | No autonomous animation timer, extra worker, network access or dependency. |
| Fit in every supported terminal | Existing 60 × 20 minimum | Reserve nine columns within the existing header. Keep the brand, counts, table and confirmation controls readable. |

The mascot is presentation-only. It cannot trigger actions and does not replace a status label. Demo screenshots cover wide, compact, minimum, paused, busy and error states in addition to the bilingual interface. No image protocol, raster asset, emoji font or terminal extension is required.
