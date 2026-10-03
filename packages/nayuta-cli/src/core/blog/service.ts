import { mkdir, rm } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { stringify } from 'yaml';
import type { I18n } from '../../i18n';
import { NayutaError } from '../errors';
import { discoverProject } from '../project/discover';
import { exists } from '../../infrastructure/filesystem';
import {
  gitIdentity,
  initializeGit,
  initialCommit,
} from '../../infrastructure/git';
import {
  createGithubRepository,
  validateGithub,
} from '../../infrastructure/github';
import {
  defaultTemplate,
  defaultTemplateRef,
  copyTemplate,
} from '../../infrastructure/template';
import {
  readSiteConfig,
  prepareSiteConfig,
  writeSiteConfig,
} from '../../infrastructure/site-config';
import { run } from '../../infrastructure/process';
import {
  siteSettingsSchema,
  today,
  validate,
  type SiteSettings,
} from './schema';
export interface CreateBlogOptions {
  directory: string;
  settings: SiteSettings;
  i18n: I18n;
  template?: string;
  templateRef?: string;
  install?: boolean;
  commit?: boolean;
  githubRepo?: string;
  visibility?: 'private' | 'public';
  push?: boolean;
}
export async function createBlog(options: CreateBlogOptions) {
  const settings = validate(siteSettingsSchema, options.settings);
  for (const field of ['title', 'author', 'site_url'] as const) {
    if (!settings[field])
      throw new NayutaError('errorValidation', { field, detail: 'Required' });
  }
  validateGithub(
    options.githubRepo,
    options.push ?? false,
    options.commit !== false,
  );
  const root = resolve(options.directory);
  if (await exists(root)) throw new NayutaError('errorExists', { path: root });
  await mkdir(dirname(root), { recursive: true });
  await mkdir(root);
  let initialized = false;
  try {
    const template = options.template ?? defaultTemplate;
    const ref = options.templateRef ?? defaultTemplateRef;
    const revision = await copyTemplate(root, template, ref);
    await discoverProject(root);
    // Author content is never inherited from the remote template.
    await rm(join(root, 'src/content'), { recursive: true, force: true });
    await rm(join(root, 'src/contents'), { recursive: true, force: true });
    const content = join(root, 'src/content');
    for (const folder of ['posts', 'pages', 'assets']) {
      await mkdir(join(content, folder), { recursive: true });
      await Bun.write(join(content, folder, '.gitkeep'), '');
    }
    await Bun.write(
      join(content, 'index.md'),
      `---\n${stringify({ title: settings.title })}---\n\n${options.i18n.t('homeBody')}\n`,
    );
    await Bun.write(
      join(content, '_left.astro'),
      `---\nimport SearchWidget from '@widgets/sidebar/SearchWidget.astro';\nimport RecentPosts from '@widgets/sidebar/RecentPosts.astro';\nimport TagCloud from '@widgets/sidebar/TagCloud.astro';\nimport WebsiteStatus from '@widgets/sidebar/WebsiteStatus.astro';\n---\n\n<SearchWidget />\n<RecentPosts />\n<TagCloud />\n<WebsiteStatus />\n`,
    );
    const config = await readSiteConfig(root);
    await writeSiteConfig(
      config,
      prepareSiteConfig(config, {
        avatar: '/assets/images/avatar.png',
        description: '',
        theme: 'nayuta',
        since: today(),
        postsPerPage: 10,
        copyright: { enabled: false },
        links: [{ text: options.i18n.t('postsLink'), url: '/posts' }],
        ...Object.fromEntries(
          Object.entries(settings).filter(([, value]) => value !== undefined),
        ),
      }),
    );
    await Bun.write(
      join(root, '.nayuta-template.json'),
      `${JSON.stringify({ repository: template, ref, revision }, null, 2)}\n`,
    );
    await initializeGit(root);
    initialized = true;
    if (options.commit !== false) await gitIdentity(root);
    if (options.install !== false) await run(['bun', 'install'], { cwd: root });
    if (options.commit !== false) await initialCommit(root);
    if (options.githubRepo)
      await createGithubRepository(
        root,
        options.githubRepo,
        options.visibility ?? 'private',
        options.push ?? false,
      );
    return {
      path: root,
      templateRevision: revision,
      committed: options.commit !== false,
    };
  } catch (error) {
    if (!initialized) await rm(root, { recursive: true, force: true });
    if (
      initialized &&
      !(error instanceof NayutaError && error.code === 'errorIdentity')
    )
      throw new NayutaError('errorPartial', {
        path: root,
        detail:
          error instanceof NayutaError
            ? options.i18n.t(error.code, error.params)
            : String(error),
      });
    throw error;
  }
}
export async function setupBlog(directory: string, settings: SiteSettings) {
  const root = await discoverProject(directory);
  const snapshot = await readSiteConfig(root);
  await writeSiteConfig(snapshot, prepareSiteConfig(snapshot, settings));
  return { path: snapshot.path };
}
