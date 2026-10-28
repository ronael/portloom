const platformsByName = {
  'darwin-arm64': 'macOS · Apple Silicon (ARM64)',
  'darwin-x64': 'macOS · Intel (x64)',
  'linux-arm64': 'Linux · ARM64 · glibc (Ubuntu 22.04 or newer)',
  'linux-x64': 'Linux · x64 · glibc (Ubuntu 22.04 or newer)',
};
export function createManifest(metadata, version, platforms) {
  for (const field of ['name', 'description', 'license', 'homepage']) {
    if (typeof metadata[field] !== 'string' || !metadata[field].trim()) throw new Error(`Métadonnée manquante : ${field}`);
  }
  if (!metadata.author?.name || !metadata.author?.url) throw new Error('Auteur et profil public requis.');
  if (!metadata.bugs?.url && !metadata.bugs?.email) throw new Error('Contact de support requis.');
  if (!Array.isArray(metadata.keywords) || !metadata.keywords.length) throw new Error('Mots-clés requis.');
  if (!/^\d+\.\d+\.\d+(?:[-+][\w.-]+)?$/.test(version)) throw new Error('Version de release invalide.');
  for (const url of [metadata.homepage, metadata.author.url, metadata.bugs.url].filter(Boolean)) {
    if (new URL(url).protocol !== 'https:') throw new Error('Les liens publics doivent utiliser HTTPS.');
  }
  if (metadata.repository && (metadata.repository.type !== 'git' || !/^git\+https:\/\//.test(metadata.repository.url))) throw new Error('Le dépôt doit désigner une URL Git réelle en git+https.');
  if (!platforms.length || platforms.some(p => !platformsByName[p])) throw new Error('Plateforme de publication invalide.');
  const os = [...new Set(platforms.map(p => p.split('-')[0]))];
  const cpu = [...new Set(platforms.map(p => p.split('-')[1]))];
  for (const system of os) for (const arch of cpu) {
    if (!platforms.includes(`${system}-${arch}`)) throw new Error(`Combinaison annoncée sans binaire : ${system}-${arch}`);
  }
  const {name, description, license, author, homepage, bugs, keywords, repository} = metadata;
  return {
    name, version, description, license, author, homepage, bugs,
    ...(repository ? {repository} : {}), keywords,
    bin: {portloom: 'bin/portloom'},
    files: ['bin/', 'native/', 'source/', 'README.md', 'LICENSE', 'SHA256SUMS'],
    os, cpu, publishConfig: {access: 'public', registry: 'https://registry.npmjs.org/'},
  };
}
export function renderReadme(template, platforms) {
  const marker = '<!-- SUPPORTED_PLATFORMS -->';
  if (!template.includes(marker)) throw new Error('Bloc des plateformes manquant dans le README npm.');
  const table = ['| Platform included in this release |', '| --- |', ...platforms.map(p => `| ${platformsByName[p]} |`)];
  return template.replace(marker, table.join('\n'));
}
