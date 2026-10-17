import fs from 'node:fs';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const run = (name,args) => execFileSync(name,args,{encoding:'utf8'}).trim();
const artifacts=[];
for(const name of fs.readdirSync('dist').filter(name=>name.startsWith('bun-') || name.startsWith('rust-'))) {
 const path=`dist/${name}`;const format=run('file',[path]);const record={path,bytes:fs.statSync(path).size,format,sha256:createHash('sha256').update(fs.readFileSync(path)).digest('hex')};
 if(format.includes('Mach-O')) {
  const commands=run('otool',['-l',path]).split('\n');
  record.buildVersion=commands.flatMap((line,index)=>(line.includes('cmd LC_BUILD_VERSION') || line.includes('cmd LC_VERSION_MIN_MACOSX')) ? [commands.slice(index,index+7).join('\n')] : []);
  record.linkedLibraries=run('otool',['-L',path]);
 }
 artifacts.push(record);
}
const rustCompiler=run('rustup',['which','--toolchain','stable','rustc']);
const metadata={host:run('sw_vers',[]),architecture:run('uname',['-m']),bun:run('bun',['--version']),node:run('node',['--version']),rustCompiler,rust:run(rustCompiler,['-vV']),cargo:run('rustup',['run','stable','cargo','-vV']),targets:run('rustup',['target','list','--installed']),artifacts};
fs.writeFileSync('results/artifacts-rust.json',JSON.stringify(metadata,null,2));
console.log(JSON.stringify({rust:metadata.rust,artifacts:artifacts.map(({path,bytes,format,buildVersion})=>({path,bytes,format,buildVersion}))},null,2));
