#!/usr/bin/env bun
import { Command } from 'commander';
import { detectLocale, I18n } from './i18n';

const args = process.argv.slice(2);
const languageIndex = args.indexOf('--locale');
const i18n = new I18n(
  detectLocale({
    explicit: languageIndex < 0 ? undefined : args[languageIndex + 1],
  }),
);
const command = new Command('nayuta')
  .description(i18n.t('appDescription'))
  .version('0.1.0', '-V, --version', i18n.t('versionHelp'))
  .helpOption('-h, --help', i18n.t('help'))
  .option('--locale <locale>', i18n.t('localeHelp'));
command.parse();
if (!args.length) command.outputHelp();
