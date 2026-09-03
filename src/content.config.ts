import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const postsCollection = defineCollection({
  loader: glob({ pattern: '**/[^_]*.{md,mdx}', base: './src/content/posts' }),
  schema: z.object({
    title: z.string(),
    publishDate: z.string(),
    views: z.string(),
    readingTime: z.string(),
    cover: z.string().optional(),
    description: z.string().optional(),
  }),
});

const pagesCollection = defineCollection({
  loader: glob({ pattern: '**/[^_]*.{md,mdx}', base: './src/content/pages' }),
  schema: z
    .object({
      title: z.string(),
      description: z.string().optional(),
      template: z.string().optional().default('default'),
      withRightSidebar: z.boolean().optional(),
      rightWidgets: z.array(z.string()).optional().default([]),
      breadcrumbs: z
        .array(z.object({ text: z.string(), href: z.string() }))
        .optional(),
      friends: z
        .array(
          z.object({
            name: z.string(),
            site: z.string(),
            href: z.string(),
            desc: z.string(),
            avatar: z.string().optional(),
            avatarText: z.string().optional(),
            avatarBg: z.string().optional(),
            avatarColor: z.string().optional(),
            tags: z.array(z.string()).optional(),
            category: z.string().optional().default('tech'),
          }),
        )
        .optional()
        .default([]),
      mySite: z
        .object({
          name: z.string(),
          url: z.string(),
          avatar: z.string().optional(),
          desc: z.string().optional(),
        })
        .optional(),
    })
    .passthrough(),
});

export const collections = {
  posts: postsCollection,
  pages: pagesCollection,
};
