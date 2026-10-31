# Nayuta CLI / TUI

A Rust companion to the Nayuta theme. Its version follows the theme's root
`package.json`. The executable is always `nayuta`.

From the repository root, use Bun's entry point to build and launch automatically:

```sh
bun run cli
bun run cli --help
bun run cli --cwd ./my-blog post new hello --title "Hello"
```

Arguments and terminal input/output pass through to the native binary. Cargo's
incremental build reuses the executable when sources have not changed. To create
release binaries and packages for the current platform, run `bun run cli:release`.
You can also invoke Cargo directly:

```sh
cargo +1.95.0 build --manifest-path packages/nayuta-cli/Cargo.toml --release --locked
packages/nayuta-cli/target/release/nayuta --help
```

The toolchain is pinned in `rust-toolchain.toml`. Run Cargo inside this directory,
or use `cargo +1.95.0` when invoking it from the repository root. Git is required
for blog creation. Bun is required to install and run a generated Astro blog.

## Commands and modes

No arguments opens the Ratatui interface. Only global interface flags also open
TUI. Invalid arguments print help and exit 2. TUI requires interactive stdin and
stdout. `--help` and `--version` never access the network.

The closest Nayuta blog in the current directory or its ancestors selects Ready
mode. Discovery uses theme files and Oxc parsing; it works without an upstream
remote, including newly created single-commit blogs. Outside a blog, only
`create` is offered. Inside a blog, only `post new` is offered in CLI; TUI offers
settings, article creation, and frontmatter editing.

```sh
nayuta create my-blog --set 'title="My Blog"' \
  --set 'author="Writer"' --set 'site_url="https://example.com"'
nayuta create older-blog --ref v0.2.0 --config ./config.json
nayuta create cached-blog --ref v0.3.0 --offline --config ./config.json
nayuta --cwd ./my-blog post new hello --title "Hello" --tag Nayuta
nayuta --cwd ./my-blog post new notes --title "Notes" --format mdx \
  --draft false --date-prefix false --set 'exclude_in_search=true'
nayuta --cwd ./my-blog --locale zh-HK --no-mouse
```

Creation selects the latest stable GitHub Release by default. `--prerelease`
includes prereleases, and `--ref` accepts an explicit tag or branch. Configuration
precedence is safe defaults, a JSON object from `--config`, then repeated
`--set path=JSON`. Template code is parsed, never executed to read schemas.

Official releases with a dedicated template asset use its SHA-256 checksum.
Older releases and branches use Git partial fetch and sparse checkout; the
`src/content/` and `public/` blobs are excluded. The starter generates an empty
homepage, pages/posts directories, and `src/content/assets/logo.svg`, imported
through Vite's `?url` pipeline. Demo content and upstream Git history are absent.
Creation writes exactly one root commit with the user's local Git identity for
both author and committer. Configure `git config user.name` and `user.email`
before creating a blog; configured signing is respected.

Articles follow [Writing Posts](https://github.com/yuanzui-cf/nayuta/wiki/Writing-Posts):
`src/content/posts/YYYY-MM-DD-slug/index.md` or `.mdx` and an `assets/` directory.
New articles default to drafts. Dates are quoted YAML strings; tags are normalized
from the selected site's Zod definition. Existing URL collisions are rejected.
`--frontmatter file.json` and `--set` support additional version-specific fields.
These operations do not make Git commits.

## TUI controls

Choose a release, tab to Continue, fill configuration, then Save. The final
version item accepts another tag or branch. Ready mode opens the action menu.
White outer borders, `>` focus markers, double borders, bold text and optional
blue accents keep selection and keyboard focus distinct, including without color.

| Key                          | Action                                                             |
| ---------------------------- | ------------------------------------------------------------------ |
| Tab / Shift+Tab              | Move focus; toggle list/button focus in version selection          |
| Up / Down, PageUp / PageDown | Navigate fields or list items                                      |
| Enter                        | Edit a value, cycle an enum/toggle, or activate the focused button |
| Space / Left / Right         | Cycle a boolean or enum                                            |
| Ctrl+U                       | Unset an optional field to inherit its value                       |
| Ctrl+S                       | Save the current form                                              |
| Esc                          | Cancel input, go back, or confirm discarding unsaved changes       |
| Ctrl+C                       | Exit 130, restoring the terminal                                   |
| R                            | Retry loading releases on the version page                         |

Text editing supports CJK graphemes, cursor movement, bracketed paste, and
Ctrl+A/E/K for start/end/clear. Arrays and complex unions use JSON inputs.
Type in the article list to filter by title or path. Copyright license controls
appear when copyright is enabled, including an inherited article setting.
Hidden values are retained. Computed configuration fields are read only.

Network and file operations run on a worker thread. Esc cancels a read operation
after it finishes; file writes finish before Ctrl+C exits, so a partially committed
blog is not abandoned. Terminal state is restored on normal exit, interrupts,
and panics. Configuration edits patch Oxc source spans; frontmatter edits use a
YAML concrete syntax tree. Untouched code, unknown metadata and article bodies
are retained. Concurrent changes fail instead of being overwritten.

## Language, logs and cache

Embedded catalogs support English, Simplified Chinese (mainland China), Traditional
Chinese (Taiwan), and Traditional Chinese (Hong Kong). Explicit `--locale` wins,
then `NAYUTA_LOCALE`, then OS preferences. macOS reads preferred languages through
CoreFoundation; Windows reads preferred UI languages; Linux reads locale environment
variables. `LC_ALL` / `LC_MESSAGES` override native preferences. Unmatched locales
fall back to English; `C` / `POSIX` selects English.

`--color auto|always|never` controls color; `NO_COLOR` disables the default.
`--verbose` enables diagnostic tracing. TUI logs go to the OS cache directory,
never onto the active screen; `--log-file` overrides the destination.
`NAYUTA_CACHE_DIR` overrides the release/template cache. `--offline` requires a
previously cached exact template. Optional `GH_TOKEN` / `GITHUB_TOKEN` authenticates
GitHub API requests. A failed release request can use the last cached list.

Exit codes: 0 success/help/version, 1 operation error, 2 argument/mode/TTY error,
130 interrupted. See [CLI development](../../docs/develop/cli.md) for static
parsing support, tests, release packaging and platform verification limits.
