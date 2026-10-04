# Nayuta npm launcher

This package starts the matching native Rust `nayuta` binary with inherited
standard input/output, so CLI and TUI behavior is identical to the native release.
Development and tests use Bun; the bundled launcher also runs under Node.js 22+.

The wrapper contains the JavaScript launcher. Its platform-specific optional
dependency contains the Rust binary, which already embeds the four locale
catalogs, starter logo and other CLI assets. No separate assets directory is needed
at runtime. Installing the wrapper selects the matching binary automatically.
Theme templates are fetched from the selected GitHub release or local cache.

GitHub Releases include the wrapper `.tgz`, all six platform `.tgz` packages, native
archives and the clean template archive. npm publishes the platform packages and
wrapper together at the same version.

After the scoped packages are published:

```sh
bunx --bun @yuanzui-cf/nayuta --help
npx @yuanzui-cf/nayuta --help
```

For a short command after one global installation:

```sh
bun add --global @yuanzui-cf/nayuta
nayuta
```

With no arguments, `nayuta` opens the interactive interface. Use `create` outside
a Nayuta blog and `post new` inside one. Configuration and frontmatter editing
are available in TUI only. See the [native CLI guide](https://github.com/yuanzui-cf/nayuta/blob/main/packages/nayuta-cli/README.md)
in the source repository.

The unscoped npm name `nayuta` is already occupied. These sources prepare
`@yuanzui-cf/nayuta`; publishing requires access to that scope. Nothing is
published by local build/pack commands. Install optional dependencies so the
platform binary is included. No `postinstall` download is required.

The development workspace has no platform dependencies. `scripts/package-cli.ts`
adds six exact-version optional dependencies to the generated publish package.
Use `NAYUTA_BINARY=/absolute/path/to/nayuta` for local launcher testing; its version
must match the wrapper.

For everyday development from the repository root, `bun run cli` automatically
builds and launches the native CLI/TUI. `bun run cli:release` prepares native and
npm archives, the template and checksums for the current platform in `dist/cli`.

```sh
bun run cli:wrapper:check
bun run cli:wrapper:test
bun run cli:wrapper:build
NAYUTA_BINARY="$PWD/packages/nayuta-cli/target/debug/nayuta" \
  bun packages/nayuta-cli-npm/dist/cli.js --version
```
