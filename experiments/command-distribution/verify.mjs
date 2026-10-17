import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync, readdirSync, mkdirSync, writeFileSync, mkdtempSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { tmpdir } from 'node:os';
const cwd = resolve('.');
const root = resolve('../..');
const binaries = ['bun-host', 'scriptc-host', 'rust-host'].map(name => resolve('dist', name));
const clean = mkdtempSync(join(tmpdir(), 'sdev-comparison-'));
const invoke = (bin, args, expected = 0) => {
  const out = spawnSync(bin, args, { cwd: clean, env: { PATH: '/usr/bin:/bin', HOME: clean }, encoding: 'utf8', timeout: 15000 });
  assert.equal(out.status, expected, `${bin}: ${args}: ${out.stderr}`);
  return out.stdout;
};
const list = dir => readdirSync(dir, {withFileTypes: true}).flatMap(d => d.isDirectory() ? list(join(dir, d.name)) : d.name.endsWith('.md') ? [join(dir, d.name)] : []);
const files = [join(root, 'README.md'), join(root, 'AGENTS.md'), ...list(join(root, 'docs'))];
const outputs = [];
const parseRecords = text => text.trim().split('\n').filter(Boolean).map(JSON.parse);
for (const bin of binaries) {
  assert.match(invoke(bin, ['--help']), /docs/);
  assert.equal(invoke(bin, ['--version']).trim(), '0.0.0-comparison');
  const names = invoke(bin, ['docs']).trim().split('\n');
  assert.equal(names.length, 10);
  for (const name of names) assert.equal(invoke(bin, ['docs', name]), readFileSync(join(root, name), 'utf8') + '\n');
  invoke(bin, ['docs', 'nonexistent'], 2);
  invoke(bin, ['check', join(clean, 'nonexistent.md')], 2);
  outputs.push(invoke(bin, ['check', ...files]));
}
for (const output of outputs.slice(1)) assert.deepEqual(parseRecords(output), parseRecords(outputs[0]));
const links = outputs[0].trim().split('\n').map(JSON.parse);
assert(links.every(x => x.exists));
writeFileSync(join(clean, 'exists.md'), '# exists');
writeFileSync(join(clean, 'with space.md'), '# exists');
const fixture = [
  '[reference][ref]', '[ref]: exists.md', '![image](exists.md)', '[nested **text**](exists.md)',
  '[escaped](with%20space.md)', '`[inline](not-a-link)`', '```md', '[code](not-a-link)', '```',
  '[external](https://example.com)', '[anchor](#section)', '[missing](missing.md)'
].join('\n\n');
writeFileSync(join(clean, 'fixture.md'), fixture);
const fixtureOutputs = binaries.map(bin => invoke(bin, ['check', join(clean, 'fixture.md')], 1));
for (const output of fixtureOutputs.slice(1)) assert.deepEqual(parseRecords(output), parseRecords(fixtureOutputs[0]));
const records = fixtureOutputs[0].trim().split('\n').map(JSON.parse);
assert.equal(records.length, 5);
assert.equal(records.filter(x => !x.exists).length, 1);
for (const bin of [binaries[0], binaries[2]]) {
  const result = JSON.parse(invoke(bin, ['profile', join(root, 'docs/README.md'), join(root, 'docs/products/sdev/product.md')]));
  assert.equal(result.files, 2);
  assert.equal(result.missing, 0);
  const expected = parseRecords(invoke(bin, ['check', join(root, 'docs/README.md'), join(root, 'docs/products/sdev/product.md')])).length;
  assert.equal(result.links, expected);
  for (const key of ['read_ms','parse_ms','references_ms','total_ms']) assert(result[key] >= 0 && Number.isFinite(result[key]));
}
const summary = { files: files.length, links: links.length, bundledDocuments: 10, fixtureLinks: records.length, isolatedCwd: clean, checks: 'All three binaries agree; help/version/docs/errors/reference links/images/nesting/percent paths/code exclusion verified' };
mkdirSync('results', { recursive: true });
writeFileSync('results/verification.json', JSON.stringify(summary, null, 2));
console.log(JSON.stringify(summary, null, 2));
