import { cp, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { basename, join } from 'node:path';
import { NayutaError } from '../core/errors';
import { exists } from './filesystem';
import { gitEnvironment } from './git';
import { run } from './process';
export const defaultTemplate = 'https://github.com/yuanzui-cf/nayuta.git';
export const defaultTemplateRef = 'v0.2.0';
export async function copyTemplate(
  destination: string,
  repository: string,
  ref: string,
): Promise<string> {
  if (!ref || ref.startsWith('-'))
    throw new NayutaError('errorTemplate', { path: ref });
  const temporary = await mkdtemp(join(tmpdir(), 'nayuta-template-'));
  const checkout = join(temporary, 'checkout');
  try {
    await run(
      [
        'git',
        'clone',
        '--quiet',
        '--depth=1',
        '--single-branch',
        '--branch',
        ref,
        '--',
        repository,
        checkout,
      ],
      { env: gitEnvironment() },
    );
    for (const required of [
      'package.json',
      'src/config.ts',
      'src/content.config.ts',
      'src/layouts/frame.astro',
    ]) {
      if (!(await exists(join(checkout, required))))
        throw new NayutaError('errorTemplate', { path: required });
    }
    const revision = (
      await run(['git', 'rev-parse', 'HEAD'], {
        cwd: checkout,
        env: gitEnvironment(),
      })
    ).stdout;
    const excluded = new Set(['.git', 'node_modules', 'dist', '.astro']);
    await cp(checkout, destination, {
      recursive: true,
      filter: (source) => !excluded.has(basename(source)),
    });
    return revision;
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}
