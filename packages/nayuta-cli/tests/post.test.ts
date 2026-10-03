import { afterEach, expect, test } from 'bun:test';
import { mkdir, symlink } from 'node:fs/promises';
import { join } from 'node:path';
import { listPosts } from '../src/core/post/service';
import { parseFrontmatter } from '../src/core/post/frontmatter';
import { exists } from '../src/infrastructure/filesystem';
import {
  createArgs,
  projectFixture,
  type ProjectFixture,
} from './helpers/project';
let item: ProjectFixture | undefined;
afterEach(async () => {
  await item?.cleanup();
  item = undefined;
});
async function freshBlog() {
  item = await projectFixture();
  const result = await item.cli([...createArgs(item), '--no-commit']);
  expect(result.exitCode).toBe(0);
  return item;
}

test('creates localized MDX folder bundles with explicit date, draft and local cover', async () => {
  const fixture = await freshBlog();
  const cover = join(fixture.root, 'image.svg');
  await Bun.write(
    cover,
    '<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"/>',
  );
  const result = await fixture.cli([
    'post',
    'create',
    '--cwd',
    fixture.blog,
    '--title',
    '測試文章',
    '--slug',
    'first-post',
    '--publish-date',
    '2026-10-04',
    '--format',
    'mdx',
    '--cover',
    cover,
    '--tags',
    'Astro, 筆記,Astro',
    '--locale',
    'zh-TW',
    '--json',
  ]);
  expect(result.exitCode).toBe(0);
  const metadata = JSON.parse(result.stdout);
  expect(metadata.url).toBe('/posts/2026-10-04-first-post');
  const path = join(
    fixture.blog,
    'src/content/posts/2026-10-04-first-post/index.mdx',
  );
  const snapshot = parseFrontmatter(await Bun.file(path).text(), path);
  expect(snapshot.values).toEqual({
    title: '測試文章',
    publishDate: '2026-10-04',
    tags: ['Astro', '筆記'],
    draft: true,
    cover: './assets/cover.svg',
  });
  expect(snapshot.body).toContain('在這裡開始撰寫');
  expect(await exists(join(path, '../assets/cover.svg'))).toBe(true);
  expect((await listPosts(fixture.blog)).map((post) => post.id)).toEqual([
    '2026-10-04-first-post',
  ]);
});

test('prevents collisions with single-file, MDX and custom slug posts', async () => {
  const fixture = await freshBlog();
  await Bun.write(
    join(fixture.blog, 'src/content/posts/existing.mdx'),
    '---\ntitle: Existing\npublishDate: "2026-10-04"\nslug: 2026-10-04-collision\n---\nBody',
  );
  const result = await fixture.cli([
    '--cwd',
    fixture.blog,
    'post',
    'create',
    '--title',
    'Collision',
    '--slug',
    'collision',
    '--publish-date',
    '2026-10-04',
    '--json',
  ]);
  expect(JSON.parse(result.stdout).error).toBe('errorExists');
  expect(
    await exists(join(fixture.blog, 'src/content/posts/2026-10-04-collision')),
  ).toBe(false);
});

test('validates dates, tags, formats and explicit paths before creating folders', async () => {
  const fixture = await freshBlog();
  for (const flags of [
    ['--publish-date', '2026-02-30'],
    ['--slug', '../escape'],
    ['--slug', '_hidden'],
    ['--format', 'astro'],
    ['--tags', 'Astro,,Notes'],
  ]) {
    expect(
      (
        await fixture.cli([
          '--cwd',
          fixture.blog,
          'post',
          'create',
          '--title',
          'Test',
          ...flags,
          '--json',
        ])
      ).exitCode,
    ).toBe(1);
  }
  expect(await listPosts(fixture.blog)).toEqual([]);
});

test('supports no date prefix and published posts, and ignores content resource files', async () => {
  const fixture = await freshBlog();
  const result = await fixture.cli([
    '--cwd',
    fixture.blog,
    'post',
    'create',
    '--title',
    'Hello',
    '--no-date-prefix',
    '--published',
    '--json',
  ]);
  expect(JSON.parse(result.stdout).url).toBe('/posts/hello');
  await Bun.write(
    join(fixture.blog, 'src/content/posts/hello/assets/ignored.md'),
    'Not YAML',
  );
  await Bun.write(
    join(fixture.blog, 'src/content/posts/hello/_notes.md'),
    'Not YAML',
  );
  const posts = await listPosts(fixture.blog);
  expect(posts.length).toBe(1);
  expect(posts[0]!.draft).toBe(false);
});

test('does not discover external posts through symlinks', async () => {
  const fixture = await freshBlog();
  const outside = join(fixture.root, 'outside');
  await mkdir(outside);
  await symlink(outside, join(fixture.blog, 'src/content/posts/external'));
  await Bun.write(
    join(outside, 'post.md'),
    '---\ntitle: Outside\npublishDate: "2026-10-04"\n---\nBody',
  );
  expect(
    (await listPosts(fixture.blog)).some((post) => post.title === 'Outside'),
  ).toBe(false);
});
