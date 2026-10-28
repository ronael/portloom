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
  writeFileSync(path.join(destination, 'README.md'), `# Sources de Portloom\n\nLes sources Rust de cette version sont distribuées sous MIT.\n\n## Compiler\n\nAvec rustup installé, la chaîne est définie par rust-toolchain.toml :\n\n\x60\x60\x60sh\ncargo build --locked --release --bin portloom\n./target/release/portloom --demo\n\x60\x60\x60\n\nmacOS ou Linux requis ; le scanner utilise lsof et ps. Pour les tests : Python 3 est requis pour la fixture réseau ; cargo test --locked --workspace --all-targets. Les tests n’arrêtent que leurs propres fixtures.\n\nLes dossiers locaux, dépendances de compilation, anciennes versions et données du poste ne font pas partie de ces sources.\n`);
}
