import type { Command } from 'commander';
import type { AppContext } from '../../context';
import type { SiteSettings } from '../../../core/blog/schema';
import { setupBlog } from '../../../core/blog/service';
import { NayutaError } from '../../../core/errors';
import { output } from '../output';
export function settingsOptions(
  command: Command,
  context: AppContext,
): Command {
  const t = context.i18n.t.bind(context.i18n);
  return command
    .option('--title <title>', t('title'))
    .option('--author <author>', t('author'))
    .option('--avatar <path>', t('avatar'))
    .option('--description <text>', t('description'))
    .option('--site-url <url>', t('siteUrl'))
    .option('--theme <theme>', t('theme'))
    .option('--since <date>', t('since'))
    .option('--posts-per-page <count>', t('postsPerPage'))
    .option('--links <json>', t('links'))
    .option('--copyright-enabled <boolean>', t('copyrightEnabled'))
    .option('--copyright-license <license>', t('copyrightLicense'));
}
export function readSettings(
  options: Record<string, string | undefined>,
): SiteSettings {
  let links;
  try {
    if (options.links !== undefined) links = JSON.parse(options.links);
  } catch {
    throw new NayutaError('errorValidation', {
      field: 'links',
      detail: 'JSON',
    });
  }
  if (
    options.copyrightEnabled !== undefined &&
    !['true', 'false'].includes(options.copyrightEnabled)
  )
    throw new NayutaError('errorValidation', {
      field: 'copyright.enabled',
      detail: 'true / false',
    });
  return {
    title: options.title,
    author: options.author,
    avatar: options.avatar,
    description: options.description,
    site_url: options.siteUrl,
    theme: options.theme as SiteSettings['theme'],
    since: options.since,
    postsPerPage:
      options.postsPerPage === undefined
        ? undefined
        : Number(options.postsPerPage),
    links,
    copyright:
      options.copyrightEnabled === undefined &&
      options.copyrightLicense === undefined
        ? undefined
        : {
            enabled:
              options.copyrightEnabled === undefined
                ? undefined
                : options.copyrightEnabled === 'true',
            license: options.copyrightLicense as NonNullable<
              SiteSettings['copyright']
            >['license'],
          },
  };
}
export function registerBlogSetup(blog: Command, context: AppContext): void {
  settingsOptions(
    blog.command('setup').description(context.i18n.t('setupBlog')),
    context,
  ).action(async (options) =>
    output(context, await setupBlog(context.cwd, readSettings(options))),
  );
}
