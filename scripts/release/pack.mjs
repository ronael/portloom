import {chmodSync, copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {createManifest, renderReadme} from './manifest.mjs';
import {includeSources} from './sources.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const allowed=['darwin-arm64','darwin-x64','linux-arm64','linux-x64'];
const requested=process.argv.slice(2);
if(requested.some(a=>!a.startsWith('--platforms=')) || requested.length>1) throw new Error('Usage : node scripts/release/pack.mjs [--platforms=darwin-arm64,darwin-x64,linux-arm64,linux-x64]');
const source=path.join(root,'dist/native');
const platforms=requested.length?requested[0].slice('--platforms='.length).split(','):existsSync(source)?readdirSync(source).filter(p=>allowed.includes(p)):[];
if(!platforms.length || platforms.some(p=>!allowed.includes(p)) || new Set(platforms).size!==platforms.length) throw new Error('Aucun binaire valide. Exécute npm run build.');
const osSet=[...new Set(platforms.map(p=>p.split('-')[0]))];
const cpuSet=[...new Set(platforms.map(p=>p.split('-')[1]))];
for (const os of osSet) for (const cpu of cpuSet) {
  if(!platforms.includes(`${os}-${cpu}`)) throw new Error(`Matrice incomplète : ${os}-${cpu} absent. Le manifeste npm ne doit pas annoncer une combinaison manquante.`);
}
const version=/^version = "([^"]+)"/m.exec(readFileSync(path.join(root,'Cargo.toml'),'utf8'))[1];
const metadata=JSON.parse(readFileSync(path.join(root,'distribution/npm/metadata.json'),'utf8'));
const manifest=createManifest(metadata,version,platforms);
const stage=mkdtempSync(path.join(tmpdir(),'portloom-package-')); const output=path.join(root,'dist/npm');mkdirSync(output,{recursive:true});
const checksums=[];
try {
  for(const platform of platforms) {
    const binary=path.join(source,platform,'portloom');const metadata=JSON.parse(readFileSync(path.join(source,platform,'build.json'),'utf8'));
    const digest=createHash('sha256').update(readFileSync(binary)).digest('hex');
    const notices=readFileSync(path.join(source,platform,'THIRD_PARTY_NOTICES.txt'));
    if(createHash('sha256').update(notices).digest('hex')!==metadata.noticesSha256) throw new Error(`Notices périmées ou altérées : ${platform}`);
    if(metadata.platform!==platform || metadata.version!==version || metadata.sha256!==digest) throw new Error(`Binaire périmé ou altéré : ${platform}. Recompile cette cible.`);
    const target=path.join(stage,'native',platform);mkdirSync(target,{recursive:true});copyFileSync(binary,path.join(target,'portloom'));chmodSync(path.join(target,'portloom'),0o755); checksums.push(`${digest}  native/${platform}/portloom`);
    writeFileSync(path.join(target,'THIRD_PARTY_NOTICES.txt'),notices);
  }
  cpSync(path.join(root,'distribution/npm/bin'),path.join(stage,'bin'),{recursive:true});chmodSync(path.join(stage,'bin/portloom'),0o755);
  includeSources(root,stage);
  const readme=renderReadme(readFileSync(path.join(root,'distribution/npm/README.md'),'utf8'),platforms);
  writeFileSync(path.join(stage,'README.md'),readme);
  copyFileSync(path.join(root,'LICENSE'),path.join(stage,'LICENSE'));
  if(manifest.license==='MIT' && !readFileSync(path.join(stage,'LICENSE'),'utf8').startsWith('MIT License')) throw new Error('Le texte de licence ne correspond pas au manifeste.');
  writeFileSync(path.join(stage,'SHA256SUMS'),checksums.join('\n')+'\n');
  writeFileSync(path.join(stage,'package.json'),JSON.stringify(manifest,null,2)+'\n');
  const result=spawnSync('npm',['pack','--offline','--ignore-scripts','--json','--pack-destination',output],{cwd:stage,encoding:'utf8'});
  if(result.status!==0) throw new Error(result.stderr||'npm pack a échoué');
  const pack=JSON.parse(result.stdout)[0];const archive=path.join(output,pack.filename);const hash=createHash('sha256').update(readFileSync(archive)).digest('hex');
  writeFileSync(path.join(output,'SHA256SUMS'),`${hash}  ${pack.filename}\n`);
  console.log(JSON.stringify({archive,platforms,bytes:pack.size,sha256:hash,files:pack.files.map(f=>f.path)},null,2));
} finally {rmSync(stage,{recursive:true,force:true});}
