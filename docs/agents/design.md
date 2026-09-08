# Nayuta Astro Theme Paradigm

This document describes the intended UI and architecture paradigm for `nayuta`.
It is written as guidance for future agents working on the project.

This document describes architecture, boundaries, component categories, and
expected behavior.

## Positioning

`nayuta` is an Astro personal homepage and blog theme.

The theme provides a polished, responsive, reading-first foundation for
personal homepages, blogs, notes, and adjacent content pages. It combines Astro
pages and layouts, reusable components, composed widgets, templates, content
styling, design tokens, and theme support into a coherent site experience.

Short description:

> `nayuta` is a responsive Astro theme for personal homepages and blogs, with
> reading-first layouts, reusable UI components, composed profile/blog widgets,
> semantic tokens, and accessible responsive surfaces.

## Source of Truth for Visual Design and Tokens

The visual foundation and design token system are defined in:

- `src/styles/global.css`: Typography, spacing rhythm, border radii, and layout
  geometry.
- `src/styles/themes/*.css`: Modular color schemes providing semantic color
  tokens.
- Existing components in `src/components/*`, widgets in `src/widgets/*`, and
  layouts in `src/layouts/*`: Reference implementations for UI density,
  interactive states, and responsive behavior.

Agents should use these styling and component definitions to understand:

- Visual density and rhythm.
- Typography direction: Monospace aesthetic centered on JetBrains Mono and Fira
  Code.
- Layout behavior across desktop, narrow, and mobile viewports.
- Control, card, callout, prose, drawer, sidebar, and widget appearance.

Concrete CSS values, colors, spacing, and breakpoints should use semantic
tokens (`--ny-*`) rather than arbitrary ad-hoc inline styles.

## Design Goals

- Build an Astro-first personal homepage and blog theme.
- Prioritize reading and content consumption over dashboard density.
- Keep the homepage, blog, notes, and adjacent content pages cohesive.
- Preserve a clear separation between components, widgets, layouts, templates,
  styles, and content helpers.
- Provide reusable base components for controls, surfaces, media, metadata, and
  rich prose.
- Provide composed widgets for profile, navigation, recent content, links,
  status, table of contents, and related content.
- Provide layouts for adaptive pages with optional side regions.
- Support light and dark color schemes through semantic tokens and theme
  variables.
- Keep side information accessible on narrow screens through drawer-like,
  collapsible, or relocated surfaces.
- Prefer static Astro output and progressive enhancement for interactions that
  need client-side behavior.

## Architecture Layers

`nayuta` organizes repository code by responsibility:

```text
src/components/*
src/widgets/*
src/layouts/*
src/templates/*
src/styles/*
src/content/*
src/pages/*
src/config.ts
src/types/*
```

Additional helper modules may exist when needed, but they should not blur these
boundaries.

### Components

Components are small, reusable UI pieces.

A component should be simple enough that it does not need to own a whole page or
content model. Components define shape, styling, accessibility behavior, slots,
simple local state, token usage, and progressive enhancement hooks when needed.

Current implemented components:

- `Avatar`: User avatar display with consistent border and aspect ratio.
- `Badge`: Small inline badge/pill for metadata.
- `Button`: Versatile button supporting both `<button>` and `<a>` elements, with
  icon-only support.
- `Callout`: Styled admonition box supporting `note`, `tip`, `important`,
  `warning`, and `caution`.
- `Card`: Baseline card surface wrapper.
- `Icon`: Polymorphic icon renderer supporting SVG, image, and Font Awesome
  icons.
- `Input`: Single-line text input control.
- `Prose`: Typography container providing comfortable reading flow and Markdown
  styling.
- `TableContainer`: Horizontally scrollable wrapper for responsive tables.
- `Tag`: Clickable or display tag chip.
- `DynamicWidget`: Helper component that resolves widget names from frontmatter
  or configuration to widget implementations.

Planned / optional future components:

- `Collapse`
- `Mask`
- `Textarea`
- `CodeBlock`
- `Media`

### Widgets

Widgets are composed UI sections built from components, Astro components, HTML,
and data.

A widget carries homepage or blog theme semantics. Widgets can combine multiple
components, consume site configuration or content data, manage local
interaction, and expose a higher-level interface for pages and layouts.

Current implemented widgets:

- `ProfileCard`: Author identity, avatar, bio, and social/navigation link rows.
- `RecentPosts`: Recent blog posts list with dates and reading time.
- `FriendCard`: Friend link presentation card with avatar, name, description,
  and site logo.
- `WebsiteStatus`: Site operational status, uptime since date, and dynamic site
  metrics.
- `TableOfContents`: Article heading navigation hierarchy.
- `Categories`: Category listing widget.
- `TagCloud`: Aggregated tag cloud widget.
- `SearchWidget`: Site search trigger / input widget.
- `Widget`: Base card container wrapper with title header and content slot.

Planned / optional interactive widgets:

- `Dialog`
- `Toast`
- `Popover`
- `Drawer`
- `Menu`
- `Tabs`
- `Tooltip`
- `RelatedLinks`

`FriendCard` is a widget-level pattern, built using card, avatar, badge, and
tag primitives.

### Layouts

Layouts control page structure and responsive placement.

Layout code belongs in `src/layouts/*`, not in `src/components/*` or
`src/widgets/*`.

Current layout modules:

- `frame.astro`: Root HTML shell providing document structure, responsive
  three-column grid, narrow-viewport sticky header, mobile drawer surfaces, and
  progressive enhancement drawer scripts.
- `left-sidebar.astro`: Structural container for the left region dividing
  content into `header`, `widgets`, and `footer`.
- `right-sidebar.astro`: Structural container for the right contextual region.
- `head-base.astro`: Shared `<head>` element including SEO meta tags, title
  formatting, and theme CSS injection.

### Templates

Templates provide full-page content presentations driven by content collections
and frontmatter metadata.

Template code belongs in `src/templates/*`.

Current implemented templates:

- `PageTemplate.astro`: Default template wrapping general prose pages within
  the standard layout frame.
- `FriendTemplate.astro`: Specialized page template organizing friend links
  into structured categories, personal site showcase, and call-to-action
  sections.

Pages such as `src/pages/[...slug].astro` dynamically select and render
templates according to the `template` attribute defined in the content
collection entry.

### Styles and Tokens

Global styles, reset rules, prose defaults, design tokens, and theme variables
belong in `src/styles/*`.

- `src/styles/global.css`: Base resets, font declarations, typography tokens,
  and structural layout measurements.
- `src/styles/themes/*.css`: Modular theme stylesheets exposing `--ny-color-*`
  tokens (e.g. `nayuta`, `nayuta-aqua`, `midnight-blue`, `oled-dark`,
  `sakura-pink`).

The theme prefers semantic variables over hard-coded values in components and
widgets.

### Content

Content schemas, frontmatter conventions, collection loaders, and content query
helpers belong in `src/content.config.ts` and `src/content/*`.

Current content collections:

- `posts`: Markdown/MDX blog posts with title, publishDate, readingTime, cover,
  and description.
- `pages`: Markdown/MDX standalone pages supporting customizable templates,
  breadcrumbs, widget slots, and structured friend metadata.

### Pages

Astro routes belong in `src/pages/*`.

Pages compose layouts, widgets, content data, and components:

- `src/pages/index.astro`: Homepage showing author profile, recent posts, site
  status, and introductory bio.
- `src/pages/posts/index.astro`: Post listing and archives.
- `src/pages/posts/[slug].astro`: Individual post reader with table of contents
  and cover display.
- `src/pages/[...slug].astro`: Dynamic page router delegating to templates.

### Configuration and Types

Site-wide settings and TypeScript models:

- `src/config.ts`: Central site configuration instance defining site title,
  author, description, avatar, theme preset, and external links.
- `src/types/*`: TypeScript type definitions (e.g. `config.ts`, `icon.ts`).

## Core Layout Model

The main page model is:

```text
[left] [main] [right?]
```

`main` is required. `left` and `right` are optional layout regions around it.

The layout is reading-first. The center region preserves readable prose, stable
spacing, predictable heading hierarchy, usable code blocks, responsive tables,
and comfortable content rhythm. Side regions support the reading experience
without overpowering the primary content surface.

The left region holds site-level identity, navigation, and supporting widgets.
The right region holds page-local contextual content such as a table of
contents, metadata, related links, or reading tools.

## Left Region Structure

The left region has three structural subregions:

```text
left
├── header
├── widgets
└── footer
```

These are layout roles, not fixed components:

- `left.header`: Leading structural area holding user profile header, identity
  summary, or introductory content.
- `left.widgets`: Middle area for supporting widgets (navigation, recent posts,
  website status, tags, search).
- `left.footer`: Trailing structural area holding copyright, build metadata,
  and footer credits.

On mobile or narrow viewports, the layout preserves role separation:

- Leading identity/header content remains near the start of the page.
- Main content remains the primary reading flow.
- Footer content moves to the end of the mobile document flow.
- Supporting widgets move into a slide-out drawer accessible via the sticky
  header.

## Main Region

`main` is the primary content region.

It is usable for homepage content, articles, notes, listing pages, documentation
pages, prose pages, forms, or other content surfaces.

Expected properties:

- Stable readable flow.
- Good default prose behavior.
- Support for long headings, code blocks, tables, figures, media, and embedded
  components.
- Compatibility with Astro pages, layouts, slots, Markdown, and MDX.
- Clear content hierarchy and keyboard-readable structure.

## Right Region

The right region is optional and page-contextual.

Typical uses:

- Table of contents.
- Page metadata.
- Related links.
- Reading tools.
- Contextual actions.

On medium/tablet viewports (<= 1120px), the right region collapses before the
left region to preserve comfortable reading space.

## Narrow Viewport Header

A sticky header is enabled on narrower viewports (<= 1120px) to keep navigation
and collapsed-region entry points reachable.

This header contains:

- Breadcrumbs for current page navigation.
- A trigger button (`#open-widgets-btn`) to open the left widgets drawer (on
  mobile <= 768px).
- A trigger button (`#open-context-btn`) to open the right context drawer (on
  screens <= 1120px when right sidebar is present).

## Responsive Pattern

Desktop behavior (> 1120px):

```text
┌────────────┬──────────────────────┬────────────┐
│ left       │ main                 │ right?     │
│ header     │ reading content      │ widgets    │
│ widgets    │                      │            │
│ footer     │                      │            │
└────────────┴──────────────────────┴────────────┘
```

Narrow / Tablet behavior (769px - 1120px):

```text
┌───────────────────────────────────┬────────────┐
│ sticky layout header (context btn)│ (drawer)   │
├───────────────────────────────────┴────────────┤
│ left                              │ main       │
│ (header + widgets + footer)       │ content    │
└───────────────────────────────────┴────────────┘
```

Mobile behavior (<= 768px):

```text
┌─────────────────────────┐
│ sticky layout header    │
├─────────────────────────┤
│ left.header (profile)   │
├─────────────────────────┤
│ main                    │
│ reading content         │
├─────────────────────────┤
│ mobile footer           │
└─────────────────────────┘

left.widgets  -> slide-out left drawer
right.widgets -> slide-out right drawer
```

Drawer content is fed directly from the layout slots without requiring duplicate
markup from page consumers.

## Component and Widget Boundary

The boundary is based on responsibility and composition:

- Small reusable UI pieces are components.
- More complex sections composed from components, markup, and data are widgets.
- Page structure and region placement are layouts.
- Full-page presentations bound to content models are templates.
- Content querying and frontmatter conventions belong to content helpers and
  loaders.
- Global visual language and theme variables belong to styles and tokens.

Guidelines:

- An avatar is a component; a user profile header is a widget.
- A button is a component; a group of site actions is a widget or page
  composition.
- A card is a component; a friend link card list is a widget.
- A table container is a component; an entire friends directory page is a
  template.

## Theme and Tokens

`nayuta` exposes semantic tokens and allows project-level customization.

The public styling surface is semantic first:

- Background and surface colors (`--ny-color-bg`, `--ny-color-surface`,
  `--ny-color-surface-container`, `--ny-color-surface-elevated`).
- Text colors (`--ny-color-text`, `--ny-color-text-muted`,
  `--ny-color-text-subtle`).
- Primary and accent colors (`--ny-color-primary`, `--ny-color-primary-hover`,
  `--ny-color-outline`).
- Border and shadow colors (`--ny-color-border`, `--ny-color-shadow`).
- Radius scale (`--ny-radius-xs` to `--ny-radius-xl`).
- Spacing rhythm (`--ny-space-1` to `--ny-space-6`).
- Layout dimensions (`--ny-layout-*`).
- Typography families (`--ny-font-ui`, `--ny-font-prose`, `--ny-font-code`).

### Theme Selection

The active theme is configured in `src/config.ts` via the `theme` property.
`src/layouts/head-base.astro` statically loads and inlines the corresponding CSS
from `src/styles/themes/` at build time, ensuring zero runtime layout shifts.

Runtime theme switching can be added as a progressive enhancement where needed.

## Rendering and Enhancement

`nayuta` is built around Astro's static-first model.

### Static Astro Output

The default path renders complete HTML at build time:

- Pages, layouts, and templates render complete document structure.
- Markdown and MDX content renders to readable HTML.
- Static supporting regions render without requiring client JavaScript.
- Theme tokens are emitted and applied before content renders.
- The initial page is visually complete before any client enhancement.

### Progressive Enhancement

Client-side JavaScript is used only where it improves interaction:

- Drawer open/close state transitions and backdrop dismissals.
- Keyboard shortcuts (e.g. closing drawers via Escape key).
- Local search filtering and dynamic statistics calculation.
- Future dialog, popover, menu, tabs, tooltip, and toast interactions.

Static content does not require hydration to be readable.

## Accessibility Expectations

Interactive components, widgets, and layout surfaces must be accessible.

Expected behavior:

- Triggers are real buttons or valid links.
- Icon-only controls preserve accessible names (`aria-label`).
- Drawers, dialogs, and popovers expose appropriate accessible roles and
  labels.
- Escape closes open drawers and overlays.
- Keyboard users can reach collapsed left and right region content.
- Reduced motion preferences are respected.
- Color contrast meets readability standards across all bundled themes.
- Page structure uses semantic landmarks (`<aside>`, `<main>`, `<nav>`,
  `<header>`, `<footer>`).

## Agent Guidance

When generating code or documentation for `nayuta`, preserve these boundaries:

- Treat `nayuta` as an Astro personal homepage and blog theme.
- Put small reusable UI pieces in `src/components/*`.
- Put composed homepage and blog sections in `src/widgets/*`.
- Put page and region layout shells in `src/layouts/*`.
- Put full-page content views in `src/templates/*`.
- Put global styles, theme variables, and design tokens in `src/styles/*`.
- Put content schemas, loaders, and conventions in `src/content.config.ts` and
  `src/content/*`.
- Keep page routes in `src/pages/*` focused on high-level composition.
- Keep side regions collapsible, reachable, and accessible.
- Prefer static Astro output and progressive enhancement over unnecessary
  client-side rendering.
- Use repository-defined Prettier scripts for formatting.
- Prefer semantic tokens over hard-coded styles.
