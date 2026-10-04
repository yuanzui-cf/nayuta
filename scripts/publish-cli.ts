import { mkdtemp, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { run } from './distribution';

const directory = resolve(process.argv[2] ?? 'artifacts');
const token = process.env.NPM_TOKEN;
if (!token) throw new Error('NPM_TOKEN is required.');
const tag = process.env.GITHUB_REF_NAME;
if (!tag?.startsWith('v'))
  throw new Error('Publish from the matching release tag.');
const files = (await readdir(directory)).filter((f) => f.endsWith('.tgz'));
const packages = await Promise.all(
  files.map(async (file) => {
    const path = join(directory, file);
    const pkg = JSON.parse(
      await run(['tar', '-xOf', path, 'package/package.json']),
    );
    if (pkg.version !== tag.slice(1))
      throw new Error(`${pkg.name}: package version differs from ${tag}`);
    return { path, pkg };
  }),
);
const platforms = packages.filter(({ pkg }) => pkg.os);
const wrappers = packages.filter(({ pkg }) => !pkg.os);
if (platforms.length !== 6 || wrappers.length !== 1)
  throw new Error('All six platform packages and one wrapper are required.');
const publishDirectory = await mkdtemp(join(tmpdir(), 'nayuta-publish-'));
try {
  await writeFile(
    join(publishDirectory, '.npmrc'),
    `//registry.npmjs.org/:_authToken=${token}\n`,
    { mode: 0o600 },
  );
  for (const { path, pkg } of [...platforms, ...wrappers]) {
    const response = await fetch(
      `https://registry.npmjs.org/${encodeURIComponent(pkg.name)}/${pkg.version}`,
    );
    if (response.ok) {
      console.log(`Already published: ${pkg.name}@${pkg.version}`);
      continue;
    }
    if (response.status !== 404)
      throw new Error(`Registry lookup failed: ${response.status}`);
    await run(
      [
        'bun',
        'publish',
        path,
        '--access',
        'public',
        '--tag',
        pkg.version.includes('-') ? 'next' : 'latest',
      ],
      publishDirectory,
    );
  }
} finally {
  await rm(publishDirectory, { recursive: true, force: true });
}
