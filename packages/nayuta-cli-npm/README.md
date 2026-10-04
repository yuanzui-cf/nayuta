# Nayuta npm launcher

This package starts the matching native Rust `nayuta` binary with inherited
standard input/output, so CLI and TUI behavior is identical to the native release.
Development and tests use Bun; the bundled launcher also runs under Node.js 22+.

After the scoped packages are published:

```sh
bunx --bun @yuanzui-cf/nayuta --help
npx @yuanzui-cf/nayuta --help
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

```sh
bun run cli:wrapper:check
bun run cli:wrapper:test
bun run cli:wrapper:build
NAYUTA_BINARY="$PWD/packages/nayuta-cli/target/debug/nayuta" \
  bun packages/nayuta-cli-npm/dist/cli.js --version
```
