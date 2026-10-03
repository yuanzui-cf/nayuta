import { NayutaError } from '../core/errors';
import { gitEnvironment } from './git';
import { run } from './process';
export function validateGithub(
  repository: string | undefined,
  push: boolean,
  commit: boolean,
): void {
  if (
    (repository &&
      !/^[a-z\d](?:[a-z\d-]*[a-z\d])?\/[a-z\d._-]+$/i.test(repository)) ||
    (push && (!repository || !commit))
  )
    throw new NayutaError('errorGithub');
}
export async function createGithubRepository(
  root: string,
  repository: string,
  visibility: 'private' | 'public',
  push: boolean,
): Promise<void> {
  await run(
    [
      'gh',
      'repo',
      'create',
      repository,
      `--${visibility}`,
      '--source',
      root,
      '--remote',
      'origin',
    ],
    { cwd: root, env: gitEnvironment() },
  );
  if (push)
    await run(['git', 'push', '--set-upstream', 'origin', 'main'], {
      cwd: root,
      env: gitEnvironment(),
    });
}
