import { mkdtemp, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { basename, join, resolve } from 'node:path';
import { root, run } from './distribution';
import { publishPlan, type NpmPackage } from './npm-release';
import { validateVersion } from './versions';

export async function readPublishPlan(
  directory: string,
  name: string,
  version: string,
  prerelease = false,
) {
  const files = (await readdir(directory)).filter((file) =>
    file.endsWith('.tgz'),
  );
  const results = await Promise.allSettled(
    files.map(async (file) => {
      const path = join(directory, file);
      const checksum = await Bun.file(`${path}.sha256`).text();
      const match = checksum.match(/^([a-f0-9]{64}) {2}(.+)\n?$/);
      const actual = new Bun.CryptoHasher('sha256')
        .update(await Bun.file(path).arrayBuffer())
        .digest('hex');
      if (!match || match[1] !== actual || match[2] !== basename(path))
        throw new Error(`Invalid checksum: ${file}`);
      const pkg = JSON.parse(
        await run(['tar', '-xOf', path, 'package/package.json']),
      );
      return { path, pkg };
    }),
  );
  const packages: NpmPackage[] = [];
  for (const result of results) {
    if (result.status === 'rejected') throw result.reason;
    packages.push(result.value);
  }
  return publishPlan(packages, name, version, prerelease);
}

if (import.meta.main) {
  try {
    const args = process.argv.slice(2);
    const dryRun = args.includes('--dry-run');
    const directories = args.filter((arg) => arg !== '--dry-run');
    if (
      directories.length > 1 ||
      directories.some((arg) => arg.startsWith('-'))
    )
      throw new Error(
        'Usage: bun scripts/publish-cli.ts [artifacts] [--dry-run]',
      );
    const directory = resolve(directories[0] ?? 'artifacts');
    const tag = process.env.NAYUTA_RELEASE_TAG ?? process.env.GITHUB_REF_NAME;
    if (!tag) throw new Error('Publish from the matching release tag.');
    const version = validateVersion(tag.replace(/^v/, ''));
    const pkg = await Bun.file(
      join(root, 'packages/nayuta-cli-npm/package.json'),
    ).json();
    if (pkg.version !== version)
      throw new Error(
        `Release ${tag} differs from checked-out version ${pkg.version}`,
      );
    const plan = await readPublishPlan(
      directory,
      pkg.name,
      version,
      process.env.NAYUTA_PRERELEASE === 'true',
    );
    if (dryRun) {
      for (const { pkg, tag } of plan)
        console.log(`${pkg.name}@${pkg.version} -> ${tag}`);
    } else {
      const token = process.env.NPM_TOKEN;
      if (!token)
        throw new Error(
          'Configure the repository secret NPM_TOKEN to publish.',
        );
      const publishDirectory = await mkdtemp(join(tmpdir(), 'nayuta-publish-'));
      try {
        await writeFile(
          join(publishDirectory, '.npmrc'),
          `//registry.npmjs.org/:_authToken=${token}\n`,
          { mode: 0o600 },
        );
        for (const { path, pkg, tag } of plan) {
          const response = await fetch(
            `https://registry.npmjs.org/${encodeURIComponent(pkg.name)}/${encodeURIComponent(pkg.version)}`,
          );
          if (response.ok) {
            console.log(`Already published: ${pkg.name}@${pkg.version}`);
            continue;
          }
          if (response.status !== 404)
            throw new Error(`Registry lookup failed: ${response.status}`);
          console.log(`Publishing ${pkg.name}@${pkg.version} -> ${tag}`);
          await run(
            ['bun', 'publish', path, '--access', 'public', '--tag', tag],
            publishDirectory,
          );
        }
      } finally {
        await rm(publishDirectory, { recursive: true, force: true });
      }
    }
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
