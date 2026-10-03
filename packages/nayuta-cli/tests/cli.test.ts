import { expect, test } from 'bun:test';
import { describeError, NayutaError } from '../src/core/errors';
import { I18n } from '../src/i18n';
import { projectFixture } from './helpers/project';

test('localizes nested help and returns machine-readable errors without modifying files', async () => {
  const fixture = await projectFixture();
  try {
    const help = await fixture.cli([
      '--locale=zh-TW',
      'post',
      'create',
      '--help',
    ]);
    expect(help.exitCode).toBe(0);
    expect(help.stdout).toContain('建立已發布的文章');
    expect(help.stdout).toContain('用法:');
    const unsupported = await fixture.cli(['--locale', 'fr', '--help']);
    expect(unsupported.stdout).toContain('Create and manage Nayuta blogs');
    const missing = await fixture.cli(['post', 'create', '--json']);
    expect(missing.exitCode).toBe(1);
    expect(JSON.parse(missing.stdout).error).toBe('errorArguments');
    const edit = await fixture.cli(['post', 'edit', '--json']);
    expect(edit.exitCode).toBe(1);
    const noArgs = await fixture.cli([]);
    expect(noArgs.exitCode).toBe(0);
    expect(noArgs.stdout).toContain('Usage:');
  } finally {
    await fixture.cleanup();
  }
});

test('localizes validation field names and explanations', () => {
  expect(
    describeError(
      new NayutaError('errorValidation', {
        field: 'site_url',
        detail: 'invalidValue',
      }),
      new I18n('zh-TW'),
    ),
  ).toBe('網站網址 無效：請檢查數值與格式');
  expect(
    describeError(
      new NayutaError('errorFrontmatter', {
        path: 'post.md',
        detail: 'missingDelimiters',
      }),
      new I18n('zh-HK'),
    ),
  ).toContain('缺少 --- 分隔符號');
});
