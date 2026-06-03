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

export const collections = {
  posts: postsCollection,
};
