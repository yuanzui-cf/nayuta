import { afterEach, expect, test } from 'bun:test';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { exists } from '../src/infrastructure/filesystem';
import { run } from '../src/infrastructure/process';
import { readSiteConfig } from '../src/infrastructure/site-config';
import {
  projectFixture,
  createArgs,
  type ProjectFixture,
} from './helpers/project';
const fixtures: ProjectFixture[] = [];
afterEach(async () => {
  for (const fixture of fixtures.splice(0)) await fixture.cleanup();
});
async function fixture() {
  const item = await projectFixture();
  fixtures.push(item);
  return item;
}

test('creates fresh required content and a single user-authored commit', async () => {
  const item = await fixture();
  const result = await item.cli(createArgs(item), {
    GIT_AUTHOR_NAME: 'Wrong Author',
    GIT_AUTHOR_EMAIL: 'wrong@example.com',
    GIT_COMMITTER_NAME: 'Wrong Committer',
    GIT_COMMITTER_EMAIL: 'wrong@example.com',
  });
  expect(result.exitCode).toBe(0);
  expect(JSON.parse(result.stdout).committed).toBe(true);
  expect(await exists(join(item.blog, 'src/contents'))).toBe(false);
  for (const path of [
    'index.mdx',
    '_right.astro',
    'posts/demo.mdx',
    'pages/demo.md',
  ])
    expect(await exists(join(item.blog, 'src/content', path))).toBe(false);
  for (const path of [
    'index.md',
    '_left.astro',
    'posts/.gitkeep',
    'pages/.gitkeep',
    'assets/.gitkeep',
  ])
    expect(await exists(join(item.blog, 'src/content', path))).toBe(true);
  const config = await readSiteConfig(item.blog);
  expect(config.values.author).toBe('Blog Author');
  expect(config.values.description).toBe('');
  expect(config.values.links).toEqual([{ text: 'Posts', url: '/posts' }]);
  expect(config.values.copyright?.enabled).toBe(false);
  const git = (args: string[]) =>
    run(['git', ...args], { cwd: item.blog, env: item.env });
  expect((await git(['rev-list', '--count', 'HEAD'])).stdout).toBe('1');
  expect(
    (await git(['show', '-s', '--format=%an|%ae|%cn|%ce|%P'])).stdout,
  ).toBe('Local User|local@example.com|Local User|local@example.com|');
  expect((await git(['status', '--porcelain'])).stdout).toBe('');
  expect(
    JSON.parse(await Bun.file(join(item.blog, '.nayuta-template.json')).text())
      .ref,
  ).toBe('v0.2.0');
});

test('resolves conditional Git identity in the new project directory', async () => {
  const item = await fixture();
  await Bun.write(
    join(item.root, 'identity'),
    '[user]\nname = Conditional User\nemail = conditional@example.com\n',
  );
  await Bun.write(
    item.env.GIT_CONFIG_GLOBAL,
    `[includeIf "gitdir:${item.blog}/"]\npath = ${join(item.root, 'identity')}\n`,
  );
  expect((await item.cli(createArgs(item))).exitCode).toBe(0);
  expect(
    (
      await run(['git', 'show', '-s', '--format=%an|%ae|%cn|%ce'], {
        cwd: item.blog,
        env: item.env,
      })
    ).stdout,
  ).toBe(
    'Conditional User|conditional@example.com|Conditional User|conditional@example.com',
  );
});

test('preserves the generated project when Git identity is missing', async () => {
  const item = await fixture();
  await Bun.write(item.env.GIT_CONFIG_GLOBAL, '');
  const result = await item.cli(createArgs(item));
  expect(result.exitCode).toBe(1);
  expect(JSON.parse(result.stdout).error).toBe('errorIdentity');
  expect(await exists(join(item.blog, 'src/content/index.md'))).toBe(true);
});

test('refuses existing destinations and validates settings before writes', async () => {
  const item = await fixture();
  const args = createArgs(item);
  args[args.indexOf('--site-url') + 1] = 'ftp://example.com';
  expect((await item.cli(args)).exitCode).toBe(1);
  expect(await exists(item.blog)).toBe(false);
  await mkdir(item.blog);
  await Bun.write(join(item.blog, 'keep.txt'), 'Keep');
  expect(JSON.parse((await item.cli(createArgs(item))).stdout).error).toBe(
    'errorExists',
  );
  expect(await Bun.file(join(item.blog, 'keep.txt')).text()).toBe('Keep');
});

test('sets up existing blogs without changing their content or Git history', async () => {
  const item = await fixture();
  const result = await item.cli([
    '--cwd',
    item.template,
    'blog',
    'setup',
    '--title',
    "A 'quoted' title",
    '--copyright-license',
    'CC BY 4.0',
    '--json',
  ]);
  expect(result.exitCode).toBe(0);
  const snapshot = await readSiteConfig(item.template);
  expect(snapshot.values.title).toBe("A 'quoted' title");
  expect(snapshot.values.copyright).toEqual({
    enabled: true,
    license: 'CC BY 4.0',
  });
  expect(snapshot.source).toContain('// Keep this comment');
  expect(snapshot.source).toContain("custom: 'preserve me'");
  expect(
    await Bun.file(join(item.template, 'src/content/posts/demo.mdx')).text(),
  ).toBe('Remote post');
});
