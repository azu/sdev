import { spawnSync, execFileSync } from 'node:child_process';
import { copyFileSync, writeFileSync, statSync } from 'node:fs';
import { performance } from 'node:perf_hooks';
const rustCompiler = execFileSync("rustup", ["which", "--toolchain", "stable", "rustc"], {encoding:"utf8"}).trim();
const targets = ['x86_64-apple-darwin', 'aarch64-unknown-linux-gnu', 'x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc'];
const results = [];
for (const target of targets) {
  const command = ['rustup','run','stable','cargo','build','--release','--locked','--manifest-path','rust/Cargo.toml','--target',target];
  const start = performance.now();
  const output = spawnSync(command[0],command.slice(1),{encoding:'utf8',timeout:120000,env:{...process.env,RUSTC:rustCompiler}});
  writeFileSync(`results/rust-${target}.log`, `${output.stdout ?? ''}${output.stderr ?? ''}${output.error ?? ''}`);
  const result = { target, command, exit:output.status, elapsed_ms:+(performance.now()-start).toFixed(3), executed:false };
  if (output.status === 0) {
    const extension = target.includes('windows') ? '.exe' : '';
    const path = `dist/rust-${target}${extension}`;
    copyFileSync(`rust/target/${target}/release/sdev-distribution-rust-comparison${extension}`,path);
    result.path=path;result.bytes=statSync(path).size;
  } else {
    result.diagnostic = (output.stderr ?? '').split('\n').filter(line=>/error|linker|linking|not found|unsupported/i.test(line)).slice(0,8);
  }
  results.push(result);console.log(JSON.stringify(result));
}
writeFileSync('results/rust-cross.json', JSON.stringify(results,null,2));
