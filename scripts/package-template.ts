import { mkdtemp, mkdir, copyFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { root, run, templatePath } from './distribution';

const version: string = (await Bun.file(join(root, 'package.json')).json()).version;
const output = resolve(process.argv[2] ?? join(root, 'dist/cli'));
await mkdir(output, { recursive: true });
const stage = await mkdtemp(join(tmpdir(), 'nayuta-template-'));
try {
  const files = (await run(['git', 'ls-files', '-z'])).split('\0').filter(templatePath);
  for (const file of files) {
    const destination = join(stage, file);
    await mkdir(dirname(destination), { recursive: true });
    await copyFile(join(root, file), destination);
  }
  const pkg = await Bun.file(join(stage, 'package.json')).json();
  delete pkg.workspaces;
  pkg.scripts = Object.fromEntries(Object.entries(pkg.scripts).filter(([name]) => ['dev', 'build', 'preview', 'astro', 'check'].includes(name)));
  await Bun.write(join(stage, 'package.json'), JSON.stringify(pkg, null, 2) + '\n');
  await Bun.write(join(stage, 'nayuta-template.json'), JSON.stringify({ version, commit: await run(['git', 'rev-parse', 'HEAD']) }, null, 2) + '\n');
  const filename = `nayuta-template-${version}.tar.gz`;
  await run(['tar', '-czf', join(output, filename), '-C', stage, '.']);
  const hash = new Bun.CryptoHasher('sha256').update(await Bun.file(join(output, filename)).arrayBuffer()).digest('hex');
  await Bun.write(join(output, `${filename}.sha256`), `${hash}  ${filename}\n`);
  console.log(join(output, filename));
} finally { await rm(stage, { recursive: true, force: true }); }
