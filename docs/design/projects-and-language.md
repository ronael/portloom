# Project groups and language selection

The build target is the existing Portloom terminal design system: dark canvas, compact native text, cyan focus/project identity, amber deliberate actions and red SIGKILL confirmation. No new decorative assets or palette changes.

| Decision | Reference | Application |
| --- | --- | --- |
| Keep the dominant table and right inspector | Existing Portloom; OpenCode/btop direction in `direction.md` | Group headers use the raised surface; the inspector remains 38 columns on wide terminals. |
| Indent hierarchy through explicit disclosure markers | [procs tree view](https://github.com/dalance/procs#tree-view) | Project headers have expand/collapse state and port counts. Full directory identity prevents same-name projects merging. |
| Preserve visible focus and deliberate confirmation | Refero craft details; existing stop flow | Group headers expose navigation only. Stops require an individual process, then the existing confirmation. |
| Ask only in an interactive session | [CLI guidelines](https://clig.dev/#interactivity) | Language selection occurs on first TTY launch, never during npm installation or script execution. |
| English fallback, explicit persistent choice | User requirement | A terminal locale suggests a menu selection; only confirmation saves it. `--lang` and the environment override storage. |
| Side effects must be explicit | Existing product contract | `w` sends a bounded local HTTP HEAD request; `o` verifies and opens. Demo adapters perform neither side effect. |

Native render verification covers English, French, expanded groups, a small terminal, sensitive-stop confirmation and the language picker. Public images use demo data only. The palette and text roles remain defined in `src/presentation/design.rs`.
