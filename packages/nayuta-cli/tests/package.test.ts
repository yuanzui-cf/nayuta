import { expect, test } from 'bun:test';
import { mkdir, symlink } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { run } from '../src/infrastructure/process';
import { projectFixture, createArgs } from './helpers/project';

test('packed executable runs CLI and TUI from an unrelated directory', async () => {
  const fixture = await projectFixture();
  const packageRoot = resolve(import.meta.dir, '..');
  try {
    await run(['bun', 'run', 'build'], { cwd: packageRoot });
    await run(
      [
        'bun',
        'pm',
        'pack',
        '--ignore-scripts',
        '--destination',
        fixture.root,
        '--quiet',
      ],
      { cwd: packageRoot },
    );
    const archive = join(fixture.root, 'nayuta-cli-0.1.0.tgz');
    const listing = (await run(['tar', '-tf', archive])).stdout;
    expect(listing).toContain('package/dist/index.js');
    expect(listing).toContain('package/LICENSE');
    expect(listing).not.toContain('package/src/');
    expect(listing).not.toContain('package/tests/');
    const extracted = join(fixture.root, 'extracted');
    await mkdir(extracted);
    await run(['tar', '-xf', archive, '-C', extracted]);
    const installed = join(extracted, 'package');
    // Share installed dependencies, while all executable files come from the tarball.
    await symlink(
      join(packageRoot, 'node_modules'),
      join(installed, 'node_modules'),
    );
    const binary = join(installed, 'dist/index.js');
    const execute = (args: string[]) =>
      run([binary, ...args], {
        cwd: fixture.root,
        env: fixture.env,
        allowFailure: true,
      });
    const help = await execute([
      '--locale',
      'zh-HK',
      'blog',
      'create',
      '--help',
    ]);
    expect(help.exitCode).toBe(0);
    expect(help.stdout).toContain('建立網誌');
    expect(help.stdout).toContain('略過首次提交');
    expect(
      (await execute([...createArgs(fixture), '--no-commit'])).exitCode,
    ).toBe(0);
    const article = await execute([
      '--cwd',
      fixture.blog,
      'post',
      'create',
      '--title',
      'Packed CLI',
      '--json',
    ]);
    expect(article.exitCode).toBe(0);
    expect(JSON.parse(article.stdout).draft).toBe(true);
    const tui = await execute(['tui', '--json']);
    expect(JSON.parse(tui.stdout).error).toBe('errorTerminal');
  } finally {
    await fixture.cleanup();
  }
}, 60_000);
