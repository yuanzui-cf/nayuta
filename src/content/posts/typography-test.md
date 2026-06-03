---
title: 'Typography & Kitchen Sink Test'
publishDate: '2026-06-03'
views: '120'
readingTime: '10 mins'
description: 'A comprehensive kitchen-sink verification page rendering all standard Markdown text formats including headings H2-H6, lists, callouts, horizontal rules, code blocks, keyboard inputs, and responsive tables.'
---

This page serves as a visual verification test ("kitchen
sink") demonstrating how various common article
typography elements render under the
**Nayuta UI** theme system. All elements
below mimic native Markdown output structures.

---

## Heading Levels (H2 to H6)

This is a paragraph preceding the H3 heading below.
Heading levels establish hierarchy and reading flow.

### Heading Level 3

This is a paragraph demonstrating font hierarchy. We use
a monospace typeface stack globally to preserve a clean
developer-oriented visual style.

#### Heading Level 4

A smaller sub-heading, ideal for minor structural units
or definitions inside sections.

##### Heading Level 5

Rarely used, but styled appropriately to maintain
logical hierarchy without overlapping neighboring
paragraphs.

###### Heading Level 6

The smallest available heading level, styled with muted
uppercase text for metadata or visual accents.

## Standard Lists

### Unordered List (Square Bullets)

<ul>
<li>First item in the list</li>
<li>
Second item in the list, showing word-wrapping
behavior when there is a significant amount of text.
Notice how the bullet aligns cleanly with the text.
</li>
<li>
Third item with a nested list:
<ul>
<li>Nested list item level 1</li>
<li>
Another nested list item
<ul>
<li>Double nested list item level 2</li>
</ul>
</li>
</ul>
</li>
<li>Final item in the unordered list</li>
</ul>

### Ordered List (Decimals)

<ol>
<li>
Ensure layout-container supports CSS grid correctly
</li>
<li>Implement responsive navigation breakpoints</li>
<li>
Deliver static files through zero-dependency bun
server:
<ol>
<li>Verify index.html page mapping</li>
<li>Verify posts/ subfolder routes</li>
</ol>
</li>
<li>
Confirm pixel perfection in mobile device simulators
</li>
</ol>

### Task Lists (Checkboxes)

<ul>
- [x]  Finish
implementing the responsive sticky mobile header

- [x] Resolve
      CSS Grid column layout overflow bugs

- [ ] Port Nayuta UI to
      official Astro component package

- [ ] Build automated
      testing suite for light/dark scheme seeds

</ul>

## Blockquotes & Callouts

### Standard Blockquotes

Blockquotes are used to highlight external quotes or
citations:

<blockquote>
“Simplicity is the ultimate sophistication.” — Leonardo
da Vinci. Monospace typography provides a very clean
container structure to emphasize quotes.
<blockquote>
Nested blockquotes are also supported. This helps
when citing conversations, comment threads, or email
chains.
</blockquote>
</blockquote>

### GitHub-Style Alerts / Callouts

<div class="callout callout-note">
<div class="callout-title">📝 Note</div>

This is a standard informational note. Use this for
general background context, guidelines, or helpful
explanations.

</div>

<div class="callout callout-tip">
<div class="callout-title">💡 Tip</div>

This is a performance tip. We recommend compiling
color schemes at build-time to avoid any flash of
unstyled content (FOUC).

</div>

<div class="callout callout-important">
<div class="callout-title">⚠️ Important</div>

Crucial requirement! Make sure any grid-container
column width on mobile uses
`minmax(0, 1fr)` rather than
`1fr` to contain code block widths.

</div>

<div class="callout callout-warning">
<div class="callout-title">⚡ Warning</div>

Warning details. Using arbitrary runtime script
injections will bypass security sandboxing and fail
review checks.

</div>

<div class="callout callout-caution">
<div class="callout-title">🛑 Caution</div>

High-risk warning! Deleting bun.lock or package.json
files may break local task runner pipelines.

</div>

## Media & Figures

Images should look beautiful, centered, with fine
borders and consistent corner radii:

<figure>
<img
src="../assets/images/banner.png"
alt="Nayuta abstract landscape banner"
/>
<figcaption>
Figure 1: Minimalist geometric landscape in dark
purple tones, generated to verify figure margin and
figcaption alignment.
</figcaption>
</figure>

## Responsive Comparison Tables

Below is a secondary table demonstrating the scrolling
wrapper structure under mobile viewports:

<div class="table-container">
<table>
<thead>
<tr>
<th>Framework</th>
<th>Rendering Style</th>
<th>JS Weight</th>
<th>Use Case</th>
<th>DX Score</th>
</tr>
</thead>
<tbody>
<tr>
<td>**Astro**</td>
<td>Islands Architecture</td>
<td>Zero by default</td>
<td>Blogs, Portfolios, Docs</td>
<td>9.5 / 10</td>
</tr>
<tr>
<td>**Next.js**</td>
<td>React Server Components</td>
<td>Heavy (React Hydration)</td>
<td>Web Applications, Dashboards</td>
<td>8.0 / 10</td>
</tr>
<tr>
<td>**SvelteKit**</td>
<td>Compiler-compiled components</td>
<td>Lightweight SPA</td>
<td>Interactive Tools, SaaS MVP</td>
<td>9.0 / 10</td>
</tr>
<tr>
<td>**Vite Static**</td>
<td>Vanilla Client-side JS</td>
<td>Custom size</td>
<td>Technical Prototypes, Mockups</td>
<td>8.5 / 10</td>
</tr>
</tbody>
</table>
</div>

## Code Blocks & Inline Formatting

### Inline Elements

We support standard inline decorations. You can
highlight terms using the
`&lt;code&gt;` element like
`const x = 42`. Text can be
**bolded**, <em>italicized</em>, or
<del>struck through</del>. We also style
<mark>marked highlight text</mark> and subscript
(H<sub>2</sub>O) or superscript (x<sup>2</sup>).

Keyboard combinations can be displayed using the kbd
tag: press <kbd>Ctrl</kbd> + <kbd>Shift</kbd> +
<kbd>R</kbd> to perform a hard refresh, or tap
<kbd>Esc</kbd> to dismiss drawer widgets.

### Description List (dl, dt, dd)

<dl>
<dt>Term 1: CSS Custom Properties</dt>
<dd>
Variables defined by CSS authors containing specific
values to be reused throughout a document. e.g.,
`--ny-color-bg`.
</dd>
<dt>Term 2: JIT Compilation</dt>
<dd>
Just-In-Time compilation, a method of improving the
execution performance of bytecode by compiling it
into native machine code at runtime.
</dd>
</dl>

### Syntax Highlighting Mockup

<pre>`// Simple Rust function demonstrating code formatting
fn calculate_fibonacci(n: u32) -> u64 {
let mut a = 0;
let mut b = 1;
for _ in 0..n {
let temp = a + b;
a = b;
b = temp;
}
a
}

fn main() {
let result = calculate_fibonacci(10);
println!("Fibonacci(10) is {}", result);
}`</pre>
