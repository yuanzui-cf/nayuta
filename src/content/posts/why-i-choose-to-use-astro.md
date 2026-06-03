---
title: 'Why I choose to use Astro?'
publishDate: '2026-11-11'
views: '1k'
readingTime: '4 mins'
description: 'A comprehensive exploration of why Astro is my framework of choice for building reading-first, static-first web applications. We dive into component islands, partial hydration, and developer experience.'
---

When it comes to building content-focused websites, I always look for three things: performance, developer experience, and simplicity. After experimenting with next-gen frameworks, I decided to migrate my projects to **Astro**.

Astro's core philosophy is "zero client-side JS by default." Most web frameworks render everything on the client side, shipping megabytes of JavaScript just to display static text. Astro reverses this paradigm. It renders HTML on the server and strips away unused JS, delivering insanely fast load speeds right out of the box.

### Component Islands Paradigm

The magic behind Astro is its **Islands Architecture**. Instead of making the entire page a single monolithic JS application, Astro lets you define small, isolated widgets (islands) of interactivity. If a search bar needs React, you hydrate only the search bar. The rest of the page remains raw, lightweight HTML.

This approach perfectly fits my design goals for Nayuta UI: a reading-first, distraction-free environment. Content should load instantly, and interactions should enhance the page, not drag it down.
