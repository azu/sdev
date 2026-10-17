import { spawnSync, execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync, statSync, copyFileSync, utimesSync } from 'node:fs';
import { performance } from 'node:perf_hooks';
mkdirSync('dist', { recursive: true });
mkdirSync('results', { recursive: true });
const rustCompiler = process.argv.includes("--rust") ? execFileSync("rustup", ["which", "--toolchain", "stable", "rustc"], {encoding:"utf8"}).trim() : "";
const results = [];
function fsCopyRust() { copyFileSync("rust/target/release/sdev-distribution-rust-comparison", "dist/rust-host"); }
function run(name, command, args, extraEnv = {}) {
  const start = performance.now();
  const out = spawnSync(command, args, { env: { ...process.env, ...extraEnv }, encoding: 'utf8', timeout: 120000 });
  writeFileSync(`results/${name}.log`, `${out.stdout ?? ''}${out.stderr ?? ''}${out.error ?? ''}`);
  const result = { name, command: [command, ...args], exit: out.status, elapsed_ms: +(performance.now()-start).toFixed(3) };
  results.push(result); console.log(JSON.stringify(result));
  return out.status;
}
run('prepare', 'node', ['prepare.mjs']);
const bunFlags = ['build','--compile','--minify','--bytecode','--no-compile-autoload-dotenv','--no-compile-autoload-bunfig'];
if (process.argv.includes('--rust')) {
  run('rust-release-initial', 'rustup', ['run','stable','cargo','build','--release','--locked','--manifest-path','rust/Cargo.toml'], {RUSTC: rustCompiler});
  if (results.at(-1).exit === 0) fsCopyRust();
  for (let i=1; i<=3; i++) {
    run(`bun-host-${i}`, 'bun', [...bunFlags,'cli.ts','--outfile','dist/bun-host']);
    const source = 'rust/src/main.rs'; const stat = statSync(source); utimesSync(source,stat.atime,new Date());
    run(`rust-host-${i}`, 'rustup', ['run','stable','cargo','build','--release','--locked','--manifest-path','rust/Cargo.toml']);
  }
  if (results.at(-1).exit === 0) fsCopyRust();
} else if (process.argv.includes('--cross')) {
  for (const target of ['bun-darwin-x64','bun-linux-x64','bun-linux-arm64','bun-windows-x64'])
    run(target, 'bun', [...bunFlags, `--target=${target}`, 'cli.ts','--outfile',`dist/${target}`]);
  for (const target of ['x86_64-apple-macos','x86_64-linux-gnu','aarch64-linux-gnu','x86_64-windows-gnu'])
    run(`scriptc-${target}`, 'node_modules/.bin/scriptc', ['build','cli.ts','--dynamic','--strip','-o',`dist/scriptc-${target}`], { SCRIPTC_TARGET: target });
} else {
  run('scriptc-static', 'node_modules/.bin/scriptc', ['build','cli.ts','--npm-static=auto','--strip','-o','dist/scriptc-native']);
  for (let i=1; i<=3; i++) {
    run(`bun-host-${i}`, 'bun', [...bunFlags,'cli.ts','--outfile','dist/bun-host']);
    run(`scriptc-host-${i}`, 'node_modules/.bin/scriptc', ['build','cli.ts','--dynamic','--optimization=release','--strip','-o','dist/scriptc-host']);
  }
}
writeFileSync(`results/build${process.argv.includes('--rust') ? '-rust' : process.argv.includes('--cross') ? '-cross' : ''}.json`, JSON.stringify(results, null, 2));

if (!process.argv.includes("--cross") && results.some(result => result.name !== "scriptc-static" && result.exit !== 0)) process.exitCode = 1;
