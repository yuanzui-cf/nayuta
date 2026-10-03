import { afterEach, expect, test } from 'bun:test';
import { join } from 'node:path';
import {
  prepareSiteConfig,
  readSiteConfig,
  writeSiteConfig,
} from '../src/infrastructure/site-config';
import { projectFixture, type ProjectFixture } from './helpers/project';
let item: ProjectFixture | undefined;
afterEach(async () => {
  await item?.cleanup();
  item = undefined;
});

test('reads configuration without executing it and rejects dynamic fields', async () => {
  item = await projectFixture();
  await Bun.write(
    join(item.template, 'src/config.ts'),
    `const explode = () => { throw new Error('Executed'); };\nexport default { title: explode(), author: 'Me' } satisfies Record<string, unknown>;\n`,
  );
  const snapshot = await readSiteConfig(item.template);
  expect(snapshot.dynamic).toEqual(['title']);
  expect(() => prepareSiteConfig(snapshot, { title: 'Changed' })).toThrow(
    'errorConfig',
  );
  await expect(
    prepareSiteConfig(snapshot, { author: 'Updated', description: 'Added' }),
  ).toContain('description: "Added"');
});

test('refuses concurrent file changes and retains their contents', async () => {
  item = await projectFixture();
  const snapshot = await readSiteConfig(item.template);
  await Bun.write(snapshot.path, `${snapshot.source}// External change\n`);
  expect(
    writeSiteConfig(
      snapshot,
      prepareSiteConfig(snapshot, { title: 'Changed' }),
    ),
  ).rejects.toThrow('errorChanged');
  expect(await Bun.file(snapshot.path).text()).toContain('// External change');
});
