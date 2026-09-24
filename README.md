![Nayuta banner](public/assets/images/banner.png)

<div align="center">

# Nayuta

A responsive, reading-first Astro theme for personal homepages, blogs, and notes.

[![Astro](https://img.shields.io/badge/Astro-7.x-FF5D01?style=flat-square&logo=astro&logoColor=white)](https://astro.build)
[![TypeScript](https://img.shields.io/badge/TypeScript-6.x-3178C6?style=flat-square&logo=typescript&logoColor=white)](https://www.typescriptlang.org)
[![Bun](https://img.shields.io/badge/Bun-1.x-FBF0DF?style=flat-square&logo=bun&logoColor=black)](https://bun.sh)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](LICENSE)

[Features](#features) · [Showcase](#showcase) · [Quick start](#quick-start) ·
[Configuration](#configuration) · [Content](#content) · [Contributing](#contributing)

</div>

Nayuta combines static site generation with compact monospace UI and a larger
reading scale. Its responsive layout places navigation and widgets beside the
content on desktop and makes them available through drawers on smaller screens.

The name comes from **Kani Nayuta** (可児 那由多), the novelist in Yomi Hirasaka's
_A Sister's All You Need_ (妹さえいればいい。).

## Features

- **Reading-focused typography:** JetBrains Mono and Fira Code, with separate
  scales for prose and compact interface text.
- **Responsive layout:** optional side columns, mobile drawers, and keyboard
  navigation with a no-JavaScript fallback.
- **Five color palettes:** `nayuta`, `nayuta-aqua`, `midnight-blue`, `oled-dark`,
  and `sakura-pink`, selected in the site configuration.
- **Astro content collections:** Markdown/MDX posts, authored Astro pages, and
  validated metadata.
- **Automatic reading time:** build-time estimates for CJK and Latin text.
- **Post and tag archives:** static pagination, tag pages, and sidebar widgets.
- **Custom sidebars and templates:** per-page Astro widgets, an article TOC,
  personal pages, and friend-link collections.

## Showcase

| Desktop homepage                                                                                     | Mobile homepage                                                                                  |
| ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| <img src="docs/assets/01-home-desktop-1920x1080.png" alt="Nayuta homepage on desktop" width="650" /> | <img src="docs/assets/05-home-mobile-440x956.png" alt="Nayuta homepage on mobile" width="180" /> |

<details>
<summary>More screenshots: posts, articles, and friends</summary>

| Page    | Desktop                                                                                              | Mobile                                                                                           |
| ------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| Posts   | <img src="docs/assets/02-posts-desktop-1920x1080.png" alt="Post archive on desktop" width="500" />   | <img src="docs/assets/06-posts-mobile-440x956.png" alt="Post archive on mobile" width="150" />   |
| Article | <img src="docs/assets/03-article-desktop-1920x1080.png" alt="Article on desktop" width="500" />      | <img src="docs/assets/07-article-mobile-440x956.png" alt="Article on mobile" width="150" />      |
| Friends | <img src="docs/assets/04-friends-desktop-1920x1080.png" alt="Friend links on desktop" width="500" /> | <img src="docs/assets/08-friends-mobile-440x956.png" alt="Friend links on mobile" width="150" /> |

</details>

## Quick Start

Use [Bun 1.x](https://bun.sh) (recommended), or [Node.js](https://nodejs.org)
as an alternative runtime for Astro.

The following setup uses only Bun; no separate Node.js installation is needed:

```sh
git clone https://github.com/yuanzui-cf/nayuta.git
cd nayuta
bun install
bun run dev
```

Open [localhost:4321](http://localhost:4321). To contribute to the theme, follow
[the fork setup](CONTRIBUTING.md#requirements-and-setup) instead.

| Command                | Purpose                                                         |
| ---------------------- | --------------------------------------------------------------- |
| `bun run dev`          | Start the development server.                                   |
| `bun run check`        | Check Astro templates and TypeScript.                           |
| `bun run test`         | Run the test suite.                                             |
| `bun run build`        | Generate the static site and Pagefind article index in `dist/`. |
| `bun run preview`      | Preview the production build, including search.                 |
| `bun run format`       | Format source and documentation with Prettier.                  |
| `bun run format:check` | Check formatting without changing files.                        |

The sidebar searches published posts by title and body using Pagefind. Run
`bun run build` before `bun run preview` to generate the search index; the
development server alone does not generate one. Search needs JavaScript; the
sidebar links to the post archive when search is unavailable.

## Configuration

Customize your site profile, metadata, navigation, and theme preset in [`src/config.ts`](./src/config.ts):

```typescript
import type { Config } from '@type/config';

const config: Config = {
  title: '✍️ Kani Nayuta',
  author: 'Kani Nayuta',
  avatar: '/assets/images/avatar.png',
  description: 'Genius Light Novelist | Author of "The Landscape Series"',
  site_url: 'https://nayuta.kani.dev',
  // Presets: 'nayuta' | 'nayuta-aqua' | 'midnight-blue' | 'oled-dark' | 'sakura-pink'
  theme: 'nayuta',
  since: '2024-01-01',
  postsPerPage: 10,
  links: [
    {
      text: 'Posts',
      url: '/posts',
      icon: { type: 'icon', name: 'fa-solid fa-file-lines' },
    },
    {
      text: 'Friends',
      url: '/friend',
      icon: { type: 'icon', name: 'fa-solid fa-users' },
    },
  ],
};

export default config;
```

The checked-in default is `nayuta`; all five bundled palettes are dark.
`site_url` must be an absolute HTTP(S) URL. `postsPerPage` controls post and tag
archives and defaults to 10 when omitted.

## Content

- [Writing posts](#writing-posts)
- [Homepage and custom pages](#homepage-and-custom-pages)
- [Routes and assets](#routes-and-assets)
- [Page templates](#page-templates)
- [Custom sidebars](#custom-sidebars)
- [Bundled demos](#bundled-demos)

### Writing Posts

Place `.md` or `.mdx` files into `src/content/posts/`. You can author posts as standalone files or folder bundles:

```markdown
---
title: 'The Architecture of Reading-First Design'
publishDate: '2026-09-09'
description: 'Balancing information density and typography.'
cover: '/assets/images/banner.png' # or './cover.png' in a folder bundle
tags: ['Astro', 'CSS Grid', 'Typography']
---

Article content here...
```

- **Reading time**: Calculated automatically at compile time based on bilingual (CJK + Latin) content.
- **Folder bundles**: For posts with colocated images, create `src/content/posts/<slug>/index.md` alongside your assets and use relative paths like `cover: './cover.png'`.
- **MDX components**: Reusable components such as `<Callout />` are available under `src/widgets/article/`.

Set `draft: true` to preview a post in development while excluding it from the
production site. Tags produce archive links at `/tag/<name>`.

### Homepage and Custom Pages

Edit the homepage in `src/content/index.mdx`. You can use `index.md` or
`index.astro` instead; keep exactly one homepage file. The theme supplies the
page frame, title, sidebars and responsive drawers.

```text
src/content/
├── assets/                    # Shared content resources, never routes
├── index.mdx                  # Homepage: .md / .mdx / .astro
├── _left.astro                # Default left widgets
├── _right.astro               # (Optional) Default right widgets
├── pages/
│   ├── about.astro            # /about
│   └── a/b/c/
│       ├── index.mdx          # /a/b/c
│       ├── _left.astro        # (Optional) Override the left widgets for this directory
│       ├── _right.astro       # (Optional) Override the right widgets for this directory
│       └── assets/            # (Optional) Directory-specific assets, won't be routed
└── posts/
    └── hello/
        ├── world/
        │   ├── assets/            # (Optional) Directory-specific assets, won't be routed
        │   └── index.mdx          # /posts/hello/world
        ├── index.mdx          # /posts/hello
        ├── _left.astro
        └── _right.astro       # (Optional) Additional content below the mandatory TOC
```

Standalone pages accept `.md`, `.mdx` and `.astro`. For Markdown/MDX:

```markdown
---
title: About Me
description: A short introduction
template: default
---

Page content goes here...
```

An Astro page exports the same metadata as `page` and supplies body content:

```astro
---
export const page = { title: 'About Me', description: 'A short introduction' };
---

<p>Compose Astro components here. The theme renders the page title.</p>
```

Astro pages receive `entry` and `headings` props. Their `headings` array is empty;
automatic heading extraction applies to Markdown/MDX. Article bodies remain
Markdown/MDX so reading time, drafts, tags and the article TOC retain their normal
behavior. Astro pages can contain scoped styles and normal Astro scripts, but
should not include another HTML document or Frame. Dynamic route declarations
such as `[id].astro` belong in `src/pages`, not in authored content.

### Routes and Assets

Routes follow the path below `content/pages`, without the extension or terminal
`index`. Thus `a/b/c.md` and `a/b/c/index.mdx` both describe `/a/b/c`; choose one.
Paths retain filename case. An optional `slug: published/path` overrides the URL
without changing where the theme finds sidebars or relative assets. Duplicate
standalone URLs, including different file formats, fail with the source paths.
The homepage dispatcher and generated system routes take priority over content
pages: `pages/posts.mdx` cannot replace the archive, and the build reports the
ignored route without failing. Only URLs actually emitted by system routes win;
this does not reserve every possible URL below `/posts` or `/tag`.

All `assets/` directories and underscore-prefixed files/directories are excluded
from page and post discovery. Resource files remain importable, including relative
Markdown images and MDX/Astro imports. They are not automatically copied to matching
public URLs; use `public/` for files that need stable, direct download URLs.
Only the root `index.*`, files under `pages/`, and registered posts are entries;
other files directly under `content/` do not become routes.

### Page Templates

Pages use `template: default` for ordinary reading content or `template: friend`
for a friend-link collection. See [the bundled friends page](src/content/pages/friend.mdx)
for an example of `friends`, `categories`, and `mySite` metadata.

### Custom Sidebars

Each side resolves independently: a `_left.astro` or `_right.astro` beside the
content file takes precedence over the corresponding file at `src/content/`.
There is no search through intermediate ancestor directories. Files in the same
directory share sidebars; use a folder bundle for independent per-page overrides.
The profile card and footer remain part of the layout.

For example, place this at `src/content/_right.astro` for a site-wide default,
or next to a page's `index.mdx` to replace that default for the page:

```astro
---
import RecentPosts from '@widgets/sidebar/RecentPosts.astro';
import WebsiteStatus from '@widgets/sidebar/WebsiteStatus.astro';
---

<RecentPosts />
<WebsiteStatus />
```

Without a local or root right component, ordinary pages have no right column or
right drawer. Article detail always renders its TOC first, followed by the chosen
right component. Article/tag archives and the 404 page use the root defaults.

Both sidebar components receive `entry`, `headings` and `pathname` through
`Astro.props`. `entry` is absent for system archive/404 pages, so access it with
optional chaining. Match sidebar links against `pathname`, not a collection ID.
Sidebars render in both desktop regions and mobile drawers; avoid fixed HTML IDs
and scope any client behavior to its component instance.

Optional metadata switches are `withLeftSidebar: false` (hide the entire left
region), `withProfileCard: false` (hide only the profile), and
`withRightSidebar: false` (hide optional right content). The latter never hides an
article's TOC. Setting it to `true` does not create an empty right column when no
right component exists. An empty local component replaces the root content but
still counts as an existing right component; use the switch to hide the region.

For discovery details and migration from `rightWidgets`, see
[the sidebar contract](docs/develop/content.md#sidebars).

### Bundled Demos

Open `/demo` (also linked from the homepage) to explore the examples:

| Route                    | Source                                          | Demonstrates                                                    |
| ------------------------ | ----------------------------------------------- | --------------------------------------------------------------- |
| `/demo/markdown`         | `src/content/pages/demo/markdown.md`            | Root left sidebar, no right sidebar                             |
| `/demo/notes/nested`     | `src/content/pages/demo/notes/nested/index.mdx` | Nested routing, local assets, local right sidebar               |
| `/demo/studio`           | `src/content/pages/demo/studio/index.astro`     | Astro body and both local sidebars                              |
| `/posts/typography-test` | `src/content/posts/typography-test/index.mdx`   | Bundled article, local sidebars, TOC before extra right content |

## License

[MIT](LICENSE)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for discussing changes, setting up a fork,
branch and commit conventions, checks, and submitting pull requests. The
[development guide](docs/develop/index.md) covers visual design and code conventions.
