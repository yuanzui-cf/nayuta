import { afterEach, expect, test } from 'bun:test';
import { mkdir, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { platforms, root, run } from '../../scripts/distribution';
import {
  npmTarballName,
  platformDependencies,
  platformVersion,
  publishPlan,
  type NpmPackage,
} from '../../scripts/npm-release';
import { readPublishPlan } from '../../scripts/publish-cli';

const name = '@yuanzui-cf/nayuta';
const folders: string[] = [];
afterEach(async () => {
  for (const folder of folders.splice(0))
    await rm(folder, { recursive: true, force: true });
});

function packages(version: string): NpmPackage[] {
  return [
    ...platforms.map((platform) => ({
      path: `${platform.platform}-${platform.arch}.tgz`,
      pkg: {
        name,
        version: `${version}-${platform.platform}-${platform.arch}`,
        os: [platform.platform],
        cpu: [platform.arch],
      },
    })),
    {
      path: 'wrapper.tgz',
      pkg: {
        name,
        version,
        optionalDependencies: Object.fromEntries(
          platforms.map((platform) => [
            `${name}-${platform.platform}-${platform.arch}`,
            `npm:${name}@${version}-${platform.platform}-${platform.arch}`,
          ]),
        ),
      },
    },
  ];
}

async function archives(version: string) {
  const directory = await mkdtemp(join(tmpdir(), 'nayuta npm release '));
  folders.push(directory);
  const manifests = packages(version);
  for (const [i, item] of manifests.entries()) {
    const stage = join(directory, `stage-${i}`);
    await mkdir(join(stage, 'package'), { recursive: true });
    await Bun.write(
      join(stage, 'package/package.json'),
      JSON.stringify(item.pkg),
    );
    const filename = npmTarballName(item.pkg.name, item.pkg.version);
    item.path = join(directory, filename);
    await run(['tar', '-czf', item.path, '-C', stage, 'package']);
    const digest = new Bun.CryptoHasher('sha256')
      .update(await Bun.file(item.path).arrayBuffer())
      .digest('hex');
    await Bun.write(`${item.path}.sha256`, `${digest}  ${filename}\n`);
  }
  return { directory, manifests };
}

test('platform variants use one npm name, version suffixes and exact aliases', () => {
  const aliases = platformDependencies(name, '0.4.0');
  expect(Object.keys(aliases)).toHaveLength(6);
  expect(aliases[`${name}-darwin-arm64`]).toBe(
    `npm:${name}@0.4.0-darwin-arm64`,
  );
  expect(aliases[`${name}-win32-x64`]).toBe(`npm:${name}@0.4.0-win32-x64`);
  expect(platformVersion('0.4.0-rc.1', platforms[3]!)).toBe(
    '0.4.0-rc.1-linux-x64',
  );
  expect(npmTarballName(name, '0.4.0-linux-x64')).toBe(
    'yuanzui-cf-nayuta-0.4.0-linux-x64.tgz',
  );
  expect(() => platformDependencies(name, '0.4.0+build.1')).toThrow(
    'build metadata',
  );
});

test('publish platforms first with a native tag, then the stable or prerelease wrapper', () => {
  for (const [version, tag] of [
    ['0.4.0', 'latest'],
    ['0.4.0-rc.1', 'next'],
  ]) {
    const plan = publishPlan(packages(version!), name, version!);
    expect(plan.slice(0, 6).every((item) => item.tag === 'native')).toBe(true);
    expect(plan.at(-1)?.pkg.version).toBe(version);
    expect(plan.at(-1)?.tag).toBe(tag);
  }
  expect(publishPlan(packages('0.4.0'), name, '0.4.0', true).at(-1)?.tag).toBe(
    'next',
  );
});

test('missing, duplicated, incompatible or incorrectly aliased packages cannot publish', () => {
  const valid = packages('0.4.0');
  expect(() => publishPlan(valid.slice(1), name, '0.4.0')).toThrow('All six');
  const duplicate = packages('0.4.0');
  duplicate[1] = duplicate[0]!;
  expect(() => publishPlan(duplicate, name, '0.4.0')).toThrow(
    'platform package',
  );
  const wrongOs = packages('0.4.0');
  wrongOs[0]!.pkg.os = ['linux'];
  expect(() => publishPlan(wrongOs, name, '0.4.0')).toThrow('platform package');
  const wrongAlias = packages('0.4.0');
  wrongAlias.at(-1)!.pkg.optionalDependencies![`${name}-darwin-arm64`] =
    '0.4.0';
  expect(() => publishPlan(wrongAlias, name, '0.4.0')).toThrow('aliases');
  const oldNames = packages('0.4.0');
  oldNames[0]!.pkg.name = `${name}-darwin-arm64`;
  expect(() => publishPlan(oldNames, name, '0.4.0')).toThrow(
    'platform package',
  );
});

test('the publish command verifies archives and supports a credential-free dry run', async () => {
  const version = (
    await Bun.file(join(root, 'packages/nayuta-cli-npm/package.json')).json()
  ).version;
  const { directory } = await archives(version);
  const child = Bun.spawn(
    [
      process.execPath,
      join(root, 'scripts/publish-cli.ts'),
      directory,
      '--dry-run',
    ],
    {
      env: {
        ...process.env,
        NAYUTA_RELEASE_TAG: `v${version}`,
        NAYUTA_PRERELEASE: 'false',
        NPM_TOKEN: '',
      },
      stdout: 'pipe',
      stderr: 'pipe',
    },
  );
  const [stdout, stderr, status] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  expect(status, stderr).toBe(0);
  const lines = stdout.trim().split('\n');
  expect(lines).toHaveLength(7);
  expect(lines.slice(0, 6).every((line) => line.endsWith(' -> native'))).toBe(
    true,
  );
  expect(lines.at(-1)).toBe(
    `${name}@${version} -> ${version.includes('-') ? 'next' : 'latest'}`,
  );
});

test('a corrupt npm archive fails before any publish operation', async () => {
  const { directory, manifests } = await archives('0.4.0');
  const archive = manifests.at(-1)!;
  await Bun.write(archive.path, 'Corrupted archive');
  const result: unknown = await readPublishPlan(directory, name, '0.4.0').then(
    () => undefined,
    (error: unknown) => error,
  );
  expect(result).toBeInstanceOf(Error);
  expect((result as Error).message).toContain('Invalid checksum');
});
