import type { Command } from 'commander';
import type { AppContext } from '../../context';
export function registerTui(command: Command, context: AppContext): void {
  command
    .command('tui')
    .description(context.i18n.t('tuiHelp'))
    .action(async () => {
      const { runTui } = await import('../../tui');
      await runTui(context);
    });
}
