import { afterEach, expect, test } from 'bun:test';
import { copyFile, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { bumpVersion } from '../../scripts/bump-version';
import { run } from '../../scripts/distribution';
import {
  nextVersion,
  readVersions,
  validateVersion,
  versionChanges,
  versionFiles,
  writeVersions,
} from '../../scripts/versions';

const source = resolve(import.meta.dir, '../..');
const folders: string[] = [];
afterEach(async () => {
  for (const folder of folders.splice(0))
    await rm(folder, { recursive: true, force: true });
});

async function fixture() {
  const root = await mkdtemp(join(tmpdir(), 'nayuta versions '));
  folders.push(root);
  for (const file of [
    ...versionFiles,
    'scripts/bump-version.ts',
    'scripts/versions.ts',
    'scripts/distribution.ts',
  ]) {
    await mkdir(dirname(join(root, file)), { recursive: true });
    await copyFile(join(source, file), join(root, file));
  }
  await writeVersions(root, versionChanges(await readVersions(root), '0.3.0'));
  await run(['git', 'init', '--initial-branch=main'], root);
  await run(['git', 'config', 'user.name', 'Version Fixture'], root);
  await run(['git', 'config', 'user.email', 'versions@example.org'], root);
  await run(['git', 'config', 'commit.gpgsign', 'false'], root);
  await mkdir(join(root, 'hooks'));
  await run(['git', 'config', 'core.hooksPath', join(root, 'hooks')], root);
  await run(['git', 'add', '--all'], root);
  await run(['git', 'commit', '-m', 'Fixture'], root);
  return root;
}

async function rejects(operation: Promise<unknown>, message?: string) {
  const error: unknown = await operation.then(
    () => undefined,
    (error: unknown) => error,
  );
  expect(error).toBeInstanceOf(Error);
  if (message) expect((error as Error).message).toContain(message);
}

test('versions follow SemVer, including prereleases and bump aliases', () => {
  expect(nextVersion('0.3.0', 'patch')).toBe('0.3.1');
  expect(nextVersion('0.3.5', 'minor')).toBe('0.4.0');
  expect(nextVersion('0.3.5', 'major')).toBe('1.0.0');
  expect(nextVersion('0.3.0', 'v0.4.0-rc.1')).toBe('0.4.0-rc.1');
  expect(validateVersion('1.0.0-rc.1+build.01')).toBe('1.0.0-rc.1+build.01');
  for (const invalid of ['1.2', '01.2.3', '1.2.3-01', '1.2.3-', '1.2.3+'])
    expect(() => validateVersion(invalid)).toThrow('Invalid SemVer');
});

test('root bump command commits all project versions and preserves unrelated staged work', async () => {
  const root = await fixture();
  const before = await readVersions(root);
  await writeFile(join(root, 'unrelated.txt'), 'Keep this staged\n');
  await run(['git', 'add', 'unrelated.txt'], root);
  await mkdir(join(root, 'docs/plans'), { recursive: true });
  await writeFile(join(root, 'docs/plans/local.md'), 'Never commit this\n');
  const child = Bun.spawn([process.execPath, 'run', 'bump', 'v0.4.0-rc.1'], {
    cwd: root,
    stdout: 'pipe',
    stderr: 'pipe',
  });
  const [stdout, stderr, status] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  expect(status, stderr).toBe(0);
  expect(stdout).toContain('All Nayuta versions updated to 0.4.0-rc.1');
  expect(await run(['git', 'log', '-1', '--format=%s'], root)).toBe(
    'chore: Bump version to 0.4.0-rc.1',
  );
  expect(await run(['git', 'rev-list', '--count', 'HEAD'], root)).toBe('2');
  expect(
    (await run(['git', 'diff', '--name-only', 'HEAD^', 'HEAD'], root))
      .split('\n')
      .sort(),
  ).toEqual([...versionFiles].sort());
  expect(await run(['git', 'diff', '--cached', '--name-only'], root)).toBe(
    'unrelated.txt',
  );
  const after = await readVersions(root);
  expect(after.every((file) => file.version === '0.4.0-rc.1')).toBe(true);
  for (const [i, file] of after.entries()) {
    expect(file.source.replace(file.pattern, '$1__VERSION__$3')).toBe(
      before[i]!.source.replace(before[i]!.pattern, '$1__VERSION__$3'),
    );
  }
});

test('dirty version files, invalid versions and downgrades never change files or history', async () => {
  const root = await fixture();
  const original = await readVersions(root);
  for (const version of ['0.2.0', '0.3.0', '01.4.0'])
    await rejects(bumpVersion(root, version));
  await writeFile(join(root, 'package.json'), original[0]!.source + '\n');
  await rejects(bumpVersion(root, '0.4.0'), 'uncommitted changes');
  const after = await readVersions(root);
  expect(after[0]!.source).toBe(original[0]!.source + '\n');
  expect(after.slice(1).map((file) => file.source)).toEqual(
    original.slice(1).map((file) => file.source),
  );
  expect(await run(['git', 'rev-list', '--count', 'HEAD'], root)).toBe('1');
});

test('a failed Git commit restores versions without consuming other staged files', async () => {
  const root = await fixture();
  const before = await readVersions(root);
  await writeFile(join(root, 'unrelated.txt'), 'Keep this staged\n');
  await run(['git', 'add', 'unrelated.txt'], root);
  await writeFile(join(root, 'hooks/pre-commit'), '#!/bin/sh\nexit 1\n', {
    mode: 0o755,
  });
  await rejects(bumpVersion(root, 'patch'), 'Command failed');
  expect((await readVersions(root)).map((file) => file.source)).toEqual(
    before.map((file) => file.source),
  );
  expect(
    await run(['git', 'status', '--porcelain', '--', ...versionFiles], root),
  ).toBe('');
  expect(await run(['git', 'diff', '--cached', '--name-only'], root)).toBe(
    'unrelated.txt',
  );
  expect(await run(['git', 'rev-list', '--count', 'HEAD'], root)).toBe('1');
});

test('partial updates roll back while preserving a concurrent edit', async () => {
  const root = await fixture();
  const before = await readVersions(root);
  const changes = versionChanges(before, '0.4.0');
  const conflicting = changes[2]!;
  await writeFile(join(root, conflicting.path), conflicting.before + '\n');
  await rejects(writeVersions(root, changes), 'changed while');
  for (const change of changes) {
    expect(await Bun.file(join(root, change.path)).text()).toBe(
      change.before + (change === conflicting ? '\n' : ''),
    );
  }
});
