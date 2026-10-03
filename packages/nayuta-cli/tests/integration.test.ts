import { expect, test } from 'bun:test';
import { cp, symlink } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { run } from '../src/infrastructure/process';
import { exists } from '../src/infrastructure/filesystem';
import { projectFixture, createArgs } from './helpers/project';

test('builds a fresh empty release template and CLI-created Markdown/MDX posts', async () => {
  const fixture = await projectFixture();
  try {
    const repository = resolve(import.meta.dir, '../../..');
    const args = [...createArgs(fixture), '--no-commit'];
    for (const path of [
      'src',
      'public',
      'astro.config.ts',
      'tsconfig.json',
      'LICENSE',
    ]) {
      await cp(join(repository, path), join(fixture.template, path), {
        recursive: true,
      });
    }
    const metadata = await Bun.file(join(repository, 'package.json')).json();
    delete metadata.workspaces;
    metadata.scripts.check = 'astro check && tsc -p tsconfig.json --noEmit';
    await Bun.write(
      join(fixture.template, 'package.json'),
      JSON.stringify(metadata),
    );
    await run(['git', 'add', '--all'], {
      cwd: fixture.template,
      env: fixture.env,
    });
    await run(['git', 'commit', '--quiet', '-m', 'Theme fixture'], {
      cwd: fixture.template,
      env: fixture.env,
    });
    await run(['git', 'tag', '--force', 'v0.2.0'], {
      cwd: fixture.template,
      env: fixture.env,
    });
    const created = await fixture.cli(args);
    expect(created.exitCode).toBe(0);
    await symlink(
      join(repository, 'node_modules'),
      join(fixture.blog, 'node_modules'),
    );
    const check = await run(['bun', 'run', '--bun', 'check'], {
      cwd: fixture.blog,
      env: fixture.env,
      allowFailure: true,
    });
    if (check.exitCode !== 0) throw new Error(check.stdout + check.stderr);
    const build = () =>
      run(['bun', 'run', '--bun', 'build'], {
        cwd: fixture.blog,
        env: fixture.env,
        allowFailure: true,
      });
    const empty = await build();
    if (empty.exitCode !== 0) throw new Error(empty.stdout + empty.stderr);
    expect(await exists(join(fixture.blog, 'dist/index.html'))).toBe(true);
    expect(await exists(join(fixture.blog, 'dist/posts/index.html'))).toBe(
      true,
    );
    for (const [title, slug, format, published] of [
      ['Published Article', 'published-article', 'md', true],
      ['Draft Article', 'draft-article', 'mdx', false],
    ] as const) {
      const result = await fixture.cli([
        '--cwd',
        fixture.blog,
        'post',
        'create',
        '--title',
        title,
        '--slug',
        slug,
        '--format',
        format,
        '--publish-date',
        '2026-10-04',
        '--no-date-prefix',
        ...(published ? ['--published'] : []),
        '--json',
      ]);
      expect(result.exitCode).toBe(0);
    }
    const written = await build();
    if (written.exitCode !== 0)
      throw new Error(written.stdout + written.stderr);
    expect(
      await exists(
        join(fixture.blog, 'dist/posts/published-article/index.html'),
      ),
    ).toBe(true);
    expect(
      await exists(join(fixture.blog, 'dist/posts/draft-article/index.html')),
    ).toBe(false);
    expect(
      await Bun.file(join(fixture.blog, 'dist/posts/index.html')).text(),
    ).toContain('Published Article');
  } finally {
    await fixture.cleanup();
  }
}, 120_000);
