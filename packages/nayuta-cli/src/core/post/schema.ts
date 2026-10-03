import { z } from 'zod';
import { licenses } from '../blog/schema';

// Keep this standalone contract aligned with Nayuta 0.2.x content collections.
export const postSchema = z.looseObject({
  title: z.string().min(1),
  publishDate: z.string().min(1),
  description: z.string().optional(),
  cover: z.string().min(1).optional(),
  views: z.string().optional(),
  tags: z.array(z.string().trim().min(1)).optional(),
  draft: z.boolean().optional(),
  exclude_in_search: z.boolean().optional(),
  withLeftSidebar: z.boolean().optional(),
  withProfileCard: z.boolean().optional(),
  withRightSidebar: z.boolean().optional(),
  copyright: z
    .object({
      enabled: z.boolean().optional(),
      license: z.enum(licenses).optional(),
    })
    .optional(),
});
export type PostData = z.infer<typeof postSchema>;
