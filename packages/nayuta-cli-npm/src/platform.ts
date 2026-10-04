export function platformPackage(
  name: string,
  platform: string,
  arch: string,
): string {
  if (
    !['darwin', 'linux', 'win32'].includes(platform) ||
    !['x64', 'arm64'].includes(arch)
  ) {
    throw new Error(
      `Unsupported platform: ${platform}/${arch}. Download a native binary from https://github.com/yuanzui-cf/nayuta/releases.`,
    );
  }
  return `${name}-${platform}-${arch}`;
}
