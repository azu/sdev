import { readFileSync, existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { marked } from 'marked';
import { names, bodies, version } from './generated';

function help(): void {
  console.log('Comparison prototype: --help | --version | docs [name] | check <markdown>... | profile <markdown>...');
}

function extractLinks(text: string): string[] {
  const links: string[] = [];
  const tokens = marked.lexer(text);
  marked.walkTokens(tokens, (token) => {
    if (token.type === 'link' || token.type === 'image') links.push(token.href);
  });
  return links;
}

type LinkRecord = { file: string; href: string; target: string; exists: boolean };
function inspectLinks(file: string, links: string[]): { records: LinkRecord[]; missing: number } {
  const records: LinkRecord[] = [];
  let missing = 0;
  for (let i = 0; i < links.length; i++) {
    const href = links[i];
    if (href.startsWith('#') || /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(href) || href.startsWith('//')) continue;
    const pathPart = href.split('#')[0];
    if (pathPart.length === 0) continue;
    const target = resolve(dirname(file), decodeURIComponent(pathPart));
    const exists = existsSync(target);
    records.push({ file, href, target, exists });
    if (!exists) missing++;
  }
  return { records, missing };
}

function main(): number {
  const count = process.argv.length;
  if (count < 3) { help(); return 0; }
  const command = process.argv[2];
  if (command === '--help') { help(); return 0; }
  if (command === '--version') { console.log(version); return 0; }
  if (command === 'docs') {
    if (count < 4) { console.log(names.join('\n')); return 0; }
    const name = process.argv[3];
    const index = names.indexOf(name);
    if (index < 0) { console.error('Unknown document: ' + name + '; use docs to list'); return 2; }
    const body: string = bodies[index];
    console.log(body);
    return 0;
  }
  if (command === 'check' && count > 3) {
    let missing = 0;
    for (let i = 3; i < count; i++) {
      const text = readFileSync(process.argv[i], 'utf8');
      const result = inspectLinks(process.argv[i], extractLinks(text));
      for (const record of result.records) console.log(JSON.stringify(record));
      missing += result.missing;
    }
    return missing > 0 ? 1 : 0;
  }
  if (command === 'profile' && count > 3) {
    let read_ms = 0, parse_ms = 0, references_ms = 0, links = 0, missing = 0;
    for (let i = 3; i < count; i++) {
      const t0 = performance.now();
      const text = readFileSync(process.argv[i], 'utf8');
      const t1 = performance.now();
      const hrefs = extractLinks(text);
      const t2 = performance.now();
      const result = inspectLinks(process.argv[i], hrefs);
      const t3 = performance.now();
      read_ms += t1 - t0; parse_ms += t2 - t1; references_ms += t3 - t2;
      links += result.records.length; missing += result.missing;
    }
    console.log(JSON.stringify({ read_ms, parse_ms, references_ms, total_ms: read_ms + parse_ms + references_ms, files: count - 3, links, missing }));
    return missing > 0 ? 1 : 0;
  }
  console.error('Unknown command or missing input; use --help');
  return 2;
}
try { process.exit(main()); }
catch (error) { console.error(String(error)); process.exit(2); }
