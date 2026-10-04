# CLI development

[< Development guide](index.md)

`packages/nayuta-cli` is a single Rust crate. `app/cli` and `app/tui` share
`core` use cases, schemas and forms. `infra` owns TypeScript/YAML parsing, file
transactions, Git, GitHub, cache and tracing; `i18n` embeds regional JSON catalogs.
`packages/nayuta-cli-npm` only locates and launches native binaries.

## Static parsing contract

Every version uses Oxc 0.110 to parse `src/types/config.ts`, imported types/constants,
`src/content.config.ts`, `src/config.ts` and TypeScript/JSONC path aliases. No project
JavaScript executes during detection, creation or editing. There is no separately
maintained schema JSON or version-specific parser.

Supported TypeScript includes interfaces/extends, aliases, optional fields, literal
unions, arrays, imported constants, `Date` types, `(typeof values)[number]`, object
and array literals, literal templates, static `new Date(string)` and resource
imports ending in `?url`. Computed values in existing configuration remain read
only; root spreads or computed keys cannot safely be edited.

Supported Zod operations include object/looseObject, string/number/boolean, image,
array, literal, union, enum, optional/nullable/default, trim/min/max/int/positive,
regex and the actual `tags => [...new Set(tags)]` transformation. Arbitrary calls,
callbacks and refinements are rejected, with source locations. Add a deliberate
AST adapter and behavior tests when the theme introduces another operation.

The post loader's base comes from its actual AST. The currently supported loader
pattern is `['**/[^_]*.{md,mdx}', '!**/assets/**', '!**/_*/**']`, with Astro's default
ID function. Custom patterns and ID callbacks fail explicitly rather than silently
creating content outside the collection. ID generation uses github-slugger 2's
vendored expression (with its license), including UTF-16 behavior. Configuration
URL/page-size checks and independently inherited copyright requirements apply
when those fields exist. The actual Astro config makes `site_url` required, and literal
defaults for theme, author, page size and copyright are inferred from consumer
ASTs. Keep these semantic adapters aligned with theme consumers.

Article operations read static configuration fields independently. Unrelated
computed fields do not prevent article edits; computed copyright settings require
an explicit article override when inheritance cannot be evaluated.

Settings patch changed value spans and nested static objects, preserving untouched
imports, comments and expressions. A changed local avatar can use
`@asset:src/content/assets/path.svg`, which creates a `?url` import. Frontmatter uses
`yaml-edit` for local CST edits and `yaml-rust2` for independent semantic checks.
Duplicate keys, anchors, aliases, tags and merges are rejected to avoid ambiguous
edits. The body and delimiters are copied byte for byte, including BOM/CRLF.

Writers use a shared advisory lock keyed by canonical project/destination path,
same-directory staging and snapshot checks before replacement. Source/schema
changes require reopening the editor. This coordinates Nayuta processes; external
editors are protected by pre-save hash checks, not by an OS-enforced mandatory lock.

## Validation

```sh
cargo +1.95.0 fmt --manifest-path packages/nayuta-cli/Cargo.toml --check
cargo +1.95.0 clippy --manifest-path packages/nayuta-cli/Cargo.toml --all-targets -- -D warnings
cargo +1.95.0 test --manifest-path packages/nayuta-cli/Cargo.toml --locked
NAYUTA_TEST_SITE_BUILD=1 cargo +1.95.0 test --manifest-path packages/nayuta-cli/Cargo.toml create_has_one
bun run cli:version:check
bun run cli:wrapper:check
bun run cli:wrapper:test
bun run cli:wrapper:build
bun run format:check
bun run check
bun run test
bun run build
```

Historical fixtures contain actual types and collection definitions from v0.1.0
and v0.2.0. Tests confirm that copyright and search-exclusion fields follow the
selected version, not the CLI version. Partial-fetch tests assert that excluded
blobs are still missing, rather than merely absent from the checkout.

## Version and distribution

The root theme version is authoritative. Run `bun run cli:version:sync` after
changing it; the check command verifies Cargo.toml, Cargo.lock, npm wrapper, Bun's
workspace lock entry and a
release tag when present. Platform packages are generated with the same version
and exact optional dependencies. They are not installed in the development
workspace before publication.

```sh
bun run cli:template:pack
cargo +1.95.0 build --manifest-path packages/nayuta-cli/Cargo.toml \
  --release --locked --target aarch64-apple-darwin
bun run cli:pack aarch64-apple-darwin
bun scripts/archive-cli.ts aarch64-apple-darwin
```

Template archives omit all authored content and public files, preserve actual
schema code, and include source version/commit metadata plus a SHA-256 sidecar.
Rust creation generates content and logo resources. The release profile disables
debug information and strips symbols; PDB/dSYM output never enters platform packs.
Windows uses the static MSVC runtime. Platform packages contain dependency license
references and the vendored github-slugger license.

The release matrix targets macOS, Windows and Linux on x64/arm64. Linux uses
static musl. Native smoke tests, binary inspection, archives and checksums are
part of the workflow. Cross-platform compilation alone does not establish locale,
IME, mouse, Ctrl+C or terminal behavior: macOS is locally exercised, while Linux
and Windows require successful CI and native interaction checks before claiming
equivalent platform coverage.

The prepared npm scope is `@yuanzui-cf`; verify account ownership before publishing.
The unscoped `nayuta` package belongs to another project, so `bunx nayuta` cannot
currently be promised. Use `bunx --bun @yuanzui-cf/nayuta` once published.
