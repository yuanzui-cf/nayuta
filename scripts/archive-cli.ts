import { join, resolve } from 'node:path';
import { root, run, platforms } from './distribution';
import { npmTarballName, platformVersion } from './npm-release';

const platform = platforms.find((p) => p.target === process.argv[2]);
if (!platform) throw new Error('Supply a supported Rust target triple.');
const output = resolve(process.argv[3] ?? join(root, 'dist/cli'));
const version = (await Bun.file(join(root, 'package.json')).json()).version;
const name = (
  await Bun.file(join(root, 'packages/nayuta-cli-npm/package.json')).json()
).name;
const directory = join(output, `${platform.platform}-${platform.arch}`);
const filename = `nayuta-${version}-${platform.target}.tar.gz`;
await run(['tar', '-czf', join(output, filename), '-C', directory, '.']);
await run(['bun', 'pm', 'pack', '--destination', output], directory);
await run(
  ['bun', 'pm', 'pack', '--destination', output],
  join(output, 'wrapper'),
);
for (const archive of [
  filename,
  npmTarballName(name, platformVersion(version, platform)),
  npmTarballName(name, version),
]) {
  const hash = new Bun.CryptoHasher('sha256')
    .update(await Bun.file(join(output, archive)).arrayBuffer())
    .digest('hex');
  await Bun.write(join(output, `${archive}.sha256`), `${hash}  ${archive}\n`);
}
console.log(join(output, filename));
