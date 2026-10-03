import { afterEach, expect, test } from 'bun:test';
import { join } from 'node:path';
import {
  parseFrontmatter,
  prepareFrontmatter,
} from '../src/core/post/frontmatter';
import { readPost, updatePost } from '../src/core/post/service';
import { readText } from '../src/infrastructure/filesystem';
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

test('preserves BOM, CRLF, comments, unknown metadata, aliases and MDX body', () => {
  const body =
    '\r\nimport Callout from "@widgets/article/Callout.astro";\r\n\r\n## Body\r\n\r\n---\r\n';
  const source = `\uFEFF---\r\n# Author note\r\ntitle: Original # inline comment\r\npublishDate: '2026-10-04'\r\ncustom: &data\r\n  value: keep\r\nalias: *data\r\ncopyright:\r\n  enabled: false # site override\r\n  license: CC BY 4.0\r\n---\r\n${body}`;
  const snapshot = parseFrontmatter(source, 'post.mdx');
  const content = prepareFrontmatter(snapshot, {
    title: '新的標題',
    tags: [' A ', 'A', 'B'],
    'copyright.enabled': undefined,
  });
  const edited = parseFrontmatter(content, 'post.mdx');
  expect(edited.body).toBe(body);
  expect(content.startsWith('\uFEFF---\r\n')).toBe(true);
  expect(content).toContain('# Author note');
  expect(content).toContain('# inline comment');
  expect(content).toContain('alias: *data');
  expect(edited.values.custom).toEqual({ value: 'keep' });
  expect(edited.values.tags).toEqual(['A', 'B']);
  expect(edited.values.copyright).toEqual({ license: 'CC BY 4.0' });
});

test('rejects invalid or duplicate YAML keys and invalid edited field types', () => {
  expect(() =>
    parseFrontmatter(
      '---\ntitle: A\ntitle: B\npublishDate: today\n---\n',
      'post.md',
    ),
  ).toThrow('errorFrontmatter');
  expect(() => parseFrontmatter('No frontmatter', 'post.md')).toThrow(
    'errorFrontmatter',
  );
  const snapshot = parseFrontmatter(
    '---\ntitle: A\npublishDate: "2026-10-04"\n---\n',
    'post.md',
  );
  expect(() => prepareFrontmatter(snapshot, { title: undefined })).toThrow(
    'errorValidation',
  );
  expect(() => prepareFrontmatter(snapshot, { draft: 'true' })).toThrow(
    'errorValidation',
  );
  expect(() => prepareFrontmatter(snapshot, { unknown: 'changed' })).toThrow(
    'errorValidation',
  );
});

test('atomically saves edits, refuses stale snapshots and validates copyright', async () => {
  item = await projectFixture();
  await item.cli([...createArgs(item), '--no-commit']);
  const path = join(item.blog, 'src/content/posts/test.md');
  const body = '\n## Preserved body\n';
  await Bun.write(
    path,
    `---\ntitle: A\npublishDate: "2026-10-04"\n---\n${body}`,
  );
  const snapshot = await readPost(item.blog, path);
  await updatePost(item.blog, snapshot, { title: 'Changed', draft: false });
  expect(parseFrontmatter(await readText(path), path).body).toBe(body);
  await expect(
    updatePost(item.blog, snapshot, { title: 'Stale' }),
  ).rejects.toThrow('errorChanged');
  const latest = await readPost(item.blog, path);
  // The generated blog disables the card but retains the theme license.
  await updatePost(item.blog, latest, {
    'copyright.enabled': true,
    'copyright.license': 'CC BY-SA 4.0',
  });
  expect((await readPost(item.blog, path)).values.copyright).toEqual({
    enabled: true,
    license: 'CC BY-SA 4.0',
  });
});
