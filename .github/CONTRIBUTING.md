# Contributing

Bug reports and feature proposals are welcome through GitHub issues. Describe the problem, a concrete example and how the change can be validated. Discussions may be in English or French.

## Development

Install rustup, Node 22 or newer, Python 3 and `lsof`. `rust-toolchain.toml` pins Rust. Scripts use the optional isolated `.tooling` toolchain without changing global tools. No npm packages are needed for development.

```sh
npm run verify
npm run build
python3 scripts/dev/terminal_smoke.py
python3 scripts/dev/language_smoke.py
npm run pack
node scripts/release/smoke.mjs
```

## Conventions

- Keep domain and application free of system access; inject dependencies through focused ports.
- Keep at most ten files directly inside each source directory, grouped by responsibility.
- Test business rules and regressions. Integration tests may only stop their own fixtures.
- Preserve confirmations, system protections, keyboard access and terminal restoration.
- Use the existing Ratatui design tokens.
- English is the default; keep French messages and interpolation fields aligned. Domain decisions must not depend on translated strings. Keep JSON keys stable.
- Never prompt during installation or in scripts. Persist preferences only after an explicit user action.
- Update documentation when commands, platforms or public behavior change.
- Never commit build caches, binaries, real process exports, secrets or personal contact addresses.

## Pull requests

Explain the problem, resulting behavior and validation. CI tests macOS and Linux on ARM64 and x64; dependency updates follow the same checks. Contributions are covered by the project's MIT license.
