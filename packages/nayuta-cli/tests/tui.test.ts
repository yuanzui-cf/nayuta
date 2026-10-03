import { afterEach, expect, test } from 'bun:test';
import { InputRenderable, SelectRenderable } from '@opentui/core';
import {
  createTestRenderer,
  type TestRendererSetup,
} from '@opentui/core/testing';
import { join } from 'node:path';
import { I18n } from '../src/i18n';
import { TuiApp } from '../src/app/tui/app';
import { frontmatterForm } from '../src/app/tui/screens/frontmatter';
import { readPost } from '../src/core/post/service';
import {
  createArgs,
  projectFixture,
  type ProjectFixture,
} from './helpers/project';

let fixture: ProjectFixture | undefined;
let setup: TestRendererSetup | undefined;
let app: TuiApp | undefined;
afterEach(async () => {
  app?.destroy();
  app = undefined;
  setup = undefined;
  await fixture?.cleanup();
  fixture = undefined;
});
async function start(width = 90, height = 28) {
  fixture = await projectFixture();
  expect(
    (await fixture.cli([...createArgs(fixture), '--no-commit'])).exitCode,
  ).toBe(0);
  setup = await createTestRenderer({
    width,
    height,
    exitOnCtrlC: false,
    consoleMode: 'disabled',
  });
  app = new TuiApp(setup.renderer, {
    cwd: fixture.blog,
    i18n: new I18n('zh-TW'),
    json: false,
  });
  app.start();
  await setup.renderOnce();
  return { fixture, setup, app };
}

test('renders regional menus and switches fields with keyboard at small sizes', async () => {
  const { setup, app } = await start(48, 16);
  expect(setup.captureCharFrame()).toContain('建立部落格');
  await app.open('createPost');
  await setup.renderOnce();
  await setup.mockInput.pasteBracketedText('我的文章');
  setup.mockInput.pressTab();
  await setup.mockInput.typeText('my-post');
  await setup.renderOnce();
  expect(
    (setup.renderer.root.findDescendantById('field-title') as InputRenderable)
      .value,
  ).toBe('我的文章');
  expect(
    (setup.renderer.root.findDescendantById('field-slug') as InputRenderable)
      .value,
  ).toBe('my-post');
  for (let index = 0; index < 7; index++) setup.mockInput.pressTab();
  await setup.renderOnce();
  expect(setup.captureCharFrame()).toContain('日期前綴');
});

test('edits actual frontmatter, previews the diff and saves through keyboard', async () => {
  const { fixture, setup, app } = await start();
  const path = join(fixture.blog, 'src/content/posts/example.md');
  await Bun.write(
    path,
    '---\n# Keep comment\ntitle: Original\npublishDate: "2026-10-04"\ncustom: keep\n---\n\n## Body stays\n',
  );
  app.openForm(await frontmatterForm(app.context, path));
  const input = setup.renderer.root.findDescendantById(
    'field-title',
  ) as InputRenderable;
  input.value = '更新後的標題';
  setup.mockInput.pressKey('s', { ctrl: true });
  await setup.waitFor(() => app.screen === 'review');
  await setup.renderOnce();
  expect(setup.captureCharFrame()).toContain('+title: 更新後的標題');
  setup.mockInput.pressKey('s', { ctrl: true });
  await setup.waitFor(() => app.screen === 'home' && !app.busy);
  const snapshot = await readPost(fixture.blog, path);
  expect(snapshot.values.title).toBe('更新後的標題');
  expect(snapshot.values.custom).toBe('keep');
  expect(snapshot.body).toBe('\n## Body stays\n');
  expect(snapshot.source).toContain('# Keep comment');
});

test('warns before discarding edits and cancel restores entered values', async () => {
  const { setup, app } = await start();
  await app.open('createPost');
  (
    setup.renderer.root.findDescendantById('field-title') as InputRenderable
  ).value = 'Unsaved';
  setup.mockInput.pressEscape();
  await setup.waitFor(() => app.screen === 'confirm');
  await setup.renderOnce();
  expect(app.screen).toBe('confirm');
  expect(setup.captureCharFrame()).toContain('尚未儲存');
  setup.mockInput.pressEnter();
  await setup.waitFor(() => app.screen === 'form');
  expect(app.screen).toBe('form');
  expect(
    (setup.renderer.root.findDescendantById('field-title') as InputRenderable)
      .value,
  ).toBe('Unsaved');
  setup.mockInput.pressEscape();
  await setup.waitFor(() => app.screen === 'confirm');
  const menu = setup.renderer.root.findDescendantById(
    'menu',
  ) as SelectRenderable;
  menu.setSelectedIndex(1);
  menu.selectCurrent();
  expect(app.screen).toBe('home');
});

test('searches posts and opens the selected editor without a CLI editing command', async () => {
  const { fixture, setup, app } = await start();
  await Bun.write(
    join(fixture.blog, 'src/content/posts/first.md'),
    '---\ntitle: First post\npublishDate: "2026-10-04"\n---\nBody',
  );
  await Bun.write(
    join(fixture.blog, 'src/content/posts/second.md'),
    '---\ntitle: 第二篇文章\npublishDate: "2026-10-03"\n---\nBody',
  );
  await app.open('editFrontmatter');
  await setup.mockInput.pasteBracketedText('第二');
  await setup.renderOnce();
  const menu = setup.renderer.root.findDescendantById(
    'menu',
  ) as SelectRenderable;
  expect(menu.options.length).toBe(1);
  expect(menu.options[0]!.name).toContain('第二篇文章');
  setup.mockInput.pressEnter();
  setup.mockInput.pressEnter();
  await setup.waitFor(() => app.screen === 'form');
  expect(
    (setup.renderer.root.findDescendantById('field-title') as InputRenderable)
      .value,
  ).toBe('第二篇文章');
});

test('creates an article through the TUI and gives explicit non-TTY errors', async () => {
  const { fixture, setup, app } = await start();
  await app.open('createPost');
  (
    setup.renderer.root.findDescendantById('field-title') as InputRenderable
  ).value = 'TUI post';
  (
    setup.renderer.root.findDescendantById('field-slug') as InputRenderable
  ).value = 'tui-post';
  await app.review();
  await app.save();
  expect(app.screen).toBe('home');
  expect(
    (await fixture.cli(['--cwd', fixture.blog, 'tui', '--json'])).stdout,
  ).toContain('errorTerminal');
});
