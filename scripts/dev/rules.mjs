// Cargo enforces the crate graph; these checks guard the remaining module boundaries.
const allowedDependencies = new Map([
  ['portloom-domain', new Set(['regex'])],
  ['portloom-application', new Set(['portloom-domain'])],
]);

export function dependencyErrors(packages) {
  const errors = [];
  for (const [name, allowed] of allowedDependencies) {
    const pkg = packages.find(pkg => pkg.name === name);
    if (!pkg) { errors.push(`Crate manquante : ${name}`); continue; }
    for (const dep of pkg.dependencies) {
      if (!allowed.has(dep.name)) errors.push(`${name}: dépendance interdite ${dep.name}`);
    }
  }
  return errors;
}

export function sourceErrors(filename, source) {
  // A conservative guard, not a Rust parser. Compile-time crate isolation is authoritative.
  const code = source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
  const core = filename.startsWith('crates/domain/src/') || filename.startsWith('crates/application/src/');
  const forbidden = core ? ['infrastructure', 'presentation', 'composition', 'ratatui', 'crossterm', 'nix']
    : filename.startsWith('src/presentation/') ? ['infrastructure', 'composition']
    : filename.startsWith('src/infrastructure/') ? ['presentation', 'composition'] : [];
  const imports = [...code.matchAll(/\buse\s+([^;]+);/g)].map(match => match[1]).join('\n');
  const errors = [];
  for (const name of forbidden) {
    if (new RegExp(`\\b${name}\\s*::`).test(code) || new RegExp(`\\b${name}\\b`).test(imports)) {
      errors.push(`${filename}: dépendance interdite ${name}`);
    }
  }
  if (core) {
    const io = '(?:fs|io|net|process|env|thread|time|os)';
    const direct = new RegExp(`\\bstd\\s*::\\s*${io}\\b`);
    const grouped = [...code.matchAll(/\buse\s+std\s*::\s*\{([^;]+);/g)];
    if (direct.test(code) || grouped.some(match => new RegExp(`\\b${io}\\b`).test(match[1]))) {
      errors.push(`${filename}: accès système dans le cœur métier ; injecter un port`);
    }
    if (/\binclude_(?:str|bytes)!/.test(code)) errors.push(`${filename}: fichier externe embarqué dans le cœur métier`);
  }
  return errors;
}
