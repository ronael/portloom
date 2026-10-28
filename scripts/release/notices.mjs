import {readFileSync,readdirSync,writeFileSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const [platform,metadata]=process.argv.slice(2);
if(!['darwin-arm64','darwin-x64','linux-arm64','linux-x64'].includes(platform)) throw new Error('Plateforme invalide');
const graph=JSON.parse(readFileSync(metadata,'utf8'));
const nodes=new Map(graph.resolve.nodes.map(n=>[n.id,n]));const reached=new Set();
function visit(id){if(reached.has(id))return;reached.add(id);for(const dep of nodes.get(id)?.dependencies||[])visit(dep);}
visit(graph.resolve.root);
const packages=graph.packages.filter(p=>p.source && reached.has(p.id)).sort((a,b)=>a.name.localeCompare(b.name));
const sections=['Portloom — third-party notices',`Platform: ${platform}`, 'License texts supplied with the dependencies used for this target.\n'];
for(const pkg of packages){
  const dir=path.dirname(pkg.manifest_path);
  const files=readdirSync(dir,{withFileTypes:true}).filter(f=>f.isFile() && /^(licen[cs]e|copying|notice|copyright|unlicense)/i.test(f.name)).map(f=>f.name).sort();
  if(!files.length)throw new Error(`Licence manquante : ${pkg.name} ${pkg.version}`);
  sections.push(`\n${'='.repeat(72)}\n${pkg.name} ${pkg.version}\nSPDX: ${pkg.license||'See license text'}\n`);
  for(const file of files)sections.push(`--- ${file} ---\n${readFileSync(path.join(dir,file),'utf8')}`);
}
writeFileSync(path.join(root,'dist/native',platform,'THIRD_PARTY_NOTICES.txt'),sections.join('\n'));
console.log(`Notices de ${packages.length} dépendances jointes pour ${platform}.`);
