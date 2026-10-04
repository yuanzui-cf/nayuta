import { join, resolve } from 'node:path';
import { root, run, platforms } from './distribution';

const platform = platforms.find((p) => p.target === process.argv[2]);
if (!platform) throw new Error('Supply a supported Rust target triple.');
const output = resolve(process.argv[3] ?? join(root, 'dist/cli'));
const version = (await Bun.file(join(root, 'package.json')).json()).version;
const directory = join(output, `${platform.platform}-${platform.arch}`);
const filename = `nayuta-${version}-${platform.target}.tar.gz`;
await run(['tar', '-czf', join(output, filename), '-C', directory, '.']);
const hash = new Bun.CryptoHasher('sha256')
  .update(await Bun.file(join(output, filename)).arrayBuffer())
  .digest('hex');
await Bun.write(join(output, `${filename}.sha256`), `${hash}  ${filename}\n`);
await run(['bun', 'pm', 'pack', '--destination', output], directory);
await run(
  ['bun', 'pm', 'pack', '--destination', output],
  join(output, 'wrapper'),
);
console.log(join(output, filename));
