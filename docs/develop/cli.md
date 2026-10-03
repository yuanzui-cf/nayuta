# CLI and Terminal Interface

[< Development guide](index.md)

[`packages/nayuta-cli`](../../packages/nayuta-cli/README.md) is an independent Bun
package. Theme and CLI share a workspace lockfile but use separate TypeScript
configurations. Root checks and tests cover both.

## Ownership

| Directory             | Responsibility                                                       |
| --------------------- | -------------------------------------------------------------------- |
| `src/app/cli/`        | Arguments, localized help, text/JSON output and exit status          |
| `src/app/tui/`        | OpenTUI lifecycle, focus, forms, search, previews and cancellation   |
| `src/core/blog/`      | Blog creation and settings                                           |
| `src/core/post/`      | Discovery, naming, schema, YAML editing and article writes           |
| `src/core/project/`   | Project detection and supported theme versions                       |
| `src/infrastructure/` | Files, processes, Git, GitHub, template retrieval and preferences    |
| `src/i18n/`           | Translation contracts, registration, locale selection and formatting |
| `src/templates/`      | Generated article bodies                                             |
| `scripts/`            | Package build                                                        |
| `tests/`              | Isolated project, Git, terminal, build and package verification      |

Both interfaces call core workflows. Core errors carry message keys and
parameters; interfaces translate them. Do not import Astro runtime modules such
as `astro:content` into the executable. Standalone schemas match supported theme
contracts, verified with actual Astro builds.

## Template creation

Retrieve a branch/tag into a temporary checkout and copy files without Git history
or generated directories. Verify the project, delete `src/content/` and legacy
`src/contents/`, then create a fresh homepage, root sidebar and posts/pages/assets
directories. Generate the sidebar from available theme widgets instead of copying
authored template content. Retain theme code, public assets and licenses.

Initialize Git at the destination before resolving effective `user.name` and
`user.email`, so `includeIf` uses the final directory. Use that identity for both
author and committer. Never modify global Git settings or copy template history.
Optional GitHub creation starts with an empty remote.

Record provenance in `.nayuta-template.json`. Preserve initialized projects after
identity, installation, commit or remote failures and report incomplete steps.

## Content editing

Create folder bundles following the
[Wiki](https://github.com/yuanzui-cf/nayuta/wiki/Writing-Posts). New posts default
to drafts. Match Astro collection IDs when detecting collisions, including
single files, folder indexes and custom slugs. Ignore assets and underscore helpers.

Frontmatter editing is TUI-only. Use YAML nodes to preserve comments, unknown
metadata, aliases and order. Keep body text separate, retaining exact whitespace,
BOM and line endings. Match Astro's YAML 1.1 semantics, including quoted dates and
ordinary maps for nested copyright settings. Only changed fields enter patches;
inheritance, enabled and disabled values are distinct.

Validate metadata, cover paths and effective copyright licensing before saving.
Writes acquire a temporary lock, check the original snapshot, write a replacement,
check again and rename. Reject project paths escaping through symbolic links.

Parse site configuration as TypeScript rather than executing it. Modify static
initializers and retain unrelated source. Reject unsafe dynamic expressions
and object spreads.

## Add a language

1. Create a locale definition in `src/i18n/locale/`.
2. Provide every key in `src/i18n/types.ts` with matching interpolation parameters.
3. Register it and exact locale aliases in `src/i18n/register.ts`.
4. Extend locale-detection tests for new matching rules.

Taiwan and Hong Kong translations are maintained separately. Do not generate them
by converting Simplified Chinese. Select the highest-priority locale source first,
then match it or fall back to English.

## Build and validate

`bun run build:cli` emits a Bun ESM executable and lazy modules. OpenTUI native
assets remain dependencies rather than paths embedded from the developer's
machine. Package metadata declares the executable, runtime, dependencies, license,
published files and prepack build.

Run root formatting, checks, tests and site build after code changes. Terminal
tests exercise OpenTUI keyboard input and narrow layouts. Integration tests build
fresh empty and populated themes. Packaging tests unpack and run the executable
from an unrelated directory while sharing installed dependencies.
