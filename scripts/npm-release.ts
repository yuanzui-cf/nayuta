import { platforms } from './distribution';
import { validateVersion } from './versions';

export type Platform = (typeof platforms)[number];

export function platformVersion(version: string, platform: Platform): string {
  validateVersion(version);
  if (version.includes('+'))
    throw new Error('npm release versions must not include build metadata.');
  return `${version}-${platform.platform}-${platform.arch}`;
}

export function platformDependencies(name: string, version: string) {
  return Object.fromEntries(
    platforms.map((platform) => [
      `${name}-${platform.platform}-${platform.arch}`,
      `npm:${name}@${platformVersion(version, platform)}`,
    ]),
  );
}

export function npmTarballName(name: string, version: string): string {
  return `${name.replace(/^@/, '').replace('/', '-')}-${version}.tgz`;
}

export interface NpmPackage {
  path: string;
  pkg: {
    name: string;
    version: string;
    os?: string[];
    cpu?: string[];
    optionalDependencies?: Record<string, string>;
  };
}

export function publishPlan(
  packages: NpmPackage[],
  name: string,
  version: string,
  prerelease = false,
): (NpmPackage & { tag: string })[] {
  const dependencies = platformDependencies(name, version);
  if (packages.length !== platforms.length + 1)
    throw new Error('All six platform packages and one wrapper are required.');
  const wrapper = packages.find(
    ({ pkg }) => pkg.name === name && pkg.version === version,
  );
  if (!wrapper || wrapper.pkg.os || wrapper.pkg.cpu)
    throw new Error('The matching platform-independent wrapper is missing.');
  const actual = wrapper.pkg.optionalDependencies;
  if (
    !actual ||
    Object.keys(actual).length !== platforms.length ||
    Object.entries(dependencies).some(([key, value]) => actual[key] !== value)
  ) {
    throw new Error(
      'Wrapper platform aliases do not match the release version.',
    );
  }
  const native = platforms.map((platform) => {
    const expected = platformVersion(version, platform);
    const matches = packages.filter(
      ({ pkg }) => pkg.name === name && pkg.version === expected,
    );
    const archive = matches[0];
    if (
      matches.length !== 1 ||
      archive?.pkg.os?.length !== 1 ||
      archive.pkg.os[0] !== platform.platform ||
      archive.pkg.cpu?.length !== 1 ||
      archive.pkg.cpu[0] !== platform.arch
    ) {
      throw new Error(
        `Missing or invalid platform package: ${name}@${expected}`,
      );
    }
    return { ...archive, tag: 'native' };
  });
  return [
    ...native,
    {
      ...wrapper,
      tag: prerelease || version.includes('-') ? 'next' : 'latest',
    },
  ];
}
