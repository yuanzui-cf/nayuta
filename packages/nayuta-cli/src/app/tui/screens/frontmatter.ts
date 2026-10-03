import { licenses } from '../../../core/blog/schema';
import { postFields } from '../../../core/post/fields';
import { prepareFrontmatter } from '../../../core/post/frontmatter';
import {
  readPost,
  updatePost,
  validatePostContent,
} from '../../../core/post/service';
import { discoverProject } from '../../../core/project/discover';
import type { AppContext } from '../../context';
import { textDiff } from '../components/diff';
import { changedFields, type FormModel } from '../state';

export async function frontmatterForm(
  context: AppContext,
  path: string,
): Promise<FormModel> {
  const root = await discoverProject(context.cwd);
  const snapshot = await readPost(root, path);
  const t = context.i18n.t.bind(context.i18n);
  const initial: Record<string, string> = {};
  const fields = postFields.map((field) => {
    const [parent, child] = field.name.split('.');
    const value = child
      ? snapshot.values.copyright?.[child as 'enabled' | 'license']
      : snapshot.values[parent!];
    const text =
      field.kind === 'boolean'
        ? value === undefined
          ? 'inherit'
          : String(value)
        : Array.isArray(value)
          ? value.join(', ')
          : value === undefined
            ? ''
            : String(value);
    initial[field.name] = text;
    return {
      name: field.name,
      label: field.label,
      value: text,
      choices:
        field.kind === 'boolean'
          ? [
              { value: 'inherit', label: t('inherit') },
              { value: 'true', label: t('enabled') },
              { value: 'false', label: t('disabled') },
            ]
          : field.kind === 'license'
            ? [
                { value: '', label: t('inherit') },
                ...licenses.map((license) => ({
                  value: license,
                  label: license,
                })),
              ]
            : undefined,
    };
  });
  return {
    title: 'editFrontmatter',
    fields,
    prepare: async (values) => {
      const patch: Record<string, unknown> = {};
      for (const name of changedFields(initial, values)) {
        const field = postFields.find((item) => item.name === name)!;
        const text = values[name]!;
        patch[name] =
          field.kind === 'boolean'
            ? text === 'inherit'
              ? undefined
              : text === 'true'
            : field.kind === 'tags'
              ? text
                ? text.split(',')
                : []
              : text || (field.required ? '' : undefined);
      }
      const content = prepareFrontmatter(snapshot, patch);
      await validatePostContent(root, snapshot.path, content);
      return {
        preview: textDiff(snapshot.path, snapshot.source, content),
        save: () => updatePost(root, snapshot, patch),
      };
    },
  };
}
