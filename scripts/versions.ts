import { randomUUID } from 'node:crypto';
import { rename, rm, stat, writeFile } from 'node:fs/promises';
import { basename, dirname, join } from 'node:path';

export const versionFiles = [
  'package.json',
  'packages/nayuta-cli/Cargo.toml',
  'packages/nayuta-cli/Cargo.lock',
  'packages/nayuta-cli-npm/package.json',
  'bun.lock',
] as const;

const patterns = [
  /^(\s{2}"version"\s*:\s*")([^"\r\n]+)(")/gm,
  /(\[package\][\s\S]*?^version\s*=\s*")([^"\r\n]+)(")/gm,
  /(^name = "nayuta-cli"\r?\nversion = ")([^"\r\n]+)(")/gm,
  /^(\s{2}"version"\s*:\s*")([^"\r\n]+)(")/gm,
  /("packages\/nayuta-cli-npm"\s*:\s*\{[^}]*?"version"\s*:\s*")([^"\r\n]+)(")/g,
];

export interface VersionFile {
  path: string;
  source: string;
  version: string;
  pattern: RegExp;
}

export interface VersionChange {
  path: string;
  before: string;
  after: string;
}

export function validateVersion(version: string): string {
  const match = version.match(
    /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$/,
  );
  if (
    !match ||
    match.slice(1, 4).some((part) => !Number.isSafeInteger(Number(part))) ||
    match[4]?.split('.').some((part) => /^0\d+$/.test(part))
  ) {
    throw new Error(`Invalid SemVer version: ${version}`);
  }
  return version;
}

export function nextVersion(current: string, requested: string): string {
  validateVersion(current);
  if (['major', 'minor', 'patch'].includes(requested)) {
    const parts = current.split(/[+-]/)[0]!.split('.').map(Number);
    const index = ['major', 'minor', 'patch'].indexOf(requested);
    parts[index] = parts[index]! + 1;
    for (let i = index + 1; i < parts.length; i++) parts[i] = 0;
    return validateVersion(parts.join('.'));
  }
  return validateVersion(requested.replace(/^v/, ''));
}

export async function readVersions(root: string): Promise<VersionFile[]> {
  return Promise.all(
    versionFiles.map(async (path, i) => {
      const source = await Bun.file(join(root, path)).text();
      const pattern = patterns[i]!;
      const matches = [...source.matchAll(pattern)];
      if (matches.length !== 1) {
        throw new Error(`${path}: expected exactly one project version entry`);
      }
      const version = validateVersion(matches[0]![2]!);
      let parsed: string;
      if (path.endsWith('.json')) {
        parsed = JSON.parse(source).version;
      } else if (path.endsWith('.toml')) {
        const manifest = Bun.TOML.parse(source) as {
          package: { version: string };
        };
        parsed = manifest.package.version;
      } else if (path.endsWith('Cargo.lock')) {
        const lock = Bun.TOML.parse(source) as {
          package: { name: string; version: string }[];
        };
        const own = lock.package.filter((pkg) => pkg.name === 'nayuta-cli');
        if (own.length !== 1) throw new Error(`${path}: missing CLI package`);
        parsed = own[0]!.version;
      } else {
        const lock = Bun.JSONC.parse(source) as {
          workspaces: Record<string, { version: string }>;
        };
        parsed = lock.workspaces['packages/nayuta-cli-npm']!.version;
      }
      if (parsed !== version) {
        throw new Error(
          `${path}: version entry does not match parsed manifest`,
        );
      }
      return { path, source, version, pattern };
    }),
  );
}

export function versionChanges(
  files: VersionFile[],
  version: string,
): VersionChange[] {
  validateVersion(version);
  return files.map(({ path, source, pattern }) => ({
    path,
    before: source,
    after: source.replace(
      pattern,
      (_match, prefix, _version, suffix) => prefix + version + suffix,
    ),
  }));
}

async function replace(path: string, source: string): Promise<void> {
  const temp = join(dirname(path), `.${basename(path)}.${randomUUID()}.tmp`);
  try {
    await writeFile(temp, source, {
      flag: 'wx',
      mode: (await stat(path)).mode,
    });
    await rename(temp, path);
  } finally {
    await rm(temp, { force: true });
  }
}

export async function restoreVersions(
  root: string,
  changes: VersionChange[],
): Promise<void> {
  for (const change of [...changes].reverse()) {
    const path = join(root, change.path);
    if ((await Bun.file(path).text()) === change.after) {
      await replace(path, change.before);
    }
  }
}

export async function writeVersions(
  root: string,
  changes: VersionChange[],
): Promise<void> {
  try {
    for (const change of changes) {
      const path = join(root, change.path);
      if ((await Bun.file(path).text()) !== change.before) {
        throw new Error(`${change.path}: changed while updating versions`);
      }
      if (change.before !== change.after) await replace(path, change.after);
    }
  } catch (error) {
    await restoreVersions(root, changes);
    throw error;
  }
}
