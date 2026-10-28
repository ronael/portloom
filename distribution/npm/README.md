# Portloom

**Your ports, under control.** A native Rust terminal monitor for projects, applications and system services, with deliberate process stopping.

[Source & documentation](https://github.com/ronael/portloom) · [Français](https://github.com/ronael/portloom/blob/main/docs/i18n/README.fr.md)

![Portloom terminal interface](https://raw.githubusercontent.com/ronael/portloom/main/docs/assets/preview.png)

## Install

```sh
npm install --global portloom
portloom
```

The package includes native binaries. There are no npm dependencies, install scripts, extra downloads or compilation during installation. The launcher uses `exec`, so no Node or Bun process stays in memory.

### Platforms included in this archive

<!-- SUPPORTED_PLATFORMS -->

`lsof` and `ps` are required. macOS provides them; Linux may need `lsof` installed separately. Minimum terminal: **60 × 20**. Windows and Alpine/musl are not supported.

## English and French

English is the default. The first interactive launch offers English / Français, with a suggestion based on the terminal locale. Enter saves your choice; Escape continues once. There is no installation prompt and no picker in scripts, JSON, help, completions, demo mode or CI.

```sh
portloom --lang fr         # one invocation in French
portloom language fr       # remember French
portloom language en       # remember English
```

Press `L` to change language in the interface. Precedence: `--lang` → `PORTLOOM_LANG` → saved preference → English. Preferences use `$XDG_CONFIG_HOME/portloom/settings.json` or `~/.config/portloom/settings.json`; an absolute `PORTLOOM_CONFIG_DIR` overrides the directory. JSON keys and category identifiers are stable across languages.

## Projects and direct commands

```sh
portloom list --projects
portloom inspect 3000 --json
portloom stop 3000                 # review and confirm in a terminal
portloom stop 3000 --pid 8421       # select an owner if a port is shared
portloom watch 3000
portloom watch 3000 --json --count 3 --interval 1000
portloom probe 3000                # verify local HTTP on demand
portloom open 3000                 # verify, then open the browser
portloom --demo
portloom --help
portloom completions zsh
```

The TUI groups projects by directory and displays their Git branch when available. `1` opens the project view; `g` toggles grouping; Enter expands a group. Group rows cannot stop processes collectively.

`inspect`, `stop`, `probe` and `open` accept `--pid` and `--protocol tcp|udp`. Actions with multiple matching owners require a PID. Stopping requires interactive confirmation; piped input and JSON mode cannot authorize a stop.

## Keyboard

| Key | Action |
| --- | --- |
| `0`–`5` | All / Projects / Services / Apps / System / Unknown |
| `Tab` / `/` | Filter protocol / search |
| `↑↓` / `Enter` | Select / details or expand a group |
| `g` / `w` / `o` | Group projects / verify HTTP / verify and open |
| `k` / `K` | Prepare SIGTERM / SIGKILL |
| `L` / `Esc` | Choose language / cancel |
| `Space` / `r` | Pause / refresh |
| `?` / `q` / `Ctrl+C` | Help / quit |

## HTTP and process control

The HTTP probe sends HEAD / only to numeric loopback addresses, with a 2-second network deadline and an 8 KiB header limit. It sends no credentials or cookies and follows no redirects. UDP, HTTPS and ambiguous local HTTP ownership are not supported. A valid response confirms HTTP, not application health. Opening a browser requires an explicit action and a fresh identity check; demo mode performs no network or browser actions.

Stopping affects one process and all its ports. A project stop requires `Y` in English or `O` in French; sensitive categories require the exact PID. Identity, ownership, classification and newly opened ports are checked again before signalling. System processes, other users' processes, Portloom and its terminal are protected. SIGTERM never automatically escalates to SIGKILL. No process groups or containers are stopped collectively.

No telemetry or sudo escalation. Commands and JSON exports may contain private arguments: review them before sharing. Process visibility depends on permissions. Identity checks and signals are not atomic; a residual race remains, especially on macOS where start times have one-second precision.

## Source, support and license

Rust sources, manifests, the lockfile, toolchain definition and Rust tests are included in `source/`. `source/README.md` explains how to build them. Dependency notices are included in `native/`.

Created by [ronael](https://github.com/ronael), npm [ronaeld](https://www.npmjs.com/~ronaeld). [Report a bug](https://github.com/ronael/portloom/issues) or [report a vulnerability privately](https://github.com/ronael/portloom/security/advisories/new).

Released under the **MIT license**, included in `LICENSE`.
