# Pip, the Portloom bat

The owner selected the bat from their avatar studio and approved its Portloom palette before implementation. This is the build target: the exact 16 × 16 matrices, silhouette, eyes, wing shapes and frame durations. The earlier humanoid companion is superseded.

| Decision | Reference | Implementation |
| --- | --- | --- |
| Bat silhouette and movement | Owner-supplied `animaux/animations/chauve-souris.json` | 59 unchanged frames: arrival (1280 ms), rest (2000 ms), listening (1500 ms), alert (840 ms), departure (1200 ms). The source fingerprint is recorded in `animation/bat.rs`. |
| Palette | Approved `animaux/portloom-theme` preview; Portloom design tokens | Cyan `#79dbc8`, shadow `#56988c`, pale eyes `#e7e9ee`, canvas `#101216`; amber `#f2c47b` with shadow `#a68859` for confirmation. Existing red error/force-stop and muted pause roles are preserved. |
| Faithful terminal raster | Approved half-block preview | Each cell encodes two original pixels as foreground/background colors. The complete sprite occupies 16 columns × 8 rows, with no resampling, font-dependent emoji or terminal image extensions. |
| Small screens | Existing 60 × 20 minimum and accessible actions | At 24 rows or more, show the complete bat beside the title and counts. Shorter windows use a text header and preserve the port table and controls. The bat is never clipped into a different silhouette. |
| Motion | Approved frame timings; Refero motion guidance | Existing event loop, no animation thread or dependency. Equal pixel grids do not request a redraw. Startup plays arrival, activity selects listening, and idle loops rest. Confirmations freeze motion. |
| Reduced motion and exit | Existing `a`, `--no-motion`, `PORTLOOM_NO_MOTION` controls | Still poses keep state colors. A normal `q` plays the 1.2-second departure; any key skips it. Ctrl+C, OS signals, reduced motion and small screens exit without a farewell delay. |

Art and timing are pure presentation data. Rendering translates those pixels into Ratatui cells. Domain logic, process safety and operating-system adapters do not depend on the companion. English and French documentation and previews use this same imported artwork.

For future visual changes, show a preview and wait for the owner's approval before implementation.
