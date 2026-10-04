import { resolve } from 'node:path';

const root = resolve(import.meta.dir, '..');
const version: string = (await Bun.file(resolve(root, 'package.json')).json())
  .version;
if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version))
  throw new Error(`Invalid theme version: ${version}`);
const tag = process.env.GITHUB_REF?.startsWith('refs/tags/')
  ? process.env.GITHUB_REF.slice(10).replace(/^v/, '')
  : undefined;
if (tag && tag !== version)
  throw new Error(`Tag ${tag} differs from theme ${version}`);
const write = process.argv.includes('--write');
const manifestPath = resolve(root, 'packages/nayuta-cli/Cargo.toml');
const lockPath = resolve(root, 'packages/nayuta-cli/Cargo.lock');
const wrapperPath = resolve(root, 'packages/nayuta-cli-npm/package.json');
const bunLockPath = resolve(root, 'bun.lock');
const bunLock = await Bun.file(bunLockPath).text();
const workspacePattern =
  /("packages\/nayuta-cli-npm"\s*:\s*\{[\s\S]*?"version"\s*:\s*")([^"]+)/;
const workspaceVersion = bunLock.match(workspacePattern)?.[2];
const manifest = await Bun.file(manifestPath).text();
const lock = await Bun.file(lockPath).text();
const wrapper = await Bun.file(wrapperPath).json();
const cargoVersion = manifest.match(
  /\[package\][\s\S]*?\nversion = "([^"]+)"/,
)?.[1];
const lockVersion = lock.match(/name = "nayuta-cli"\nversion = "([^"]+)"/)?.[1];
if (
  !write &&
  [cargoVersion, lockVersion, wrapper.version, workspaceVersion].some(
    (v) => v !== version,
  )
) {
  throw new Error(
    `Versions differ: theme ${version}, Cargo ${cargoVersion}, lock ${lockVersion}, wrapper ${wrapper.version}. Run bun run cli:version:sync.`,
  );
}
if (write) {
  await Bun.write(
    manifestPath,
    manifest.replace(
      /(\[package\][\s\S]*?\nversion = ")[^"]+/,
      (_match, prefix) => prefix + version,
    ),
  );
  await Bun.write(
    lockPath,
    lock.replace(
      /(name = "nayuta-cli"\nversion = ")[^"]+/,
      (_match, prefix) => prefix + version,
    ),
  );
  wrapper.version = version;
  await Bun.write(wrapperPath, JSON.stringify(wrapper, null, 2) + '\n');
  if (!workspaceVersion)
    throw new Error('Bun workspace lock entry is missing; run bun install.');
  await Bun.write(
    bunLockPath,
    bunLock.replace(workspacePattern, (_match, prefix) => prefix + version),
  );
}
console.log(`Nayuta theme, Rust and npm versions: ${version}`);
