import { z } from 'zod';
import { NayutaError } from '../errors';
export const themes = [
  'nayuta',
  'nayuta-aqua',
  'midnight-blue',
  'oled-dark',
  'sakura-pink',
] as const;
export const licenses = [
  'CC BY 4.0',
  'CC BY-SA 4.0',
  'CC BY-ND 4.0',
  'CC BY-NC 4.0',
  'CC BY-NC-SA 4.0',
  'CC BY-NC-ND 4.0',
] as const;
export const date = z
  .string()
  .regex(/^\d{4}-\d{2}-\d{2}$/)
  .refine((value) => {
    const parsed = new Date(`${value}T00:00:00Z`);
    return (
      Number.isFinite(parsed.getTime()) &&
      parsed.toISOString().slice(0, 10) === value
    );
  }, 'Use a valid YYYY-MM-DD date');
export const siteSettingsSchema = z.object({
  title: z.string().trim().min(1).optional(),
  author: z.string().trim().min(1).optional(),
  avatar: z.string().min(1).optional(),
  description: z.string().optional(),
  site_url: z.url({ protocol: /^https?$/ }).optional(),
  theme: z.enum(themes).optional(),
  since: date.optional(),
  postsPerPage: z.number().int().positive().optional(),
  links: z
    .array(
      z.looseObject({
        text: z.string().optional(),
        url: z.string().optional(),
        href: z.string().optional(),
        target: z.string().optional(),
        ariaLabel: z.string().optional(),
        iconOnly: z.boolean().optional(),
        icon: z.unknown().optional(),
      }),
    )
    .optional(),
  copyright: z
    .object({
      enabled: z.boolean().optional(),
      license: z.enum(licenses).optional(),
    })
    .optional(),
});
export type SiteSettings = z.infer<typeof siteSettingsSchema>;
export function validate<T>(schema: z.ZodType<T>, value: unknown): T {
  const result = schema.safeParse(value);
  if (!result.success) {
    const issue = result.error.issues[0]!;
    throw new NayutaError('errorValidation', {
      field: issue.path.join('.') || 'input',
      detail: 'invalidValue',
    });
  }
  return result.data;
}
export function today(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
}
