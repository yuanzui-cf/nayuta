import { resolve } from 'node:path';

export const root = resolve(import.meta.dir, '..');
export const platforms = [
  { platform: 'darwin', arch: 'arm64', target: 'aarch64-apple-darwin' },
  { platform: 'darwin', arch: 'x64', target: 'x86_64-apple-darwin' },
  { platform: 'linux', arch: 'arm64', target: 'aarch64-unknown-linux-musl' },
  { platform: 'linux', arch: 'x64', target: 'x86_64-unknown-linux-musl' },
  { platform: 'win32', arch: 'arm64', target: 'aarch64-pc-windows-msvc' },
  { platform: 'win32', arch: 'x64', target: 'x86_64-pc-windows-msvc' },
] as const;

export async function run(args: string[], cwd = root): Promise<string> {
  const process = Bun.spawn(args, { cwd, stdout: 'pipe', stderr: 'inherit' });
  const output = await new Response(process.stdout).text();
  if (await process.exited !== 0) throw new Error(`Command failed: ${args.join(' ')}`);
  return output.trim();
}

export function templatePath(path: string): boolean {
  return (path.startsWith('src/') && !path.startsWith('src/content/')) ||
    ['package.json', 'bun.lock', 'astro.config.ts', 'tsconfig.json', 'LICENSE', '.gitignore'].includes(path);
}
