import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
const root = resolve('../..');
const files = ['docs/README.md', ...['docs/guidelines', 'docs/templates'].flatMap(dir => readdirSync(resolve(root, dir)).filter(f => f.endsWith('.md')).map(f => `${dir}/${f}`))].sort();
const entries = files.map(name => ({ name, body: readFileSync(resolve(root, name), 'utf8') }));
writeFileSync('generated.ts', `// Generated at build time from repository documents.\nexport const version: string = "0.0.0-comparison";\nexport const names: string[] = ${JSON.stringify(entries.map(x => x.name))};\nexport const bodies: string[] = ${JSON.stringify(entries.map(x => x.body))};\n`);
writeFileSync("bundled.json", JSON.stringify(entries));
console.log(`Embedded ${entries.length} documents (${entries.reduce((n, x) => n + Buffer.byteLength(x.body), 0)} UTF-8 bytes)`);
