import {readdirSync, readFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {dependencyErrors, sourceErrors} from './rules.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const skip = new Set(['node_modules', '.git', '.tooling', 'target', 'dist', 'bin']);
const errors = [];
function walk(directory) {
  const entries = readdirSync(directory, {withFileTypes: true});
  const files = entries.filter(entry => entry.isFile() && !skip.has(entry.name));
  if (files.length > 10) errors.push(`${path.relative(root, directory) || '.'}: ${files.length} fichiers (maximum 10)`);
  for (const file of files.filter(file => file.name.endsWith('.rs'))) {
    const filename = path.join(directory, file.name);
    errors.push(...sourceErrors(path.relative(root, filename), readFileSync(filename, 'utf8')));
  }
  for (const entry of entries.filter(entry => entry.isDirectory() && !skip.has(entry.name))) walk(path.join(directory, entry.name));
}
walk(root);
const cargo = spawnSync('sh', ['scripts/dev/rust.sh', 'metadata', '--locked', '--offline', '--no-deps', '--format-version', '1'], {cwd: root, encoding: 'utf8'});
if (cargo.status !== 0) errors.push(cargo.stderr || 'Impossible de lire le graphe Cargo');
else errors.push(...dependencyErrors(JSON.parse(cargo.stdout).packages));
if (errors.length) { console.error(errors.join('\n')); process.exit(1); }
console.log('Architecture : graphe Cargo et frontières respectés, maximum 10 fichiers par dossier.');
