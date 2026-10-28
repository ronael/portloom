import {mkdtempSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const version=/^version = "([^"]+)"/m.exec(readFileSync(path.join(root,'Cargo.toml'),'utf8'))[1];
const archive=path.resolve(process.argv[2]||path.join(root,`dist/npm/portloom-${version}.tgz`));
const dir=mkdtempSync(path.join(tmpdir(),'portloom-install-'));
function run(cmd,args) {const r=spawnSync(cmd,args,{encoding:'utf8'});assert.equal(r.status,0,`${cmd}: ${r.stderr}`);return r.stdout;}
try {
  run('npm',['install','--global','--prefix',dir,'--offline','--ignore-scripts','--no-audit','--no-fund',archive]);
  const cli=path.join(dir,'bin/portloom');assert.equal(run(cli,['--version']).trim(),`portloom ${version}`);
  const json=JSON.parse(run(cli,['--demo','--json']));assert.equal(json.endpoints.length,11);
  const pkg=JSON.parse(readFileSync(path.join(dir,'lib/node_modules/portloom/package.json'),'utf8'));assert.deepEqual(Object.keys(pkg.bin),['portloom']);assert.equal(pkg.scripts,undefined);assert.equal(pkg.dependencies,undefined);
  const installed=path.join(dir,'lib/node_modules/portloom');
  const metadata=JSON.parse(readFileSync(path.join(root,'distribution/npm/metadata.json'),'utf8'));
  for(const field of ['description','license','author','homepage','bugs','keywords','repository']) assert.deepEqual(pkg[field],metadata[field],`Métadonnée manquante dans l’archive : ${field}`);
  const license=readFileSync(path.join(installed,'LICENSE'),'utf8');assert.match(license,/MIT License/);assert.match(license,/Copyright \(c\) 2026 ronaeld/);
  const readme=readFileSync(path.join(installed,'README.md'),'utf8');assert.ok(readme.includes('npm install --global portloom'));assert.ok(!readme.includes('<!-- SUPPORTED_PLATFORMS -->'));assert.ok(readme.includes('licence MIT'));
  assert.match(readFileSync(path.join(installed,'source/Cargo.toml'),'utf8'),new RegExp(`version = \"${version.replaceAll('.', '\\.')}\"`));
  assert.ok(readFileSync(path.join(installed,'source/src/main.rs'),'utf8').includes('composition::run'));
  assert.ok(readFileSync(path.join(installed,'source/crates/domain/src/lib.rs'),'utf8').includes('pub mod model'));
  assert.ok(readFileSync(path.join(installed,'source/crates/application/Cargo.toml'),'utf8').includes('portloom-domain'));
  assert.match(readFileSync(path.join(installed,'source/LICENSE'),'utf8'),/MIT License/);
  console.log('Archive npm : installation globale isolée, métadonnées, licence, README, commande native et JSON validés.');
} finally {rmSync(dir,{recursive:true,force:true});}
