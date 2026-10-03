import { mkdir, mkdtemp, realpath, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { run } from '../../src/infrastructure/process';

export const cliPath = resolve(import.meta.dir, '../../src/index.ts');
export async function projectFixture() {
  const root = await realpath(
    await mkdtemp(join(tmpdir(), 'nayuta-cli-test-')),
  );
  const template = join(root, 'template');
  const blog = join(root, 'blog');
  const globalConfig = join(root, 'gitconfig');
  const env = {
    ...process.env,
    GIT_CONFIG_GLOBAL: globalConfig,
    GIT_CONFIG_NOSYSTEM: '1',
    XDG_CONFIG_HOME: join(root, 'preferences'),
    LC_ALL: 'en_US.UTF-8',
  };
  await Bun.write(
    globalConfig,
    '[user]\n  name = Local User\n  email = local@example.com\n',
  );
  for (const folder of [
    'src/layouts',
    'src/content/posts',
    'src/content/pages',
    'src/contents',
    'public/assets/images',
  ])
    await mkdir(join(template, folder), { recursive: true });
  await Bun.write(
    join(template, 'package.json'),
    JSON.stringify({ name: 'nayuta-theme', version: '0.2.0' }),
  );
  await Bun.write(
    join(template, '.gitignore'),
    'node_modules/\ndist/\n.astro/\n',
  );
  await Bun.write(
    join(template, 'src/content.config.ts'),
    'export const collections = {};\n',
  );
  await Bun.write(join(template, 'src/layouts/frame.astro'), '<slot />\n');
  await Bun.write(
    join(template, 'src/config.ts'),
    `// Keep this comment\nconst config = {\n  title: 'Template',\n  author: 'Template Author',\n  avatar: '/assets/images/avatar.png',\n  description: 'Template biography',\n  site_url: 'https://template.example.com',\n  theme: 'nayuta',\n  copyright: { enabled: true, license: 'CC BY-NC-SA 4.0' },\n  links: [{ text: 'Demo', url: '/demo' }],\n  custom: 'preserve me',\n};\nexport default config;\n`,
  );
  await Bun.write(
    join(template, 'src/content/index.mdx'),
    '# Remote biography',
  );
  await Bun.write(
    join(template, 'src/content/_right.astro'),
    '<p>Remote sidebar</p>',
  );
  await Bun.write(join(template, 'src/content/posts/demo.mdx'), 'Remote post');
  await Bun.write(join(template, 'src/content/pages/demo.md'), 'Remote page');
  await Bun.write(
    join(template, 'src/contents/private-note.md'),
    'Remote legacy content',
  );
  await run(['git', 'init', '--quiet', '--initial-branch=main', template], {
    env,
  });
  await run(['git', 'add', '.'], { cwd: template, env });
  await run(
    [
      'git',
      '-c',
      'user.name=Template Author',
      '-c',
      'user.email=template@example.com',
      'commit',
      '--quiet',
      '-m',
      'Template history',
    ],
    { cwd: template, env },
  );
  await run(['git', 'tag', 'v0.2.0'], { cwd: template, env });
  return {
    root,
    template,
    blog,
    env,
    cli: (args: string[], overrides: Record<string, string> = {}) =>
      run(['bun', cliPath, ...args], {
        env: { ...env, ...overrides },
        allowFailure: true,
      }),
    cleanup: () => rm(root, { recursive: true, force: true }),
  };
}
export type ProjectFixture = Awaited<ReturnType<typeof projectFixture>>;
export function createArgs(fixture: ProjectFixture): string[] {
  return [
    'blog',
    'create',
    fixture.blog,
    '--template',
    fixture.template,
    '--no-install',
    '--title',
    'My Blog',
    '--author',
    'Blog Author',
    '--site-url',
    'https://blog.example.com',
    '--json',
  ];
}
