import { Command, CommanderError } from 'commander';
import type { AppContext } from '../context';
import { NayutaError } from '../../core/errors';
import { registerBlogCreate } from './commands/blog-create';
import { registerBlogSetup } from './commands/blog-setup';
import { registerPostCreate } from './commands/post-create';
import { registerTui } from './commands/tui';
import { fail } from './output';
export function createCommand(context: AppContext): Command {
  const t = context.i18n.t.bind(context.i18n);
  const command = new Command('nayuta')
    .description(t('appDescription'))
    .version('0.1.0', '-V, --version', t('versionHelp'))
    .helpOption('-h, --help', t('help'))
    .option('--cwd <directory>', t('cwdHelp'))
    .option('--locale <locale>', t('localeHelp'))
    .option('--json', t('jsonHelp'))
    .option('--no-color', t('colorHelp'))
    .exitOverride();
  command.configureOutput({ writeErr: () => {}, outputError: () => {} });
  const blog = command.command('blog').description(t('blogDescription'));
  registerBlogCreate(blog, context);
  registerBlogSetup(blog, context);
  registerPostCreate(command, context);
  registerTui(command, context);
  return command;
}
export async function runCli(
  args: string[],
  context: AppContext,
): Promise<void> {
  const command = createCommand(context);
  try {
    if (!args.length) {
      command.outputHelp();
      return;
    }
    await command.parseAsync(args, { from: 'user' });
  } catch (error) {
    if (error instanceof CommanderError) {
      if (error.exitCode === 0) return;
      fail(
        context,
        new NayutaError('errorArguments', {
          detail: error.message.replace(/^error: /, ''),
        }),
      );
    } else fail(context, error);
  }
}
