import { mkdir, copyFile, chmod } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { platforms, root, run } from './distribution';
import { platformDependencies, platformVersion } from './npm-release';

const target = process.argv[2];
const platform = platforms.find((p) => p.target === target);
if (!platform) throw new Error('Supply a supported Rust target triple.');
const pkg = await Bun.file(
  join(root, 'packages/nayuta-cli-npm/package.json'),
).json();
const output = resolve(process.argv[3] ?? join(root, 'dist/cli'));
await mkdir(output, { recursive: true });
const name = pkg.name;
const nativeVersion = platformVersion(pkg.version, platform);
const directory = join(output, `${platform.platform}-${platform.arch}`);
await mkdir(directory, { recursive: true });
const executable = platform.platform === 'win32' ? 'nayuta.exe' : 'nayuta';
const binaryRoot =
  process.env.NAYUTA_BUILD_DIR ?? join(root, 'packages/nayuta-cli/target');
await copyFile(
  join(binaryRoot, target!, 'release', executable),
  join(directory, executable),
);
await chmod(join(directory, executable), 0o755);
await copyFile(join(root, 'LICENSE'), join(directory, 'LICENSE'));
await copyFile(
  join(root, 'packages/nayuta-cli/assets/github-slugger-LICENSE'),
  join(directory, 'github-slugger-LICENSE'),
);
const metadata = JSON.parse(
  await run([
    'cargo',
    '+1.95.0',
    'metadata',
    '--format-version',
    '1',
    '--locked',
    '--manifest-path',
    join(root, 'packages/nayuta-cli/Cargo.toml'),
  ]),
);
const licenses = metadata.packages
  .filter((p: { name: string }) => p.name !== 'nayuta-cli')
  .sort((a: { name: string }, b: { name: string }) =>
    a.name.localeCompare(b.name),
  )
  .map(
    (p: { name: string; version: string; license: string | null }) =>
      `${p.name} ${p.version}\n  License: ${p.license ?? 'See crate source'}\n  Source: https://crates.io/crates/${p.name}/${p.version}`,
  );
await Bun.write(
  join(directory, 'THIRD-PARTY.txt'),
  `Nayuta Rust dependencies\n\n${licenses.join('\n\n')}\n`,
);
await Bun.write(
  join(directory, 'package.json'),
  JSON.stringify(
    {
      name,
      version: nativeVersion,
      description: `Nayuta native binary for ${platform.platform}/${platform.arch}`,
      license: 'MIT',
      os: [platform.platform],
      cpu: [platform.arch],
      files: [
        executable,
        'LICENSE',
        'github-slugger-LICENSE',
        'THIRD-PARTY.txt',
      ],
      publishConfig: { access: 'public' },
    },
    null,
    2,
  ) + '\n',
);
const wrapper = join(output, 'wrapper');
await mkdir(join(wrapper, 'dist'), { recursive: true });
const result = await Bun.build({
  entrypoints: [join(root, 'packages/nayuta-cli-npm/src/cli.ts')],
  target: 'node',
  format: 'esm',
  outdir: join(wrapper, 'dist'),
});
if (!result.success)
  throw new AggregateError(result.logs, 'Wrapper build failed');
await chmod(join(wrapper, 'dist/cli.js'), 0o755);
pkg.optionalDependencies = platformDependencies(pkg.name, pkg.version);
delete pkg.scripts;
await Bun.write(
  join(wrapper, 'package.json'),
  JSON.stringify(pkg, null, 2) + '\n',
);
await copyFile(join(root, 'LICENSE'), join(wrapper, 'LICENSE'));
await copyFile(
  join(root, 'packages/nayuta-cli-npm/README.md'),
  join(wrapper, 'README.md'),
);
console.log(
  `Prepared ${name}@${nativeVersion} and ${pkg.name}@${pkg.version} in ${output}`,
);
