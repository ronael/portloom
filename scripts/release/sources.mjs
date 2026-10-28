import {copyFileSync, cpSync, mkdirSync, readdirSync, writeFileSync} from 'node:fs';
import path from 'node:path';
export function includeSources(root, stage) {
  const destination = path.join(stage, 'source');
  mkdirSync(destination);
  for (const name of ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'LICENSE']) copyFileSync(path.join(root, name), path.join(destination, name));
  for (const directory of ['src', 'crates', 'tests', 'examples']) {
    cpSync(path.join(root, directory), path.join(destination, directory), {
      recursive: true,
      filter: file => !path.extname(file) || file.endsWith('.rs') || path.basename(file) === 'Cargo.toml',
    });
  }
  // npm-specific tests require the development scripts; only Rust tests belong here.
  const sourceFiles = [];
  function collect(directory) {
    for (const entry of readdirSync(directory, {withFileTypes: true})) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) collect(file);
      else sourceFiles.push(path.relative(destination, file));
    }
  }
  collect(destination);
  if (!sourceFiles.includes('src/main.rs') || !sourceFiles.includes('Cargo.lock')) throw new Error('Sources Rust incomplètes.');
  writeFileSync(path.join(destination, 'README.md'), `# Portloom source

This release includes the Rust source under the MIT license.

## Build

Install rustup; rust-toolchain.toml selects the toolchain:

\x60\x60\x60sh
cargo build --locked --release --bin portloom
./target/release/portloom --demo
\x60\x60\x60

macOS or Linux is required. The scanner uses lsof and ps. Tests also need Python 3 for network fixtures:

\x60\x60\x60sh
cargo test --locked --workspace --all-targets
\x60\x60\x60

Tests only stop their own fixtures. No build caches, historical versions or local machine data are included.
`);
}
