import { NayutaError } from '../core/errors';
import { run } from './process';
export function gitEnvironment(): Record<string, string | undefined> {
  const env = { ...process.env };
  for (const key of [
    'GIT_DIR',
    'GIT_WORK_TREE',
    'GIT_INDEX_FILE',
    'GIT_AUTHOR_NAME',
    'GIT_AUTHOR_EMAIL',
    'GIT_COMMITTER_NAME',
    'GIT_COMMITTER_EMAIL',
    'GIT_AUTHOR_DATE',
    'GIT_COMMITTER_DATE',
  ])
    delete env[key];
  return env;
}
export async function initializeGit(root: string): Promise<void> {
  await run(['git', 'init', '--quiet', '--initial-branch=main', root], {
    env: gitEnvironment(),
  });
}
export async function gitIdentity(
  root: string,
): Promise<{ name: string; email: string }> {
  const options = { cwd: root, env: gitEnvironment(), allowFailure: true };
  const name = await run(['git', 'config', '--get', 'user.name'], options);
  const email = await run(['git', 'config', '--get', 'user.email'], options);
  if (!name.stdout || !email.stdout) throw new NayutaError('errorIdentity');
  return { name: name.stdout, email: email.stdout };
}
export async function initialCommit(root: string): Promise<void> {
  const identity = await gitIdentity(root);
  const env = {
    ...gitEnvironment(),
    GIT_AUTHOR_NAME: identity.name,
    GIT_AUTHOR_EMAIL: identity.email,
    GIT_COMMITTER_NAME: identity.name,
    GIT_COMMITTER_EMAIL: identity.email,
  };
  await run(['git', 'add', '--all'], { cwd: root, env });
  await run(
    ['git', 'commit', '--quiet', '-m', 'feat(blog): Initialize Nayuta blog'],
    { cwd: root, env },
  );
}
