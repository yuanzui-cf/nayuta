# Nayuta Astro Theme Paradigm

This document describes the intended UI and architecture paradigm for `nayuta`.
It is written as guidance for future agents working on the project.

The concrete visual style and page examples live in `design/`. This document
should describe architecture, boundaries, component categories, and expected
behavior. It should not duplicate exact style values from the design files.

## Positioning

`nayuta` is an Astro personal homepage and blog theme.

The theme should provide a polished, responsive, reading-first foundation for
personal homepages, blogs, notes, and adjacent content pages. It should combine
Astro pages and layouts, reusable components, composed widgets, content styling,
design tokens, and theme support into a coherent site experience.

Short description:

> `nayuta` is a responsive Astro theme for personal homepages and blogs, with
> reading-first layouts, reusable UI components, composed profile/blog widgets,
> semantic tokens, and accessible responsive surfaces.

## Source of Truth for Visual Design

The reference visual direction comes from the files under `design/`.

Agents should use `design/` to understand:

- Visual density and rhythm.
- Typography direction.
- Layout behavior across desktop, narrow, and mobile viewports.
- Button, input, card, prose, drawer, sidebar, and widget appearance.
- Homepage and blog UI patterns that should be represented by components,
  widgets, layouts, styles, or content conventions.

This document should not copy concrete CSS values, exact colors, exact spacing,
or exact breakpoints. Those details should be read from the design files when
implementing or refining the theme.

The reference typeface direction is JetBrains Mono. Fallback stacks may be used,
but implementations should preserve the same compact monospace character unless
the theme is configured otherwise.

## Design Goals

- Build an Astro-first personal homepage and blog theme.
- Prioritize reading and content consumption over dashboard density.
- Keep the homepage, blog, notes, and adjacent content pages cohesive.
- Preserve a clear separation between components, widgets, layouts, styles, and
  content helpers.
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

`nayuta` should keep the project organized by responsibility:

```text
src/components/*
src/widgets/*
src/layouts/*
src/styles/*
src/content/*
src/pages/*
```

Additional helper modules may exist when needed, but they should not blur these
boundaries.

### Components

Components are small, reusable UI pieces.

A component should be simple enough that it does not need to own a whole page or
content model. Components define shape, styling, accessibility behavior, slots,
simple local state, token usage, and progressive enhancement hooks when needed.

Examples:

- `Avatar`
- `Button`
- `IconButton`
- `Input`
- `Textarea`
- `Card`
- `Badge`
- `Collapse`
- `Mask`
- `Prose`
- `CodeBlock`
- `TableContainer`
- `Callout`
- `Tag` or `Chip`
- `Media`

Components may be implemented as Astro components, CSS modules or global style
patterns, TypeScript helpers, or small client-side scripts when interaction
requires it.

### Widgets

Widgets are composed UI sections built from components, Astro components, HTML,
and data.

A widget may carry homepage or blog theme semantics. Widgets can combine
multiple components, consume site configuration or content data, manage local
interaction, and expose a higher-level interface for pages and layouts.

Examples of widgets that may belong in `nayuta`:

- `UserProfileHeader`
- `IdentitySummary` or `ProfileSummary`
- `NavigationGroup`
- `RecentPosts`
- `RecentNotes`
- `FriendLinks` or `LinkCards`
- `SiteStatus`
- `TableOfContents`
- `RelatedLinks`
- `Dialog`
- `Toast`
- `Popover`
- `Drawer`
- `Menu`
- `Tabs`
- `Tooltip`

`link-card` is a widget-level pattern, not a primitive component. It can be
built from an anchor element and a card component, so it should live with
composed UI unless a smaller reusable primitive emerges.

### Layouts

Layouts control page structure and responsive placement.

Layout code belongs in `src/layouts/*`, not in `src/components/*` or
`src/widgets/*`.

Examples:

- Main reading layout.
- Homepage layout.
- Blog post layout.
- Listing or archive layout.
- Left, main, and right region placement.
- Sidebar behavior.
- Responsive relocation of side-region content.
- Sticky narrow-viewport header behavior.
- Drawer entry point placement for collapsed regions.

A backdrop or overlay behind a sidebar or drawer can be a generic `Mask`
component, but the sidebar and region placement behavior belongs to the layout
layer.

### Styles and Tokens

Global styles, reset rules, prose defaults, design tokens, theme variables, and
color-scheme behavior belong in `src/styles/*` or another clearly named styling
area.

The theme should prefer semantic variables over hard-coded values in components
and widgets. Component-level styles may derive from global semantic tokens when
that keeps customization straightforward.

### Content

Content schemas, frontmatter conventions, collection helpers, and content query
helpers belong in `src/content/*` or another content-focused area.

Content-related code should make homepage and blog pages easy to assemble while
keeping rendering logic out of primitive components.

### Pages

Astro routes belong in `src/pages/*`.

Pages should compose layouts, widgets, content data, and components. They should
avoid becoming the only place where reusable theme behavior exists; shared page
patterns should move into layouts, widgets, styles, or content helpers.

## Core Layout Model

The main page model is:

```text
[left] [main] [right?]
```

`main` is required. `left` and `right` are optional layout regions around it.

The layout is reading-first. The center region should preserve readable prose,
stable spacing, predictable heading hierarchy, usable code blocks, responsive
tables, and comfortable content rhythm. Side regions should support the reading
experience; they should not become the primary content surface.

The left region commonly holds site-level identity, navigation, and supporting
widgets. The right region commonly holds page-local contextual content such as a
table of contents, metadata, related links, or reading tools.

## Left Region Structure

The left region has three structural subregions:

```text
left
├── header
├── widgets
└── footer
```

These are layout roles, not fixed components.

`left.header` is the leading structural area of the left region. It may hold a
user profile header, identity summary, site title, or introductory content.

`left.widgets` is the middle area for optional supporting content. It may hold
navigation, links, search entry points, recent content, controls, status, or
other widgets.

`left.footer` is the trailing structural area of the left region. It may hold
copyright, secondary links, build metadata, theme controls, or other footer
content.

On mobile or narrow viewports, the layout should not simply move the entire left
region into a drawer. It should preserve the role separation:

- Leading identity/header content can remain near the start of the page.
- Main content remains the primary reading flow.
- Footer content can remain near the end of the page.
- Supporting widgets can move into a collapsed surface.

## Main Region

`main` is the primary content region.

It should be usable for homepage content, articles, notes, listing pages,
documentation-like pages, prose pages, forms, or other content surfaces.

Expected properties:

- Stable readable flow.
- Good default prose behavior.
- Support for long headings, code blocks, tables, figures, media, and embedded
  components.
- Compatibility with Astro pages, layouts, slots, Markdown, and MDX when used.
- Clear content hierarchy and keyboard-readable structure.

## Right Region

The right region is optional and page-contextual.

Typical uses:

- Table of contents.
- Page metadata.
- Related links.
- Reading tools.
- Contextual actions.
- Secondary navigation.

The right region should not contain content required to understand the page. On
narrow screens it may collapse before the left region to preserve reading space.

## Narrow Viewport Header

The design reference uses a sticky header on narrower viewports to keep
navigation and collapsed-region entry points reachable.

This header is a layout affordance. It may contain:

- Breadcrumbs or page location.
- A trigger for collapsed left-region widgets.
- A trigger for collapsed right-region widgets.
- Compact theme or navigation actions.
- Other compact layout-level actions supplied by the page or layout.

The layout should allow this behavior without forcing every page to use the same
breadcrumb or action content.

## Responsive Pattern

Desktop behavior:

```text
┌────────────┬──────────────────────┬────────────┐
│ left       │ main                 │ right?     │
│ header     │ reading content      │ widgets    │
│ widgets    │                      │            │
│ footer     │                      │            │
└────────────┴──────────────────────┴────────────┘
```

Narrow and mobile behavior:

```text
┌──────────────────────┐
│ sticky layout header │
├──────────────────────┤
│ left.header?         │
├──────────────────────┤
│ main                 │
│ reading content      │
├──────────────────────┤
│ left.footer?         │
└──────────────────────┘

left.widgets  -> left collapsed surface
right.widgets -> right collapsed surface
```

The exact breakpoints and sizes should be taken from `design/` when implementing
the reference preset.

Pages should not be required to duplicate widget markup for desktop side regions
and mobile drawers. The layout should support responsive projection, relocation,
or another single-source pattern where practical.

## Component and Widget Boundary

The boundary is based on responsibility and composition:

- Small reusable UI pieces are components.
- More complex sections composed from components, markup, and data are widgets.
- Page structure and region placement are layouts.
- Content querying and frontmatter conventions belong to content helpers.
- Global visual language and theme variables belong to styles and tokens.

Guidelines:

- An avatar is a component; a user profile header is a widget.
- A button is a component; a group of site actions is a widget or page
  composition.
- A card is a component; a friend link card list is a widget.
- A mask is a component; a sidebar layout is layout.
- A collapse primitive is a component; a full navigation/sidebar section is a
  widget or layout composition.
- Dialog and toast may live in widgets because they are common composed
  interaction patterns.

## Reference Component Coverage

The design reference implies support for the following component areas. This
section names coverage only; exact visual values should come from `design/`.

### Controls

- Buttons.
- Icon buttons.
- Inputs.
- Textareas.
- Checkboxes.
- Compact action groups.
- Accessible focus-visible states.

### Surfaces

- Cards.
- Masks and backdrops.
- Collapsible surfaces.
- Drawer-like surfaces.
- Popover-like surfaces.

### Content

- Prose wrapper.
- Headings.
- Paragraph rhythm.
- Links.
- Lists.
- Task lists.
- Blockquotes.
- Callouts.
- Figures and captions.
- Tables with responsive overflow handling.
- Inline code.
- Code blocks.
- Keyboard input styling.
- Definition lists.

### Data and Metadata Display

- Badges.
- Tags or chips.
- Link rows.
- Simple metadata rows.
- Progress or meter-like rows when needed.

### Media

- Avatar-like media.
- Generic thumbnails.
- Responsive images.
- Cover media inside content or cards.

## Reference Widget Coverage

The design reference implies support for composed homepage and blog widgets.
This section names coverage only; exact visual values should come from
`design/`.

- User profile header.
- Identity or profile summary.
- Navigation groups.
- Recent posts or recent notes.
- Friend links or link cards.
- Site status.
- Table of contents.
- Related links.
- Theme or color-scheme controls.
- Drawer, dialog, popover, menu, tabs, tooltip, and toast when interaction calls
  for them.

## Theme and Tokens

`nayuta` should expose semantic tokens and allow project-level customization.
The reference design should be implemented through tokens and theme variables,
not through scattered hard-coded values.

The public styling surface should be semantic first:

- Background and surface colors.
- Text colors.
- Accent and primary colors.
- Border and outline colors.
- Shadow colors.
- Radius scale.
- Space scale.
- Typography families.
- Component-level derived tokens where needed.

`nayuta` should support adaptive light and dark modes. Colors may be generated
from a source color, provided explicitly by configuration, or emitted by
build-time tooling.

Preferred direction:

- Build-time or static token generation should be the main path when possible.
- Runtime theme switching can exist as a progressive enhancement.
- Explicit token input should be supported for projects that need full control.

## Rendering and Enhancement

`nayuta` should be designed around Astro's static-first model.

### Static Astro Output

The default path should render useful HTML at build time:

- Pages and layouts render complete document structure.
- Markdown or MDX content renders to readable HTML.
- Static supporting regions render without requiring client JavaScript.
- Theme tokens can be emitted before content renders.
- The initial page should be visually complete before any client enhancement.

### Progressive Enhancement

Client-side JavaScript should be used when it improves interaction:

- Drawer state.
- Collapse state.
- Dialog, popover, menu, tabs, tooltip, and toast behavior.
- Theme switching when runtime switching is supported.
- Small controls that need local browser state.

Interactive widgets should keep sensible fallback behavior where practical.
Static content should not require hydration just to be readable.

### Astro Islands

Astro islands or client directives may be used for interactive widgets when a
plain Astro component and minimal script are not enough.

Use islands selectively. Avoid turning static layout, prose, metadata, or simple
cards into client-rendered UI without a concrete interaction need.

## Implementation Direction

The preferred implementation direction is Astro, TypeScript, CSS, Bun, and Prettier.

Astro is a good fit because it:

- Supports static-first sites and content-heavy pages.
- Provides layouts, components, pages, slots, and content collections.
- Works well with Markdown and MDX.
- Allows selective client-side enhancement through islands and directives.
- Keeps the baseline site lightweight when components are mostly static.

TypeScript should be used for configuration, content helpers, data shaping, and
interactive behavior where it improves maintainability.

CSS should prefer semantic variables, readable cascade boundaries, and component
or layout ownership. Avoid scattering one-off visual values across unrelated
files.

Bun is the required package manager and script runner for repository commands.

Astro owns the frontend build and development flow. Treat Vite as Astro's
internal integration unless the repository explicitly defines Vite-specific
configuration or scripts that need attention.

Prettier is the repository formatter for Astro, Markdown, CSS, TypeScript,
JavaScript, JSON, and other supported text files. Run formatting through
repository-defined Bun scripts when available, and never allow automated
formatting or lint fixes to modify `design/`.

## Accessibility Expectations

Interactive components, widgets, and layout surfaces must be accessible.

Expected behavior:

- Triggers are real buttons or appropriate interactive elements.
- Icon-only controls preserve accessible names.
- Drawers, dialogs, popovers, and similar widgets expose appropriate labels.
- Modal-like surfaces trap focus when required.
- Escape closes modal-like surfaces where appropriate.
- Closing a modal-like surface returns focus to the trigger where practical.
- Keyboard users can reach collapsed left and right region content.
- Reduced motion preferences are respected.
- Color contrast works in both light and dark modes.
- Page structure uses semantic landmarks and heading hierarchy.

## Agent Guidance

When generating code or documentation for `nayuta`, preserve these boundaries:

- Treat `nayuta` as an Astro personal homepage and blog theme.
- Treat concrete pages in `design/` as reference examples for visual direction
  and interaction expectations.
- Do not copy exact style values into this document; inspect `design/` when
  implementation needs visual details.
- Put small reusable UI pieces in `src/components/*`.
- Put composed homepage and blog sections in `src/widgets/*`.
- Put page and region layout code in `src/layouts/*`.
- Put global styles, theme variables, and design tokens in `src/styles/*` or an
  equivalent styling area.
- Put content schemas, frontmatter conventions, and query helpers in
  content-focused areas.
- Keep page routes in `src/pages/*` focused on composition.
- Keep side regions collapsible, reachable, and accessible.
- Keep narrow viewport entry points reachable.
- Prefer static Astro output and progressive enhancement over unnecessary
  client-side rendering.
- Use repository-defined Prettier scripts for formatting, and keep automated
  formatting away from `design/`.
- Prefer semantic tokens over hard-coded styles.

Avoid generating APIs or implementations that make future content maintenance,
theme customization, responsive behavior, or Astro static output difficult.
