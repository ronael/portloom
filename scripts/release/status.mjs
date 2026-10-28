import {readFileSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const {name} = JSON.parse(readFileSync(path.join(root, 'distribution/npm/metadata.json'), 'utf8'));
const version = /^version = "([^"]+)"/m.exec(readFileSync(path.join(root, 'Cargo.toml'), 'utf8'))[1];
const result = spawnSync('npm', ['view', name, 'version', 'dist-tags', 'versions', 'license', 'description', 'author', 'homepage', 'repository', 'bugs', 'keywords', '--json'], {encoding: 'utf8', timeout: 15000});
if (result.status !== 0) { console.error(result.error?.message || result.stderr); process.exit(1); }
const metadata = JSON.parse(result.stdout);
const versions = Array.isArray(metadata.versions) ? metadata.versions : [metadata.versions].filter(Boolean);
const latest = metadata['dist-tags']?.latest || metadata.version;
const status = {name, localVersion: version, latest, published: versions.includes(version), latestIsLocalVersion: latest === version, metadata};
console.log(JSON.stringify(status, null, 2));
if (!status.published) {
  console.error(latest === '0.0.0-stage'
    ? 'npm affiche encore son placeholder. La version locale n’est pas disponible publiquement. Consulte le résultat de publication et npm stage list portloom --json avant toute nouvelle action.'
    : `La version locale ${version} n’est pas publiée ; latest pointe sur ${latest}.`);
  process.exitCode = 2;
}
