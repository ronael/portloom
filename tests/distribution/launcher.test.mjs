import test from 'node:test';
import assert from 'node:assert/strict';
import {chmodSync,copyFileSync,mkdirSync,mkdtempSync,rmSync,symlinkSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
function fixture(fn) {
  const dir=mkdtempSync(path.join(tmpdir(),'portloom npm test '));
  try {
    const platform=`${process.platform}-${process.arch}`;mkdirSync(path.join(dir,'package/bin'),{recursive:true});mkdirSync(path.join(dir,'package/native',platform),{recursive:true});mkdirSync(path.join(dir,'links'));
    copyFileSync(path.join(root,'distribution/npm/bin/portloom'),path.join(dir,'package/bin/portloom'));chmodSync(path.join(dir,'package/bin/portloom'),0o755);
    const binary=path.join(dir,'package/native',platform,'portloom');writeFileSync(binary,'#!/bin/sh\nprintf \'<%s>\\n\' "$@"\nexit 37\n');chmodSync(binary,0o755);
    symlinkSync('../package/bin/portloom',path.join(dir,'links/portloom'));symlinkSync('portloom',path.join(dir,'links/portscope'));fn(dir);
  } finally {rmSync(dir,{recursive:true,force:true});}
}
test('npm symlinks, paths with spaces, literal arguments and exit status survive exec',()=>fixture(dir=>{
  const result=spawnSync(path.join(dir,'links/portscope'),['--demo','arg with spaces','$(echo unsafe)','`echo unsafe`'],{encoding:'utf8'});
  assert.equal(result.status,37);assert.equal(result.stdout,'<--demo>\n<arg with spaces>\n<$(echo unsafe)>\n<`echo unsafe`>\n');
}));
test('unsupported platform fails visibly before trying another binary',()=>fixture(dir=>{
  const mock=path.join(dir,'mock');mkdirSync(mock);writeFileSync(path.join(mock,'uname'),'#!/bin/sh\necho Unknown\n');chmodSync(path.join(mock,'uname'),0o755);
  const result=spawnSync(path.join(dir,'links/portloom'),[],{encoding:'utf8',env:{...process.env,PATH:`${mock}:${process.env.PATH}`}});assert.equal(result.status,1);assert.match(result.stderr,/unsupported system/);
}));
