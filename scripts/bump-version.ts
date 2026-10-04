import { root, run } from './distribution';
import {
  nextVersion,
  readVersions,
  restoreVersions,
  versionChanges,
  versionFiles,
  writeVersions,
} from './versions';

export async function bumpVersion(
  directory: string,
  requested: string,
): Promise<string> {
  const files = await readVersions(directory);
  const current = files[0]!.version;
  const version = nextVersion(current, requested);
  if (Bun.semver.order(version, current) <= 0) {
    throw new Error(`Version ${version} must be newer than ${current}`);
  }
  const tracked = await run(
    ['git', 'ls-files', '--', ...versionFiles],
    directory,
  );
  if (tracked.split('\n').length !== versionFiles.length) {
    throw new Error('All project version files must be tracked in Git.');
  }
  const dirty = await run(
    ['git', 'status', '--porcelain', '--', ...versionFiles],
    directory,
  );
  if (dirty) {
    throw new Error(`Version files have uncommitted changes:\n${dirty}`);
  }
  await run(['git', 'var', 'GIT_AUTHOR_IDENT'], directory);
  await run(['git', 'var', 'GIT_COMMITTER_IDENT'], directory);
  const changes = versionChanges(files, version);
  await writeVersions(directory, changes);
  try {
    console.log(
      await run(
        [
          'git',
          'commit',
          '--only',
          '-m',
          `chore: Bump version to ${version}`,
          '--',
          ...versionFiles,
        ],
        directory,
      ),
    );
  } catch (error) {
    await restoreVersions(directory, changes);
    throw error;
  }
  return version;
}

if (import.meta.main) {
  const args = process.argv.slice(2);
  if (args.length === 1 && ['--help', '-h'].includes(args[0]!)) {
    console.log('Usage: bun run bump <version|major|minor|patch>');
  } else if (args.length !== 1) {
    console.error('Usage: bun run bump <version|major|minor|patch>');
    process.exitCode = 2;
  } else {
    try {
      const version = await bumpVersion(root, args[0]!);
      console.log(`All Nayuta versions updated to ${version}.`);
    } catch (error) {
      console.error(error instanceof Error ? error.message : error);
      process.exitCode = 1;
    }
  }
}
