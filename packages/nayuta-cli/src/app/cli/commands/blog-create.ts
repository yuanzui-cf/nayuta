import type { Command } from 'commander';
import { resolve } from 'node:path';
import type { AppContext } from '../../context';
import { createBlog } from '../../../core/blog/service';
import {
  defaultTemplate,
  defaultTemplateRef,
} from '../../../infrastructure/template';
import { NayutaError } from '../../../core/errors';
import { settingsOptions, readSettings } from './blog-setup';
import { output } from '../output';
export function registerBlogCreate(blog: Command, context: AppContext): void {
  const t = context.i18n.t.bind(context.i18n);
  settingsOptions(
    blog.command('create <directory>').description(t('createBlog')),
    context,
  )
    .option('--template <repository>', t('template'), defaultTemplate)
    .option('--template-ref <ref>', t('templateRef'), defaultTemplateRef)
    .option('--no-install', t('install'))
    .option('--no-commit', t('commit'))
    .option('--github-repo <owner/name>', t('githubRepo'))
    .option('--visibility <visibility>', t('visibility'), 'private')
    .option('--push', t('push'))
    .action(async (directory: string, options) => {
      if (!['private', 'public'].includes(options.visibility))
        throw new NayutaError('errorArguments', {
          detail: '--visibility private / public',
        });
      output(
        context,
        await createBlog({
          directory: resolve(context.cwd, directory),
          settings: readSettings(options),
          i18n: context.i18n,
          template: options.template,
          templateRef: options.templateRef,
          install: options.install,
          commit: options.commit,
          githubRepo: options.githubRepo,
          visibility: options.visibility,
          push: options.push,
        }),
        true,
      );
    });
}
