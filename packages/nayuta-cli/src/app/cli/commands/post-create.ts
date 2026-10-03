import type { Command } from 'commander';
import type { AppContext } from '../../context';
import { createPost } from '../../../core/post/service';
import { output } from '../output';
export function registerPostCreate(
  command: Command,
  context: AppContext,
): void {
  const t = context.i18n.t.bind(context.i18n);
  const post = command.command('post').description(t('createPost'));
  post
    .command('create')
    .description(t('createPost'))
    .requiredOption('--title <title>', t('title'))
    .option('--slug <slug>', t('slug'))
    .option('--format <format>', t('format'), 'md')
    .option('--publish-date <date>', t('publishDate'))
    .option('--description <text>', t('description'))
    .option('--tags <tags>', t('tags'))
    .option('--cover <path-or-url>', t('cover'))
    .option('--published', t('published'))
    .option('--no-date-prefix', t('noDatePrefix'))
    .action(async (options) =>
      output(
        context,
        await createPost({
          directory: context.cwd,
          i18n: context.i18n,
          title: options.title,
          slug: options.slug,
          format: options.format,
          publishDate: options.publishDate,
          description: options.description,
          tags:
            options.tags === undefined ? undefined : options.tags.split(','),
          cover: options.cover,
          published: options.published,
          datePrefix: options.datePrefix,
          assetBase: context.cwd,
        }),
        true,
      ),
    );
}
