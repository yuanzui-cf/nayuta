# Nayuta CLI

A standalone TypeScript package for creating and managing Nayuta 0.2.x blogs.
Run with Bun 1.3 or newer and Git; native Windows arm64 needs Bun 1.4 or newer.
The interface supports English, Simplified Chinese, Traditional Chinese (Taiwan),
and Traditional Chinese (Hong Kong).

## Run from the checkout

```sh
bun install
bun run cli --help
bun run cli tui
```

The examples below use the installed executable. In this checkout, replace
`nayuta` with `bun run cli`.

## Install the packaged executable

Pack locally and install the tarball with Bun. Registry publication is not required:

```sh
cd packages/nayuta-cli
bun pm pack --destination /tmp
bun install --global /tmp/nayuta-cli-0.1.0.tgz
nayuta --help
```

Choose another output directory if `/tmp` is unavailable. Add Bun's global bin
directory to PATH if needed. The prepack script builds the executable and lazy
TUI modules; Bun remains the runtime after installation.

## Create a blog

```sh
nayuta blog create my-blog \
  --title "My Notebook" --author "Alex" --site-url https://example.com
```

The default template is `yuanzui-cf/nayuta` at `v0.2.0`. Override it with
`--template <repository-or-local-git-path>` and `--template-ref <branch-or-tag>`.
The destination must not exist. Dependencies are installed by default;
`--no-install` leaves installation for later.

Both **`src/content/` and `src/contents/` are removed from the template**. A new
blog receives this fresh content tree:

```text
src/content/
├── index.md
├── _left.astro
├── posts/
│   └── .gitkeep
├── pages/
│   └── .gitkeep
└── assets/
    └── .gitkeep
```

The homepage uses your title and localized introductory text. The generated
sidebar imports only widgets available in the selected template. Navigation
starts with the post archive, and the template biography is cleared. Theme code,
public assets, and licenses remain. `.nayuta-template.json` records provenance.

A new Git repository starts on `main` with one initial commit. Author and
committer both use the effective local Git `user.name` and `user.email` in the
destination, including `includeIf` rules. Template history and inherited author
environment overrides are discarded. The profile `--author` is independent of
Git identity. Global Git settings are never changed.

Set a missing Git identity first, or use `--no-commit`. Failures after Git
initialization preserve the local project for recovery. For Vercel, use a Git
email associated with your connected account.

Optional GitHub creation uses your existing authenticated `gh` installation:

```sh
nayuta blog create my-blog \
  --title "My Notebook" --author "Alex" --site-url https://example.com \
  --github-repo YOUR_USERNAME/my-blog --visibility private --push
```

The remote starts empty and receives your locally authored commit. `--push`
requires a repository name and an initial commit. Without `--push`, the command
creates the repository and sets `origin` without uploading commits.

## Configure a blog

```sh
nayuta --cwd ./my-blog blog setup \
  --title "Alex's Notes" --theme nayuta-aqua --posts-per-page 12
```

Settings include title, author, avatar, description, site URL, palette, start
date, page size, navigation links as JSON, and copyright-card defaults. Use
`--copyright-enabled true|false` and `--copyright-license "CC BY 4.0"` independently.
See `nayuta blog setup --help` for every flag.

Configuration is parsed as TypeScript, never executed. Static object exports,
named `config` variables and `satisfies` are supported. Unsafe dynamic edits and
object spreads are rejected. Other properties and surrounding comments remain.
Setting up an existing blog retains its content and Git history.

## Create an article

```sh
nayuta --cwd ./my-blog post create \
  --title "My first article" --slug my-first-article \
  --publish-date 2026-10-04 --format mdx --tags "Nayuta,Notes"
```

This follows the [Wiki's bundle convention](https://github.com/yuanzui-cf/nayuta/wiki/Writing-Posts):

```text
src/content/posts/2026-10-04-my-first-article/
├── index.mdx
└── assets/
    └── .gitkeep
```

Defaults are Markdown, today's date in the local timezone, and `draft: true`.
Use `--published` for `draft: false` or `--no-date-prefix` to omit the date from
the folder name. Folder names and `publishDate` are independent; a future date
does not schedule publication.

Slugs accept letters, numbers and hyphens, including Chinese characters. Without
`--slug`, the name is derived from the title. Existing standalone posts, folder
posts, file formats and custom slug overrides are checked for URL collisions.

`--cover` accepts HTTP(S), an existing public URL, or a local image file. Local
files are copied into `assets/` and referenced relatively. `--description` sets
the summary.

## Terminal interface

```sh
nayuta tui --cwd ./my-blog --locale zh-TW
```

Running `nayuta` without arguments opens the TUI in an interactive terminal;
piped execution displays help. Explicit TUI execution requires interactive
stdin/stdout and a terminal other than `TERM=dumb`.

The menu offers blog creation, settings, article creation, article search,
Frontmatter editing and language settings. Frontmatter editing is TUI-only.

| Key                 | Action                                       |
| ------------------- | -------------------------------------------- |
| Tab / Shift+Tab     | Move between fields                          |
| Arrow keys          | Choose values or navigate lists              |
| Enter               | Select an action or advance from a field     |
| Ctrl+S in a form    | Validate and preview                         |
| Ctrl+S in a preview | Save                                         |
| Tab in a preview    | Switch between the diff and actions          |
| Esc                 | Return or cancel, confirming unsaved changes |
| Ctrl+C              | Exit, confirming unsaved changes             |

In article search, type a query, then press Down or Enter to focus results.
Saving a preview finishes before another action can start.

The editor covers title, date, summary, cover, tags, views, draft/search visibility,
sidebar switches and copyright fields. Optional booleans offer inheritance,
enabled and disabled states. Unknown metadata, comments, key order, body text,
BOM and line endings are preserved. YAML quoting/indentation may be normalized;
dates and ambiguous strings use Astro-compatible YAML 1.1 semantics.

Writes reject snapshots changed by another editor. Reopen a modified article
before saving. Invalid YAML or metadata is reported instead of overwritten.

## Languages and script usage

Locale priority is `--locale`, saved TUI preference, `LC_ALL`, `LC_MESSAGES`,
`LANG`, then system locale. Normalize the selected value and fall back to English
when it is unsupported.

| Language                       | Accepted examples                               |
| ------------------------------ | ----------------------------------------------- |
| Simplified Chinese             | `zh-CN`, `zh_CN.UTF-8`, `zh-Hans`, `zh-Hans-CN` |
| Traditional Chinese, Taiwan    | `zh-TW`, `zh_TW.UTF-8`, `zh-Hant-TW`            |
| Traditional Chinese, Hong Kong | `zh-HK`, `zh_HK.UTF-8`, `zh-Hant-HK`            |
| English                        | `en` and `en-*`                                 |

Ambiguous `zh`/`zh-Hant`, `C`, `POSIX` and unsupported locales use English.
Taiwan and Hong Kong translations are independently authored. External Git,
YAML and package-manager diagnostic details may retain their original language.
Preferences use `$XDG_CONFIG_HOME/nayuta/config.json`, defaulting to
`~/.config/nayuta/config.json`; Windows uses `%APPDATA%/nayuta/config.json`.

`--cwd` selects the working directory; project discovery searches its ancestors.
CLI operations support `--json` with stable results/errors and nonzero failure
status. `--no-color` is accepted for script use. Blog setup and article creation
write files without automatically committing them.

## Development

Run `bun run check`, `bun run test` and `bun run build` in this package. Root
checks include package type checking; root tests include package tests. Fixtures
use temporary projects and Git repositories, OpenTUI's actual renderer, Astro
builds and a tarball executable. The package fixture shares installed dependencies
and does not access a live GitHub account. Packing tests require `tar`.

See [the CLI development guide](../../docs/develop/cli.md) for module ownership
and language registration.
